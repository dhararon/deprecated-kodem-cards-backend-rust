use axum::{
    routing::get,
    Router, 
};
use std::sync::Arc;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;

use crate::utils::response::{ApiResponse, json_response};
use crate::domain::cards::{CardSetService, PgCardSetRepository};
use crate::api::card_sets::{AppState, card_sets_routes};

pub fn create_router() -> Router {
    // Sólo mantener la ruta de health check
    Router::new()
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
}

pub fn create_router_with_db(pool: PgPool) -> Router {
    // Crear repositorio y servicio
    let card_set_repository = PgCardSetRepository::new(pool);
    let card_set_service = Arc::new(CardSetService::new(card_set_repository));
    
    // Estado de la aplicación
    let app_state = Arc::new(AppState {
        card_set_service,
    });
    
    // Router con rutas
    Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", card_sets_routes(app_state.clone()))
        .layer(CorsLayer::permissive())
}

async fn health_check() -> ApiResponse<&'static str> {
    json_response("Server is running OK")
}
