use crate::{
    Node, import,
    model::*,
    random_secret,
    wire::{self, Request, Response},
};
use anyhow::{Context, Result, bail, ensure};
use std::path::Path;
use std::{
    pin::Pin,
    time::{Duration, Instant},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

pub(crate) type MediaReader = Pin<Box<dyn AsyncRead + Send>>;

/// Inclusive HTTP byte range. Reject multi-ranges and impossible requests.
pub fn byte_range(header: Option<&str>, size: u64) -> Result<(u64, u64, bool)> {
    ensure!(size > 0, "empty media");
    let Some(value) = header else {
        return Ok((0, size, false));
    };
    let range = value.strip_prefix("bytes=").context("invalid range unit")?;
    ensure!(!range.contains(','), "multiple ranges unsupported");
    let (left, right) = range.split_once('-').context("invalid byte range")?;
    if left.is_empty() {
        let suffix = right.parse::<u64>()?;
        ensure!(suffix > 0, "empty suffix range");
        let len = suffix.min(size);
        return Ok((size - len, len, true));
    }
    let start = left.parse::<u64>()?;
    ensure!(start < size, "range starts after end");
    let end = if right.is_empty() {
        size - 1
    } else {
        right.parse::<u64>()?.min(size - 1)
    };
    ensure!(end >= start, "range ends before start");
    Ok((start, end - start + 1, true))
}

impl Node {
    pub fn playback_grant(&self, id: &str) -> Result<String> {
        ensure!(
            self.library()?.items.iter().any(|i| i.media.id == id),
            "unknown media"
        );
        let secret = random_secret();
        let mut grants = self.0.grants.lock().unwrap();
        grants.retain(|_, (_, until)| *until > Instant::now());
        ensure!(grants.len() < 20000, "too many active playback grants");
        // Long audiobook queues can stay locked for days; scoped grants expire after seven days.
        grants.insert(
            secret.clone(),
            (
                id.to_owned(),
                Instant::now() + Duration::from_secs(7 * 86400),
            ),
        );
        Ok(secret)
    }

    pub(crate) fn check_grant(&self, id: &str, grant: &str) -> bool {
        self.0
            .grants
            .lock()
            .unwrap()
            .get(grant)
            .is_some_and(|(item, until)| item == id && *until > Instant::now())
    }

    pub(crate) async fn read_media(
        &self,
        id: &str,
        offset: u64,
        length: u64,
    ) -> Result<MediaReader> {
        ensure!(valid_id(id), "invalid media ID");
        if let Some(local) = self.local_file(id)? {
            ensure!(
                offset < local.media.bytes && length <= local.media.bytes - offset,
                "invalid range"
            );
            let mut file = tokio::fs::File::open(local.path).await?;
            file.seek(std::io::SeekFrom::Start(offset)).await?;
            return Ok(Box::pin(file.take(length)));
        }
        let item = self
            .library()?
            .items
            .into_iter()
            .find(|i| i.media.id == id)
            .context("unknown media")?;
        let mut last = "no paired device currently holds this media".to_owned();
        for source in item.sources.into_iter().filter(|s| !s.local) {
            let Some(peer) = self.0.store.get::<Peer>("peer", &source.device_id)? else {
                continue;
            };
            let result = async {
                let (_, mut recv) = self
                    .request(
                        peer.address,
                        Request::Media {
                            id: id.into(),
                            offset,
                            length,
                        },
                    )
                    .await?;
                match tokio::time::timeout(Duration::from_secs(10), wire::read_json(&mut recv))
                    .await??
                {
                    Response::Media {
                        total,
                        length: received,
                    } if total == item.media.bytes && received == length => Ok(recv),
                    Response::Error { message } => bail!("{message}"),
                    _ => bail!("invalid media response"),
                }
            }
            .await;
            match result {
                Ok(recv) => return Ok(Box::pin(recv.take(length))),
                Err(e) => last = e.to_string(),
            }
        }
        bail!("Cannot reach a copy: {last}")
    }

    pub async fn keep_local(&self, id: &str) -> Result<()> {
        let _guard = self.0.downloads.lock().await;
        if self
            .0
            .store
            .get::<LocalMedia>("local", id)?
            .is_some_and(|m| import::available(&m))
        {
            return Ok(());
        }
        let media = self
            .library()?
            .items
            .into_iter()
            .find(|i| i.media.id == id)
            .context("unknown media")?
            .media;
        let local = self
            .download_verified(&media, &self.0.dir.join("media"))
            .await?;
        self.0.store.put("local", id, &local)
    }

    pub(crate) async fn download_verified(
        &self,
        media: &Media,
        directory: &Path,
    ) -> Result<LocalMedia> {
        let id = &media.id;
        let mut reader = self.read_media(id, 0, media.bytes).await?;
        let temp = tempfile::NamedTempFile::new_in(directory)?;
        let mut dest = tokio::fs::File::from_std(temp.reopen()?);
        let mut hasher = blake3::Hasher::new();
        let mut buffer = vec![0u8; 256 * 1024];
        let mut copied = 0u64;
        loop {
            let count = tokio::time::timeout(Duration::from_secs(60), reader.read(&mut buffer))
                .await
                .context("download stalled")??;
            if count == 0 {
                break;
            }
            dest.write_all(&buffer[..count]).await?;
            hasher.update(&buffer[..count]);
            copied += count as u64;
        }
        ensure!(
            copied == media.bytes && hasher.finalize().to_hex().as_str() == id,
            "download failed integrity verification"
        );
        dest.flush().await?;
        dest.sync_all().await?;
        drop(dest);
        let ext = std::path::Path::new(&media.filename)
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or("media");
        ensure!(
            ext.len() <= 12 && ext.bytes().all(|b| b.is_ascii_alphanumeric()),
            "invalid media extension"
        );
        let path = directory.join(format!("{id}.{ext}"));
        import::persist_managed(temp, &path, id).await?;
        let modified = import::modified(&std::fs::metadata(&path)?);
        Ok(LocalMedia {
            media: media.clone(),
            path,
            modified,
            managed: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ranges_cover_seeking_and_reject_overflows() {
        assert_eq!(byte_range(None, 100).unwrap(), (0, 100, false));
        assert_eq!(
            byte_range(Some("bytes=10-19"), 100).unwrap(),
            (10, 10, true)
        );
        assert_eq!(byte_range(Some("bytes=90-"), 100).unwrap(), (90, 10, true));
        assert_eq!(byte_range(Some("bytes=-20"), 100).unwrap(), (80, 20, true));
        assert_eq!(
            byte_range(Some("bytes=0-999"), 100).unwrap(),
            (0, 100, true)
        );
        for bad in [
            "bytes=100-",
            "bytes=9-1",
            "bytes=-0",
            "bytes=0-1,3-4",
            "bytes=18446744073709551615-",
            "nonsense",
        ] {
            assert!(byte_range(Some(bad), 100).is_err(), "{bad}");
        }
    }
}
