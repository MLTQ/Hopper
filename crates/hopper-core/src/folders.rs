use crate::{Node, import, model::LocalMedia, store::Store};
use anyhow::{Context, Result, ensure};
use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock, RwLock},
    time::Duration,
};

#[derive(Clone, Serialize, Deserialize)]
struct Folder {
    id: String,
    path: PathBuf,
    files: BTreeMap<String, LocalMedia>,
    warning: Option<String>,
}

#[derive(Serialize)]
pub struct FolderInfo {
    pub id: String,
    pub path: PathBuf,
    pub files: usize,
    pub warning: Option<String>,
}

impl Folder {
    fn info(&self) -> FolderInfo {
        FolderInfo {
            id: self.id.clone(),
            path: self.path.clone(),
            files: self.files.len(),
            warning: self.warning.clone(),
        }
    }
}

pub type FolderPicker = Box<dyn Fn() -> BoxFuture<'static, Option<PathBuf>> + Send + Sync>;

#[derive(Default)]
pub(crate) struct SharedFolders {
    roots: RwLock<BTreeMap<String, Folder>>,
    editing: Mutex<()>,
    pub picker: OnceLock<FolderPicker>,
}

impl SharedFolders {
    pub fn open(store: &Store) -> Result<Self> {
        let roots = store
            .all::<Folder>("shared_folder")?
            .into_iter()
            .map(|f| (f.id.clone(), f))
            .collect();
        Ok(Self {
            roots: RwLock::new(roots),
            ..Default::default()
        })
    }
}

// Check the stored canonical path at serving time too: an ancestor may since have
// been replaced by a symlink or junction leading outside the designated folder.
fn accessible(file: &LocalMedia) -> bool {
    import::available(file)
        && file
            .path
            .canonicalize()
            .is_ok_and(|current| current == file.path)
}

impl Node {
    pub fn set_folder_picker(&self, picker: FolderPicker) {
        let _ = self.0.folders.picker.set(picker);
    }

    pub fn shared_folders(&self) -> Vec<FolderInfo> {
        self.0
            .folders
            .roots
            .read()
            .unwrap()
            .values()
            .map(Folder::info)
            .collect()
    }

    pub async fn share_folder(&self, path: PathBuf) -> Result<FolderInfo> {
        let node = self.clone();
        tokio::task::spawn_blocking(move || {
            ensure!(!cfg!(target_os = "ios"), "Choose a folder on a desktop");
            let _lock = node.0.folders.editing.lock().unwrap();
            ensure!(path.is_absolute(), "Choose an absolute folder path");
            let path = path.canonicalize().context("Folder could not be opened")?;
            ensure!(path.is_dir(), "Choose a folder");
            let id = blake3::hash(path.as_os_str().as_encoded_bytes())
                .to_hex()
                .to_string();
            let old = node.0.folders.roots.read().unwrap().get(&id).cloned();
            let folder = old.unwrap_or(Folder {
                id,
                path,
                files: BTreeMap::new(),
                warning: None,
            });
            node.scan_folder(folder)
        })
        .await?
    }

    pub async fn rescan_folder(&self, id: String) -> Result<FolderInfo> {
        let node = self.clone();
        tokio::task::spawn_blocking(move || {
            let _lock = node.0.folders.editing.lock().unwrap();
            let folder = node
                .0
                .folders
                .roots
                .read()
                .unwrap()
                .get(&id)
                .cloned()
                .context("Folder is no longer shared")?;
            node.scan_folder(folder)
        })
        .await?
    }

