use super::timeline::{Span, duration};
use crate::model::LocalMedia;
use anyhow::{Context, Result, ensure};
use std::{path::Path, time::Duration};

pub async fn probe_duration(path: &Path) -> Result<f64> {
    let mut cmd = tokio::process::Command::new(crate::media_tools::executable("ffprobe"));
    cmd.args([
        "-v",
        "error",
        "-protocol_whitelist",
        "file,pipe",
        "-select_streams",
        "a:0",
        "-show_entries",
        "format=duration:stream=codec_type",
        "-of",
        "json",
    ])
    .arg(path)
    .kill_on_drop(true);
    let output = tokio::time::timeout(Duration::from_secs(60), cmd.output())
        .await
        .context("audio inspection timed out")??;
    ensure!(
        output.status.success(),
        "ffprobe could not inspect this episode"
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    ensure!(
        v["streams"].as_array().is_some_and(|s| !s.is_empty()),
        "episode has no supported audio stream"
    );
    let d = v["format"]["duration"]
        .as_str()
        .and_then(|v| v.parse::<f64>().ok())
        .context("episode duration is unknown")?;
    ensure!(d.is_finite() && d > 0.0, "invalid audio duration");
    Ok(d)
}
pub async fn render(
    source: &LocalMedia,
    spans: &[Span],
    fade_ms: u32,
    work: &Path,
) -> Result<LocalMedia> {
    let output = work.join("cleaned.m4a");
    let mut graph = String::new();
    for (i, s) in spans.iter().enumerate() {
        let length = s.source_end - s.source_start;
        let fade = (fade_ms as f64 / 1000.0).min(length / 2.0);
        graph.push_str(&format!(
            "[0:a:0]atrim=start={:.9}:end={:.9},asetpts=PTS-STARTPTS",
            s.source_start, s.source_end
        ));
        if fade > 0.0 && i > 0 {
            graph.push_str(&format!(",afade=t=in:st=0:d={fade:.9}"));
        }
        if fade > 0.0 && i + 1 < spans.len() {
            graph.push_str(&format!(",afade=t=out:st={:.9}:d={fade:.9}", length - fade));
        }
        graph.push_str(&format!("[p{i}];\n"));
    }
    for i in 0..spans.len() {
        graph.push_str(&format!("[p{i}]"));
    }
    graph.push_str(&format!("concat=n={}:v=0:a=1[out]", spans.len()));
    let graph_path = work.join("edit.ffscript");
    std::fs::write(&graph_path, graph)?;
    // Older FFmpeg uses a dedicated script option; new builds use file-valued options.
    let help = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::process::Command::new(crate::media_tools::executable("ffmpeg"))
            .args(["-hide_banner", "-h", "full"])
            .kill_on_drop(true)
            .output(),
    )
    .await
    .context("FFmpeg capability check timed out")??;
    let graph_option = if String::from_utf8_lossy(&help.stdout).contains("-filter_complex_script ")
    {
        "-filter_complex_script"
    } else {
        "-/filter_complex"
    };
    let log = std::fs::File::create(work.join("render.log"))?;
    let mut cmd = tokio::process::Command::new(crate::media_tools::executable("ffmpeg"));
    cmd.args([
        "-nostdin",
        "-hide_banner",
        "-v",
        "error",
        "-y",
        "-protocol_whitelist",
        "file,pipe",
        "-i",
    ])
    .arg(&source.path)
    .arg(graph_option)
    .arg(&graph_path)
    .args([
        "-map",
        "[out]",
        "-map_metadata",
        "-1",
        "-map_chapters",
        "-1",
        "-c:a",
        "aac",
        "-b:a",
        "128k",
        "-movflags",
        "+faststart",
    ])
    .arg(&output)
    .stdin(std::process::Stdio::null())
    .stdout(std::process::Stdio::null())
    .stderr(log)
    .kill_on_drop(true);
    ensure!(
        tokio::time::timeout(Duration::from_secs(4 * 3600), cmd.status())
            .await
            .context("audio render timed out")??
            .success(),
        "audio render failed; see this job's render.log"
    );
    let measured = probe_duration(&output).await?;
    ensure!(
        (measured - duration(spans)).abs() < 0.2,
        "rendered duration does not match the edit timeline"
    );
    let path = output;
    let title = source.media.title.clone();
    let folder = source.media.folder.clone();
    let mut result = tokio::task::spawn_blocking(move || {
        crate::import::inspect(&path, &format!("{title}.m4a"), &folder, true)
    })
    .await??;
    result.media.title = source.media.title.clone();
    result.media.creator = source.media.creator.clone();
    result.media.album = source.media.album.clone();
    Ok(result)
}
