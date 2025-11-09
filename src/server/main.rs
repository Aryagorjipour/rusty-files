use actix_web::{middleware, web, App, HttpServer};
use actix_cors::Cors;
use rusty_files::SearchEngine;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Import server modules from the library
use rusty_files::server::{api, config, state, websocket};

use config::ServerConfig;
use state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info,actix_web=info".into()),
            ),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = ServerConfig::load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load config: {}, using defaults", e);
        ServerConfig::default()
    });

    let bind_addr = format!("{}:{}", config.server.host, config.server.port);

    tracing::info!("Initializing search engine...");

    // Initialize search engine
    let engine = SearchEngine::new(&config.database.path).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to initialize search engine: {}", e),
        )
    })?;

    // Create application state
    let state = web::Data::new(AppState::new(engine, config.clone()));

    tracing::info!("Starting server on {}", bind_addr);
    tracing::info!("API endpoints available at http://{}/api/v1", bind_addr);
    tracing::info!("WebSocket available at ws://{}/ws", bind_addr);

    // Start HTTP server
    HttpServer::new(move || {
        let cors = if config.server.enable_cors {
            Cors::permissive()
        } else {
            Cors::default()
        };

        App::new()
            .app_data(state.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            // API routes
            .service(
                web::scope("/api/v1")
                    .route("/search", web::post().to(api::search))
                    .route("/index", web::post().to(api::index))
                    .route("/update", web::post().to(api::update))
                    .route("/watch", web::post().to(api::start_watch))
                    .route("/watch/{id}", web::delete().to(api::stop_watch))
                    .route("/stats", web::get().to(api::get_stats))
                    .route("/health", web::get().to(api::health_check)),
            )
            // WebSocket route
            .route("/ws", web::get().to(websocket::websocket_handler))
    })
    .workers(config.server.workers)
    .keep_alive(std::time::Duration::from_secs(config.server.keep_alive))
    .bind(&bind_addr)?
    .run()
    .await
}
