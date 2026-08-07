//! Раздача статики веб-дашборда.
//!
//! По умолчанию файлы `web/build` встроены в бинарь (rust-embed), поэтому cfgy
//! работает как single-binary. Если задан `CFGY_WEB_ROOT`, статика читается с
//! диска из этого каталога — это позволяет подменить фронтенд без пересборки.
//!
//! В debug-сборке rust-embed читает файлы с диска на каждый запрос, так что
//! `task build-web` подхватывается без перекомпиляции.

use axum::{
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "web/build"]
struct WebAssets;

/// Каталог со статикой на диске, если задан CFGY_WEB_ROOT
fn disk_root() -> Option<std::path::PathBuf> {
    let root = std::env::var("CFGY_WEB_ROOT").ok()?;
    let path = std::path::PathBuf::from(root);
    path.is_dir().then_some(path)
}

/// Прочитать файл: сначала с диска (если задан CFGY_WEB_ROOT), затем из бинаря
async fn read_asset(path: &str) -> Option<Vec<u8>> {
    if let Some(root) = disk_root() {
        // Защита от выхода за пределы каталога
        if !path.split('/').any(|c| c == ".." || c.is_empty())
            && let Ok(body) = tokio::fs::read(root.join(path)).await
        {
            return Some(body);
        }
    }

    WebAssets::get(path).map(|f| f.data.into_owned())
}

fn respond(path: &str, body: Vec<u8>, immutable: bool) -> Response<Body> {
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.as_ref());

    if immutable {
        // Файлы в _app/ содержат хеш в имени — кешируем навсегда
        builder = builder.header(header::CACHE_CONTROL, "public, max-age=31536000, immutable");
    } else {
        builder = builder.header(header::CACHE_CONTROL, "no-cache");
    }

    builder.body(Body::from(body)).unwrap()
}

/// SPA-фолбэк: отдать index.html
pub async fn serve_index() -> Response<Body> {
    match read_asset("index.html").await {
        Some(body) => respond("index.html", body, false),
        None => {
            tracing::error!("index.html не найден: веб не собран (task build-web)");
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(
                    "Веб-дашборд не собран. Выполните `task build-web` и пересоберите cfgy.",
                ))
                .unwrap()
        }
    }
}

/// Раздача ассетов SvelteKit из _app/
pub async fn serve_app_asset(Path(path): Path<String>) -> Response<Body> {
    let full = format!("_app/{}", path);
    match read_asset(&full).await {
        Some(body) => respond(&full, body, true),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}

/// Раздача файлов из корня статики (favicon.svg, logo.svg, robots.txt и т.п.)
pub async fn serve_root_asset(Path(path): Path<String>) -> Response<Body> {
    match read_asset(&path).await {
        Some(body) => respond(&path, body, false),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}
