// Agent-related Tauri commands
// T032-T035: Hardware detection commands with caching

use crate::error::AppError;
use crate::models::hardware_device::HardwareDevice;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::State;
use tracing::{error, info};

// Cache structure for hardware detection results
struct HardwareCache {
    devices: Vec<HardwareDevice>,
    last_updated: Instant,
}

// Global cache with 5-minute expiration
pub struct CacheState {
    cache: Mutex<Option<HardwareCache>>,
}

impl CacheState {
    fn new() -> Self {
        CacheState {
            cache: Mutex::new(None),
        }
    }

    fn get(&self) -> Option<Vec<HardwareDevice>> {
        let cache = self.cache.lock().unwrap();
        if let Some(cached) = cache.as_ref() {
            // Cache expires after 5 minutes
            if cached.last_updated.elapsed() < Duration::from_secs(300) {
                return Some(cached.devices.clone());
            }
        }
        None
    }

    fn set(&self, devices: Vec<HardwareDevice>) {
        let mut cache = self.cache.lock().unwrap();
        *cache = Some(HardwareCache {
            devices,
            last_updated: Instant::now(),
        });
    }

    fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        *cache = None;
    }
}

// Initialize cache state (called from main.rs)
pub fn init_cache_state() -> CacheState {
    CacheState::new()
}

/// T032: Detect hardware devices using the agent binary
/// Returns a list of detected hardware accelerators and video devices
#[tauri::command]
pub async fn detect_hardware(
    cache_state: State<'_, CacheState>,
    force_refresh: Option<bool>,
) -> Result<Vec<HardwareDevice>, AppError> {
    let force = force_refresh.unwrap_or(false);

    // Check cache first (unless force refresh)
    if !force {
        if let Some(cached_devices) = cache_state.get() {
            info!("Returning cached hardware detection results");
            return Ok(cached_devices);
        }
    }

    info!("Running hardware detection via agent");

    // Find agent binary
    let agent_path = find_agent_binary()?;

    // Execute agent detect command
    let output = Command::new(&agent_path)
        .arg("detect")
        .output()
        .map_err(|e| AppError::Agent(format!("Failed to execute agent: {}", e)))?;

    // Check if command succeeded
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Agent command failed: {}", stderr);
        return Err(AppError::Agent(format!("Agent execution failed: {}", stderr)));
    }

    // Parse JSON output
    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| AppError::Agent(format!("Invalid UTF-8 output: {}", e)))?;

    let detection_result: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| AppError::Config(format!("Failed to parse agent JSON: {}", e)))?;

    // Extract devices array
    let devices_json = detection_result
        .get("devices")
        .ok_or_else(|| AppError::Agent("Missing 'devices' field in agent output".to_string()))?;

    let devices: Vec<HardwareDevice> = serde_json::from_value(devices_json.clone())
        .map_err(|e| AppError::Config(format!("Failed to deserialize devices: {}", e)))?;

    info!("Detected {} hardware devices", devices.len());

    // Update cache
    cache_state.set(devices.clone());

    Ok(devices)
}

/// T033: Get details for a specific hardware device
#[tauri::command]
pub async fn get_device_details(
    device_id: String,
    cache_state: State<'_, CacheState>,
) -> Result<HardwareDevice, AppError> {
    info!("Getting details for device: {}", device_id);

    // Try to get from cache first
    let devices = if let Some(cached) = cache_state.get() {
        cached
    } else {
        // Run detection if cache is empty
        detect_hardware(cache_state, None).await?
    };

    // Find device by ID
    devices
        .into_iter()
        .find(|d| d.id == device_id)
        .ok_or_else(|| AppError::NotFound(format!("Device not found: {}", device_id)))
}

/// T034: Get detailed device capabilities
#[tauri::command]
pub async fn get_device_capabilities(
    device_id: String,
    capability_type: String,
) -> Result<serde_json::Value, AppError> {
    info!("Getting capabilities for device {}: {}", device_id, capability_type);

    let agent_path = find_agent_binary()?;

    let command = match capability_type.as_str() {
        "gpu" => "detect-gpu-capabilities",
        "camera" => "detect-camera-capabilities",
        "tpu" => "detect-tpu-capabilities",
        "capture_card" => "detect-capture-card-capabilities",
        "availability" => "detect-availability",
        _ => return Err(AppError::Agent(format!("Unknown capability type: {}", capability_type))),
    };

    let output = Command::new(&agent_path)
        .arg(command)
        .output()
        .map_err(|e| AppError::Agent(format!("Failed to execute agent: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Agent command failed: {}", stderr);
        return Err(AppError::Agent(format!("Agent execution failed: {}", stderr)));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| AppError::Agent(format!("Invalid UTF-8 output: {}", e)))?;

    let result: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| AppError::Config(format!("Failed to parse agent JSON: {}", e)))?;

    Ok(result)
}

/// T035: Get hardware availability status
#[tauri::command]
pub async fn get_hardware_availability() -> Result<serde_json::Value, AppError> {
    info!("Getting hardware availability status");

    let agent_path = find_agent_binary()?;

    let output = Command::new(&agent_path)
        .arg("detect-availability")
        .output()
        .map_err(|e| AppError::Agent(format!("Failed to execute agent: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Agent command failed: {}", stderr);
        return Err(AppError::Agent(format!("Agent execution failed: {}", stderr)));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| AppError::Agent(format!("Invalid UTF-8 output: {}", e)))?;

    let result: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| AppError::Config(format!("Failed to parse agent JSON: {}", e)))?;

    Ok(result)
}

/// T036: Clear hardware detection cache
#[tauri::command]
pub async fn clear_hardware_cache(cache_state: State<'_, CacheState>) -> Result<(), AppError> {
    info!("Clearing hardware detection cache");
    cache_state.clear();
    Ok(())
}

/// T037: Get agent version information
#[tauri::command]
pub async fn get_agent_version() -> Result<String, AppError> {
    info!("Getting agent version");

    let agent_path = find_agent_binary()?;

    let output = Command::new(&agent_path)
        .arg("version")
        .output()
        .map_err(|e| AppError::Agent(format!("Failed to execute agent: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Agent command failed: {}", stderr);
        return Err(AppError::Agent(format!("Agent execution failed: {}", stderr)));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| AppError::Agent(format!("Invalid UTF-8 output: {}", e)))?;

    Ok(stdout.trim().to_string())
}

// T034: Helper function to find agent binary
fn find_agent_binary() -> Result<String, AppError> {
    // Try different possible locations
    let possible_paths = vec![
        "../agent/agent",              // Development (from src-tauri)
        "../../agent/agent",           // From target directory
        "./agent/agent",               // Current directory
        "../src-tauri/bin/agent",      // After build/install
        "agent",                       // In PATH
    ];

    for path in possible_paths {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }

    // Try to find in PATH
    #[cfg(target_os = "windows")]
    let agent_name = "agent.exe";
    #[cfg(not(target_os = "windows"))]
    let agent_name = "agent";

    if let Ok(output) = Command::new("which").arg(agent_name).output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(path);
            }
        }
    }

    Err(AppError::Agent(
        "Agent binary not found. Please build it first: cd agent && go build -o agent ./cmd/agent"
            .to_string(),
    ))
}
