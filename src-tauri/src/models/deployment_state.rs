// T101: DeploymentState model
// Represents the current deployment status
// REQUIREMENT: FR-032, FR-038, FR-040 (Deployment Module - State management)

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatus {
    Pending,
    Running,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentState {
    pub id: String,
    pub status: DeploymentStatus,
    pub container_id: Option<String>,
    pub container_name: String,
    pub config_path: String,
    pub deployment_method: DeploymentMethod,
    pub start_time: Option<SystemTime>,
    pub end_time: Option<SystemTime>,
    pub logs: Vec<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentMethod {
    DockerRun,
    DockerCompose,
}

impl DeploymentState {
    pub fn new(container_name: String, config_path: String, method: DeploymentMethod) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            status: DeploymentStatus::Pending,
            container_id: None,
            container_name,
            config_path,
            deployment_method: method,
            start_time: None,
            end_time: None,
            logs: Vec::new(),
            error_message: None,
        }
    }

    pub fn mark_running(&mut self, container_id: String) {
        self.status = DeploymentStatus::Running;
        self.container_id = Some(container_id);
        self.start_time = Some(SystemTime::now());
    }

    pub fn mark_completed(&mut self) {
        self.status = DeploymentStatus::Completed;
        self.end_time = Some(SystemTime::now());
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = DeploymentStatus::Failed;
        self.error_message = Some(error);
        self.end_time = Some(SystemTime::now());
    }

    pub fn mark_rolled_back(&mut self) {
        self.status = DeploymentStatus::RolledBack;
        self.end_time = Some(SystemTime::now());
    }

    pub fn add_log(&mut self, log_line: String) {
        self.logs.push(log_line);
    }

    pub fn is_running(&self) -> bool {
        self.status == DeploymentStatus::Running
    }

    pub fn is_completed(&self) -> bool {
        matches!(
            self.status,
            DeploymentStatus::Completed | DeploymentStatus::RolledBack
        )
    }

    pub fn is_failed(&self) -> bool {
        self.status == DeploymentStatus::Failed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_deployment_state() {
        let state = DeploymentState::new(
            "frigate".to_string(),
            "/config/frigate.yml".to_string(),
            DeploymentMethod::DockerRun,
        );

        assert_eq!(state.status, DeploymentStatus::Pending);
        assert!(state.container_id.is_none());
        assert_eq!(state.container_name, "frigate");
        assert!(state.logs.is_empty());
    }

    #[test]
    fn test_mark_running() {
        let mut state = DeploymentState::new(
            "frigate".to_string(),
            "/config/frigate.yml".to_string(),
            DeploymentMethod::DockerRun,
        );

        state.mark_running("container_123".to_string());

        assert_eq!(state.status, DeploymentStatus::Running);
        assert_eq!(state.container_id, Some("container_123".to_string()));
        assert!(state.start_time.is_some());
        assert!(state.is_running());
    }

    #[test]
    fn test_mark_completed() {
        let mut state = DeploymentState::new(
            "frigate".to_string(),
            "/config/frigate.yml".to_string(),
            DeploymentMethod::DockerRun,
        );

        state.mark_running("container_123".to_string());
        state.mark_completed();

        assert_eq!(state.status, DeploymentStatus::Completed);
        assert!(state.end_time.is_some());
        assert!(state.is_completed());
    }

    #[test]
    fn test_mark_failed() {
        let mut state = DeploymentState::new(
            "frigate".to_string(),
            "/config/frigate.yml".to_string(),
            DeploymentMethod::DockerRun,
        );

        state.mark_failed("Docker not available".to_string());

        assert_eq!(state.status, DeploymentStatus::Failed);
        assert_eq!(
            state.error_message,
            Some("Docker not available".to_string())
        );
        assert!(state.is_failed());
    }

    #[test]
    fn test_add_log() {
        let mut state = DeploymentState::new(
            "frigate".to_string(),
            "/config/frigate.yml".to_string(),
            DeploymentMethod::DockerRun,
        );

        state.add_log("Starting deployment...".to_string());
        state.add_log("Container created".to_string());

        assert_eq!(state.logs.len(), 2);
        assert_eq!(state.logs[0], "Starting deployment...");
        assert_eq!(state.logs[1], "Container created");
    }
}
