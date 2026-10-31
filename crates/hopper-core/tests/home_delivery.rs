use hopper_core::{
    Config, Hello, Node,
    home::{AddPodcasts, Settings},
};
use serde_json::{Value, json};
use std::time::Duration;

async fn start(path: &std::path::Path, name: &str) -> (Node, Hello) {
    Node::start(Config {
        data_dir: path.into(),
        name: name.into(),
        local_only: true,
    })
    .await
    .unwrap()
}
async fn delivered(node: &Node, id: &str) {
    tokio::time::timeout(Duration::from_secs(40), async {
        loop {
            let job = node
                .deliveries()
                .unwrap()
                .into_iter()
                .find(|j| j.id == id)
                .unwrap();
            assert_ne!(job.state, "failed", "{}", job.message);
            if job.state == "delivered" {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap();
}
fn input(opml: String) -> AddPodcasts {
    AddPodcasts {
        owner_id: None,
        url: None,
        opml: Some(opml),
        batch_id: Some("castamatic-export".into()),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn laptop_delivers_subscriptions_and_verified_files_to_home_for_another_player() {
    let root = tempfile::tempdir().unwrap();
    let (home, hh) = start(&root.path().join("home"), "Home").await;
    let (laptop, hl) = start(&root.path().join("laptop"), "Laptop").await;
    let (player, _) = start(&root.path().join("player"), "Player").await;
    laptop.pair(&home.invite().await.unwrap()).await.unwrap();
    player.pair(&home.invite().await.unwrap()).await.unwrap();
    laptop
        .set_home(Settings {
            device_id: Some(hh.device.id.clone()),
        })
        .unwrap();
    assert_eq!(laptop.library().unwrap().home.unwrap().id, hh.device.id);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let feed_url = format!("http://{}/feed", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, axum::Router::new().route("/feed", axum::routing::get(|| async { "<rss><channel><title>Imported campaign</title><description>Fixture</description></channel></rss>" }))).await.unwrap();
    });
    let opml = format!(
        "<opml><body><outline text='Folder'><outline xmlUrl='{feed_url}'/><outline xmlUrl='{feed_url}'/></outline></body></opml>"
    );
    let first = laptop.add_podcasts(input(opml.clone())).unwrap();
    let repeated = laptop.add_podcasts(input(opml)).unwrap();
    assert_eq!(first.delivery_ids.len(), 1);
    assert_eq!(first.delivery_ids, repeated.delivery_ids);
    delivered(&laptop, &first.delivery_ids[0]).await;
    assert!(
        laptop.podcast_catalogue().unwrap().shows.is_empty(),
        "Home owns the feed"
    );
    assert_eq!(home.podcast_catalogue().unwrap().shows.len(), 1);
    player.refresh().await.unwrap();
    assert_eq!(
        player.library().unwrap().podcasts.shows[0].owner.id,
        hh.device.id
    );
    // Replay a completed delivery: receipt, not a second subscription.
    laptop.retry_delivery(&first.delivery_ids[0]).unwrap();
    delivered(&laptop, &first.delivery_ids[0]).await;
    assert_eq!(home.podcast_catalogue().unwrap().shows.len(), 1);
    let file = root.path().join("Chapter.wav");
    let bytes: Vec<u8> = (0..600000).map(|i| (i % 251) as u8).collect();
    std::fs::write(&file, &bytes).unwrap();
    let client = reqwest::Client::new();
    let report: Value = client
        .post(format!("http://127.0.0.1:{}/api/import", hl.port))
        .bearer_auth(&hl.token)
        .json(&json!({"paths":[file]}))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(report["added"], 1);
    assert_eq!(report["errors"], json!([]));
    assert_eq!(
        std::fs::read(&file).unwrap(),
        bytes,
        "original is unchanged"
    );
    let id = report["item_ids"][0].as_str().unwrap();
    let job = laptop
        .deliveries()
        .unwrap()
        .into_iter()
        .find(|j| matches!(j.payload, hopper_core::home::Payload::Media { .. }))
        .unwrap();
    // Staged copy survives the source moving, before or during transfer.
    std::fs::rename(&file, root.path().join("Moved.wav")).unwrap();
    delivered(&laptop, &job.id).await;
    laptop.close().await;
    player.refresh().await.unwrap();
    player.keep_local(id).await.unwrap();
    assert!(
        player
            .library()
            .unwrap()
            .items
            .iter()
            .find(|i| i.media.id == id)
            .unwrap()
            .sources
            .iter()
            .any(|s| s.local && s.online)
    );
    assert!(
        home.library()
            .unwrap()
            .items
            .iter()
            .find(|i| i.media.id == id)
            .unwrap()
            .sources
            .iter()
            .any(|s| s.local && s.online)
    );
    home.close().await;
    player.close().await;
    server.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn offline_additions_survive_sender_restart() {
    let root = tempfile::tempdir().unwrap();
    let (home, hh) = start(&root.path().join("home"), "Home").await;
    let (laptop, _) = start(&root.path().join("laptop"), "Laptop").await;
    laptop.pair(&home.invite().await.unwrap()).await.unwrap();
    laptop
        .set_home(Settings {
            device_id: Some(hh.device.id.clone()),
        })
        .unwrap();
    home.close().await;
    let added = laptop
        .add_podcasts(AddPodcasts {
            owner_id: None,
            url: Some("https://example.org/podcast.xml".into()),
            opml: None,
            batch_id: Some("offline".into()),
        })
        .unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(
        laptop
            .deliveries()
            .unwrap()
            .iter()
            .all(|j| j.state != "delivered")
    );
    laptop.close().await;
    drop(laptop);
    let (restarted, _) = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            match Node::start(Config {
                data_dir: root.path().join("laptop"),
                name: "Laptop".into(),
                local_only: true,
            })
            .await
            {
                Ok(node) => break node,
                Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
    })
    .await
    .unwrap();
    assert_eq!(restarted.home_device().unwrap().unwrap().id, hh.device.id);
    let job = restarted
        .deliveries()
        .unwrap()
        .into_iter()
        .find(|j| j.id == added.delivery_ids[0])
        .unwrap();
    assert!(matches!(job.state.as_str(), "waiting" | "sending"));
    restarted.close().await;
}
