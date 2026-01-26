use anyhow::Result;
use axum::{
    http::{header, Method},
    routing::get,
    Router,
};
use keystone_api::{config::Config, routes, services, AppState};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "keystone_api=debug,tower_http=debug".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app_state = AppState {
        pool: pool.clone(),
        config: config.clone(),
    };

    // Start alert worker in background
    let worker_pool = pool.clone();
    let worker_config = config.clone();
    tokio::spawn(async move {
        services::run_alert_worker(worker_pool, worker_config).await;
    });

    // Configure CORS
    let cors = if config.allowed_origins.is_empty() {
        tracing::warn!("No ALLOWED_ORIGINS configured, using permissive CORS (not recommended for production)");
        CorsLayer::permissive()
    } else {
        let origins: Vec<_> = config
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();

        tracing::info!("CORS allowed origins: {:?}", config.allowed_origins);

        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
            .allow_credentials(true)
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", routes::api_routes(app_state.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(RequestBodyLimitLayer::new(1024 * 1024)) // 1MB max body
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "ok"
}
