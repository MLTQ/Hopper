use super::Store;
use crate::{import, model::LocalMedia};
use anyhow::Result;
use rusqlite::{Connection, params};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
pub(crate) struct ImportedFolder {
    pub id: String,
    pub name: String,
    pub files: usize,
}

fn folder_id(name: &str) -> String {
    blake3::hash(name.as_bytes()).to_hex().to_string()
}

fn put_file(db: &Connection, name: &str, file: &LocalMedia) -> Result<()> {
    db.execute(
        "INSERT OR REPLACE INTO folder_imports(folder_id, name, media_id, value) VALUES (?1, ?2, ?3, ?4)",
        params![folder_id(name), name, file.media.id, serde_json::to_string(file)?],
    )?;
    Ok(())
}

pub(super) fn initialize(db: &mut Connection) -> Result<()> {
    let tx = db.transaction()?;
    let initialized: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='folder_imports')",
        [],
        |row| row.get(0),
    )?;
    if initialized {
        return Ok(());
    }
    tx.execute_batch(
        "CREATE TABLE folder_imports(folder_id TEXT NOT NULL, name TEXT NOT NULL, media_id TEXT NOT NULL,
         value TEXT NOT NULL, PRIMARY KEY(folder_id, media_id));
         CREATE INDEX folder_import_media ON folder_imports(media_id);
         CREATE TRIGGER release_delivered_folder_media AFTER INSERT ON records
         WHEN NEW.kind='delivery' AND NEW.value ->> '$.state'='delivered' BEGIN
             DELETE FROM folder_imports WHERE folder_id='' AND media_id=NEW.value ->> '$.payload.id'
             AND NOT EXISTS (SELECT 1 FROM records WHERE kind='delivery'
                 AND value ->> '$.payload.id'=NEW.value ->> '$.payload.id' AND value ->> '$.state'!='delivered');
         END;",
    )?;
    if !cfg!(target_os = "ios") {
        // Legacy uploads stored only a logical folder. Preserve records with evidence of
        // independent podcast, Home, or peer ownership in the original local collection.
        let mut query = tx.prepare(
            "SELECT value FROM records WHERE kind='local' AND json_extract(value,'$.media.folder') != ''
             AND (NOT json_extract(value,'$.managed') OR key NOT IN (
                SELECT value ->> '$.original_id' FROM records WHERE kind='podcast_episode' AND value ->> '$.original_id' IS NOT NULL
                UNION SELECT value ->> '$.cleaned_id' FROM records WHERE kind='podcast_episode' AND value ->> '$.cleaned_id' IS NOT NULL
                UNION SELECT value ->> '$.media_id' FROM records WHERE kind='received_delivery' AND value ->> '$.media_id' IS NOT NULL
                UNION SELECT item.value ->> '$.id' FROM records r, json_each(r.value,'$.items') item WHERE r.kind='snapshot' AND item.value ->> '$.id' IS NOT NULL
             ))",
        )?;
        let legacy = query
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(query);
        for json in legacy {
            let file: LocalMedia = serde_json::from_str(&json)?;
            let Some(root) = file
                .media
                .folder
                .split('/')
                .next()
                .filter(|s| !s.is_empty())
            else {
                continue;
            };
            let mut name = root.to_owned();
            if !file.managed
                && let Some(parent) = file.path.parent()
                && parent.ends_with(Path::new(&file.media.folder))
                && let Some(path) = parent
                    .ancestors()
                    .nth(file.media.folder.split('/').count() - 1)
            {
                name = path.to_string_lossy().into_owned();
            }
            put_file(&tx, &name, &file)?;
            tx.execute(
                "DELETE FROM records WHERE kind='local' AND key=?1",
                [&file.media.id],
            )?;
        }
    }
    tx.commit()?;
    Ok(())
}

impl Store {
    pub(crate) fn put_import(&self, folder: Option<&str>, file: &LocalMedia) -> Result<()> {
        match folder {
            Some(name) => put_file(&self.0.lock().unwrap(), name, file),
            None => self.put("local", &file.media.id, file),
        }
    }

