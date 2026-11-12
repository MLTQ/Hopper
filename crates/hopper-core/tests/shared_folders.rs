use hopper_core::{Config, Hello, Node};
use serde_json::Value;
use std::path::Path;

async fn start(root: &Path, name: &str) -> (Node, Hello) {
    Node::start(Config {
        data_dir: root.join(name),
        name: name.into(),
        local_only: true,
    })
    .await
    .unwrap()
}

async fn qr_invitation(client: &reqwest::Client, host: &Hello) -> String {
    let url = format!("http://127.0.0.1:{}/api/invite/qr", host.port);
    assert_eq!(client.post(&url).send().await.unwrap().status(), 401);
    let qr: Value = client
        .post(url)
        .bearer_auth(&host.token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        qr["image"]
            .as_str()
            .unwrap()
            .starts_with("data:image/svg+xml;base64,")
    );
    assert_eq!(qr["expires_in"], 600);
    qr["code"].as_str().unwrap().to_string()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_desktops_share_scoped_folders_and_stream_to_one_player() {
    let tmp = tempfile::tempdir().unwrap();
    let (desktop, desktop_hello) = start(tmp.path(), "Studio").await;
    let (laptop, laptop_hello) = start(tmp.path(), "Laptop").await;
    let (player, player_hello) = start(tmp.path(), "Player").await;
    let archive = tmp.path().join("Old campaign");
    let season = archive.join("Season one");
    let tapes = tmp.path().join("Other show");
    std::fs::create_dir_all(&season).unwrap();
    std::fs::create_dir(&tapes).unwrap();
    let content = vec![42u8; 48000];
    let source = season.join("01.wav");
    std::fs::write(&source, &content).unwrap();
    std::fs::write(tapes.join("02.wav"), b"second desktop recording").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&tapes, archive.join("Outside")).unwrap();
    let folder = desktop.share_folder(archive.clone()).await.unwrap();
    assert_eq!(folder.files, 1);
    assert_eq!(
        desktop.share_folder(archive.clone()).await.unwrap().id,
        folder.id
    );
    assert_eq!(desktop.shared_folders().len(), 1);
    laptop.share_folder(tapes).await.unwrap();
    let http = reqwest::Client::new();
    let code = qr_invitation(&http, &desktop_hello).await;
    player.pair(&code).await.unwrap();
    assert!(player.pair(&code).await.is_err());
    player
        .pair(&qr_invitation(&http, &laptop_hello).await)
        .await
        .unwrap();
    let library = player.library().unwrap();
    assert_eq!(library.peers.len(), 2);
    assert_eq!(library.items.len(), 2);
    let item = library
        .items
        .iter()
        .find(|i| i.media.folder == "Old campaign/Season one")
        .unwrap();
    assert_eq!(item.sources[0].device_id, desktop_hello.device.id);
    let grant = player.playback_grant(&item.media.id).unwrap();
    let url = format!(
        "http://127.0.0.1:{}/media/{}?grant={}",
        player_hello.port, item.media.id, grant
    );
    let response = http
        .get(&url)
        .header("Range", "bytes=10-99")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 206);
    assert_eq!(response.bytes().await.unwrap().as_ref(), &content[10..100]);
    let snapshot = serde_json::to_string(&desktop.snapshot().unwrap()).unwrap();
    assert!(!snapshot.contains(tmp.path().to_str().unwrap()));
    std::fs::write(archive.join("03.wav"), b"newly added recording").unwrap();
    assert_eq!(
        desktop
            .rescan_folder(folder.id.clone())
            .await
            .unwrap()
            .files,
        2
    );
    player.refresh().await.unwrap();
    assert_eq!(player.library().unwrap().items.len(), 3);
    // A separate local import of the same bytes still belongs to the library after unsharing.
    laptop.import(vec![source.clone()]).await.unwrap();
    desktop.stop_sharing_folder(folder.id).await.unwrap();
    assert!(desktop.snapshot().unwrap().items.is_empty());
    assert_eq!(std::fs::read(source).unwrap(), content);
    player.refresh().await.unwrap();
    assert_eq!(player.library().unwrap().items.len(), 2);
    let response = http
        .get(&url)
        .header("Range", "bytes=4-20")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 206);
    assert_eq!(response.bytes().await.unwrap().as_ref(), &content[4..21]);
    player.close().await;
    desktop.close().await;
    laptop.close().await;
}

#[tokio::test]
async fn share_survives_restart_and_offline_folder_returns() {
    let tmp = tempfile::tempdir().unwrap();
    let archive = tmp.path().join("Archive");
    let unplugged = tmp.path().join("Unavailable");
    std::fs::create_dir(&archive).unwrap();
    std::fs::write(archive.join("01.wav"), b"one recording").unwrap();
    let (desktop, _) = start(tmp.path(), "Host").await;
    let folder = desktop.share_folder(archive.clone()).await.unwrap();
    desktop.close().await;
    drop(desktop);

    // Wait for the loopback/peer workers to release the existing library lock.
    let (desktop, _) = tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            let config = Config {
                data_dir: tmp.path().join("Host"),
                name: "Host".into(),
                local_only: true,
            };
            if let Ok(node) = Node::start(config).await {
                break node;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(desktop.shared_folders()[0].id, folder.id);
    assert_eq!(desktop.snapshot().unwrap().items.len(), 1);
    std::fs::rename(&archive, &unplugged).unwrap();
    let offline = desktop.rescan_folder(folder.id.clone()).await.unwrap();
    assert_eq!(offline.files, 0);
    assert!(offline.warning.is_some());
    assert!(desktop.snapshot().unwrap().items.is_empty());
    std::fs::rename(&unplugged, &archive).unwrap();
    let online = desktop.rescan_folder(folder.id).await.unwrap();
    assert_eq!(online.files, 1);
    assert!(online.warning.is_none());
    desktop.close().await;
}
