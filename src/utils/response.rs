use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Standardized API response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// HTTP status code
    pub status_code: u16,
    /// Optional message, typically used for errors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Optional error code, if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<u16>,
    /// Response data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// Create a success response with data
    pub fn success(data: T, status_code: StatusCode) -> Self {
        Self {
            status_code: status_code.as_u16(),
            message: None,
            error: None,
            data: Some(data),
        }
    }

    /// Create an error response
    pub fn error(message: String, status_code: StatusCode, error_code: Option<u16>) -> ApiResponse<T> {
        Self {
            status_code: status_code.as_u16(),
            message: Some(message),
            error: error_code,
            data: None,
        }
    }

    /// Create a success response with status code 200 OK
    pub fn ok(data: T) -> Self {
        Self::success(data, StatusCode::OK)
    }

    /// Create a success response with status code 201 CREATED
    pub fn created(data: T) -> Self {
        Self::success(data, StatusCode::CREATED)
    }

    /// Create a not found error response
    pub fn not_found(message: String) -> Self {
        Self::error(message, StatusCode::NOT_FOUND, Some(404))
    }

    /// Create a bad request error response
    pub fn bad_request(message: String) -> Self {
        Self::error(message, StatusCode::BAD_REQUEST, Some(400))
    }

    /// Create an internal server error response
    pub fn internal_error(message: String) -> Self {
        Self::error(message, StatusCode::INTERNAL_SERVER_ERROR, Some(500))
    }

    /// Create an unauthorized error response
    pub fn unauthorized(message: String) -> Self {
        Self::error(message, StatusCode::UNAUTHORIZED, Some(401))
    }

    /// Create a forbidden error response
    pub fn forbidden(message: String) -> Self {
        Self::error(message, StatusCode::FORBIDDEN, Some(403))
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status_code = StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let json = Json(json!({
            "status_code": self.status_code,
            "message": self.message,
            "error": self.error,
            "data": self.data,
        }));

        (status_code, json).into_response()
    }
}

/// Helper function to wrap any serializable data in a standard API response
pub fn json_response<T: Serialize>(data: T) -> ApiResponse<T> {
    ApiResponse::ok(data)
}

/// Helper function to create a list response
pub fn list_response<T: Serialize>(items: Vec<T>) -> ApiResponse<Vec<T>> {
    ApiResponse::ok(items)
}

/// Helper function to create an error response
pub fn error_response<T>(message: String, status_code: u16) -> ApiResponse<T> {
    let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    ApiResponse::error(message, status, Some(status_code))
}

/// Helper function to create a validation error response with detailed information
pub fn validation_error<T>(message: String, details: Option<Value>) -> ApiResponse<T> {
    ApiResponse::<T>::bad_request(message)
}
