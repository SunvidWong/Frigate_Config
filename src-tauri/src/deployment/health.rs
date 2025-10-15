// T113-T115: Health check and retry logic for deployments
// Monitors container health, Frigate API, and implements retry with exponential backoff
// REQUIREMENT: FR-039 (Deployment Module - Health checks after deployment)

use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};

/// Health status of a deployment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Starting,
    Unknown,
}

/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub details: Option<String>,
}

/// Complete health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub checks: Vec<HealthCheck>,
    pub timestamp: SystemTime,
    pub response_time_ms: u64,
}

/// Configuration for health check with retry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub endpoint_url: String,
    pub timeout: Duration,
    pub retry_count: u32,
    pub retry_delay: Duration,
    pub expected_status_code: u16,
}

impl HealthCheckResult {
    pub fn new(status: HealthStatus) -> Self {
        Self {
            status,
            checks: Vec::new(),
            timestamp: SystemTime::now(),
            response_time_ms: 0,
        }
    }

    pub fn add_check(
        &mut self,
        name: String,
        passed: bool,
        message: String,
        details: Option<String>,
    ) {
        self.checks.push(HealthCheck {
            name,
            passed,
            message,
            details,
        });
    }
}

// ========== Container Health Checks (T114) ==========

/// Check the health status of a Docker container
pub fn check_container_health(container_id: &str) -> Result<HealthStatus, String> {
    let output = Command::new("docker")
        .args([
            "inspect",
            "--format",
            "{{.State.Health.Status}}",
            container_id,
        ])
        .output()
        .map_err(|e| format!("Failed to inspect container: {}", e))?;

    if !output.status.success() {
        // Container might not have health check defined, check if it's running
        return check_container_running(container_id).map(|running| {
            if running {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unknown
            }
        });
    }

    let health_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    match health_str.as_str() {
        "healthy" => Ok(HealthStatus::Healthy),
        "unhealthy" => Ok(HealthStatus::Unhealthy),
        "starting" => Ok(HealthStatus::Starting),
        "" => {
            // No health check defined, check if running
            check_container_running(container_id).map(|running| {
                if running {
                    HealthStatus::Healthy
                } else {
                    HealthStatus::Unknown
                }
            })
        }
        _ => Ok(HealthStatus::Unknown),
    }
}

/// Check if a container is currently running
pub fn check_container_running(container_id: &str) -> Result<bool, String> {
    let output = Command::new("docker")
        .args(["inspect", "--format", "{{.State.Running}}", container_id])
        .output()
        .map_err(|e| format!("Failed to check container status: {}", e))?;

    if !output.status.success() {
        return Err(format!("Container not found: {}", container_id));
    }

    let running_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(running_str == "true")
}

// ========== Port Response Check ==========

/// Check if a port is responding (TCP connection test)
pub fn check_port_responding(host: &str, port: u16, timeout: Duration) -> Result<bool, String> {
    if port == 0 {
        return Err("Port 0 is invalid".to_string());
    }

    let addr = format!("{}:{}", host, port);

    // Set timeout for connection attempt
    let start = Instant::now();

    match TcpStream::connect_timeout(
        &addr
            .parse()
            .map_err(|e| format!("Invalid address: {}", e))?,
        timeout,
    ) {
        Ok(_) => {
            let elapsed = start.elapsed();
            if elapsed > timeout {
                Ok(false)
            } else {
                Ok(true)
            }
        }
        Err(_) => Ok(false),
    }
}

// ========== Frigate API Health Checks ==========

/// Check Frigate API health (basic endpoint check)
pub fn check_frigate_api(endpoint: &str) -> Result<HealthCheckResult, String> {
    let start = Instant::now();

    // Try to make HTTP request (using curl for now)
    let output = Command::new("curl")
        .args([
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "--connect-timeout",
            "5",
            "--max-time",
            "10",
            endpoint,
        ])
        .output()
        .map_err(|e| format!("Failed to check API: {}", e))?;

    let elapsed = start.elapsed();
    let status_code_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let status_code: u16 = status_code_str.parse().unwrap_or(0);

    // Connection failed - return error instead of Ok with status
    if status_code == 0 || status_code_str == "000" {
        return Err("Failed to connect to API endpoint".to_string());
    }

    let mut result = HealthCheckResult::new(if status_code == 200 {
        HealthStatus::Healthy
    } else {
        HealthStatus::Unknown
    });

    result.response_time_ms = elapsed.as_millis() as u64;
    result.add_check(
        "API Endpoint".to_string(),
        status_code == 200,
        format!("HTTP status: {}", status_code),
        Some(format!("Response time: {}ms", result.response_time_ms)),
    );

    Ok(result)
}

/// Check Frigate API with retry logic and exponential backoff (T115)
pub fn check_frigate_api_with_retry(
    config: &HealthCheckConfig,
) -> Result<HealthCheckResult, String> {
    let mut last_error = String::new();
    let mut retry_delay = config.retry_delay;

    for attempt in 0..=config.retry_count {
        if attempt > 0 {
            std::thread::sleep(retry_delay);
            // Exponential backoff: double the delay each time
            retry_delay *= 2;
        }

        match check_frigate_api(&config.endpoint_url) {
            Ok(result) => {
                if result.status == HealthStatus::Healthy {
                    return Ok(result);
                }
                last_error = "API returned unhealthy status".to_string();
            }
            Err(e) => {
                last_error = e;
            }
        }
    }

    Err(format!(
        "API check failed after {} retry attempts (timeout): {}",
        config.retry_count, last_error
    ))
}

