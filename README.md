# Kodem Cards Backend

Backend API for the Kodem Cards game.

## Development

### Hot Reloading

This project supports hot reloading during development, which allows you to make changes to your code without having to manually rebuild and restart the application.

To use hot reloading:

1. Install the required tools:
   ```bash
   cargo install cargo-watch
   cargo install systemfd
   ```

2. Run the development script:
   ```bash
   ./dev.sh
   ```

For more details on how hot reloading works and alternative setups, see [hot-reload.md](hot-reload.md).

## Standardized API Response Structure

All API endpoints in this application follow a standardized response structure to ensure consistency across the entire API. This makes it easier for frontend developers to handle responses and provides a predictable format for error handling.

### Response Format

All API responses follow this JSON structure:

```json
{
    "status_code": 200,
    "message": "Optional message, typically for errors",
    "error": 400,
    "data": [
        {
            "id": "123",
            "name": "Example"
        }
    ]
}
```

Where:

- `status_code`: The HTTP status code of the response (e.g., 200, 201, 400, 404, 500)
- `message`: An optional message, typically used for error responses
- `error`: An optional error code, if applicable
- `data`: The response data, which can be a single object or an array of objects

### Success Responses

For successful responses, the `status_code` will be in the 2xx range, and the `data` field will contain the response data. The `message` and `error` fields will be omitted.

Example success response:

```json
{
    "status_code": 200,
    "data": [
        {
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "Fire Dragon",
            "description": "A powerful dragon that breathes fire"
        },
        {
            "id": "223e4567-e89b-12d3-a456-426614174001",
            "name": "Water Elemental",
            "description": "A creature made of pure water"
        }
    ]
}
```

### Error Responses

For error responses, the `status_code` will be in the 4xx or 5xx range, the `message` field will contain a description of the error, and the `error` field will contain the error code. The `data` field will be omitted.

Example error response:

```json
{
    "status_code": 404,
    "message": "Card not found",
    "error": 404
}
```

## Using the Standardized Response Structure in Your Code

The application provides utility functions and types to make it easy to create standardized responses in your API endpoints.

### ApiResponse Struct

The `ApiResponse<T>` struct is a generic type that represents a standardized API response. It has the following fields:

- `status_code`: The HTTP status code of the response
- `message`: An optional message, typically used for error responses
- `error`: An optional error code, if applicable
- `data`: The response data, which can be of any serializable type

### Helper Functions

The application provides several helper functions to create standardized responses:

- `json_response(data)`: Creates a success response with status code 200 OK
- `list_response(items)`: Creates a success response with status code 200 OK for a list of items

### Example Usage

Here's an example of how to use the standardized response structure in your API endpoints:

```rust
use crate::utils::response::{ApiResponse, json_response, list_response};

// Success response with a single item
async fn get_item(Path(id): Path<String>) -> ApiResponse<Item> {
    let item = Item {
        id,
        name: "Example Item".to_string(),
        description: Some("This is an example item".to_string()),
        created_at: "2023-01-01T00:00:00Z".to_string(),
    };
    
    json_response(item)
}

// Success response with a list of items
async fn list_items() -> ApiResponse<Vec<Item>> {
    let items = vec![
        Item {
            id: "123".to_string(),
            name: "Item 1".to_string(),
            description: Some("Description 1".to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
        },
        Item {
            id: "456".to_string(),
            name: "Item 2".to_string(),
            description: Some("Description 2".to_string()),
            created_at: "2023-01-02T00:00:00Z".to_string(),
        },
    ];
    
    list_response(items)
}

// Created response
async fn create_item(Json(payload): Json<CreateItemRequest>) -> ApiResponse<Item> {
    let item = Item {
        id: "123".to_string(),
        name: payload.name,
        description: payload.description,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    ApiResponse::created(item)
}

// Error response
async fn get_item_with_error(Path(id): Path<String>) -> Result<ApiResponse<Item>, AppError> {
    if id == "not-found" {
        return Err(AppError::NotFound("Item not found".to_string()));
    }
    
    let item = Item {
        id,
        name: "Example Item".to_string(),
        description: Some("This is an example item".to_string()),
        created_at: "2023-01-01T00:00:00Z".to_string(),
    };
    
    Ok(json_response(item))
}
```

### Error Handling

The application also provides a standardized way to handle errors using the `AppError` enum. When an `AppError` is returned from an API endpoint, it will be automatically converted to a standardized error response.

Example error handling:

```rust
async fn get_item(Path(id): Path<String>) -> Result<ApiResponse<Item>, AppError> {
    // In a real app, this would fetch an item from the database
    if id == "not-found" {
        return Err(AppError::NotFound("Item not found".to_string()));
    }
    
    let item = Item {
        id,
        name: "Example Item".to_string(),
        description: Some("This is an example item".to_string()),
        created_at: "2023-01-01T00:00:00Z".to_string(),
    };
    
    Ok(json_response(item))
}
```

## Example Endpoints

The application includes several example endpoints that demonstrate how to use the standardized response structure:

- `POST /examples/items`: Creates a new item
- `GET /examples/items`: Lists all items
- `GET /examples/items/:id`: Gets an item by ID
- `PUT /examples/items/:id`: Updates an item
- `DELETE /examples/items/:id`: Deletes an item
- `POST /examples/validate`: Validates an item

You can use these endpoints as references when implementing your own API endpoints.
