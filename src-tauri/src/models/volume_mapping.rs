// T102: VolumeMapping model
// Represents a host-to-container path mapping
// REQUIREMENT: FR-037 (Deployment Module - Volume mappings)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VolumeMappingType {
    Recordings,
    Clips,
    Cache,
    Config,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMapping {
    pub host_path: PathBuf,
    pub container_path: String,
    pub mapping_type: VolumeMappingType,
    pub read_only: bool,
    pub description: Option<String>,
}

impl VolumeMapping {
    pub fn new(
        host_path: PathBuf,
        container_path: String,
        mapping_type: VolumeMappingType,
    ) -> Self {
        Self {
            host_path,
            container_path,
            mapping_type,
            read_only: false,
            description: None,
        }
    }

    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn to_docker_arg(&self) -> String {
        let host = self.host_path.display();
        let container = &self.container_path;

        if self.read_only {
            format!("{}:{}:ro", host, container)
        } else {
            format!("{}:{}", host, container)
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        // Check if host path exists
        if !self.host_path.exists() {
            return Err(format!(
                "Host path does not exist: {}",
                self.host_path.display()
            ));
        }

        // Check if host path is a directory
        if !self.host_path.is_dir() {
            return Err(format!(
                "Host path is not a directory: {}",
                self.host_path.display()
            ));
        }

        // Check container path is absolute
        if !self.container_path.starts_with('/') {
            return Err(format!(
                "Container path must be absolute: {}",
                self.container_path
            ));
        }

        // Check if writable (if not read-only)
        if !self.read_only {
            // Try to write a test file
            let test_file = self.host_path.join(".write_test");
            if std::fs::write(&test_file, "test").is_err() {
                return Err(format!(
                    "Host path is not writable: {}",
                    self.host_path.display()
                ));
            }
            let _ = std::fs::remove_file(test_file);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMappingSet {
    pub mappings: Vec<VolumeMapping>,
}

impl VolumeMappingSet {
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }

    pub fn add(&mut self, mapping: VolumeMapping) {
        self.mappings.push(mapping);
    }

    pub fn add_config(&mut self, host_path: PathBuf) {
        self.add(VolumeMapping::new(
            host_path,
            "/config".to_string(),
            VolumeMappingType::Config,
        ));
    }

    pub fn add_recordings(&mut self, host_path: PathBuf) {
        self.add(VolumeMapping::new(
            host_path,
            "/media/frigate/recordings".to_string(),
            VolumeMappingType::Recordings,
        ));
    }

    pub fn add_clips(&mut self, host_path: PathBuf) {
        self.add(VolumeMapping::new(
            host_path,
            "/media/frigate/clips".to_string(),
            VolumeMappingType::Clips,
        ));
    }

    pub fn add_cache(&mut self, host_path: PathBuf) {
        self.add(VolumeMapping::new(
            host_path,
            "/tmp/cache".to_string(),
            VolumeMappingType::Cache,
        ));
    }

    pub fn to_docker_args(&self) -> Vec<String> {
        self.mappings.iter().map(|m| m.to_docker_arg()).collect()
    }

    pub fn validate_all(&self) -> Result<(), Vec<String>> {
        let errors: Vec<String> = self
            .mappings
            .iter()
            .filter_map(|m| m.validate().err())
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for VolumeMappingSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_new_volume_mapping() {
        let mapping = VolumeMapping::new(
            PathBuf::from("/host/path"),
            "/container/path".to_string(),
            VolumeMappingType::Config,
        );

        assert_eq!(mapping.host_path, PathBuf::from("/host/path"));
        assert_eq!(mapping.container_path, "/container/path");
        assert_eq!(mapping.mapping_type, VolumeMappingType::Config);
        assert!(!mapping.read_only);
    }

    #[test]
    fn test_read_only_mapping() {
        let mapping = VolumeMapping::new(
            PathBuf::from("/host/path"),
            "/container/path".to_string(),
            VolumeMappingType::Config,
        )
        .read_only();

        assert!(mapping.read_only);
    }

    #[test]
    fn test_to_docker_arg_read_write() {
        let mapping = VolumeMapping::new(
            PathBuf::from("/host/config"),
            "/config".to_string(),
            VolumeMappingType::Config,
        );

        assert_eq!(mapping.to_docker_arg(), "/host/config:/config");
    }

    #[test]
    fn test_to_docker_arg_read_only() {
        let mapping = VolumeMapping::new(
            PathBuf::from("/etc/localtime"),
            "/etc/localtime".to_string(),
            VolumeMappingType::Custom,
        )
        .read_only();

        assert_eq!(mapping.to_docker_arg(), "/etc/localtime:/etc/localtime:ro");
    }

    #[test]
    fn test_validate_nonexistent_path() {
        let mapping = VolumeMapping::new(
            PathBuf::from("/nonexistent/path"),
            "/config".to_string(),
            VolumeMappingType::Config,
        );

        let result = mapping.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_validate_valid_path() {
        let temp_dir = TempDir::new().unwrap();
        let mapping = VolumeMapping::new(
            temp_dir.path().to_path_buf(),
            "/config".to_string(),
            VolumeMappingType::Config,
        );

        let result = mapping.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_not_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test").unwrap();

        let mapping = VolumeMapping::new(
            file_path,
            "/config".to_string(),
            VolumeMappingType::Config,
        );

        let result = mapping.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a directory"));
    }

    #[test]
    fn test_volume_mapping_set() {
        let mut set = VolumeMappingSet::new();
        let temp_dir = TempDir::new().unwrap();

        set.add_config(temp_dir.path().join("config"));
        fs::create_dir(temp_dir.path().join("config")).unwrap();

        assert_eq!(set.mappings.len(), 1);
        assert_eq!(set.mappings[0].mapping_type, VolumeMappingType::Config);
    }

    #[test]
    fn test_to_docker_args() {
        let mut set = VolumeMappingSet::new();
        set.add(VolumeMapping::new(
            PathBuf::from("/host/config"),
            "/config".to_string(),
            VolumeMappingType::Config,
        ));
        set.add(
            VolumeMapping::new(
                PathBuf::from("/etc/localtime"),
                "/etc/localtime".to_string(),
                VolumeMappingType::Custom,
            )
            .read_only(),
        );

        let args = set.to_docker_args();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "/host/config:/config");
        assert_eq!(args[1], "/etc/localtime:/etc/localtime:ro");
    }
}