/// Check if Frigate config is loaded
pub fn check_frigate_config_loaded(endpoint: &str) -> Result<bool, String> {
    let output = Command::new("curl")
        .args([
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "--connect-timeout",
            "5",
            endpoint,
        ])
        .output()
        .map_err(|e| format!("Failed to check config endpoint: {}", e))?;

    let status_code = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(status_code == "200")
}

/// Check which cameras are initialized in Frigate
pub fn check_cameras_initialized(endpoint: &str) -> Result<Vec<String>, String> {
    // For now, return empty vec - would need JSON parsing in real implementation
    // This would call /api/config and parse the cameras section
    let _output = Command::new("curl")
        .args(["-s", endpoint])
        .output()
        .map_err(|e| format!("Failed to check cameras: {}", e))?;

    // Placeholder: would parse JSON response here
    Ok(Vec::new())
}

// ========== Wait for Healthy (T113) ==========

/// Wait for container to become healthy with timeout
pub fn wait_for_healthy(
    container_id: &str,
    timeout: Duration,
    check_interval: Duration,
) -> Result<bool, String> {
    let start = Instant::now();

    loop {
        if start.elapsed() >= timeout {
            return Ok(false); // Timeout
        }

        match check_container_health(container_id) {
            Ok(HealthStatus::Healthy) => return Ok(true),
            Ok(HealthStatus::Unhealthy) => return Ok(false),
            Ok(_) => {
                // Starting or Unknown - keep waiting
                std::thread::sleep(check_interval);
            }
            Err(e) => {
                // Error checking health - might be transient
                if start.elapsed() >= timeout {
                    return Err(format!("Timeout while checking health: {}", e));
                }
                std::thread::sleep(check_interval);
            }
        }
    }
}

// ========== Comprehensive Health Check ==========

/// Perform comprehensive health check covering all aspects
pub fn perform_comprehensive_health_check(container_id: &str) -> Result<HealthCheckResult, String> {
    let start = Instant::now();
    let mut result = HealthCheckResult::new(HealthStatus::Healthy);

    // 1. Check container is running
    match check_container_running(container_id) {
        Ok(true) => {
            result.add_check(
                "container running".to_string(),
                true,
                "Container is running".to_string(),
                None,
            );
        }
        Ok(false) => {
            result.status = HealthStatus::Unhealthy;
            result.add_check(
                "container running".to_string(),
                false,
                "Container is not running".to_string(),
                None,
            );
            result.response_time_ms = start.elapsed().as_millis() as u64;
            return Ok(result);
        }
        Err(e) => {
            result.status = HealthStatus::Unknown;
            result.add_check(
                "container running".to_string(),
                false,
                format!("Failed to check container: {}", e),
                None,
            );
        }
    }

    // 2. Check container health status
    match check_container_health(container_id) {
        Ok(health_status) => {
            let passed = health_status == HealthStatus::Healthy;
            if !passed && result.status == HealthStatus::Healthy {
                result.status = health_status.clone();
            }
            result.add_check(
                "container health".to_string(),
                passed,
                format!("Health status: {:?}", health_status),
                None,
            );
        }
        Err(e) => {
            result.add_check(
                "container health".to_string(),
                false,
                format!("Health check failed: {}", e),
                None,
            );
        }
    }

    // 3. Check port 5000 is responding (default Frigate port)
    match check_port_responding("localhost", 5000, Duration::from_secs(2)) {
        Ok(true) => {
            result.add_check(
                "port 5000".to_string(),
                true,
                "Port is responding".to_string(),
                None,
            );
        }
        Ok(false) => {
            result.add_check(
                "port 5000".to_string(),
                false,
                "Port is not responding".to_string(),
                None,
            );
        }
        Err(e) => {
            result.add_check(
                "port 5000".to_string(),
                false,
                format!("Port check failed: {}", e),
                None,
            );
        }
    }

    // 4. Check Frigate API
    match check_frigate_api("http://localhost:5000/api") {
        Ok(api_result) => {
            let passed = api_result.status == HealthStatus::Healthy;
            result.add_check(
                "frigate api".to_string(),
                passed,
                format!("API status: {:?}", api_result.status),
                Some(format!("Response time: {}ms", api_result.response_time_ms)),
            );
        }
        Err(e) => {
            result.add_check(
                "frigate api".to_string(),
                false,
                format!("API check failed: {}", e),
                None,
            );
        }
    }

    result.response_time_ms = start.elapsed().as_millis() as u64;

    // Determine overall status based on all checks
    let all_passed = result.checks.iter().all(|c| c.passed);
    if !all_passed && result.status == HealthStatus::Healthy {
        result.status = HealthStatus::Unhealthy;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus::Healthy;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Healthy"));
    }

    #[test]
    fn test_health_check_result_creation() {
        let mut result = HealthCheckResult::new(HealthStatus::Starting);
        assert_eq!(result.status, HealthStatus::Starting);
        assert_eq!(result.checks.len(), 0);

        result.add_check("Test".to_string(), true, "Test passed".to_string(), None);

        assert_eq!(result.checks.len(), 1);
        assert!(result.checks[0].passed);
    }

    #[test]
    fn test_port_validation() {
        let result = check_port_responding("localhost", 0, Duration::from_secs(1));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Port 0"));
    }

    #[test]
    fn test_health_check_config() {
        let config = HealthCheckConfig {
            endpoint_url: "http://localhost:5000/api".to_string(),
            timeout: Duration::from_secs(5),
            retry_count: 3,
            retry_delay: Duration::from_secs(1),
            expected_status_code: 200,
        };

        assert_eq!(config.retry_count, 3);
        assert_eq!(config.expected_status_code, 200);
    }
}
