mod auth;
mod common;
mod user;

use crate::auth::handler::AuthHandler;
use crate::auth::service::{AuthReadServiceImpl, AuthWriteServiceImpl};
use crate::common::config::Config;
use crate::common::database::Database;
use crate::common::state::AppState;
use crate::user::handler::UserHandler;
use crate::user::repo::PostgresUserReadRepo;
use crate::user::repo::PostgresUserWriteRepo;
use crate::user::service::UserReadServiceImpl;
use crate::user::service::UserWriteServiceImpl;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::{error, info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize structured message
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG) // Set the maximum log level
        .init();

    // Load configuration
    let config = Arc::new(Config::init());
    info!(?config, "Using configuration");

    // Initialize database
    let database = match Database::init(Arc::clone(&config)).await {
        Ok(db) => Arc::new(db.pool),
        Err(err) => {
            error!(error = %err, "Failed to initialize database");
            return;
        }
    };

    // Initialize repositories
    let user_read_repo = Arc::new(PostgresUserReadRepo::new(Arc::clone(&database)));
    let user_write_repo = Arc::new(PostgresUserWriteRepo::new(Arc::clone(&database)));

    // Initialize services
    let user_read_service = Arc::new(UserReadServiceImpl::new(Arc::clone(&user_read_repo)));
    let user_write_service = Arc::new(UserWriteServiceImpl::new(
        Arc::clone(&user_write_repo),
        Arc::clone(&user_read_repo),
    ));
    let auth_write_service = Arc::new(AuthWriteServiceImpl::new(
        Arc::clone(&user_write_repo),
        Arc::clone(&user_read_repo),
        Arc::clone(&config),
    ));
    let auth_read_service = Arc::new(AuthReadServiceImpl::new(Arc::clone(&config)));

    // Initialize handlers
    let user_handler = Arc::new(UserHandler::new(
        Arc::clone(&user_write_service),
        Arc::clone(&user_read_service),
    ));
    let auth_handler = Arc::new(AuthHandler::new(Arc::clone(&auth_write_service)));

    let app_state = AppState {
        auth_read_service: Arc::clone(&auth_read_service),
    };

    // Create Axum app
    let app = Router::new()
        .route("/", get(|| async { "Welcome to social media" }))
        .merge(UserHandler::create_route(
            user_handler,
            Router::new().with_state(app_state.clone()),
        ))
        .merge(AuthHandler::create_route(
            auth_handler,
            Router::new().with_state(app_state.clone()),
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server
    let address = format!("0.0.0.0:{}", config.port);
    info!(%address, "Starting server");
    match tokio::net::TcpListener::bind(&address).await {
        Ok(listener) => {
            if let Err(err) = axum::serve(listener, app).await {
                error!(error = %err, "Server encountered an error");
            }
        }
        Err(err) => {
            error!(error = %err, "Failed to bind to address");
        }
    }
}
