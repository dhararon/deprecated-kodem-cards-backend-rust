use anyhow::Result;
use uuid::Uuid;

use super::model::CardSet;
use super::repository::CardSetRepository;

pub struct CardSetService<R: CardSetRepository> {
    repository: R,
}

impl<R: CardSetRepository> CardSetService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_all_card_sets(&self) -> Result<Vec<CardSet>> {
        self.repository.get_all_card_sets().await
    }

    pub async fn get_card_set_by_id(&self, id: Uuid) -> Result<Option<CardSet>> {
        self.repository.get_card_set_by_id(id).await
    }

    pub async fn create_card_set(&self, card_set: CardSet) -> Result<CardSet> {
        self.repository.create_card_set(card_set).await
    }

    pub async fn update_card_set(&self, card_set: CardSet) -> Result<CardSet> {
        self.repository.update_card_set(card_set).await
    }

    pub async fn delete_card_set(&self, id: Uuid) -> Result<bool> {
        self.repository.delete_card_set(id).await
    }
} 