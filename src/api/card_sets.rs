use axum::{
    extract::{Path, State},
    routing::{get, post, put, delete, patch},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;
use axum::http::StatusCode;

use crate::domain::cards::{CardSet, CardSetService, PgCardSetRepository, CreateCardSetDto, UpdateCardSetDto, PatchCardSetDto, Validable};
use crate::utils::response::{ApiResponse, json_response, error_response, validation_error};
use crate::utils::extractors::ValidatedJson;

pub struct AppState {
    pub card_set_service: Arc<CardSetService<PgCardSetRepository>>,
}

pub fn card_sets_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/cards/sets", get(get_all_card_sets))
        .route("/cards/sets", post(create_card_set))
        .route("/cards/sets/:id", get(get_card_set_by_id))
        .route("/cards/sets/:id", put(update_card_set))
        .route("/cards/sets/:id", patch(patch_card_set))
        .route("/cards/sets/:id", delete(delete_card_set))
        .with_state(app_state)
}

async fn get_all_card_sets(
    State(state): State<Arc<AppState>>,
) -> ApiResponse<Vec<CardSet>> {
    match state.card_set_service.get_all_card_sets().await {
        Ok(card_sets) => json_response(card_sets),
        Err(e) => error_response(e.to_string(), 500),
    }
}

async fn get_card_set_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResponse<CardSet> {
    match state.card_set_service.get_card_set_by_id(id).await {
        Ok(Some(card_set)) => json_response(card_set),
        Ok(None) => error_response(format!("Conjunto de cartas con ID {} no encontrado", id), 404),
        Err(e) => error_response(e.to_string(), 500),
    }
}

async fn create_card_set(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<CreateCardSetDto>,
) -> ApiResponse<CardSet> {
    // Validamos los datos de entrada
    if let Err(e) = payload.validate() {
        return validation_error(format!("Error de validación: {}", e), None);
    }
    
    // Verificamos si ya existe un conjunto con el mismo código
    match check_unique_code(&state, &payload.code, None).await {
        Ok(true) => {}, // Código único, continúa
        Ok(false) => return validation_error(format!("El código '{}' ya está en uso", payload.code), None),
        Err(e) => return error_response(e.to_string(), 500),
    }
    
    let card_set = payload.to_model();
    
    match state.card_set_service.create_card_set(card_set).await {
        Ok(created) => {
            let response = ApiResponse::success(created, StatusCode::CREATED);
            response
        },
        Err(e) => error_response(e.to_string(), 500),
    }
}

async fn update_card_set(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateCardSetDto>,
) -> ApiResponse<CardSet> {
    // Validamos los datos de entrada
    if let Err(e) = payload.validate() {
        return validation_error(format!("Error de validación: {}", e), None);
    }
    
    // Verificamos si ya existe un conjunto con el mismo código (excluyendo el actual)
    match check_unique_code(&state, &payload.code, Some(id)).await {
        Ok(true) => {}, // Código único, continúa
        Ok(false) => return validation_error(format!("El código '{}' ya está en uso por otro conjunto", payload.code), None),
        Err(e) => return error_response(e.to_string(), 500),
    }
    
    // Primero, verificamos si el conjunto de cartas existe
    match state.card_set_service.get_card_set_by_id(id).await {
        Ok(Some(existing)) => {
            // Actualizamos el conjunto de cartas
            let card_set = payload.to_model(id, existing.created_at);
            match state.card_set_service.update_card_set(card_set).await {
                Ok(updated) => json_response(updated),
                Err(e) => error_response(e.to_string(), 500),
            }
        },
        Ok(None) => error_response(format!("Conjunto de cartas con ID {} no encontrado", id), 404),
        Err(e) => error_response(e.to_string(), 500),
    }
}

// Nuevo endpoint para actualizaciones parciales (PATCH)
async fn patch_card_set(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<PatchCardSetDto>,
) -> ApiResponse<CardSet> {
    // Validamos los datos de entrada
    if let Err(e) = payload.validate() {
        return validation_error(format!("Error de validación: {}", e), None);
    }
    
    // Si estamos actualizando el código, verificamos que sea único
    if let Some(code) = &payload.code {
        match check_unique_code(&state, code, Some(id)).await {
            Ok(true) => {}, // Código único, continúa
            Ok(false) => return validation_error(format!("El código '{}' ya está en uso por otro conjunto", code), None),
            Err(e) => return error_response(e.to_string(), 500),
        }
    }
    
    // Primero, verificamos si el conjunto de cartas existe
    match state.card_set_service.get_card_set_by_id(id).await {
        Ok(Some(existing)) => {
            // Aplicamos los cambios parciales al modelo existente
            let updated_card_set = payload.apply_to_model(existing);
            
            // Guardamos los cambios
            match state.card_set_service.update_card_set(updated_card_set).await {
                Ok(updated) => json_response(updated),
                Err(e) => error_response(e.to_string(), 500),
            }
        },
        Ok(None) => error_response(format!("Conjunto de cartas con ID {} no encontrado", id), 404),
        Err(e) => error_response(e.to_string(), 500),
    }
}

async fn delete_card_set(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResponse<String> {
    match state.card_set_service.delete_card_set(id).await {
        Ok(true) => json_response(format!("Conjunto de cartas con ID {} eliminado correctamente", id)),
        Ok(false) => error_response(format!("Conjunto de cartas con ID {} no encontrado", id), 404),
        Err(e) => error_response(e.to_string(), 500),
    }
}

// Función auxiliar para verificar la unicidad del código
async fn check_unique_code(state: &Arc<AppState>, code: &str, exclude_id: Option<Uuid>) -> Result<bool, String> {
    let card_sets = match state.card_set_service.get_all_card_sets().await {
        Ok(sets) => sets,
        Err(e) => return Err(e.to_string()),
    };
    
    for set in card_sets {
        if set.code == code {
            // Si estamos excluyendo un ID (actualización) y ese ID es el mismo que el conjunto actual,
            // entonces está bien que el código sea el mismo
            if let Some(id) = exclude_id {
                if set.id == id {
                    continue;
                }
            }
            return Ok(false); // Código ya existe
        }
    }
    
    Ok(true) // Código es único
} 