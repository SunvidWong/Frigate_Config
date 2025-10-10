// Concurrency tests for Agent
// Tests agent behavior under concurrent execution and thread safety

package contract

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestAgentConcurrentExecutionAdvanced tests agent can handle multiple simultaneous executions
func TestAgentConcurrentExecutionAdvanced(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	const (
		numGoroutines = 20
		numRunsPerGoroutine = 5
		totalRuns = numGoroutines * numRunsPerGoroutine
	)

	var wg sync.WaitGroup
	results := make(chan TestResult, totalRuns)
	errors := make(chan error, totalRuns)

	// Launch concurrent goroutines
	for i := 0; i < numGoroutines; i++ {
		wg.Add(1)
		go func(goroutineID int) {
			defer wg.Done()

			for j := 0; j < numRunsPerGoroutine; j++ {
				result := TestResult{
					GoroutineID: goroutineID,
					RunID:       j,
				}

				start := time.Now()
				cmd := exec.Command(agentPath, "detect")
				output, err := cmd.CombinedOutput()
				result.Duration = time.Since(start)
				result.OutputSize = len(output)

				if err != nil {
					errors <- err
					return
				}

				// Parse JSON output
				var detectionResult DetectionResult
				if err := json.Unmarshal(output, &detectionResult); err != nil {
					errors <- err
					return
				}

				result.DeviceCount = len(detectionResult.Devices)
				result.Platform = detectionResult.Platform
				result.Architecture = detectionResult.Architecture

				results <- result
			}
		}(i)
	}

	wg.Wait()
	close(results)
	close(errors)

	// Collect and analyze results
	var allResults []TestResult
	for result := range results {
		allResults = append(allResults, result)
	}

	// Check for errors
	for err := range errors {
		assert.NoError(t, err, "Concurrent execution should not produce errors")
	}

	// Validate results
	assert.Equal(t, totalRuns, len(allResults), "Should have results from all runs")

	// Analyze consistency
	platforms := make(map[string]int)
	architectures := make(map[string]int)
	deviceCounts := make(map[int]int)

	for _, result := range allResults {
		platforms[result.Platform]++
		architectures[result.Architecture]++
		deviceCounts[result.DeviceCount]++

		// Validate each result
		assert.NotEmpty(t, result.Platform, "Platform should be set")
		assert.NotEmpty(t, result.Architecture, "Architecture should be set")
		assert.GreaterOrEqual(t, result.DeviceCount, 0, "Device count should be non-negative")
		assert.True(t, result.Duration > 0, "Duration should be positive")
		assert.Greater(t, result.OutputSize, 0, "Output size should be positive")
	}

	// All runs should have the same platform and architecture
	assert.Len(t, platforms, 1, "All runs should have the same platform")
	assert.Len(t, architectures, 1, "All runs should have the same architecture")

	t.Logf("Concurrency test results:")
	t.Logf("  Total runs: %d", totalRuns)
	t.Logf("  Concurrent goroutines: %d", numGoroutines)
	t.Logf("  Runs per goroutine: %d", numRunsPerGoroutine)
	t.Logf("  Platform consistency: %v", len(platforms) == 1)
	t.Logf("  Architecture consistency: %v", len(architectures) == 1)
	t.Logf("  Device count distribution: %v", deviceCounts)
}

