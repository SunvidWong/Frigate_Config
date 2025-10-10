// CameraConfiguration model
// Represents a single camera's complete configuration for Frigate

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HWAccelType {
    Vaapi,
    Cuda,
    Qsv,
    Videotoolbox,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    Valid,
    Warning,
    Error,
}

/// Camera configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfiguration {
    /// Unique identifier
    pub id: String,

    /// Camera name (e.g., "front_door")
    pub name: String,

    /// Whether camera is enabled
    pub enabled: bool,

    /// RTSP stream URL (with credentials)
    pub rtsp_url: String,

    /// RTSP URL for display (credentials masked)
    pub rtsp_url_display: String,

    /// Video resolution
    pub resolution: Resolution,

    /// Target FPS (1-60)
    pub fps: u8,

    /// Assigned hardware device ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_device_id: Option<String>,

    /// Hardware acceleration type (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hwaccel: Option<HWAccelType>,

    /// Detection enabled
    #[serde(default = "default_true")]
    pub detect_enabled: bool,

    /// Recording enabled
    #[serde(default = "default_true")]
    pub record_enabled: bool,

    /// Snapshots enabled
    #[serde(default = "default_true")]
    pub snapshots_enabled: bool,

    /// Custom FFmpeg arguments (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_ffmpeg_args: Option<String>,

    /// Creation timestamp
    pub created_at: String,

    /// Last update timestamp
    pub updated_at: String,

    /// Whether manually edited in YAML
    #[serde(default)]
    pub manually_edited: bool,

    /// Validation status
    pub validation_status: ValidationStatus,

    /// Validation error messages
    #[serde(default)]
    pub validation_errors: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl CameraConfiguration {
    /// Create a new camera configuration
    pub fn new(id: String, name: String, rtsp_url: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();

        Self {
            id,
            name: name.clone(),
            enabled: true,
            rtsp_url: rtsp_url.clone(),
            rtsp_url_display: Self::mask_credentials(&rtsp_url),
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
            fps: 15,
            hardware_device_id: None,
            hwaccel: None,
            detect_enabled: true,
            record_enabled: true,
            snapshots_enabled: true,
            custom_ffmpeg_args: None,
            created_at: now.clone(),
            updated_at: now,
            manually_edited: false,
            validation_status: ValidationStatus::Valid,
            validation_errors: Vec::new(),
        }
    }

    /// Mask credentials in RTSP URL for display
    fn mask_credentials(url: &str) -> String {
        if let Some(at_pos) = url.find('@') {
            if let Some(proto_end) = url.find("://") {
                let proto = &url[..proto_end + 3];
                let after_at = &url[at_pos..];
                return format!("{}***{}", proto, after_at);
            }
        }
        url.to_string()
    }

    /// Validate camera configuration
    pub fn validate(&mut self) {
        self.validation_errors.clear();

        // Validate name
        if self.name.is_empty() || self.name.len() > 50 {
            self.validation_errors
                .push("Name must be 1-50 characters".to_string());
        }

        // Validate RTSP URL
        if !self.rtsp_url.starts_with("rtsp://") {
            self.validation_errors
                .push("RTSP URL must start with rtsp://".to_string());
        }

        // Validate resolution
        if self.resolution.width % 2 != 0 || self.resolution.height % 2 != 0 {
            self.validation_errors
                .push("Resolution width and height must be even numbers".to_string());
        }

        // Validate FPS
        if self.fps < 1 || self.fps > 60 {
            self.validation_errors
                .push("FPS must be between 1 and 60".to_string());
        }

        // Update validation status
        self.validation_status = if self.validation_errors.is_empty() {
            ValidationStatus::Valid
        } else {
            ValidationStatus::Error
        };

        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_credentials() {
        let url = "rtsp://admin:password@192.168.1.100:554/stream";
        let masked = CameraConfiguration::mask_credentials(url);
        assert_eq!(masked, "rtsp://***@192.168.1.100:554/stream");
    }

    #[test]
    fn test_camera_validation() {
        let mut camera = CameraConfiguration::new(
            "test-id".to_string(),
            "front_door".to_string(),
            "rtsp://camera.local/stream".to_string(),
        );

        camera.validate();
        assert_eq!(camera.validation_status, ValidationStatus::Valid);
        assert!(camera.validation_errors.is_empty());

        // Test invalid FPS
        camera.fps = 100;
        camera.validate();
        assert_eq!(camera.validation_status, ValidationStatus::Error);
        assert!(camera.validation_errors.iter().any(|e| e.contains("FPS")));
    }

    // T022: Extended unit tests for CameraConfiguration
    // These tests validate camera configuration logic

    #[test]
    fn test_camera_creation() {
        let camera = CameraConfiguration::new(
            "cam-1".to_string(),
            "backyard".to_string(),
            "rtsp://admin:pass@192.168.1.100:554/stream1".to_string(),
        );

        assert_eq!(camera.id, "cam-1");
        assert_eq!(camera.name, "backyard");
        assert!(camera.enabled);
        assert_eq!(camera.fps, 15); // Default FPS
        assert_eq!(camera.validation_status, ValidationStatus::Valid);
    }

    #[test]
    fn test_credential_masking_various_formats() {
        // Standard RTSP URL
        let url1 = "rtsp://user:pass@192.168.1.100:554/stream";
        assert_eq!(
            CameraConfiguration::mask_credentials(url1),
            "rtsp://***@192.168.1.100:554/stream"
        );

        // URL with special characters in password (but no @ in password)
        let url2 = "rtsp://admin:passw0rd!@camera.local/live";
        assert_eq!(
            CameraConfiguration::mask_credentials(url2),
            "rtsp://***@camera.local/live"
        );

        // URL without credentials
        let url3 = "rtsp://192.168.1.100/stream";
        assert_eq!(
            CameraConfiguration::mask_credentials(url3),
            "rtsp://192.168.1.100/stream"
        );
    }

    #[test]
    fn test_validation_invalid_rtsp_url() {
        let mut camera = CameraConfiguration::new(
            "cam-1".to_string(),
            "test_cam".to_string(),
            "http://invalid.url/stream".to_string(), // Invalid protocol
        );

        camera.validate();
        assert_eq!(camera.validation_status, ValidationStatus::Error);
        assert!(camera
            .validation_errors
            .contains(&"RTSP URL must start with rtsp://".to_string()));
    }

    #[test]
    fn test_validation_empty_camera_name() {
        let mut camera = CameraConfiguration::new(
            "cam-1".to_string(),
            "".to_string(), // Empty name
            "rtsp://camera.local/stream".to_string(),
        );

        camera.validate();
        assert_eq!(camera.validation_status, ValidationStatus::Error);
        assert!(camera
            .validation_errors
            .iter()
            .any(|e| e.contains("Name")));
    }

    #[test]
    fn test_validation_odd_resolution() {
        let mut camera = CameraConfiguration::new(
            "cam-1".to_string(),
            "test_cam".to_string(),
            "rtsp://camera.local/stream".to_string(),
        );

        // Set odd resolution (invalid for video encoding)
        camera.resolution = Resolution {
            width: 1921,  // Odd width
            height: 1080,
        };

        camera.validate();
        assert_eq!(camera.validation_status, ValidationStatus::Error);
        assert!(camera.validation_errors.iter().any(|e| e.contains("even")));
    }

    #[test]
    fn test_validation_invalid_fps() {
        let mut camera = CameraConfiguration::new(
            "cam-1".to_string(),
            "test_cam".to_string(),
            "rtsp://camera.local/stream".to_string(),
        );

        // Test FPS too low
        camera.fps = 0;
        camera.validate();
        assert_eq!(camera.validation_status, ValidationStatus::Error);
        assert!(camera.validation_errors.iter().any(|e| e.contains("FPS")));

        // Test FPS too high
        camera.validation_errors.clear();
        camera.fps = 100;
        camera.validate();
        assert_eq!(camera.validation_status, ValidationStatus::Error);
        assert!(camera.validation_errors.iter().any(|e| e.contains("FPS")));
    }

    #[test]
    fn test_validation_multiple_errors() {
        let mut camera = CameraConfiguration::new(
            "cam-1".to_string(),
            "".to_string(), // Empty ID
            "http://invalid".to_string(), // Invalid protocol
        );

        camera.fps = 0; // Invalid FPS
        camera.resolution = Resolution {
            width: 1921,  // Odd width
            height: 1081, // Odd height
        };

        camera.validate();
        assert_eq!(camera.validation_status, ValidationStatus::Error);
        assert!(camera.validation_errors.len() >= 3); // Multiple errors
    }

    #[test]
    fn test_camera_serialization() {
        let camera = CameraConfiguration::new(
            "cam-1".to_string(),
            "test_cam".to_string(),
            "rtsp://camera.local/stream".to_string(),
        );

        let json = serde_json::to_string(&camera).unwrap();
        assert!(json.contains("\"name\":\"test_cam\""));
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"rtsp_url\":\"rtsp://camera.local/stream\""));
    }

    #[test]
    fn test_camera_deserialization() {
        let json = r#"{
            "id": "cam-1",
            "name": "garage",
            "rtsp_url": "rtsp://camera.local/stream",
            "rtsp_url_display": "rtsp://***@camera.local/stream",
            "enabled": true,
            "hardware_device_id": null,
            "hwaccel": null,
            "resolution": {
                "width": 1920,
                "height": 1080
            },
            "fps": 10,
            "detect_enabled": true,
            "record_enabled": true,
            "snapshots_enabled": true,
            "custom_ffmpeg_args": null,
            "created_at": "2025-01-09T10:00:00Z",
            "updated_at": "2025-01-09T10:00:00Z",
            "manually_edited": false,
            "validation_status": "valid",
            "validation_errors": []
        }"#;

        let camera: CameraConfiguration = serde_json::from_str(json).unwrap();
        assert_eq!(camera.name, "garage");
        assert_eq!(camera.fps, 10);
        assert_eq!(camera.resolution.width, 1920);
        assert_eq!(camera.validation_status, ValidationStatus::Valid);
    }

    #[test]
    fn test_validation_status_enum() {
        assert_eq!(
            serde_json::to_string(&ValidationStatus::Valid).unwrap(),
            "\"valid\""
        );
        assert_eq!(
            serde_json::to_string(&ValidationStatus::Error).unwrap(),
            "\"error\""
        );
        assert_eq!(
            serde_json::to_string(&ValidationStatus::Warning).unwrap(),
            "\"warning\""
        );
    }

    #[test]
    fn test_hardware_assignment() {
        let mut camera = CameraConfiguration::new(
            "cam-1".to_string(),
            "test_cam".to_string(),
            "rtsp://camera.local/stream".to_string(),
        );

        // Initially no hardware assigned
        assert!(camera.hardware_device_id.is_none());

        // Assign hardware
        camera.hardware_device_id = Some("gpu-0".to_string());
        assert_eq!(camera.hardware_device_id, Some("gpu-0".to_string()));
    }

    #[test]
    fn test_detection_and_recording_flags() {
        let mut camera = CameraConfiguration::new(
            "cam-1".to_string(),
            "test_cam".to_string(),
            "rtsp://camera.local/stream".to_string(),
        );

        // Default values
        assert!(camera.detect_enabled);
        assert!(camera.record_enabled);

        // Toggle flags
        camera.detect_enabled = false;
        camera.record_enabled = false;
        assert!(!camera.detect_enabled);
        assert!(!camera.record_enabled);
    }
}
