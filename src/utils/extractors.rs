use axum::{
    async_trait,
    extract::{FromRequest, Request},
    http::StatusCode,
    Json,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::utils::response::{ApiResponse, validation_error};

// Extractor personalizado para JSON
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiResponse<Value>;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Primero intentamos extraer el JSON usando el extractor normal de Axum
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(ValidatedJson(value)),
            Err(err) => {
                // Si hay un error de deserialización, lo convertimos en un error de validación
                let error_message = match err.status() {
                    StatusCode::UNPROCESSABLE_ENTITY => {
                        // Error de deserialización de JSON
                        let message = err.to_string();
                        
                        // Mejoramos los mensajes para casos específicos
                        if message.contains("missing field") {
                            let field = extract_field_name(&message);
                            format!("Falta el campo requerido: '{}'", field)
                        } else if message.contains("invalid type") {
                            format!("Tipo de dato inválido en el JSON: {}", message)
                        } else if message.contains("data did not match") || message.contains("invalid value") {
                            if message.contains("release_date") {
                                "El formato de la fecha debe ser ISO 8601 con zona horaria (ejemplo: '2025-01-01T00:00:00Z')".to_string()
                            } else {
                                format!("Formato de datos inválido: {}", message)
                            }
                        } else {
                            format!("Error al procesar el JSON: {}", message)
                        }
                    }
                    StatusCode::BAD_REQUEST => {
                        // JSON sintácticamente inválido
                        "El JSON enviado no es válido. Verifica la sintaxis.".to_string()
                    }
                    StatusCode::UNSUPPORTED_MEDIA_TYPE => {
                        // Content-Type incorrecto
                        "El Content-Type debe ser application/json".to_string()
                    }
                    _ => format!("Error en la solicitud: {}", err),
                };

                Err(validation_error(error_message, None))
            }
        }
    }
}

// Función auxiliar para extraer el nombre del campo de un mensaje de error
fn extract_field_name(message: &str) -> &str {
    if let Some(start) = message.find('`') {
        if let Some(end) = message[start + 1..].find('`') {
            return &message[start + 1..start + 1 + end];
        }
    }
    "desconocido"
} 