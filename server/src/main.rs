//! Tarot Battler local server.
//!
//! One process that serves the static UI (`tools/ui/`), exposes an `/api`
//! namespace, and persists state in an embedded **redb** database file. Built as
//! a sibling crate that depends on `battle_engine` directly (native battles via
//! `run_battle_json` — no server-side WASM).
//!
//! Run it from the `server/` directory:
//!   cargo run
//! Config (env):
//!   TAROT_PORT    listen port (default 8080)
//!   TAROT_DB      embedded DB file path (default ./tarot-data/tarot.redb)
//!   TAROT_UI_DIR  static UI directory (default ../tools/ui relative to crate)

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use axum::{routing::get, Json, Router};
use tower_http::services::ServeDir;

mod db;
mod draft;
mod models;
mod runner;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("TAROT_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let db_path = std::env::var("TAROT_DB").unwrap_or_else(|_| "tarot-data/tarot.redb".to_string());
    let ui_dir = std::env::var("TAROT_UI_DIR")
        .unwrap_or_else(|_| format!("{}/../tools/ui", env!("CARGO_MANIFEST_DIR")));

    // Open (creating if needed) the embedded database; tables are created lazily.
    let db = db::Db::open(&db_path).unwrap_or_else(|e| {
        eprintln!("failed to open database at {db_path}: {e}");
        std::process::exit(1);
    });
    // Hold the handle for the process lifetime (state wiring lands with the data model).
    let _db = db;

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/api/version", get(version))
        // Everything else is served from the static UI directory.
        .fallback_service(ServeDir::new(&ui_dir).append_index_html_on_directories(true));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("failed to bind {addr}: {e}");
            std::process::exit(1);
        });
    let ui_display = PathBuf::from(&ui_dir);
    println!("Tarot server listening on http://{addr}");
    println!("  UI:  {}", ui_display.display());
    println!("  DB:  {db_path}");
    axum::serve(listener, app).await.expect("server error");
}

async fn healthz() -> &'static str {
    "ok"
}

async fn version() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "tarot-server",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Resolve a path relative to the crate root when it isn't absolute, so the
/// server works regardless of the current working directory.
#[allow(dead_code)]
fn crate_relative(path: &str) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
    }
}
