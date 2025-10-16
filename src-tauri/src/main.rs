// Prevents additional console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;
mod config_engine;
mod database;
mod deployment;
mod error;
mod http_server; // HTTP server for Docker mode
mod models;
mod network; // Network utilities and camera discovery
mod state;
mod utils;

fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Frigate Configuration Tool");

    // Check if we should run in HTTP mode (Docker)
    if http_server::should_run_http_mode() {
        info!("Running in HTTP server mode (Docker)");
        run_http_mode();
        return;
    }

    // Run in Tauri mode (Desktop)
    info!("Running in Tauri mode (Desktop)");
    run_tauri_mode();
}

/// Run HTTP server mode for Docker
#[tokio::main]
async fn run_http_mode() {
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "1420".to_string())
        .parse()
        .unwrap_or(1420);

    if let Err(e) = http_server::run_http_server(port).await {
        error!("HTTP server error: {}", e);
        std::process::exit(1);
    }
}

/// Run Tauri desktop application mode
fn run_tauri_mode() {
    // Initialize application state
    let app_state = state::AppState::new().expect("Failed to initialize application state");

    // Initialize hardware detection cache
    let cache_state = commands::agent::init_cache_state();

    tauri::Builder::default()
        .manage(app_state)
        .manage(cache_state)
        .invoke_handler(tauri::generate_handler![
            // Agent commands (Phase 3 - T032-T037)
            commands::agent::detect_hardware,
            commands::agent::get_device_details,
            commands::agent::get_device_capabilities,
            commands::agent::get_hardware_availability,
            commands::agent::clear_hardware_cache,
            commands::agent::get_agent_version,
            // Configuration commands (Phase 4 - T058)
            commands::config::load_config,
            commands::config::save_config,
            commands::config::merge_configurations,
            commands::config::resolve_conflicts,
            commands::config::create_snapshot,
            commands::config::list_snapshots,
            commands::config::restore_from_snapshot,
            commands::config::delete_snapshot,
            commands::config::compare_snapshots,
            commands::config::get_snapshot_statistics,
            commands::config::validate_config,
            // Deployment commands (Phase 5 - T120-T124)
            commands::deploy::deploy_frigate,
            commands::deploy::get_deployment_status_cmd,
            commands::deploy::stop_deployment_cmd,
            commands::deploy::get_deployment_logs,
            commands::deploy::check_deployment_health,
            commands::deploy::rollback_deployment,
            commands::deploy::list_deployment_history,
            commands::deploy::get_deployment_by_id_cmd,
            commands::deploy::add_hardware_device_to_config,
            commands::deploy::get_saved_hardware_devices,
            commands::deploy::remove_hardware_device,
            commands::deploy::validate_hardware_devices,
            commands::deploy::scan_pci_devices,
            commands::deploy::set_docker_compose_path,
            commands::deploy::get_docker_compose_path,
            commands::deploy::save_docker_compose_content,
            // Disk commands (Phase 8 - T175, T180)
            commands::disk::get_disk_info_command,
            commands::disk::validate_volume_path_command,
            commands::disk::create_volume_mapping,
            commands::disk::get_default_volume_paths,
            commands::disk::get_recommended_paths,
            // Camera discovery commands (Phase 8.5 - Camera Scanner)
            commands::camera::get_network_interfaces,
            commands::camera::get_local_network_ip,
            commands::camera::guess_network_range_command,
            commands::camera::scan_for_cameras,
            commands::camera::quick_scan_cameras,
            // System status commands (Phase 4 - T034)
            commands::system::check_system_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
