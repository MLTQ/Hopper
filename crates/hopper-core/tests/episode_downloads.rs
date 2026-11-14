use axum::routing::get;
use hopper_core::{
    Config, Node,
    podcast::{Command, agent::AgentConfig},
};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

async fn start(path: &Path) -> Node {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let Ok((node, _)) = Node::start(Config {
                data_dir: path.into(),
                name: "Home".into(),
                local_only: true,
            })
            .await
            {
                return node;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap()
}

async fn until(mut check: impl FnMut() -> bool) {
    tokio::time::timeout(Duration::from_secs(20), async {
        while !check() {
            tokio::time::sleep(Duration::from_millis(50)).await
        }
    })
    .await
    .unwrap();
}

fn wave() -> Vec<u8> {
    let mut b = Vec::new();
    b.extend(b"RIFF");
    b.extend(1636u32.to_le_bytes());
    b.extend(b"WAVEfmt ");
    b.extend(16u32.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(8000u32.to_le_bytes());
    b.extend(16000u32.to_le_bytes());
    b.extend(2u16.to_le_bytes());
    b.extend(16u16.to_le_bytes());
    b.extend(b"data");
    b.extend(1600u32.to_le_bytes());
    b.resize(1644, 0);
    b
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn newest_download_survives_restart_and_manual_download_runs_during_agent_work() {
    let root = tempfile::tempdir().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let feed = format!(
        r#"<rss><channel><title>Download check</title>
      <item><title>Older</title><guid>old</guid><pubDate>Mon, 01 Jan 2024 00:00:00 +0000</pubDate><enclosure url="{base}/old" type="audio/wav"/></item>
      <item><title>Latest</title><guid>latest</guid><pubDate>Thu, 01 Feb 2024 00:00:00 +0000</pubDate><enclosure url="{base}/latest" type="audio/wav"/></item>
    </channel></rss>"#
    );
    let allow = Arc::new(AtomicBool::new(false));
    let latest_calls = Arc::new(AtomicUsize::new(0));
    let older_calls = Arc::new(AtomicUsize::new(0));
    let (gate, calls) = (allow.clone(), latest_calls.clone());
    let old = older_calls.clone();
    let router = axum::Router::new()
        .route(
            "/feed",
            get(move || {
                let feed = feed.clone();
                async move { feed }
            }),
        )
        .route(
            "/latest",
            get(move || {
                let (gate, calls) = (gate.clone(), calls.clone());
                async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    while !gate.load(Ordering::SeqCst) {
                        tokio::time::sleep(Duration::from_millis(20)).await
                    }
                    wave()
                }
            }),
        )
        .route(
            "/old",
            get(move || {
                let old = old.clone();
                async move {
                    old.fetch_add(1, Ordering::SeqCst);
                    wave()
                }
            }),
        );
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let path = root.path().join("owner");
    let host = start(&path).await;
    let phone = start(&root.path().join("phone")).await;
    phone.pair(&host.invite().await.unwrap()).await.unwrap();
    let owner = host.library().unwrap().device.id;
    let result = phone
        .podcast_command(
            &owner,
            Command::Subscribe {
                url: format!("{base}/feed"),
            },
        )
        .await
        .unwrap();
    let catalogue = host.podcast_catalogue().unwrap();
    let latest = catalogue
        .episodes
        .iter()
        .find(|e| e.title == "Latest")
        .unwrap()
        .id
        .clone();
    let old = catalogue
        .episodes
        .iter()
        .find(|e| e.title == "Older")
        .unwrap()
        .id
        .clone();
    assert_eq!(catalogue.downloads.len(), 1);
    assert_eq!(catalogue.downloads[0].episode_id, latest);
    assert!(catalogue.jobs.is_empty());
    assert_eq!(older_calls.load(Ordering::SeqCst), 0);
    phone
        .podcast_command(
            &owner,
            Command::QueueDownload {
                episode_id: latest.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(host.podcast_catalogue().unwrap().downloads.len(), 1);
    until(|| latest_calls.load(Ordering::SeqCst) > 0).await;
    host.close().await;
    drop(host);
    allow.store(true, Ordering::SeqCst);
    let host = start(&path).await;
    until(|| host.podcast_catalogue().unwrap().downloads[0].state == "ready").await;
    let e = host
        .podcast_catalogue()
        .unwrap()
        .episodes
        .into_iter()
        .find(|e| e.id == latest)
        .unwrap();
    let original = e.original_id.unwrap();
    assert!(
        host.library()
            .unwrap()
            .items
            .iter()
            .any(|i| i.media.id == original)
    );
    // Hold an actual cut job in its external inbox while the separate download worker proceeds.
    host.set_agent_config(AgentConfig {
        mode: "inbox".into(),
        ..Default::default()
    })
    .unwrap();
    let cut = host
        .podcast_command(
            &owner,
            Command::CutEpisode {
                episode_id: latest.clone(),
                options: Default::default(),
            },
        )
        .await
        .unwrap();
    until(|| {
        host.podcast_catalogue()
            .unwrap()
            .jobs
            .iter()
            .any(|j| j.id == cut.job_ids[0] && j.state == "analysing")
    })
    .await;
    host.podcast_command(&owner, Command::QueueDownload { episode_id: old })
        .await
        .unwrap();
    until(|| {
        host.podcast_catalogue()
            .unwrap()
            .downloads
            .iter()
            .all(|d| d.state == "ready")
    })
    .await;
    assert_eq!(older_calls.load(Ordering::SeqCst), 1);
    let counts = latest_calls.load(Ordering::SeqCst);
    host.podcast_command(
        &owner,
        Command::Subscribe {
            url: format!("{base}/feed"),
        },
    )
    .await
    .unwrap();
    assert_eq!(host.podcast_catalogue().unwrap().downloads.len(), 2);
    assert_eq!(latest_calls.load(Ordering::SeqCst), counts);
    assert_eq!(
        host.podcast_catalogue().unwrap().shows[0].id,
        result.show_id.unwrap()
    );
    phone.refresh().await.unwrap();
    assert_eq!(phone.library().unwrap().podcasts.downloads.len(), 2);
    host.podcast_command(
        &owner,
        Command::Cancel {
            job_id: cut.job_ids[0].clone(),
        },
    )
    .await
    .unwrap();
    phone.close().await;
    host.close().await;
    server.abort();
}
