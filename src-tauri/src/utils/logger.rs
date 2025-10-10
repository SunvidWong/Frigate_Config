// Logging utilities
// Provides structured logging for the application

use tracing::{info, warn, error, debug};

/// Log an audit event
/// These are written to both application logs and potentially the audit log table
pub fn log_audit_event(event_type: &str, action: &str, success: bool, details: Option<&str>) {
    if success {
        info!(
            event_type = event_type,
            action = action,
            details = details,
            "Audit event: success"
        );
    } else {
        warn!(
            event_type = event_type,
            action = action,
            details = details,
            "Audit event: failed"
        );
    }
}

/// Log a configuration change
pub fn log_config_change(action: &str, file_path: &str, success: bool) {
    log_audit_event(
        "config_change",
        action,
        success,
        Some(file_path)
    );
}

/// Log a deployment event
pub fn log_deployment(action: &str, container_id: Option<&str>, success: bool) {
    log_audit_event(
        "deployment",
        action,
        success,
        container_id
    );
}

/// Log a hardware detection event
pub fn log_hardware_detection(device_count: usize, platform: &str) {
    info!(
        device_count = device_count,
        platform = platform,
        "Hardware detection completed"
    );
}

/// Log an error with context
pub fn log_error(context: &str, error: &dyn std::error::Error) {
    error!(
        context = context,
        error = %error,
        "Error occurred"
    );
}

/// Log a debug message
pub fn log_debug(message: &str) {
    debug!("{}", message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_functions() {
        // These just ensure the functions compile and don't panic
        log_audit_event("test", "test_action", true, Some("test details"));
        log_config_change("save", "/test/path", true);
        log_deployment("start", Some("container123"), true);
        log_hardware_detection(5, "linux");
        log_debug("test debug message");
    }
}
