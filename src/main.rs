//! Run with
//!
//! ```not_rust
//! cargo run --release
//! ```

mod routes;
mod utils;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};

use clap::{arg, Parser};
use tokio::signal;
use tokio::sync::RwLock;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use tracing_subscriber::EnvFilter;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use shadow_rs::shadow;

use crate::utils::{LogLevel, User};

// Get build information
shadow!(build);

type GlobalState = Arc<RwLock<HashMap<String, User>>>;

/// Command line arguments
///
/// Basic info is read from `Cargo.toml`
/// See Clap `Derive` documentation for details:
/// <https://docs.rs/clap/latest/clap/_derive/index.html>
#[derive(Parser)]
#[command(
    author,
    about = "Rust Axum REST API example.",
    long_about = "Rust Axum REST API example.",
    arg_required_else_help = false,
    disable_version_flag = true
)]
struct Args {
    /// Optional host IP to listen to (for example "0.0.0.0")
    #[arg(long, value_name = "HOST")]
    host: Option<String>,

    /// Log level to use
    #[arg(value_enum, short, long, value_name = "LEVEL")]
    log: Option<LogLevel>,

    /// Optional port number to use (default is 3000)
    #[arg(short, long, value_name = "PORT")]
    port: Option<u16>,

    /// Custom version flag instead of clap default
    #[arg(short, long, help = "Print version info and exit")]
    version: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    if args.version {
        println!("{}", utils::api_version_info());
        return Ok(());
    }

    let host = args.host.unwrap_or_else(|| "127.0.0.1".to_string());
    let port_number = args.port.unwrap_or(3000);

    let mut filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    if let Some(ref level) = args.log {
        filter_layer = filter_layer.add_directive(level.to_filter().into());
    }

    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter(filter_layer).init();

    // Initialize your global state
    let state: GlobalState = Arc::new(RwLock::new(HashMap::new()));
    // Clone the state to move into the closure
    let app_state = state.clone();

    // Build application with routes
    let app = Router::new()
        .route("/", get(routes::root))
        .route("/version", get(routes::version))
        .route("/user", get(routes::query_user))
        .route("/users", post(routes::create_user))
        .layer(axum::Extension(app_state))
        .layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete.
            // Add a timeout so requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
        ));

    // Run app with Hyper
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port_number}")).await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
