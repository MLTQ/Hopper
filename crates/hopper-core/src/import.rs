#[cfg(not(target_os = "ios"))]
use crate::model::Chapter;
use crate::{
    model::{ImportReport, LocalMedia, Media},
    store::Store,
};
use anyhow::{Context, Result, bail};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub fn format(path: &Path) -> Option<(&'static str, &'static str)> {
    Some(
        match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
            "mp3" => ("audio/mpeg", "audio"),
            "m4a" => ("audio/mp4", "audio"),
            "aac" => ("audio/aac", "audio"),
            "m4b" => ("audio/mp4", "audiobook"),
            "flac" => ("audio/flac", "audio"),
            "ogg" | "oga" | "opus" => ("audio/ogg", "audio"),
            "wav" => ("audio/wav", "audio"),
            "aif" | "aiff" => ("audio/aiff", "audio"),
            "mp4" | "m4v" => ("video/mp4", "video"),
            "mov" => ("video/quicktime", "video"),
            "webm" => ("video/webm", "video"),
            "mkv" => ("video/x-matroska", "video"),
            _ => return None,
        },
    )
}

pub fn modified(meta: &std::fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|t| t.as_nanos().min(u64::MAX as u128) as u64)
        .unwrap_or(0)
}

pub fn available(local: &LocalMedia) -> bool {
    std::fs::metadata(&local.path).is_ok_and(|m| {
        m.is_file() && m.len() == local.media.bytes && modified(&m) == local.modified
    })
}

// A second import of the same bytes can reference the already published blob.
// Replacing it would change its mtime and invalidate other sources for this ID.
pub(crate) async fn persist_managed(
    temp: tempfile::NamedTempFile,
    dest: &Path,
    id: &str,
) -> Result<()> {
    match temp.persist_noclobber(dest) {
        Ok(_) => Ok(()),
        Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
            let dest = dest.to_owned();
            let id = id.to_owned();
            tokio::task::spawn_blocking(move || {
                let mut hasher = blake3::Hasher::new();
                hasher.update_reader(File::open(&dest)?)?;
                if hasher.finalize().to_hex().as_str() != id {
                    error.file.persist(dest)?;
                }
                Ok(())
            })
            .await?
        }
        Err(error) => Err(error.into()),
    }
}

pub fn inspect(path: &Path, filename: &str, folder: &str, managed: bool) -> Result<LocalMedia> {
    let (mime, kind) = format(Path::new(filename)).context("unsupported media extension")?;
    let path = path.canonicalize()?;
    let mut file = File::open(&path)?;
    let before = file.metadata()?;
    if !before.is_file() || before.len() == 0 {
        bail!("media file is empty or not a regular file");
    }
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 256 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let after = file.metadata()?;
    if before.len() != after.len() || modified(&before) != modified(&after) {
        bail!("file changed during import; retry when copying finishes");
    }
    let title = Path::new(filename)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut media = Media {
        id: hasher.finalize().to_hex().to_string(),
        title,
        creator: String::new(),
        album: String::new(),
        kind: kind.into(),
        filename: filename.into(),
        folder: folder.into(),
        mime: mime.into(),
        bytes: before.len(),
        duration: None,
        chapters: vec![],
    };
    read_metadata(&path, &mut media);
    Ok(LocalMedia {
        media,
        path,
        modified: modified(&after),
        managed,
    })
}

#[cfg(not(target_os = "ios"))]
fn read_metadata(path: &Path, media: &mut Media) {
    use serde_json::Value;
    let output = std::process::Command::new(crate::media_tools::executable("ffprobe"))
        .args([
            "-v",
            "quiet",
            "-protocol_whitelist",
            "file,pipe",
            "-show_format",
            "-show_chapters",
            "-print_format",
            "json",
        ])
        .arg(path)
        .output();
    let Ok(output) = output else { return };
    if !output.status.success() {
        return;
    }
    let Ok(v) = serde_json::from_slice::<Value>(&output.stdout) else {
        return;
    };
    let f = &v["format"];
    media.duration = f["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|d| d.is_finite() && *d >= 0.0);
    for (key, dest) in [
        ("title", &mut media.title),
        ("artist", &mut media.creator),
        ("album", &mut media.album),
    ] {
        if let Some(s) = f["tags"][key].as_str().filter(|s| !s.is_empty()) {
            *dest = s.to_owned();
        }
    }
    if let Some(chapters) = v["chapters"].as_array() {
        media.chapters = chapters
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                let start = c["start_time"].as_str()?.parse::<f64>().ok()?;
                if !start.is_finite() || start < 0.0 {
                    return None;
                }
                Some(Chapter {
                    title: c["tags"]["title"]
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("Chapter {}", i + 1)),
                    start,
                    end: c["end_time"]
                        .as_str()
                        .and_then(|v| v.parse::<f64>().ok())
                        .filter(|v| v.is_finite() && *v >= start),
                })
            })
            .collect();
    }
}

#[cfg(target_os = "ios")]
fn read_metadata(_: &Path, _: &mut Media) {}

pub fn import_paths(store: &Store, paths: &[PathBuf]) -> ImportReport {
    let mut report = ImportReport::default();
    for root in paths {
        if !root.exists() {
            report.errors.push(format!("{}: not found", root.display()));
            continue;
        }
        let import_folder = root.is_dir().then(|| {
            root.canonicalize()
                .unwrap_or_else(|_| root.clone())
                .to_string_lossy()
                .into_owned()
        });
        let base = root.parent().unwrap_or(root);
        for entry in walkdir::WalkDir::new(root)
            .follow_links(false)
            .max_depth(32)
        {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    report.errors.push(e.to_string());
                    continue;
                }
            };
            if !entry.file_type().is_file() {
                continue;
            }
            if format(entry.path()).is_none() {
                report.skipped += 1;
                continue;
            }
            let filename = entry.file_name().to_string_lossy();
            let folder = entry
                .path()
                .parent()
                .and_then(|p| p.strip_prefix(base).ok())
                .unwrap_or(Path::new(""))
                .to_string_lossy()
                .replace('\\', "/");
            match inspect(entry.path(), &filename, &folder, false).and_then(|m| {
                store.put_import(import_folder.as_deref(), &m)?;
                Ok(m.media.id)
            }) {
                Ok(id) => {
                    report.added += 1;
                    report.item_ids.push(id);
                }
                Err(e) => report
                    .errors
                    .push(format!("{}: {e}", entry.path().display())),
            }
        }
    }
    report
}
