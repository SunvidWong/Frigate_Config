// HardwareDevice model
// Represents a detected hardware accelerator or video device

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Gpu,
    Tpu,
    Camera,
    CaptureCard,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Linux,
    Windows,
    Darwin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Architecture {
    #[serde(rename = "x86_64")]
    X86_64,
    #[serde(rename = "arm64")]
    Arm64,
    #[serde(rename = "arm32")]
    Arm32,
}

/// Hardware device detected by the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareDevice {
    /// Unique identifier (UUID)
    pub id: String,

    /// Device type
    #[serde(rename = "type")]
    pub device_type: DeviceType,

    /// Human-readable name
    pub name: String,

    /// System device path
    pub device_path: String,

    /// Device capabilities
    pub capabilities: Vec<String>,

    /// Driver name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,

    /// Platform
    pub platform: Platform,

    /// CPU architecture
    pub architecture: Architecture,

    /// Vendor/Product ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_id: Option<String>,

    /// Detection timestamp (ISO 8601)
    pub detected_at: String,

    /// Detection source command/method
    pub detection_source: String,

    /// Whether device is currently accessible
    pub available: bool,

    /// Whether device is assigned to a camera
    #[serde(default)]
    pub in_use: bool,

    /// Error message if detection failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl HardwareDevice {
    /// Create a new hardware device
    pub fn new(
        id: String,
        device_type: DeviceType,
        name: String,
        device_path: String,
        platform: Platform,
        architecture: Architecture,
    ) -> Self {
        Self {
            id,
            device_type,
            name,
            device_path,
            capabilities: Vec::new(),
            driver: None,
            platform,
            architecture,
            vendor_id: None,
            detected_at: chrono::Utc::now().to_rfc3339(),
            detection_source: String::new(),
            available: true,
            in_use: false,
            error: None,
        }
    }
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::Gpu => write!(f, "GPU"),
            DeviceType::Tpu => write!(f, "TPU"),
            DeviceType::Camera => write!(f, "Camera"),
            DeviceType::CaptureCard => write!(f, "Capture Card"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_creation() {
        let device = HardwareDevice::new(
            "test-id".to_string(),
            DeviceType::Gpu,
            "NVIDIA RTX 3060".to_string(),
            "/dev/nvidia0".to_string(),
            Platform::Linux,
            Architecture::X86_64,
        );

        assert_eq!(device.id, "test-id");
        assert_eq!(device.device_type, DeviceType::Gpu);
        assert!(device.available);
        assert!(!device.in_use);
    }

    #[test]
    fn test_device_serialization() {
        let device = HardwareDevice::new(
            "test-id".to_string(),
            DeviceType::Tpu,
            "Coral USB".to_string(),
            "/dev/apex_0".to_string(),
            Platform::Linux,
            Architecture::Arm64,
        );

        let json = serde_json::to_string(&device).unwrap();
        assert!(json.contains("\"type\":\"tpu\""));
        assert!(json.contains("\"platform\":\"linux\""));
    }

    // T021: Extended unit tests for HardwareDevice validation
    // These tests MUST pass after implementation

    #[test]
    fn test_device_type_enum_serialization() {
        assert_eq!(serde_json::to_string(&DeviceType::Gpu).unwrap(), "\"gpu\"");
        assert_eq!(serde_json::to_string(&DeviceType::Tpu).unwrap(), "\"tpu\"");
        assert_eq!(
            serde_json::to_string(&DeviceType::Camera).unwrap(),
            "\"camera\""
        );
        assert_eq!(
            serde_json::to_string(&DeviceType::CaptureCard).unwrap(),
            "\"capturecard\"" // lowercase renaming converts CaptureCard -> capturecard
        );
    }

    #[test]
    fn test_platform_enum_serialization() {
        assert_eq!(
            serde_json::to_string(&Platform::Linux).unwrap(),
            "\"linux\""
        );
        assert_eq!(
            serde_json::to_string(&Platform::Windows).unwrap(),
            "\"windows\""
        );
        assert_eq!(
            serde_json::to_string(&Platform::Darwin).unwrap(),
            "\"darwin\""
        );
    }

    #[test]
    fn test_architecture_enum_serialization() {
        assert_eq!(
            serde_json::to_string(&Architecture::X86_64).unwrap(),
            "\"x86_64\""
        );
        assert_eq!(
            serde_json::to_string(&Architecture::Arm64).unwrap(),
            "\"arm64\""
        );
        assert_eq!(
            serde_json::to_string(&Architecture::Arm32).unwrap(),
            "\"arm32\""
        );
    }

    #[test]
    fn test_device_with_capabilities() {
        let mut device = HardwareDevice::new(
            "gpu-0".to_string(),
            DeviceType::Gpu,
            "NVIDIA RTX 3060".to_string(),
            "/dev/nvidia0".to_string(),
            Platform::Linux,
            Architecture::X86_64,
        );

        device.capabilities = vec![
            "h264_decode".to_string(),
            "h264_encode".to_string(),
            "hevc_decode".to_string(),
        ];

        assert_eq!(device.capabilities.len(), 3);
        assert!(device.capabilities.contains(&"h264_decode".to_string()));
    }

    #[test]
    fn test_device_availability_flags() {
        let mut device = HardwareDevice::new(
            "cam-0".to_string(),
            DeviceType::Camera,
            "USB Camera".to_string(),
            "/dev/video0".to_string(),
            Platform::Linux,
            Architecture::X86_64,
        );

        // Initially available and not in use
        assert!(device.available);
        assert!(!device.in_use);

        // Mark as in use
        device.in_use = true;
        assert!(device.in_use);

        // Mark as unavailable
        device.available = false;
        assert!(!device.available);
    }

    #[test]
    fn test_device_with_error() {
        let mut device = HardwareDevice::new(
            "gpu-0".to_string(),
            DeviceType::Gpu,
            "Unknown GPU".to_string(),
            "/dev/unknown".to_string(),
            Platform::Linux,
            Architecture::X86_64,
        );

        device.error = Some("Driver not loaded".to_string());
        assert!(device.error.is_some());
        assert_eq!(device.error.unwrap(), "Driver not loaded");
    }

    #[test]
    fn test_device_full_deserialization() {
        let json = r#"{
            "id": "gpu-1",
            "type": "gpu",
            "name": "Intel UHD Graphics",
            "device_path": "/dev/dri/renderD128",
            "capabilities": ["vaapi"],
            "driver": "i915",
            "platform": "linux",
            "architecture": "x86_64",
            "vendor_id": "8086",
            "detected_at": "2025-01-09T10:00:00Z",
            "detection_source": "lspci",
            "available": true,
            "in_use": false,
            "error": null
        }"#;

        let device: HardwareDevice = serde_json::from_str(json).unwrap();
        assert_eq!(device.id, "gpu-1");
        assert_eq!(device.device_type, DeviceType::Gpu);
        assert_eq!(device.name, "Intel UHD Graphics");
        assert_eq!(device.driver, Some("i915".to_string()));
        assert_eq!(device.vendor_id, Some("8086".to_string()));
    }

    #[test]
    fn test_device_display_trait() {
        assert_eq!(DeviceType::Gpu.to_string(), "GPU");
        assert_eq!(DeviceType::Tpu.to_string(), "TPU");
        assert_eq!(DeviceType::Camera.to_string(), "Camera");
        assert_eq!(DeviceType::CaptureCard.to_string(), "Capture Card");
    }
}
