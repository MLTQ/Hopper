use anyhow::{Context, Result, ensure};
use futures_util::StreamExt;
use reqwest::{Client, Url};
use std::time::Duration;
use tokio::io::AsyncWriteExt;

pub fn url(raw: &str) -> Result<Url> {
    let u = Url::parse(raw.trim()).context("enter a complete podcast feed URL")?;
    ensure!(
        matches!(u.scheme(), "http" | "https")
            && u.host_str().is_some()
            && u.username().is_empty()
            && u.password().is_none(),
        "use an HTTP or HTTPS URL without embedded login credentials"
    );
    Ok(u)
}
fn client(timeout: u64) -> Result<Client> {
    Ok(Client::builder()
        .user_agent("Hopper/0.1 podcast reader")
        .redirect(reqwest::redirect::Policy::limited(5))
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(timeout))
        .build()?)
}
pub async fn document(raw: &str) -> Result<(String, Vec<u8>)> {
    let mut r = client(45)?
        .get(url(raw)?)
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("could not reach the podcast server"))?;
    ensure!(
        r.status().is_success(),
        "podcast server returned HTTP {}",
        r.status().as_u16()
    );
    let final_url = r.url().to_string();
    let mut bytes = vec![];
    while let Some(chunk) = r
        .chunk()
        .await
        .map_err(|_| anyhow::anyhow!("podcast document transfer interrupted"))?
    {
        ensure!(
            bytes.len() + chunk.len() <= 8 * 1024 * 1024,
            "podcast document exceeds 8 MB"
        );
        bytes.extend_from_slice(&chunk);
    }
    Ok((final_url, bytes))
}
pub async fn download(raw: &str, path: &std::path::Path) -> Result<()> {
    let r = client(3600)?
        .get(url(raw)?)
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("could not reach the episode server"))?;
    ensure!(
        r.status().is_success(),
        "episode server returned HTTP {}",
        r.status().as_u16()
    );
    ensure!(
        r.content_length()
            .is_none_or(|s| s <= 4 * 1024 * 1024 * 1024),
        "episode exceeds 4 GB"
    );
    let mut stream = r.bytes_stream();
    let mut file = tokio::fs::File::create(path).await?;
    let mut size = 0u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| anyhow::anyhow!("episode download interrupted"))?;
        size += chunk.len() as u64;
        ensure!(size <= 4 * 1024 * 1024 * 1024, "episode exceeds 4 GB");
        file.write_all(&chunk).await?;
    }
    ensure!(size > 0, "episode was empty");
    file.flush().await?;
    file.sync_all().await?;
    Ok(())
}