    pub(crate) fn imported_folders(&self) -> Result<Vec<ImportedFolder>> {
        let db = self.0.lock().unwrap();
        let mut q = db.prepare(
            "SELECT folder_id, name, COUNT(*) FROM folder_imports WHERE folder_id != '' GROUP BY folder_id ORDER BY name",
        )?;
        Ok(q.query_map([], |r| {
            Ok(ImportedFolder {
                id: r.get(0)?,
                name: r.get(1)?,
                files: r.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?)
    }

    pub(crate) fn imported_files(&self) -> Result<Vec<LocalMedia>> {
        let db = self.0.lock().unwrap();
        let mut q = db.prepare("SELECT value FROM folder_imports ORDER BY folder_id, media_id")?;
        q.query_map([], |r| r.get::<_, String>(0))?
            .map(|s| Ok(serde_json::from_str(&s?)?))
            .collect()
    }

    pub(crate) fn imported_file(&self, id: &str) -> Result<Option<(String, LocalMedia)>> {
        let db = self.0.lock().unwrap();
        let mut q = db.prepare(
            "SELECT name, value FROM folder_imports WHERE media_id=?1 ORDER BY folder_id",
        )?;
        let mut rows = q.query([id])?;
        while let Some(row) = rows.next()? {
            let file = serde_json::from_str(&row.get::<_, String>(1)?)?;
            if import::available(&file) {
                return Ok(Some((row.get(0)?, file)));
            }
        }
        Ok(None)
    }

    pub(crate) fn remove_imported_folder(&self, id: &str) -> Result<()> {
        let mut db = self.0.lock().unwrap();
        let tx = db.transaction()?;
        // Home must still be able to fetch an already queued ID. An unlisted source
        // holds its staged bytes until the last delivery completes, even across restart.
        tx.execute(
            "INSERT OR IGNORE INTO folder_imports(folder_id, name, media_id, value)
             SELECT '', '', media_id, value FROM folder_imports f WHERE folder_id=?1
             AND value ->> '$.managed' AND EXISTS (
                 SELECT 1 FROM records r WHERE r.kind='delivery' AND r.value ->> '$.state'!='delivered'
                 AND r.value ->> '$.payload.id'=f.media_id)",
            [id],
        )?;
        tx.execute("DELETE FROM folder_imports WHERE folder_id=?1", [id])?;
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn upgrades_existing_imports_and_retains_only_needed_delivery_sources() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("library.sqlite");
        let root = dir.path().join("Archive");
        std::fs::create_dir(&root).unwrap();
        let source = root.join("one.wav");
        std::fs::write(&source, b"legacy media").unwrap();
        let file = import::inspect(&source, "one.wav", "Archive", false).unwrap();
        let other = root.join("two.wav");
        std::fs::write(&other, b"managed media").unwrap();
        let managed = import::inspect(&other, "two.wav", "Copied/Season", true).unwrap();
        let id = &managed.media.id;
        {
            let conn = Connection::open(&db).unwrap();
            conn.execute_batch(
                "CREATE TABLE records(kind TEXT, key TEXT, value TEXT, PRIMARY KEY(kind,key));",
            )
            .unwrap();
            for media in [&file, &managed] {
                conn.execute(
                    "INSERT INTO records VALUES('local', ?1, ?2)",
                    params![media.media.id, serde_json::to_string(media).unwrap()],
                )
                .unwrap();
            }
        }
        let store = Store::open(&db).unwrap();
        let folders = store.imported_folders().unwrap();
        assert_eq!(folders.len(), 2);
        assert!(folders.iter().any(|f| f.name == root.to_string_lossy()));
        let copied = folders.iter().find(|f| f.name == "Copied").unwrap();
        let mut delivery = json!({"payload":{"kind":"media","id":id}, "state":"waiting"});
        store.put("delivery", "first", &delivery).unwrap();
        store.put("delivery", "second", &delivery).unwrap();
        store.remove_imported_folder(&copied.id).unwrap();
        assert_eq!(store.imported_folders().unwrap().len(), 1);
        assert!(store.imported_file(id).unwrap().is_some());
        drop(store);
        let store = Store::open(&db).unwrap();
        delivery["state"] = json!("delivered");
        store.put("delivery", "first", &delivery).unwrap();
        assert!(store.imported_file(id).unwrap().is_some());
        store.put("delivery", "second", &delivery).unwrap();
        assert!(store.imported_file(id).unwrap().is_none());
        assert_eq!(std::fs::read(&other).unwrap(), b"managed media");
        assert_eq!(std::fs::read(&source).unwrap(), b"legacy media");
    }
}
