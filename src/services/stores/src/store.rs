use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Store {
    pub id: i32,
    pub user_id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub logo_url: Option<String>,
    pub banner_urls: Vec<String>,
    pub phone: Option<String>,
    pub social_links: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Store {
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}
