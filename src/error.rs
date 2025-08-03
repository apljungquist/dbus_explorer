use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("D-Bus connection failed: {0}")]
    DbusConnection(#[from] dbus::Error),

    #[error("Service introspection failed: {0}")]
    ServiceIntrospection(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Object not found: {0}")]
    ObjectNotFound(String),

    #[error("URL decode error: {0}")]
    UrlDecode(String),

    #[error("Internal server error: {0}")]
    #[allow(dead_code)]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::DbusConnection(_) => {
                (StatusCode::SERVICE_UNAVAILABLE, "D-Bus service unavailable")
            }
            AppError::ServiceIntrospection(_) => {
                (StatusCode::BAD_GATEWAY, "Failed to introspect service")
            }
            AppError::InvalidInput(_) => (StatusCode::BAD_REQUEST, "Invalid input provided"),
            AppError::ServiceNotFound(_) => (StatusCode::NOT_FOUND, "Service not found"),
            AppError::ObjectNotFound(_) => (StatusCode::NOT_FOUND, "Object not found"),
            AppError::UrlDecode(_) => (StatusCode::BAD_REQUEST, "Invalid URL encoding"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>D-Bus Explorer - Error</title>
    <style>
        body {{ font-family: 'Courier New', 'Monaco', 'Menlo', monospace; margin: 40px; }}
        .error {{ color: #d32f2f; background: #ffebee; padding: 20px; border-radius: 4px; }}
    </style>
</head>
<body>
    <h1>Error</h1>
    <div class="error">
        <h2>{}</h2>
        <p>{}</p>
        <p><a href="/local/dbus_explorer/app">← Back to Home</a></p>
    </div>
</body>
</html>"#,
            status.as_u16(),
            message
        );

        (status, Html(html)).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
