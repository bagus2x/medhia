#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Participant {
    pub id: i64,
    pub conversation_id: i64,
    pub user_id: i64,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub roles: String,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
