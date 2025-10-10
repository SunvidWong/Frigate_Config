// ConfigurationSnapshot model
// Represents a complete Frigate configuration at a specific point in time

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CreationSource {
    Ui,
    Template,
    ManualEdit,
    Rollback,
}

/// Configuration snapshot (backup)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationSnapshot {
    /// Unique identifier
    pub id: String,

    /// Sequential version number
    pub version: i32,

    /// Full YAML configuration text
    pub yaml_content: String,

    /// SHA-256 checksum of yaml_content
    pub checksum: String,

    /// Creation timestamp
    pub created_at: String,

    /// How this snapshot was created
    pub created_by: CreationSource,

    /// User-provided description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether this is an automatic backup
    pub is_backup: bool,

    /// Reason for backup (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_reason: Option<String>,

    /// Whether this config was ever deployed
    #[serde(default)]
    pub deployed: bool,

    /// Deployment timestamp (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployed_at: Option<String>,

    /// Whether deployment was successful (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_success: Option<bool>,
}

impl ConfigurationSnapshot {
    /// Create a new configuration snapshot
    pub fn new(
        id: String,
        version: i32,
        yaml_content: String,
        created_by: CreationSource,
    ) -> Self {
        let checksum = Self::calculate_checksum(&yaml_content);

        Self {
            id,
            version,
            yaml_content,
            checksum,
            created_at: chrono::Utc::now().to_rfc3339(),
            created_by,
            description: None,
            is_backup: false,
            backup_reason: None,
            deployed: false,
            deployed_at: None,
            deployment_success: None,
        }
    }

    /// Calculate SHA-256 checksum of content
    pub fn calculate_checksum(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify checksum matches content
    pub fn verify_checksum(&self) -> bool {
        self.checksum == Self::calculate_checksum(&self.yaml_content)
    }

    /// Mark as backup
    pub fn mark_as_backup(&mut self, reason: &str) {
        self.is_backup = true;
        self.backup_reason = Some(reason.to_string());
    }

    /// Mark as deployed
    pub fn mark_as_deployed(&mut self, success: bool) {
        self.deployed = true;
        self.deployed_at = Some(chrono::Utc::now().to_rfc3339());
        self.deployment_success = Some(success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let yaml = "mqtt:\n  enabled: true\n  host: mqtt.local";
        let snapshot = ConfigurationSnapshot::new(
            "test-id".to_string(),
            1,
            yaml.to_string(),
            CreationSource::Ui,
        );

        assert_eq!(snapshot.version, 1);
        assert!(!snapshot.deployed);
        assert!(!snapshot.is_backup);
    }

    #[test]
    fn test_checksum_verification() {
        let yaml = "cameras:\n  front_door:\n    enabled: true";
        let snapshot = ConfigurationSnapshot::new(
            "test-id".to_string(),
            1,
            yaml.to_string(),
            CreationSource::Ui,
        );

        assert!(snapshot.verify_checksum());
    }

    #[test]
    fn test_mark_as_backup() {
        let yaml = "test config";
        let mut snapshot = ConfigurationSnapshot::new(
            "test-id".to_string(),
            1,
            yaml.to_string(),
            CreationSource::Ui,
        );

        snapshot.mark_as_backup("pre_deploy");
        assert!(snapshot.is_backup);
        assert_eq!(snapshot.backup_reason, Some("pre_deploy".to_string()));
    }
}