// TestAgentResourceContention tests agent behavior under resource contention
func TestAgentResourceContention(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	const numWorkers = 50
	const workDuration = 5 * time.Second

	var wg sync.WaitGroup
	results := make(chan time.Duration, numWorkers)
	errors := make(chan error, numWorkers)

	startTime := time.Now()

	// Launch many workers to create contention
	for i := 0; i < numWorkers; i++ {
		wg.Add(1)
		go func(workerID int) {
			defer wg.Done()

			// Run agent until time limit
			for time.Since(startTime) < workDuration {
				runStart := time.Now()

				cmd := exec.Command(agentPath, "detect")
				_, err := cmd.CombinedOutput()

				duration := time.Since(runStart)
				results <- duration

				if err != nil {
					errors <- err
					return
				}
			}
		}(i)
	}

	wg.Wait()
	close(results)
	close(errors)

	// Analyze results
	var durations []time.Duration
	for duration := range results {
		durations = append(durations, duration)
	}

	// Check for errors
	errorCount := 0
	for range errors {
		errorCount++
	}

	assert.Equal(t, 0, errorCount, "No errors should occur under resource contention")

	if len(durations) > 0 {
		avgDuration := averageDuration(durations)
		maxDuration := maxDuration(durations)
		minDuration := minDuration(durations)

		t.Logf("Resource contention test results:")
		t.Logf("  Workers: %d", numWorkers)
		t.Logf("  Test duration: %v", workDuration)
		t.Logf("  Total runs: %d", len(durations))
		t.Logf("  Average duration: %v", avgDuration)
		t.Logf("  Min duration: %v", minDuration)
		t.Logf("  Max duration: %v", maxDuration)
		t.Logf("  Throughput: %.2f runs/second", float64(len(durations))/workDuration.Seconds())

		// Performance should not degrade significantly under contention
		assert.Less(t, avgDuration, 5*time.Second, "Average duration should remain reasonable")
		assert.Greater(t, float64(len(durations))/workDuration.Seconds(), 0.5,
			"Should maintain reasonable throughput under contention")
	}
}

// TestAgentParallelExecution tests agent behavior when executed in parallel processes
func TestAgentParallelExecution(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	const numProcesses = 10

	var wg sync.WaitGroup
	results := make(chan ProcessResult, numProcesses)
	errors := make(chan error, numProcesses)

	// Launch parallel processes
	for i := 0; i < numProcesses; i++ {
		wg.Add(1)
		go func(processID int) {
			defer wg.Done()

			result := ProcessResult{ProcessID: processID}

			start := time.Now()
			cmd := exec.Command(agentPath, "detect")
			output, err := cmd.CombinedOutput()
			result.Duration = time.Since(start)

			if err != nil {
				errors <- err
				return
			}

			// Parse output
			var detectionResult DetectionResult
			if err := json.Unmarshal(output, &detectionResult); err != nil {
				errors <- err
				return
			}

			result.DeviceCount = len(detectionResult.Devices)
			result.Platform = detectionResult.Platform
			result.Architecture = detectionResult.Architecture
			result.OutputSize = len(output)

			results <- result
		}(i)
	}

	wg.Wait()
	close(results)
	close(errors)

	// Collect results
	var allResults []ProcessResult
	for result := range results {
		allResults = append(allResults, result)
	}

	// Check for errors
	for err := range errors {
		assert.NoError(t, err, "Parallel execution should not produce errors")
	}

	// Validate results
	assert.Equal(t, numProcesses, len(allResults), "Should have results from all processes")

	// Analyze consistency
	platforms := make(map[string]int)
	architectures := make(map[string]int)

	for _, result := range allResults {
		platforms[result.Platform]++
		architectures[result.Architecture]++

		assert.NotEmpty(t, result.Platform, "Platform should be set")
		assert.NotEmpty(t, result.Architecture, "Architecture should be set")
		assert.GreaterOrEqual(t, result.DeviceCount, 0, "Device count should be non-negative")
	}

	assert.Len(t, platforms, 1, "All processes should report the same platform")
	assert.Len(t, architectures, 1, "All processes should report the same architecture")

	t.Logf("Parallel execution test results:")
	t.Logf("  Processes: %d", numProcesses)
	t.Logf("  Platform consistency: %v", len(platforms) == 1)
	t.Logf("  Architecture consistency: %v", len(architectures) == 1)
}

