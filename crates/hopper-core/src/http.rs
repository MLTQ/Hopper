use crate::{Node, import, media::byte_range, model::*};
use anyhow::Result;
use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, Query, Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

struct ApiError(anyhow::Error);
impl<E: Into<anyhow::Error>> From<E> for ApiError {
    fn from(e: E) -> Self {
        Self(e.into())
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error":self.0.to_string()})),
        )
            .into_response()
    }
}
type Api<T> = std::result::Result<Json<T>, ApiError>;

pub(crate) async fn serve(node: Node) -> Result<u16> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let shutdown = node.0.shutdown.clone();
    let api = Router::new()
        .route("/library", get(library))
        .route("/refresh", post(refresh))
        .route("/invite", post(invite))
        .route("/invite/qr", post(invite_qr))
        .route("/folders", get(folders).post(share_folder))
        .route("/folders/choose", post(choose_folder))
        .route("/folders/:id", delete(stop_sharing))
        .route("/folders/:id/rescan", post(rescan_folder))
        .route("/imported-folders/:id", delete(remove_imported_folder))
        .route("/pair", post(pair))
        .route("/peers/:id", delete(unpair))
        .route("/import", post(import_paths))
        .route("/upload", post(upload))
        .route("/progress", post(progress))
        .route("/queue", post(queue))
        .route("/play/:id", post(play))
        .route("/keep/:id", post(keep))
        .route("/podcasts/command", post(podcast_command))
        .route("/home", post(set_home))
        .route("/add-podcasts", post(add_podcasts))
        .route("/deliveries/:id/retry", post(retry_delivery))
        .route("/podcasts/agent", get(agent_status).post(agent_config))
        .layer(middleware::from_fn_with_state(node.clone(), authenticate));
    let app = Router::new()
        .nest("/api", api)
        .route("/media/:id", get(media))
        .route(
            "/",
            get(|| async {
                asset(
                    "text/html; charset=utf-8",
                    include_str!("../../../ui/index.html"),
                )
            }),
        )
        .route(
            "/app.js",
            get(|| async {
                asset(
                    "text/javascript; charset=utf-8",
                    include_str!("../../../ui/app.js"),
                )
            }),
        )
        .route(
            "/api.js",
            get(|| async {
                asset(
                    "text/javascript; charset=utf-8",
                    include_str!("../../../ui/api.js"),
                )
            }),
        )
        .route(
            "/pairing.js",
            get(|| async {
                asset(
                    "text/javascript; charset=utf-8",
                    include_str!("../../../ui/pairing.js"),
                )
            }),
        )
        .route(
            "/folders.js",
            get(|| async {
                asset(
                    "text/javascript; charset=utf-8",
                    include_str!("../../../ui/folders.js"),
                )
            }),
        )
        .route(
            "/player.js",
            get(|| async {
                asset(
                    "text/javascript; charset=utf-8",
                    include_str!("../../../ui/player.js"),
                )
            }),
        )
        .route(
            "/styles.css",
            get(|| async {
                asset(
                    "text/css; charset=utf-8",
                    include_str!("../../../ui/styles.css"),
                )
            }),
        )
        .route(
            "/material.css",
            get(|| async {
                asset(
                    "text/css; charset=utf-8",
                    include_str!("../../../ui/material.css"),
                )
            }),
        )
        .route(
            "/glass.js",
            get(|| async {
                asset(
                    "text/javascript; charset=utf-8",
                    include_str!("../../../ui/glass.js"),
                )
            }),
        )
        .route(
            "/glass-lens.js",
            get(|| async {
                asset(
                    "text/javascript; charset=utf-8",
                    include_str!("../../../ui/glass-lens.js"),
                )
            }),
        )
        .route(
            "/agents.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript")],
                    include_str!("../../../ui/agents.js"),
                )
            }),
        )
        .route(
            "/home.js",
            get(|| async {
                asset(
                    "text/javascript; charset=utf-8",
                    include_str!("../../../ui/home.js"),
                )
            }),
        )
        .route(
            "/podcasts.js",
            get(|| async {
                asset(
                    "text/javascript; charset=utf-8",
                    include_str!("../../../ui/podcasts.js"),
                )
            }),
        )
        .layer(DefaultBodyLimit::disable())
        .with_state(node);
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown.cancelled_owned())
            .await
        {
            eprintln!("Hopper HTTP stopped: {e}");
        }
    });
    Ok(port)
}

fn asset(mime: &'static str, body: &'static str) -> Response {
    ([(header::CONTENT_TYPE,mime),(header::CACHE_CONTROL,"no-store"),(header::REFERRER_POLICY,"no-referrer"),
      (header::CONTENT_SECURITY_POLICY,"default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; media-src 'self' blob:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'" )],body).into_response()
}

