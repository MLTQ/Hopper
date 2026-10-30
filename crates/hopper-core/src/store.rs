use crate::model::{Progress, now};
use anyhow::{Result, bail};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Serialize, de::DeserializeOwned};
use std::{path::Path, sync::Mutex};

pub struct Store(Mutex<Connection>);

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let c = Connection::open(path)?;
        c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;
            CREATE TABLE IF NOT EXISTS records(kind TEXT NOT NULL, key TEXT NOT NULL, value TEXT NOT NULL,
            PRIMARY KEY(kind,key));")?;
        Ok(Self(Mutex::new(c)))
    }
    pub fn get<T: DeserializeOwned>(&self, kind: &str, key: &str) -> Result<Option<T>> {
        let c = self.0.lock().unwrap();
        let value: Option<String> = c
            .query_row(
                "SELECT value FROM records WHERE kind=?1 AND key=?2",
                params![kind, key],
                |r| r.get(0),
            )
            .optional()?;
        value.map(|s| Ok(serde_json::from_str(&s)?)).transpose()
    }
    pub fn all<T: DeserializeOwned>(&self, kind: &str) -> Result<Vec<T>> {
        let c = self.0.lock().unwrap();
        let mut q = c.prepare("SELECT value FROM records WHERE kind=?1 ORDER BY key")?;
        q.query_map([kind], |r| r.get::<_, String>(0))?
            .map(|s| Ok(serde_json::from_str(&s?)?))
            .collect()
    }
    pub fn put(&self, kind: &str, key: &str, value: &impl Serialize) -> Result<()> {
        self.0.lock().unwrap().execute(
            "INSERT OR REPLACE INTO records VALUES(?1,?2,?3)",
            params![kind, key, serde_json::to_string(value)?],
        )?;
        Ok(())
    }
    pub fn remove(&self, kind: &str, key: &str) -> Result<()> {
        self.0.lock().unwrap().execute(
            "DELETE FROM records WHERE kind=?1 AND key=?2",
            params![kind, key],
        )?;
        Ok(())
    }
    pub fn progress(
        &self,
        id: &str,
        device: &str,
        position: f64,
        duration: Option<f64>,
        completed: bool,
    ) -> Result<Progress> {
        let mut c = self.0.lock().unwrap();
        let tx = c.transaction()?;
        let clock: String = tx
            .query_row(
                "SELECT value FROM records WHERE kind='meta' AND key='clock'",
                [],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or_else(|| "0".into());
        let p = Progress {
            item_id: id.into(),
            position,
            duration,
            completed,
            revision: clock
                .parse::<u64>()?
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("revision exhausted"))?,
            device_id: device.into(),
            updated_at: now(),
        };
        if !p.validate() {
            bail!("invalid playback progress");
        }
        tx.execute(
            "INSERT OR REPLACE INTO records VALUES('progress',?1,?2)",
            params![id, serde_json::to_string(&p)?],
        )?;
        tx.execute(
            "INSERT OR REPLACE INTO records VALUES('meta','clock',?1)",
            [p.revision.to_string()],
        )?;
        tx.commit()?;
        Ok(p)
    }
    pub fn merge_progress(&self, records: &[Progress]) -> Result<()> {
        let mut c = self.0.lock().unwrap();
        let tx = c.transaction()?;
        let clock: String = tx
            .query_row(
                "SELECT value FROM records WHERE kind='meta' AND key='clock'",
                [],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or_else(|| "0".into());
        let mut clock = clock.parse::<u64>()?;
        for p in records {
            if !p.validate() {
                bail!("peer sent invalid playback progress");
            }
            clock = clock.max(p.revision);
            let old: Option<String> = tx
                .query_row(
                    "SELECT value FROM records WHERE kind='progress' AND key=?1",
                    [&p.item_id],
                    |r| r.get(0),
                )
                .optional()?;
            let replace = match old {
                Some(s) => p.newer_than(&serde_json::from_str(&s)?),
                None => true,
            };
            if replace {
                tx.execute(
                    "INSERT OR REPLACE INTO records VALUES('progress',?1,?2)",
                    params![p.item_id, serde_json::to_string(p)?],
                )?;
            }
        }
        tx.execute(
            "INSERT OR REPLACE INTO records VALUES('meta','clock',?1)",
            [clock.to_string()],
        )?;
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rewind_wins_after_sync_and_old_offline_updates_do_not() {
        let dir = tempfile::tempdir().unwrap();
        let a = Store::open(&dir.path().join("a.sqlite")).unwrap();
        let b = Store::open(&dir.path().join("b.sqlite")).unwrap();
        let id = "a".repeat(64);
        let first = a.progress(&id, "a", 300.0, None, false).unwrap();
        b.merge_progress(&[first.clone()]).unwrap();
        let rewind = b.progress(&id, "b", 120.0, None, false).unwrap();
        a.merge_progress(&[rewind.clone(), first]).unwrap();
        assert_eq!(a.get::<Progress>("progress", &id).unwrap().unwrap(), rewind);
        assert!(a.progress(&id, "a", -1.0, None, false).is_err());
    }
    #[test]
    fn concurrent_edits_converge() {
        let dir = tempfile::tempdir().unwrap();
        let a = Store::open(&dir.path().join("a.sqlite")).unwrap();
        let b = Store::open(&dir.path().join("b.sqlite")).unwrap();
        let id = "b".repeat(64);
        let pa = a.progress(&id, "a", 200.0, None, false).unwrap();
        let pb = b.progress(&id, "b", 50.0, None, false).unwrap();
        a.merge_progress(&[pb.clone()]).unwrap();
        b.merge_progress(&[pa]).unwrap();
        assert_eq!(
            a.all::<Progress>("progress").unwrap(),
            b.all::<Progress>("progress").unwrap()
        );
    }
}
