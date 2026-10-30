use hopper_core::{Config, Hello, Node, model::Queue};
use serde_json::{Value, json};

async fn start(dir: &std::path::Path, name: &str) -> (Node, Hello) {
    Node::start(Config {
        data_dir: dir.into(),
        name: name.into(),
        local_only: true,
    })
    .await
    .unwrap()
}
fn api(hello: &Hello, path: &str) -> String {
    format!("http://127.0.0.1:{}/api/{path}", hello.port)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_desktops_pair_stream_seek_cache_and_sync_progress() {
    let root = tempfile::tempdir().unwrap();
    let (a, ha) = start(&root.path().join("a"), "Studio Mac").await;
    let (b, hb) = start(&root.path().join("b"), "Linux box").await;
    let file = root.path().join("Chapter one.wav");
    let bytes: Vec<u8> = (0..600000).map(|i| (i % 251) as u8).collect();
    std::fs::write(&file, &bytes).unwrap();
    assert_eq!(a.import(vec![file.clone()]).await.unwrap().added, 1);
    let id = a.library().unwrap().items[0].media.id.clone();
    let invite = a.invite().await.unwrap();
    b.pair(&invite).await.unwrap();
    assert!(
        b.pair(&invite).await.is_err(),
        "invitation must be consumed"
    );
    assert_eq!(b.library().unwrap().items.len(), 1);
    assert!(!b.library().unwrap().items[0].sources[0].local);

    let client = reqwest::Client::new();
    assert_eq!(
        client
            .get(api(&ha, "library"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        client
            .get(api(&hb, "library"))
            .bearer_auth(&hb.token)
            .header("Origin", "https://untrusted.example")
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let grant: Value = client
        .post(api(&hb, &format!("play/{id}")))
        .bearer_auth(&hb.token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let url = format!(
        "http://127.0.0.1:{}/{}",
        hb.port,
        grant["path"].as_str().unwrap()
    );
    let response = client
        .get(&url)
        .header("Range", "bytes=12000-24000")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 206);
    assert_eq!(
        response.headers()["content-range"],
        "bytes 12000-24000/600000"
    );
    assert_eq!(
        response.bytes().await.unwrap().as_ref(),
        &bytes[12000..24001]
    );
    let response = client.head(&url).send().await.unwrap();
    assert_eq!(response.headers()["content-length"], "600000");
    assert!(response.bytes().await.unwrap().is_empty());
    assert_eq!(
        client
            .get(&url)
            .header("Range", "bytes=600000-")
            .send()
            .await
            .unwrap()
            .status(),
        416
    );

    a.set_progress(&id, 300.0, Some(600.0), false).unwrap();
    b.refresh().await.unwrap();
    b.set_progress(&id, 90.0, Some(600.0), false).unwrap();
    a.refresh().await.unwrap();
    assert_eq!(
        a.library().unwrap().items[0]
            .progress
            .as_ref()
            .unwrap()
            .position,
        90.0
    );
    b.set_queue(Queue {
        items: vec![id.clone()],
        current: Some(id.clone()),
    })
    .unwrap();
    b.keep_local(&id).await.unwrap();
    assert!(
        b.library().unwrap().items[0]
            .sources
            .iter()
            .any(|s| s.local && s.online)
    );
    // A second import is another source, not a second logical item.
    assert_eq!(b.library().unwrap().items.len(), 1);
    // The invitation creator must also receive a catalogue and stream from the joining peer.
    // This uses the same cached QUIC connection with the request direction reversed.
    let second = root.path().join("From the other desktop.mp3");
    let other_bytes = vec![42u8; 4096];
    std::fs::write(&second, &other_bytes).unwrap();
    b.import(vec![second]).await.unwrap();
    a.refresh().await.unwrap();
    let library = a.library().unwrap();
    assert!(library.peers[0].online, "reverse requests must be served");
    let other = library
        .items
        .iter()
        .find(|i| i.media.title == "From the other desktop")
        .unwrap();
    assert!(
        other
            .sources
            .iter()
            .any(|s| !s.local && s.device_id == hb.device.id)
    );
    let reverse_grant = a.playback_grant(&other.media.id).unwrap();
    let reverse_url = format!(
        "http://127.0.0.1:{}/media/{}?grant={reverse_grant}",
        ha.port, other.media.id
    );
    assert_eq!(
        client
            .get(reverse_url)
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap()
            .as_ref(),
        other_bytes
    );
    a.unpair(&hb.device.id).await.unwrap();
    b.refresh().await.unwrap();
    assert!(!b.library().unwrap().peers[0].online);
    assert_eq!(
        client
            .get(&url)
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap()
            .as_ref(),
        bytes
    );
    assert_eq!(
        std::fs::read(&file).unwrap(),
        bytes,
        "source remains unchanged"
    );

    // Even a known media ID gives an unpaired device no streaming capability.
    let (c, _) = start(&root.path().join("c"), "Unpaired").await;
    assert!(c.playback_grant(&id).is_err());
    // API validates invalid progress and queue state.
    let response = client
        .post(api(&hb, "progress"))
        .bearer_auth(&hb.token)
        .json(&json!({"item_id":id,"position":-1}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 400);
    a.close().await;
    b.close().await;
    c.close().await;
}