    fn scan_folder(&self, mut folder: Folder) -> Result<FolderInfo> {
        let private = self.0.dir.canonicalize()?;
        ensure!(
            !folder.path.starts_with(&private),
            "Choose a folder outside Hopper’s app data"
        );
        let previous: HashMap<_, _> = folder
            .files
            .values()
            .map(|f| (f.path.as_path(), f))
            .collect();
        let mut files = BTreeMap::new();
        let mut skipped = 0;
        let root = &folder.path;
        let base = root.parent().unwrap_or(root);
        // Never walk through a replaced or unmounted root. Keep its registration for reconnect.
        if root.canonicalize().is_ok_and(|p| p == *root && p.is_dir()) {
            for entry in walkdir::WalkDir::new(root)
                .follow_root_links(false)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| !e.path().starts_with(&private))
            {
                let Ok(entry) = entry else {
                    skipped += 1;
                    continue;
                };
                if !entry.file_type().is_file() || import::format(entry.path()).is_none() {
                    continue;
                }
                let path = entry.path();
                if !path.canonicalize().is_ok_and(|p| p == path) {
                    skipped += 1;
                    continue;
                }
                let local = match previous.get(path).filter(|f| import::available(f)) {
                    Some(file) => (**file).clone(),
                    None => {
                        let parent = path
                            .parent()
                            .and_then(|p| p.strip_prefix(base).ok())
                            .unwrap_or(Path::new(""));
                        let logical = parent
                            .components()
                            .map(|c| c.as_os_str().to_string_lossy())
                            .collect::<Vec<_>>()
                            .join("/");
                        match import::inspect(
                            path,
                            &entry.file_name().to_string_lossy(),
                            &logical,
                            false,
                        ) {
                            Ok(file) if file.path == path => file,
                            _ => {
                                skipped += 1;
                                continue;
                            }
                        }
                    }
                };
                files.entry(local.media.id.clone()).or_insert(local);
            }
            folder.warning = (skipped > 0).then(|| format!("Could not read {skipped} entries"));
        } else {
            folder.warning = Some("Folder is unavailable".into());
        }
        folder.files = files;
        self.0.store.put("shared_folder", &folder.id, &folder)?;
        let info = folder.info();
        self.0
            .folders
            .roots
            .write()
            .unwrap()
            .insert(folder.id.clone(), folder);
        Ok(info)
    }

    pub async fn stop_sharing_folder(&self, id: String) -> Result<()> {
        let node = self.clone();
        tokio::task::spawn_blocking(move || {
            let _lock = node.0.folders.editing.lock().unwrap();
            node.0.store.remove("shared_folder", &id)?;
            node.0.folders.roots.write().unwrap().remove(&id);
            Ok(())
        })
        .await?
    }

    pub(crate) fn local_file(&self, id: &str) -> Result<Option<LocalMedia>> {
        if let Some(file) = self
            .0
            .store
            .get::<LocalMedia>("local", id)?
            .filter(import::available)
        {
            return Ok(Some(file));
        }
        if let Some((_, file)) = self.0.store.imported_file(id)? {
            return Ok(Some(file));
        }
        let shared = self
            .0
            .folders
            .roots
            .read()
            .unwrap()
            .values()
            .filter_map(|folder| folder.files.get(id))
            .find(|file| accessible(file))
            .cloned();
        if shared.is_some() {
            return Ok(shared);
        }
        self.playback_copy(id)
    }

    pub(crate) fn local_files(&self) -> Result<Vec<LocalMedia>> {
        let mut files: BTreeMap<_, _> = self
            .0
            .store
            .all::<LocalMedia>("local")?
            .into_iter()
            .map(|f| (f.media.id.clone(), f))
            .collect();
        for file in self.0.store.imported_files()? {
            if !files.get(&file.media.id).is_some_and(import::available) {
                files.insert(file.media.id.clone(), file);
            }
        }
        for root in self.0.folders.roots.read().unwrap().values() {
            for (id, file) in &root.files {
                if let Some(existing) = files.get_mut(id)
                    && import::available(existing)
                {
                    if existing.media.folder.is_empty() && accessible(file) {
                        existing.media.folder.clone_from(&file.media.folder);
                    }
                } else if accessible(file) {
                    files.insert(id.clone(), file.clone());
                }
            }
        }
        for file in self.playback_copies()? {
            if !files.get(&file.media.id).is_some_and(import::available) {
                files.insert(file.media.id.clone(), file);
            }
        }
        Ok(files.into_values().collect())
    }

    pub(crate) fn start_folder_scans(&self) {
        if cfg!(target_os = "ios") {
            return;
        }
        let weak = std::sync::Arc::downgrade(&self.0);
        let shutdown = self.0.shutdown.clone();
        tokio::spawn(async move {
            loop {
                let Some(inner) = weak.upgrade() else { return };
                let node = Node(inner);
                for folder in node.shared_folders() {
                    if shutdown.is_cancelled() {
                        return;
                    }
                    let _ = node.rescan_folder(folder.id).await;
                }
                drop(node);
                tokio::select! { _ = shutdown.cancelled() => return, _ = tokio::time::sleep(Duration::from_secs(60)) => {} }
            }
        });
    }
}
