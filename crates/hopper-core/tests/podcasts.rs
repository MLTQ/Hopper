use hopper_core::{
    Config, Node,
    podcast::{Command, CutOptions},
};
use serde_json::json;
use std::time::Duration;

fn wav(seconds: u32) -> Vec<u8> {
    let length = 8000 * seconds * 2;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + length).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&8000u32.to_le_bytes());
    bytes.extend_from_slice(&16000u32.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&length.to_le_bytes());
    for i in 0..8000 * seconds {
        let sample = ((i as f32 * std::f32::consts::TAU * 220.0 / 8000.0).sin() * 1000.0) as i16;
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}
async fn node(path: &std::path::Path, name: &str) -> Node {
    Node::start(Config {
        data_dir: path.into(),
        name: name.into(),
        local_only: true,
    })
    .await
    .unwrap()
    .0
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn subscriptions_remote_jobs_cuts_metadata_and_originals() {
    // These tests render real audio, while a deterministic fixture stands in for inference.
    if std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .is_err()
    {
        eprintln!("SKIP: FFmpeg required for podcast rendering test");
        return;
    }
    let python = if std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_ok()
    {
        "python3"
    } else {
        "python"
    };
    let root = tempfile::tempdir().unwrap();
    let desktop = root.path().join("desktop");
    std::fs::create_dir(&desktop).unwrap();
    let script = root.path().join("fixture_agent.py");
    std::fs::write(&script,r#"import argparse,json
p=argparse.ArgumentParser();p.add_argument('--request');p.add_argument('--output');a=p.parse_args()
r=json.load(open(a.request))
assert r['protocol']=='hopper.ad-analysis.v1'
json.dump({'source_hash':r['source_hash'],'duration':r['duration'],'segments':[
 {'start':2,'end':4,'kind':'ad','confidence':0.99,'reason':'Synthetic test interval'},
 {'start':6,'end':8,'kind':'banter','confidence':0.99,'reason':'Synthetic banter interval'}]},open(a.output,'w'))
"#).unwrap();
    std::fs::write(
        desktop.join("agent-command.json"),
        serde_json::to_vec(&vec![
            python.to_string(),
            script.to_string_lossy().into_owned(),
        ])
        .unwrap(),
    )
    .unwrap();
    let source = wav(12);
    let original_hash = blake3::hash(&source).to_hex().to_string();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let feed = format!(
        r#"<rss xmlns:podcast="https://podcastindex.org/namespace/1.0"><channel><title>Fixture show</title><description>A test feed</description><item><title>Episode one</title><guid>stable-1</guid><enclosure url="{base}/audio" type="audio/wav"/><podcast:chapters url="{base}/chapters" type="application/json+chapters"/><podcast:transcript url="{base}/transcript" type="text/vtt"/><podcast:soundbite startTime="4" duration="2">Story</podcast:soundbite></item></channel></rss>"#
    );
    let app=axum::Router::new().route("/feed",axum::routing::get(move||{let feed=feed.clone();async move{feed}}))
        .route("/audio",axum::routing::get(move||{let source=source.clone();async move{source}}))
        .route("/chapters",axum::routing::get(||async{axum::Json(json!({"version":"1.2.0","chapters":[{"startTime":0,"title":"Opening"},{"startTime":2,"title":"Ad"},{"startTime":4,"title":"Story","img":"https://example.com/story.png"}]}))}))
        .route("/transcript",axum::routing::get(||async{"WEBVTT\n\n00:00:01.000 --> 00:00:02.000\nHello\n\n00:00:02.000 --> 00:00:04.000\nAd\n\n00:00:04.000 --> 00:00:06.000\nStory\n"}));
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let host = node(&desktop, "Podcast desktop").await;
    let phone = node(&root.path().join("phone"), "Listener").await;
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
    let show_id = result.show_id.unwrap();
    let lib = phone.library().unwrap();
    assert_eq!(lib.podcasts.shows.len(), 1);
    let episode = lib.podcasts.episodes[0].clone();
    phone
        .podcast_command(
            &owner,
            Command::Refresh {
                show_id: show_id.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(phone.library().unwrap().podcasts.episodes.len(), 1);
    let prepared = phone
        .podcast_command(
            &owner,
            Command::Download {
                episode_id: episode.id.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(prepared.media_id.as_deref(), Some(original_hash.as_str()));
    host.set_progress(&original_hash, 7.0, Some(12.0), false)
        .unwrap();
    let started = phone
        .podcast_command(
            &owner,
            Command::Cut {
                show_id: show_id.clone(),
                episode_ids: vec![episode.id.clone()],
                options: CutOptions::default(),
            },
        )
        .await
        .unwrap();
    let job_id = &started.job_ids[0];
    let job = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let j = host
                .podcast_catalogue()
                .unwrap()
                .jobs
                .into_iter()
                .find(|j| &j.id == job_id)
                .unwrap();
            assert_ne!(
                j.state,
                "failed",
                "{}: {}",
                j.message,
                std::fs::read_to_string(desktop.join("jobs").join(&j.id).join("render.log"))
                    .unwrap_or_default()
            );
            if j.state == "ready" {
                break j;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap();
    assert!((job.removed_seconds - 2.0).abs() < 0.1);
    assert_ne!(job.output_id.as_ref(), Some(&original_hash));
    assert_eq!(
        job.cleaned_chapter_document.as_ref().unwrap()["chapters"][1]["startTime"],
        2.0
    );
    assert_eq!(job.cleaned_transcript.len(), 2);
    assert_eq!(job.cleaned_transcript[1].start, 2.0);
    assert_eq!(job.cleaned_soundbites[0].start, 2.0);
    let lib = host.library().unwrap();
    let clean = lib
        .items
        .iter()
        .find(|i| Some(&i.media.id) == job.output_id.as_ref())
        .unwrap();
    assert_eq!(clean.media.chapters[1].start, 2.0);
    assert_eq!(clean.progress.as_ref().unwrap().position, 5.0);
    let original =
        std::fs::read(desktop.join("media").join(format!("{original_hash}.wav"))).unwrap();
    assert_eq!(blake3::hash(&original).to_hex().to_string(), original_hash);
    phone.refresh().await.unwrap();
    assert!(
        phone
            .library()
            .unwrap()
            .items
            .iter()
            .any(|i| Some(&i.media.id) == job.output_id.as_ref())
    );
    phone
        .podcast_command(&owner, Command::Unsubscribe { show_id })
        .await
        .unwrap();
    assert!(phone.library().unwrap().podcasts.shows.is_empty());
    assert!(
        phone
            .library()
            .unwrap()
            .items
            .iter()
            .any(|i| i.media.id == original_hash)
    );
    host.close().await;
    phone.close().await;
    server.abort();
}
