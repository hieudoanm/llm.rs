//! Binary entry point for the pure-Rust mlrs backend.

use std::net::SocketAddr;

use axum::Router;
use mlrs_api::{router, AppState};
use mlrs_core::config::{get_settings, setup_logging};
use mlrs_db::Database;

#[tokio::main]
async fn main() {
    let settings = get_settings();
    setup_logging(&settings.log_level);

    tracing::info!(
        database_url = %settings.database_url,
        data_dir = ?settings.data_dir,
        "starting mlrs backend"
    );

    let db = match Database::connect(&settings).await {
        Ok(database) => database,
        Err(error) => {
            tracing::error!("failed to connect to the database: {error}");
            std::process::exit(1);
        }
    };

    let worker = mlrs_api::worker::start(db.clone(), settings.clone()).await;

    let state = AppState::new(db, settings, worker);
    let app: Router = router(state);

    let host = std::env::var("SKL_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SKL_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8000);

    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}"))
        .await
        .expect("failed to bind the mlrs listener");
    let addr = listener.local_addr().expect("the listener has an address");
    tracing::info!("mlrs backend listening on {addr}");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("the mlrs server failed");
}