// TestAgentConcurrentResourceAccess tests agent handles concurrent resource access safely
func TestAgentConcurrentResourceAccess(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	const numWorkers = 20
	const numRounds = 3

	for round := 0; round < numRounds; round++ {
		t.Run(fmt.Sprintf("Round_%d", round+1), func(t *testing.T) {
			var wg sync.WaitGroup
			results := make(chan ResourceResult, numWorkers)

			// Launch workers that access system resources concurrently
			for i := 0; i < numWorkers; i++ {
				wg.Add(1)
				go func(workerID int) {
					defer wg.Done()

					result := ResourceResult{WorkerID: workerID}

					start := time.Now()
					cmd := exec.Command(agentPath, "detect")
					output, err := cmd.CombinedOutput()
					result.Duration = time.Since(start)

					if err != nil {
						result.Error = err.Error()
					} else {
						// Parse output to check for resource access errors
						var detectionResult DetectionResult
						if parseErr := json.Unmarshal(output, &detectionResult); parseErr != nil {
							result.Error = parseErr.Error()
						} else {
							result.DeviceCount = len(detectionResult.Devices)
							result.HasErrors = hasDeviceErrors(detectionResult.Devices)
						}
					}

					results <- result
				}(i)
			}

			wg.Wait()
			close(results)

			// Analyze results
			var allResults []ResourceResult
			for result := range results {
				allResults = append(allResults, result)
			}

			// Check for critical errors
			criticalErrors := 0
			for _, result := range allResults {
				if result.Error != "" && isCriticalError(result.Error) {
					criticalErrors++
					t.Logf("Worker %d critical error: %s", result.WorkerID, result.Error)
				}
			}

			assert.Equal(t, 0, criticalErrors, "No critical errors should occur in round %d", round+1)

			// Most workers should succeed
			successCount := 0
			for _, result := range allResults {
				if result.Error == "" {
					successCount++
				}
			}

			successRate := float64(successCount) / float64(len(allResults)) * 100
			t.Logf("Round %d results:", round+1)
			t.Logf("  Workers: %d", numWorkers)
			t.Logf("  Success rate: %.1f%%", successRate)
			t.Logf("  Critical errors: %d", criticalErrors)

			assert.Greater(t, successRate, 80.0, "Success rate should be above 80%%")
		})
	}
}

// TestAgentDeadlockPrevention tests agent doesn't deadlock under concurrent execution
func TestAgentDeadlockPrevention(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	const (
		numGoroutines = 30
		maxWaitTime = 10 * time.Second
	)

	var wg sync.WaitGroup
	completed := make(chan bool, numGoroutines)

	// Launch goroutines with timeout detection
	for i := 0; i < numGoroutines; i++ {
		wg.Add(1)
		go func(goroutineID int) {
			defer wg.Done()

			done := make(chan bool, 1)

			go func() {
				defer func() {
					if r := recover(); r != nil {
						t.Logf("Goroutine %d panicked: %v", goroutineID, r)
					}
					done <- true
				}()

				cmd := exec.Command(agentPath, "detect")
				_, err := cmd.CombinedOutput()
				if err != nil {
					t.Logf("Goroutine %d failed: %v", goroutineID, err)
				}
			}()

			// Wait for completion or timeout
			select {
			case <-done:
				completed <- true
			case <-time.After(maxWaitTime):
				t.Errorf("Goroutine %d timed out after %v - possible deadlock", goroutineID, maxWaitTime)
				completed <- false
			}
		}(i)
	}

	wg.Wait()
	close(completed)

	// Count completed goroutines
	completedCount := 0
	timeoutCount := 0
	for completed := range completed {
		if completed {
			completedCount++
		} else {
			timeoutCount++
		}
	}

	t.Logf("Deadlock prevention test results:")
	t.Logf("  Total goroutines: %d", numGoroutines)
	t.Logf("  Completed: %d", completedCount)
	t.Logf("  Timed out: %d", timeoutCount)

	// All goroutines should complete
	assert.Equal(t, numGoroutines, completedCount, "All goroutines should complete without deadlock")
	assert.Equal(t, 0, timeoutCount, "No goroutines should timeout")
}

// Helper types and functions

type TestResult struct {
	GoroutineID int
	RunID       int
	Duration    time.Duration
	OutputSize  int
	DeviceCount int
	Platform    string
	Architecture string
}

type ProcessResult struct {
	ProcessID   int
	Duration    time.Duration
	DeviceCount int
	Platform    string
	Architecture string
	OutputSize  int
}

type ResourceResult struct {
	WorkerID    int
	Duration    time.Duration
	DeviceCount int
	Error       string
	HasErrors   bool
}

func hasDeviceErrors(devices []HardwareDevice) bool {
	for _, device := range devices {
		if device.Error != nil {
			return true
		}
	}
	return false
}

func isCriticalError(errorMsg string) bool {
	criticalErrors := []string{
		"fatal",
		"panic",
		"segmentation fault",
		"cannot allocate memory",
		"permission denied",
	}

	errorLower := strings.ToLower(errorMsg)
	for _, critical := range criticalErrors {
		if strings.Contains(errorLower, critical) {
			return true
		}
	}
	return false
}