use anyhow::Result;
use log::info;

mod config;
mod dbus_introspection;
mod error;
mod handlers;
mod routes;
mod templates;
mod utils;

use config::Config;
use routes::create_routes;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    acap_logging::init_logger();

    // Load configuration
    let config = Config::from_env();
    info!("Starting D-Bus Explorer with config: {config:?}");

    // Create the web application
    let app = create_routes();

    let listener = tokio::net::TcpListener::bind(config.server_addr).await?;
    info!(
        "D-Bus Explorer server running at http://{}/local/dbus_explorer",
        config.server_addr
    );

    axum::serve(listener, app).await?;
    Ok(())
}
