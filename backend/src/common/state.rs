use crate::auth::service::AuthReadServiceImpl;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub auth_read_service: Arc<AuthReadServiceImpl>,
}
