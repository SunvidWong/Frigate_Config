// HTTP Server for Docker mode
// Provides REST API for camera scanning and hardware management

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::info;

use crate::commands::{camera, deploy};
use crate::error::AppError;
use crate::network::DiscoveredCamera;

/// API request/response types
#[derive(Debug, Deserialize)]
struct ScanRequest {
    #[serde(rename = "networkRange")]
    network_range: Option<String>,
    ports: Option<Vec<u16>>,
    #[serde(rename = "timeoutMs")]
    timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct AddDeviceRequest {
    #[serde(rename = "devicePath")]
    device_path: String,
    #[serde(rename = "deviceType")]
    device_type: String,
    #[serde(rename = "deviceName")]
    device_name: String,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(msg: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(msg),
        }
    }
}

/// Convert AppError to HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Network(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Deployment(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(ApiResponse::<()>::error(error_message));
        (status, body).into_response()
    }
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(ApiResponse::ok("OK"))
}

/// Quick scan endpoint
async fn quick_scan() -> Result<Json<ApiResponse<Vec<DiscoveredCamera>>>, AppError> {
    info!("HTTP API: Quick scan cameras");
    let cameras = camera::quick_scan_cameras().await?;
    Ok(Json(ApiResponse::ok(cameras)))
}

/// Custom scan endpoint
async fn custom_scan(
    Json(req): Json<ScanRequest>,
) -> Result<Json<ApiResponse<Vec<DiscoveredCamera>>>, AppError> {
    info!(
        "HTTP API: Custom scan - range: {:?}, ports: {:?}",
        req.network_range, req.ports
    );
    let cameras = camera::scan_for_cameras(req.network_range, req.ports, req.timeout_ms).await?;
    Ok(Json(ApiResponse::ok(cameras)))
}

/// Get network range
async fn get_network_range() -> Result<Json<ApiResponse<String>>, AppError> {
    info!("HTTP API: Get network range");
    let range = camera::guess_network_range_command().await?;
    Ok(Json(ApiResponse::ok(range)))
}

/// Add hardware device
async fn add_hardware_device(
    Json(req): Json<AddDeviceRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    info!(
        "HTTP API: Add hardware device - {} ({})",
        req.device_name, req.device_path
    );
    // Call internal implementation
    deploy::add_device_internal(req.device_path, req.device_type, req.device_name).await?;
    Ok(Json(ApiResponse::ok(())))
}

/// Get saved hardware devices
async fn get_hardware_devices(
) -> Result<Json<ApiResponse<Vec<deploy::HardwareDeviceConfig>>>, AppError> {
    info!("HTTP API: Get saved hardware devices");
    let devices = deploy::load_hardware_devices().await?;
    Ok(Json(ApiResponse::ok(devices)))
}

/// Create and configure the HTTP server
pub async fn run_http_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting HTTP server on port {}", port);

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // API routes
    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/quick_scan_cameras", post(quick_scan))
        .route("/scan_for_cameras", post(custom_scan))
        .route("/guess_network_range_command", post(get_network_range))
        .route("/add_hardware_device_to_config", post(add_hardware_device))
        .route("/get_saved_hardware_devices", post(get_hardware_devices));

    // Serve static files from /app/web
    let static_files = ServeDir::new("/app/web").append_index_html_on_directories(true);

    // Combine routes
    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(static_files)
        .layer(cors);

    // Bind to address
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("HTTP server listening on http://{}", addr);

    // Run server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Check if we should run in HTTP server mode
pub fn should_run_http_mode() -> bool {
    // Check environment variable or Docker environment
    std::env::var("FRIGATE_HTTP_MODE")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false)
        || std::path::Path::new("/.dockerenv").exists()
}