async fn authenticate(State(node): State<Node>, request: Request, next: Next) -> Response {
    let correct = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .is_some_and(|h| h.strip_prefix("Bearer ") == Some(&node.0.token));
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let origin_ok = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|h| h.to_str().ok())
        .is_none_or(|o| o == format!("http://{host}"));
    if !correct || !origin_ok || !host.starts_with("127.0.0.1:") {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error":"Hopper authorization required"})),
        )
            .into_response();
    }
    next.run(request).await
}

async fn library(State(node): State<Node>) -> Api<Library> {
    Ok(Json(node.library()?))
}
#[derive(Deserialize)]
struct PodcastInput {
    owner_id: String,
    #[serde(flatten)]
    command: crate::podcast::Command,
}
async fn podcast_command(
    State(node): State<Node>,
    Json(input): Json<PodcastInput>,
) -> Api<crate::podcast::CommandResult> {
    Ok(Json(
        node.podcast_command(&input.owner_id, input.command).await?,
    ))
}
async fn agent_status(State(node): State<Node>) -> Api<crate::podcast::agent::AgentStatus> {
    Ok(Json(node.agent_status()?))
}
async fn agent_config(
    State(node): State<Node>,
    Json(config): Json<crate::podcast::agent::AgentConfig>,
) -> Api<crate::podcast::agent::AgentStatus> {
    Ok(Json(node.set_agent_config(config)?))
}
async fn set_home(
    State(node): State<Node>,
    Json(settings): Json<crate::home::Settings>,
) -> Api<Library> {
    node.set_home(settings)?;
    Ok(Json(node.library()?))
}
async fn add_podcasts(
    State(node): State<Node>,
    Json(input): Json<crate::home::AddPodcasts>,
) -> Api<crate::home::Added> {
    Ok(Json(node.add_podcasts(input)?))
}
async fn retry_delivery(State(node): State<Node>, Path(id): Path<String>) -> Api<Value> {
    node.retry_delivery(&id)?;
    Ok(Json(json!({"ok":true})))
}
async fn refresh(State(node): State<Node>) -> Api<Library> {
    node.refresh().await?;
    Ok(Json(node.library()?))
}
async fn folders(State(node): State<Node>) -> Api<Value> {
    Ok(Json(
        json!({"folders": node.shared_folders(), "imports": node.0.store.imported_folders()?, "can_choose": node.0.folders.picker.get().is_some()}),
    ))
}
#[derive(Deserialize)]
struct FolderInput {
    path: PathBuf,
}
async fn share_folder(
    State(node): State<Node>,
    Json(input): Json<FolderInput>,
) -> Api<crate::folders::FolderInfo> {
    Ok(Json(node.share_folder(input.path).await?))
}
async fn choose_folder(State(node): State<Node>) -> Api<Option<crate::folders::FolderInfo>> {
    let picker = node
        .0
        .folders
        .picker
        .get()
        .ok_or_else(|| anyhow::anyhow!("Enter the folder path on this desktop"))?;
    let result = match picker().await {
        Some(path) => Some(node.share_folder(path).await?),
        None => None,
    };
    Ok(Json(result))
}
async fn rescan_folder(
    State(node): State<Node>,
    Path(id): Path<String>,
) -> Api<crate::folders::FolderInfo> {
    Ok(Json(node.rescan_folder(id).await?))
}
async fn stop_sharing(State(node): State<Node>, Path(id): Path<String>) -> Api<Value> {
    node.stop_sharing_folder(id).await?;
    Ok(Json(json!({"ok":true})))
}

async fn remove_imported_folder(State(node): State<Node>, Path(id): Path<String>) -> Api<Value> {
    let _guard = node.0.imports.lock().await;
    node.0.store.remove_imported_folder(&id)?;
    Ok(Json(json!({"ok":true})))
}

async fn invite_qr(State(node): State<Node>) -> Api<Value> {
    use base64::Engine;
    let code = node.invite().await?;
    let qr = qrcode::QrCode::new(code.as_bytes())?;
    let svg = qr
        .render::<qrcode::render::svg::Color>()
        .quiet_zone(true)
        .module_dimensions(6, 6)
        .build();
    let image = base64::engine::general_purpose::STANDARD.encode(svg.as_bytes());
    Ok(Json(
        json!({"code":code, "image":format!("data:image/svg+xml;base64,{image}"), "expires_in":600}),
    ))
}

async fn invite(State(node): State<Node>) -> Api<Value> {
    Ok(Json(json!({"code":node.invite().await?,"expires_in":600})))
}
#[derive(Deserialize)]
struct PairInput {
    code: String,
}
async fn pair(State(node): State<Node>, Json(input): Json<PairInput>) -> Api<Library> {
    node.pair(&input.code).await?;
    Ok(Json(node.library()?))
}
async fn unpair(State(node): State<Node>, Path(id): Path<String>) -> Api<Value> {
    node.unpair(&id).await?;
    Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]
