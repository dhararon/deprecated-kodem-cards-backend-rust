mod api;
mod config;
mod domain;
mod infrastructure;
mod utils;

use dotenv::dotenv;
use listenfd::ListenFd;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file if it exists
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = config::Config::from_env()
        .expect("Failed to load configuration from environment");

    // Inicializar base de datos con migraciones
    let pool = infrastructure::database::init_database(&config.database_url)
        .await
        .expect("Failed to initialize database");

    // Build our application con rutas completas
    let app = api::create_router_with_db(pool);

    // Create a listener using either listenfd (for hot reloading) or a new TcpListener
    let mut listenfd = ListenFd::from_env();
    let listener = match listenfd.take_tcp_listener(0)? {
        Some(listener) => {
            tracing::info!("Using socket from systemfd for hot reloading");
            tokio::net::TcpListener::from_std(listener)?
        }
        None => {
            let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
            tracing::info!("listening on {}", addr);
            tokio::net::TcpListener::bind(addr).await?
        }
    };

    tracing::info!("Server running with API endpoints");

    // Run the server
    axum::serve(listener, app).await?;
    
    Ok(())
}
