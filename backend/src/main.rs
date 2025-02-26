mod auth;
mod chat;
mod common;
mod user;

use crate::auth::handler::AuthHandler;
use crate::auth::service::{AuthReadServiceImpl, AuthWriteServiceImpl};
use crate::chat::conversation::handler::ConversationHandler;
use crate::chat::conversation::repo::read::ConversationReadRepoPg;
use crate::chat::conversation::repo::write::ConversationWriteRepoPg;
use crate::chat::conversation::service::read::ConversationReadServiceImpl;
use crate::chat::conversation::service::write::ConversationWriteServiceImpl;
use crate::chat::message::handler::MessageHandler;
use crate::chat::participant::repo::read::ParticipantReadRepoPg;
use crate::chat::participant::repo::write::ParticipantWriteRepoPg;
use crate::common::config::Config;
use crate::common::database::{Database, UnitOfWorkPg};
use crate::common::state::AppState;
use crate::user::handler::UserHandler;
use crate::user::repo::UserReadRepoPg;
use crate::user::repo::UserWriteRepoPg;
use crate::user::service::UserReadServiceImpl;
use crate::user::service::UserWriteServiceImpl;
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;
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
    let user_read_repo = Arc::new(UserReadRepoPg::new(Arc::clone(&database)));
    let user_write_repo = Arc::new(UserWriteRepoPg::new(Arc::clone(&database)));
    let participant_read_repo = Arc::new(ParticipantReadRepoPg::new(Arc::clone(&database)));
    let participant_write_repo = Arc::new(ParticipantWriteRepoPg::new(Arc::clone(&database)));
    let conversation_read_repo = Arc::new(ConversationReadRepoPg::new(Arc::clone(&database)));
    let conversation_write_repo = Arc::new(ConversationWriteRepoPg::new(Arc::clone(&database)));

    let unit_of_work = Arc::new(UnitOfWorkPg::new(Arc::clone(&database)));

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
    let conversation_write_service = Arc::new(ConversationWriteServiceImpl::new(
        Arc::clone(&conversation_write_repo),
        Arc::clone(&conversation_read_repo),
        Arc::clone(&participant_write_repo),
        Arc::clone(&unit_of_work),
    ));
    let conversation_read_service = Arc::new(ConversationReadServiceImpl::new(Arc::clone(
        &conversation_read_repo,
    )));

    // Initialize handlers
    let user_handler = Arc::new(UserHandler::new(
        Arc::clone(&user_write_service),
        Arc::clone(&user_read_service),
    ));
    let auth_handler = Arc::new(AuthHandler::new(Arc::clone(&auth_write_service)));
    let message_handler = Arc::new(MessageHandler::new());
    let conversation_handler = Arc::new(ConversationHandler::new(
        Arc::clone(&conversation_write_service),
        Arc::clone(&conversation_read_service),
    ));

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
        .merge(MessageHandler::create_route(
            message_handler,
            Router::new().with_state(app_state.clone()),
        ))
        .merge(ConversationHandler::create_route(
            conversation_handler,
            Router::new().with_state(app_state.clone()),
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!(%addr, "Starting the server");
    match tokio::net::TcpListener::bind(addr).await {
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
