pub mod error;
pub mod response;
pub mod extractors;

pub use error::AppError;
pub use response::{ApiResponse, json_response, list_response};

pub type Result<T> = std::result::Result<T, AppError>;
