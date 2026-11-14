use axum::{
    Json,
    routing::{get, post},
};
use base64::Engine;
use hopper_core::{
    Config, Node,
    podcast::{Command, CutOptions, Job, agent::AgentConfig},
};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

async fn state(node: &Node, id: &str, expected: &str) -> Job {
    tokio::time::timeout(Duration::from_secs(35), async {
        loop {
            let job = node
                .podcast_catalogue()
                .unwrap()
                .jobs
                .into_iter()
                .find(|j| j.id == id)
                .unwrap();
            if job.state == expected {
                return job;
            }
            assert!(
                !matches!(job.state.as_str(), "ready" | "failed" | "cancelled"),
                "unexpected state: {}: {}",
                job.state,
                job.message
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap()
}
async fn cut(node: &Node, owner: &str, show: &str, episode: &str) -> String {
    node.podcast_command(
        owner,
        Command::Cut {
            show_id: show.into(),
            episode_ids: vec![episode.into()],
            options: CutOptions::default(),
        },
    )
    .await
    .unwrap()
    .job_ids[0]
        .clone()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn builtin_audio_and_external_inbox_preserve_failures_and_credentials() {
    let root = tempfile::tempdir().unwrap();
    let audio = root.path().join("fixture.wav");
    assert!(
        tokio::process::Command::new("ffmpeg")
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=220:duration=12",
                "-y"
            ])
            .arg(&audio)
            .status()
            .await
            .expect("FFmpeg required")
            .success()
    );
    let bytes = std::fs::read(audio).unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let feed = format!(
        "<rss><channel><title>Agent test</title><item><title>Audio fixture</title><guid>one</guid><enclosure url=\"{base}/audio\" type=\"audio/wav\"/></item></channel></rss>"
    );
    let invalid = Arc::new(AtomicBool::new(false));
    let count = Arc::new(AtomicUsize::new(0));
    let (bad, calls) = (invalid.clone(), count.clone());
    let app = axum::Router::new()
        .route("/feed", get(move || {let feed=feed.clone(); async move {feed}}))
        .route("/audio", get(move || {let bytes=bytes.clone(); async move {bytes}}))
        .route("/llm", post(move |headers:axum::http::HeaderMap, Json(body):Json<Value>| {
            let (bad,calls)=(bad.clone(),calls.clone()); async move {
                assert_eq!(headers["authorization"], "Bearer fixture-key");
                assert_eq!(body["model"], "fixture-audio");
                let input=&body["messages"][1]["content"][1]["input_audio"];
                assert_eq!(input["format"], "wav");
                let bytes=base64::engine::general_purpose::STANDARD.decode(input["data"].as_str().unwrap()).unwrap();
                assert!(bytes.starts_with(b"RIFF") && bytes.len()>100_000, "actual audio must be sent");
                calls.fetch_add(1,Ordering::SeqCst);
                Json(json!({"choices":[{"message":{"content":json!({"segments":[{"start":2,"end":if bad.load(Ordering::SeqCst){999}else{4},"kind":"ad","confidence":0.99,"reason":"Synthetic test interval"}]}).to_string()}}]}))
            }
        }));
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let data = root.path().join("node");
    let (node, hello) = Node::start(Config {
        data_dir: data.clone(),
        name: "Agent modes".into(),
        local_only: true,
    })
    .await
    .unwrap();
    let owner = node.library().unwrap().device.id;
    let show = node
        .podcast_command(
            &owner,
            Command::Subscribe {
                url: format!("{base}/feed"),
            },
        )
        .await
        .unwrap()
        .show_id
        .unwrap();
    let episode = node.podcast_catalogue().unwrap().episodes[0].id.clone();
    let id = cut(&node, &owner, &show, &episode).await;
    state(&node, &id, "waiting_for_agent").await;
    let config = AgentConfig {
        endpoint: format!("{base}/llm"),
        model: "fixture-audio".into(),
        api_key: Some("fixture-key".into()),
        ..Default::default()
    };
    let status = node.set_agent_config(config.clone()).unwrap();
    assert!(status.configured && status.key_present);
    let client = reqwest::Client::new();
    let redacted = client
        .get(format!(
            "http://127.0.0.1:{}/api/podcasts/agent",
            hello.port
        ))
        .bearer_auth(&hello.token)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(!redacted.contains("fixture-key"));
    let ready = state(&node, &id, "ready").await;
    assert!((ready.removed_seconds - 2.0).abs() < 0.01);
    assert_eq!(count.load(Ordering::SeqCst), 1);
    let mut replacement = config.clone();
    replacement.api_key = None;
    assert!(
        node.set_agent_config(replacement.clone())
            .unwrap()
            .key_present
    );
    replacement.endpoint = format!("{base}/different");
    assert!(!node.set_agent_config(replacement).unwrap().key_present);
    node.set_agent_config(config).unwrap();
    invalid.store(true, Ordering::SeqCst);
    let failed = cut(&node, &owner, &show, &episode).await;
    state(&node, &failed, "failed").await;
    assert_eq!(
        node.podcast_catalogue().unwrap().episodes[0].cleaned_id,
        ready.output_id
    );
    // External clients receive only active inbox jobs, and must return matching source metadata.
    node.set_agent_config(AgentConfig {
        mode: "inbox".into(),
        ..Default::default()
    })
    .unwrap();
    let inbox = cut(&node, &owner, &show, &episode).await;
    state(&node, &inbox, "analysing").await;
    let request_path = data.join("jobs").join(&inbox).join("request.json");
    let request: Value = serde_json::from_slice(&std::fs::read(request_path).unwrap()).unwrap();
    let analysis = json!({"source_hash":request["source_hash"],"duration":request["duration"],"segments":[{"start":1,"end":3,"kind":"banter","confidence":0.99,"reason":"Legacy conversation classification"}]});
    let messages = [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"hopper_pending_jobs","arguments":{}}}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"hopper_get_job","arguments":{"job_id":"../outside"}}}),
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"hopper_submit_analysis","arguments":{"job_id":inbox,"analysis":{"source_hash":"wrong","duration":12,"segments":[]}}}}),
        json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"hopper_listen_clip","arguments":{"job_id":inbox,"start":11.5,"seconds":2}}}),
        json!({"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"hopper_listen_clip","arguments":{"job_id":inbox,"start":0,"seconds":600}}}),
        json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"hopper_submit_analysis","arguments":{"job_id":inbox,"analysis":analysis}}}),
    ];
    let script =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/hopper-agent-mcp.py");
    let mut child = tokio::process::Command::new("python3")
        .arg(script)
        .arg("--data-dir")
        .arg(&data)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use tokio::io::AsyncWriteExt;
    let mut input = child.stdin.take().unwrap();
    for message in messages {
        input
            .write_all(format!("{message}\n").as_bytes())
            .await
            .unwrap();
    }
    drop(input);
    let output = child.wait_with_output().await.unwrap();
    assert!(output.status.success());
    let replies: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert!(
        replies[1]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains(&inbox)
    );
    assert_eq!(replies[2]["result"]["isError"], true);
    assert_eq!(replies[3]["result"]["isError"], true);
    let clip = &replies[4]["result"]["content"][1];
    assert_eq!(clip["type"], "audio");
    assert_eq!(clip["mimeType"], "audio/wav");
    let wave = base64::engine::general_purpose::STANDARD
        .decode(clip["data"].as_str().unwrap())
        .unwrap();
    assert_eq!(&wave[0..4], b"RIFF");
    assert_eq!(
        wave.len(),
        44 + 16000,
        "the final excerpt ends at the source boundary"
    );
    assert_eq!(replies[5]["result"]["isError"], true);
    assert!(replies[6]["result"]["isError"].is_null(), "{}", replies[6]);
    let no_cuts = state(&node, &inbox, "ready").await;
    assert_eq!(no_cuts.removed_seconds, 0.0);
    let cancelled = cut(&node, &owner, &show, &episode).await;
    state(&node, &cancelled, "analysing").await;
    node.podcast_command(
        &owner,
        Command::Cancel {
            job_id: cancelled.clone(),
        },
    )
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_millis(700)).await;
    state(&node, &cancelled, "cancelled").await;
    node.close().await;
    server.abort();
}
