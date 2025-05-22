use async_trait::async_trait;
use sqlx::PgPool;
use anyhow::Result;
use uuid::Uuid;

use super::model::CardSet;

#[async_trait]
pub trait CardSetRepository {
    async fn get_all_card_sets(&self) -> Result<Vec<CardSet>>;
    async fn get_card_set_by_id(&self, id: Uuid) -> Result<Option<CardSet>>;
    async fn create_card_set(&self, card_set: CardSet) -> Result<CardSet>;
    async fn update_card_set(&self, card_set: CardSet) -> Result<CardSet>;
    async fn delete_card_set(&self, id: Uuid) -> Result<bool>;
}

pub struct PgCardSetRepository {
    pool: PgPool,
}

impl PgCardSetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CardSetRepository for PgCardSetRepository {
    async fn get_all_card_sets(&self) -> Result<Vec<CardSet>> {
        let card_sets = sqlx::query_as::<_, CardSet>(
            r#"
            SELECT id, name, code, release_date, icon_url, total_cards, created_at, updated_at
            FROM card_sets
            ORDER BY release_date DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(card_sets)
    }

    async fn get_card_set_by_id(&self, id: Uuid) -> Result<Option<CardSet>> {
        let card_set = sqlx::query_as::<_, CardSet>(
            r#"
            SELECT id, name, code, release_date, icon_url, total_cards, created_at, updated_at
            FROM card_sets
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(card_set)
    }

    async fn create_card_set(&self, card_set: CardSet) -> Result<CardSet> {
        let created = sqlx::query_as::<_, CardSet>(
            r#"
            INSERT INTO card_sets (id, name, code, release_date, icon_url, total_cards, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, name, code, release_date, icon_url, total_cards, created_at, updated_at
            "#
        )
        .bind(card_set.id)
        .bind(card_set.name)
        .bind(card_set.code)
        .bind(card_set.release_date)
        .bind(card_set.icon_url)
        .bind(card_set.total_cards)
        .bind(card_set.created_at)
        .bind(card_set.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }

    async fn update_card_set(&self, card_set: CardSet) -> Result<CardSet> {
        let now = chrono::Utc::now();
        
        let updated = sqlx::query_as::<_, CardSet>(
            r#"
            UPDATE card_sets
            SET 
                name = $1,
                code = $2,
                release_date = $3,
                icon_url = $4,
                total_cards = $5,
                updated_at = $6
            WHERE id = $7
            RETURNING id, name, code, release_date, icon_url, total_cards, created_at, updated_at
            "#
        )
        .bind(card_set.name)
        .bind(card_set.code)
        .bind(card_set.release_date)
        .bind(card_set.icon_url)
        .bind(card_set.total_cards)
        .bind(now)
        .bind(card_set.id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    async fn delete_card_set(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM card_sets
            WHERE id = $1
            "#
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
} 