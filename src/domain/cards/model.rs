use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct CardSet {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub release_date: DateTime<Utc>,
    pub icon_url: Option<String>,
    pub total_cards: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CardSet {
    pub fn new(
        name: String,
        code: String,
        release_date: DateTime<Utc>,
        icon_url: Option<String>,
        total_cards: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            code,
            release_date,
            icon_url,
            total_cards,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl<'r> sqlx::FromRow<'r, PgRow> for CardSet {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            code: row.try_get("code")?,
            release_date: row.try_get("release_date")?,
            icon_url: row.try_get("icon_url")?,
            total_cards: row.try_get("total_cards")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
} 