struct ImportInput {
    paths: Vec<PathBuf>,
}
async fn import_paths(
    State(node): State<Node>,
    Json(input): Json<ImportInput>,
) -> Api<ImportReport> {
    if input.paths.len() > 1000 {
        return Err(anyhow::anyhow!("too many import paths").into());
    }
    let report = node.import(input.paths).await?;
    Ok(Json(report))
}
#[derive(Deserialize)]
struct ProgressInput {
    item_id: String,
    position: f64,
    duration: Option<f64>,
    #[serde(default)]
    completed: bool,
}
async fn progress(State(node): State<Node>, Json(input): Json<ProgressInput>) -> Api<Progress> {
    Ok(Json(node.set_progress(
        &input.item_id,
        input.position,
        input.duration,
        input.completed,
    )?))
}
async fn queue(State(node): State<Node>, Json(input): Json<Queue>) -> Api<Value> {
    node.set_queue(input)?;
    Ok(Json(json!({"ok":true})))
}
async fn play(State(node): State<Node>, Path(id): Path<String>) -> Api<Value> {
    let grant = node.playback_grant(&id)?;
    Ok(Json(json!({"path":format!("media/{id}?grant={grant}")})))
}
async fn keep(State(node): State<Node>, Path(id): Path<String>) -> Api<Value> {
    node.keep_local(&id).await?;
    Ok(Json(json!({"ok":true})))
}

async fn upload(State(node): State<Node>, mut multipart: Multipart) -> Api<ImportReport> {
    let _guard = node.0.imports.lock().await;
    let mut report = ImportReport::default();
    while let Some(mut field) = multipart.next_field().await? {
        let filename = field.file_name().unwrap_or("media").to_owned();
        let clean = filename.replace('\\', "/");
        let (folder, name) = clean.rsplit_once('/').unwrap_or(("", &clean));
        let name = name.to_owned();
        let folder = folder.to_owned();
        if import::format(std::path::Path::new(&name)).is_none() {
            report.skipped += 1;
            continue;
        }
        let temp = tempfile::NamedTempFile::new_in(node.0.dir.join("media"))?;
        let mut file = tokio::fs::File::from_std(temp.reopen()?);
        while let Some(chunk) = field.chunk().await? {
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        file.sync_all().await?;
        drop(file);
        let path = temp.path().to_owned();
        let mut local =
            tokio::task::spawn_blocking(move || import::inspect(&path, &name, &folder, true))
                .await??;
        let ext = std::path::Path::new(&local.media.filename)
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or("media");
        let dest = node
            .0
            .dir
            .join("media")
            .join(format!("{}.{}", local.media.id, ext));
        import::persist_managed(temp, &dest, &local.media.id).await?;
        local.path = dest;
        local.modified = import::modified(&std::fs::metadata(&local.path)?);
        let root = local
            .media
            .folder
            .split('/')
            .next()
            .filter(|s| !s.is_empty());
        node.0.store.put_import(root, &local)?;
        report.item_ids.push(local.media.id);
        report.added += 1;
    }
    if let Err(error) = node.send_imports_home(&report.item_ids).await {
        report.errors.push(error.to_string());
    }
    Ok(Json(report))
}

#[derive(Deserialize)]
struct Grant {
    grant: String,
}
async fn media(
    State(node): State<Node>,
    Path(id): Path<String>,
    Query(q): Query<Grant>,
    headers: HeaderMap,
    method: axum::http::Method,
) -> Response {
    if !node.check_grant(&id, &q.grant) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let item = match node
        .library()
        .ok()
        .and_then(|l| l.items.into_iter().find(|i| i.media.id == id))
    {
        Some(item) => item.media,
        None => return StatusCode::NOT_FOUND.into_response(),
    };
    let range = headers.get(header::RANGE).and_then(|r| r.to_str().ok());
    let (offset, length, partial) = match byte_range(range, item.bytes) {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::RANGE_NOT_SATISFIABLE,
                [(header::CONTENT_RANGE, format!("bytes */{}", item.bytes))],
            )
                .into_response();
        }
    };
    let body = if method == axum::http::Method::HEAD {
        Body::empty()
    } else {
        match node.read_media(&id, offset, length).await {
            Ok(reader) => Body::from_stream(ReaderStream::new(reader)),
            Err(e) => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({"error":e.to_string()})),
                )
                    .into_response();
            }
        }
    };
    let mut response = Response::builder()
        .status(if partial { 206 } else { 200 })
        .header(header::CONTENT_TYPE, item.mime)
        .header(header::CONTENT_LENGTH, length)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CACHE_CONTROL, "private, no-store")
        .header(header::REFERRER_POLICY, "no-referrer")
        .header("X-Content-Type-Options", "nosniff");
    if partial {
        response = response.header(
            header::CONTENT_RANGE,
            format!("bytes {}-{}/{}", offset, offset + length - 1, item.bytes),
        );
    }
    response.body(body).unwrap()
}
