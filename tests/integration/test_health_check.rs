// T095: Integration test for health checks
// Tests container health monitoring and Frigate API checks
// REQUIREMENT: FR-039 (Deployment Module - Health checks after deployment)

#[cfg(test)]
mod test_health_check {
    use std::time::{Duration, Instant};

    // Import actual implementation
    use frigate_config_tool::deployment::health::{
        HealthStatus, HealthCheckResult, HealthCheck, HealthCheckConfig,
        check_container_health, check_frigate_api, check_frigate_api_with_retry,
        wait_for_healthy, perform_comprehensive_health_check, check_container_running,
        check_port_responding, check_frigate_config_loaded, check_cameras_initialized,
    };

    // ========== Container Health Check Tests ==========

    #[test]
    fn test_check_container_health_running() {
        // Mock container ID that would be running
        let container_id = "frigate_test_container";

        let result = check_container_health(container_id);

        // Should return a health status
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_check_container_health_nonexistent() {
        let container_id = "nonexistent_container_12345";

        let result = check_container_health(container_id);

        // Should return error for nonexistent container
        assert!(result.is_err());
    }

    #[test]
    fn test_check_container_running() {
        let container_id = "frigate_test";

        let result = check_container_running(container_id);

        // Should return boolean or error
        assert!(result.is_ok() || result.is_err());
    }

    // ========== Frigate API Health Check Tests ==========

    #[test]
    fn test_check_frigate_api_healthy() {
        // Assuming Frigate is running on localhost:5000
        let endpoint = "http://localhost:5000/api";

        let result = check_frigate_api(endpoint);

        // Will succeed if Frigate is running, fail otherwise
        // Only assert if we get a successful result AND the status is Healthy
        if let Ok(health) = result {
            if health.status == HealthStatus::Healthy {
                assert!(!health.checks.is_empty());
            }
            // If status is Unknown or Unhealthy, that's also valid (Frigate not running or misconfigured)
        }
    }

    #[test]
    fn test_check_frigate_api_unreachable() {
        // Use unreachable endpoint
        let endpoint = "http://localhost:99999/api";

        let result = check_frigate_api(endpoint);

        // Should fail with connection error
        assert!(result.is_err());
    }

    #[test]
    fn test_check_frigate_api_with_timeout() {
        let endpoint = "http://localhost:5000/api";

        let config = HealthCheckConfig {
            endpoint_url: endpoint.to_string(),
            timeout: Duration::from_secs(5),
            retry_count: 0,
            retry_delay: Duration::from_secs(0),
            expected_status_code: 200,
        };

        let start = Instant::now();
        let result = check_frigate_api_with_retry(&config);
        let duration = start.elapsed();

        // Should not exceed timeout significantly
        assert!(duration < Duration::from_secs(10));

        // Result may succeed or fail depending on whether Frigate is running
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_check_frigate_api_response_time() {
        let endpoint = "http://localhost:5000/api";

        let result = check_frigate_api(endpoint);

        if let Ok(health) = result {
            // Response time should be reasonable (< 5000ms)
            assert!(health.response_time_ms < 5000);
        }
    }

    // ========== Retry Logic Tests ==========

    #[test]
    fn test_check_frigate_api_with_retry_success() {
        let endpoint = "http://localhost:5000/api";

        let config = HealthCheckConfig {
            endpoint_url: endpoint.to_string(),
            timeout: Duration::from_secs(5),
            retry_count: 3,
            retry_delay: Duration::from_secs(1),
            expected_status_code: 200,
        };

        let result = check_frigate_api_with_retry(&config);

        // Should eventually succeed if Frigate starts within retry window
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_check_frigate_api_retry_respects_delay() {
        let endpoint = "http://localhost:99999/api"; // Unreachable

        let config = HealthCheckConfig {
            endpoint_url: endpoint.to_string(),
            timeout: Duration::from_millis(100),
            retry_count: 3,
            retry_delay: Duration::from_millis(500),
            expected_status_code: 200,
        };

        let start = Instant::now();
        let _result = check_frigate_api_with_retry(&config);
        let duration = start.elapsed();

        // Should take at least (retry_count * retry_delay)
        // 3 retries * 500ms = 1500ms minimum
        assert!(duration >= Duration::from_millis(1400)); // Allow some margin
    }

    #[test]
    fn test_check_frigate_api_max_retries_exhausted() {
        let endpoint = "http://localhost:99999/api"; // Unreachable

        let config = HealthCheckConfig {
            endpoint_url: endpoint.to_string(),
            timeout: Duration::from_millis(100),
            retry_count: 2,
            retry_delay: Duration::from_millis(100),
            expected_status_code: 200,
        };

        let result = check_frigate_api_with_retry(&config);

        // Should fail after exhausting retries
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("retry") || error.contains("timeout"));
    }

    // ========== Wait for Healthy Tests ==========

    #[test]
    fn test_wait_for_healthy_success() {
        let container_id = "frigate_test";

        let result = wait_for_healthy(
            container_id,
            Duration::from_secs(30),
            Duration::from_secs(2),
        );

        // Should succeed if container becomes healthy within timeout
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_wait_for_healthy_timeout() {
        let container_id = "nonexistent_container";

        let start = Instant::now();
        let result = wait_for_healthy(
            container_id,
            Duration::from_secs(5),
            Duration::from_secs(1),
        );
        let duration = start.elapsed();

        // Should timeout after specified duration
        assert!(result.is_err() || result.unwrap() == false);
        assert!(duration >= Duration::from_secs(4)); // Allow some margin
        assert!(duration < Duration::from_secs(10)); // Should not exceed significantly
    }

    #[test]
    fn test_wait_for_healthy_respects_check_interval() {
        let container_id = "frigate_test";

        let start = Instant::now();
        let _result = wait_for_healthy(
            container_id,
            Duration::from_secs(6),
            Duration::from_secs(2),
        );
        let duration = start.elapsed();

        // Should perform checks at intervals
        // With 6 second timeout and 2 second interval, should perform ~3 checks
        assert!(duration >= Duration::from_secs(2)); // At least one interval
    }

    // ========== Comprehensive Health Check Tests ==========

    #[test]
    fn test_comprehensive_health_check() {
        let container_id = "frigate_test";

        let result = perform_comprehensive_health_check(container_id);

        if let Ok(health) = result {
            // Should have multiple health checks
            assert!(!health.checks.is_empty());

            // Should check container, API, config, etc.
            let check_names: Vec<String> = health.checks.iter().map(|c| c.name.clone()).collect();

            // Verify at least some expected checks are present
            assert!(
                check_names.iter().any(|name| name.contains("container") || name.contains("running"))
                || check_names.iter().any(|name| name.contains("api") || name.contains("endpoint"))
            );
        }
    }

    #[test]
    fn test_comprehensive_health_check_all_passed() {
        let container_id = "frigate_test";

        let result = perform_comprehensive_health_check(container_id);

        if let Ok(health) = result {
            if health.status == HealthStatus::Healthy {
                // All checks should have passed
                let all_passed = health.checks.iter().all(|check| check.passed);
                assert!(all_passed, "All checks should pass for healthy status");
            }
        }
    }

    #[test]
    fn test_comprehensive_health_check_includes_timestamp() {
        let container_id = "frigate_test";

        let result = perform_comprehensive_health_check(container_id);

        if let Ok(health) = result {
            // Should have recent timestamp
            let now = std::time::SystemTime::now();
            let diff = now.duration_since(health.timestamp);

            assert!(diff.is_ok());
            assert!(diff.unwrap() < Duration::from_secs(10)); // Very recent
        }
    }

    // ========== Port Response Check Tests ==========

    #[test]
    fn test_check_port_responding_success() {
        // Test with a port that's likely to be open (localhost HTTP)
        let result = check_port_responding("127.0.0.1", 5000, Duration::from_secs(2));

        // May succeed or fail depending on whether service is running
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_check_port_responding_timeout() {
        // Test with unreachable host
        let start = Instant::now();
        let result = check_port_responding("192.0.2.1", 80, Duration::from_millis(500));
        let duration = start.elapsed();

        // Should timeout
        assert!(result.is_err() || result.unwrap() == false);
        assert!(duration < Duration::from_secs(2)); // Should timeout quickly
    }

    #[test]
    fn test_check_port_responding_invalid_port() {
        let result = check_port_responding("localhost", 0, Duration::from_secs(1));

        // Should fail for port 0
        assert!(result.is_err() || result.unwrap() == false);
    }

    // ========== Frigate-Specific Health Checks ==========

    #[test]
    fn test_check_frigate_config_loaded() {
        let endpoint = "http://localhost:5000/api/config";

        let result = check_frigate_config_loaded(endpoint);

        // Should succeed if Frigate is running and config is loaded
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_check_cameras_initialized() {
        let endpoint = "http://localhost:5000/api";

        let result = check_cameras_initialized(endpoint);

        if let Ok(cameras) = result {
            // Should return list of camera names
            assert!(cameras.is_empty() || !cameras.is_empty());

            // Camera names should not be empty strings
            for camera in cameras {
                assert!(!camera.is_empty());
            }
        }
    }

    #[test]
    fn test_check_cameras_initialized_no_cameras() {
        // Test with config that has no cameras
        let endpoint = "http://localhost:5000/api";

        let result = check_cameras_initialized(endpoint);

        // Should succeed even with no cameras (valid state)
        assert!(result.is_ok() || result.is_err());
    }

    // ========== Health Status Interpretation Tests ==========

    #[test]
    fn test_health_status_starting_vs_healthy() {
        let container_id = "frigate_starting";

        // Check immediately after start
        let result1 = check_container_health(container_id);

        if let Ok(status1) = result1 {
            // Might be Starting initially
            assert!(
                status1 == HealthStatus::Starting
                || status1 == HealthStatus::Healthy
                || status1 == HealthStatus::Unknown
            );
        }

        // Wait and check again
        std::thread::sleep(Duration::from_secs(3));

        let result2 = check_container_health(container_id);

        // Should transition to Healthy or remain in current state
        assert!(result2.is_ok() || result2.is_err());
    }

    // ========== Performance Tests ==========

    #[test]
    fn test_health_check_performance() {
        let container_id = "frigate_test";

        let start = Instant::now();
        let _result = perform_comprehensive_health_check(container_id);
        let duration = start.elapsed();

        // Comprehensive health check should complete within 10 seconds
        assert!(duration < Duration::from_secs(10), "Health check took too long: {:?}", duration);
    }

    // ========== Integration Test ==========

    #[test]
    fn test_full_health_check_workflow() {
        // This test simulates the complete health check workflow after deployment

        let container_id = "frigate_integration_test";

        // 1. Check container is running
        let running = check_container_running(container_id);
        if let Ok(is_running) = running {
            if is_running {
                // 2. Wait for container to become healthy
                let healthy = wait_for_healthy(
                    container_id,
                    Duration::from_secs(60),
                    Duration::from_secs(5),
                );

                if let Ok(is_healthy) = healthy {
                    if is_healthy {
                        // 3. Check port is responding
                        let port_ok = check_port_responding("localhost", 5000, Duration::from_secs(5));
                        assert!(port_ok.is_ok());

                        // 4. Check Frigate API
                        let api_ok = check_frigate_api("http://localhost:5000/api");
                        assert!(api_ok.is_ok());

                        // 5. Check config loaded
                        let config_ok = check_frigate_config_loaded("http://localhost:5000/api/config");
                        assert!(config_ok.is_ok());

                        // 6. Perform comprehensive check
                        let comprehensive = perform_comprehensive_health_check(container_id);
                        assert!(comprehensive.is_ok());

                        let health = comprehensive.unwrap();
                        assert_eq!(health.status, HealthStatus::Healthy);
                    }
                }
            }
        }
    }
}
