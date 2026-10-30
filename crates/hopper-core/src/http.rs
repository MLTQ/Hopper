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
        .route("/pair", post(pair))
        .route("/peers/:id", delete(unpair))
        .route("/import", post(import_paths))
        .route("/upload", post(upload))
        .route("/progress", post(progress))
        .route("/queue", post(queue))
        .route("/play/:id", post(play))
        .route("/keep/:id", post(keep))
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
async fn refresh(State(node): State<Node>) -> Api<Library> {
    node.refresh().await?;
    Ok(Json(node.library()?))
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
    Ok(Json(node.import(input.paths).await?))
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
        temp.persist(&dest)?;
        local.path = dest;
        local.modified = import::modified(&std::fs::metadata(&local.path)?);
        node.0.store.put("local", &local.media.id, &local)?;
        report.added += 1;
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
