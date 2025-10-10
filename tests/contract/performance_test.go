// Performance tests for Agent
// Tests agent performance characteristics and resource usage

package contract

import (
	"encoding/json"
	"os"
	"os/exec"
	"runtime"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestAgentExecutionTime tests agent execution time performance
func TestAgentExecutionTime(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	// Performance benchmarks
	const (
		maxExecutionTime = 5 * time.Second
		targetExecutionTime = 2 * time.Second
		numRuns = 5
	)

	var totalDuration time.Duration
	var durations []time.Duration

	for i := 0; i < numRuns; i++ {
		start := time.Now()

		cmd := exec.Command(agentPath, "detect")
		output, err := cmd.CombinedOutput()

		duration := time.Since(start)
		durations = append(durations, duration)
		totalDuration += duration

		// Verify success
		require.NoError(t, err, "Agent execution %d should succeed", i+1)

		// Verify valid JSON output
		var result DetectionResult
		err = json.Unmarshal(output, &result)
		require.NoError(t, err, "Agent output should be valid JSON on run %d", i+1)

		t.Logf("Run %d: %v", i+1, duration)
	}

	avgDuration := totalDuration / time.Duration(numRuns)

	t.Logf("Average execution time: %v", avgDuration)
	t.Logf("Min execution time: %v", minDuration(durations))
	t.Logf("Max execution time: %v", maxDuration(durations))

	// Performance assertions
	assert.Less(t, avgDuration, maxExecutionTime,
		"Average execution time should be less than %v, got %v", maxExecutionTime, avgDuration)

	if avgDuration > targetExecutionTime {
		t.Logf("Warning: Average execution time %v exceeds target %v", avgDuration, targetExecutionTime)
	}
}

// TestAgentMemoryUsage tests agent memory usage
func TestAgentMemoryUsage(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	// Get initial memory stats
	var initialMem runtime.MemStats
	runtime.GC()
	runtime.ReadMemStats(&initialMem)

	// Run agent
	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent execution should succeed")

	// Get final memory stats
	var finalMem runtime.MemStats
	runtime.GC()
	runtime.ReadMemStats(&finalMem)

	// Calculate memory usage
	memUsed := finalMem.Alloc - initialMem.Alloc

	t.Logf("Memory used during agent execution: %d bytes", memUsed)

	// Verify output is reasonable size
	outputSize := len(output)
	assert.Less(t, outputSize, 1024*1024, "Output should be less than 1MB, got %d bytes", outputSize)

	// Memory usage should be reasonable (less than 10MB for this simple operation)
	assert.Less(t, memUsed, uint64(10*1024*1024),
		"Memory usage should be less than 10MB, got %d bytes", memUsed)
}

// TestAgentConcurrentPerformance tests agent performance under concurrent load
func TestAgentConcurrentPerformance(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	const (
		numGoroutines = 10
		numRunsPerGoroutine = 3
	)

	var wg sync.WaitGroup
	results := make(chan time.Duration, numGoroutines*numRunsPerGoroutine)
	errors := make(chan error, numGoroutines*numRunsPerGoroutine)

	start := time.Now()

	// Launch concurrent goroutines
	for i := 0; i < numGoroutines; i++ {
		wg.Add(1)
		go func(goroutineID int) {
			defer wg.Done()

			for j := 0; j < numRunsPerGoroutine; j++ {
				runStart := time.Now()

				cmd := exec.Command(agentPath, "detect")
				output, err := cmd.CombinedOutput()

				duration := time.Since(runStart)
				results <- duration

				if err != nil {
					errors <- err
					return
				}

				// Verify output is valid JSON
				var result DetectionResult
				if json.Unmarshal(output, &result) != nil {
					errors <- assert.AnError
					return
				}
			}
		}(i)
	}

	wg.Wait()
	close(results)
	close(errors)

	totalDuration := time.Since(start)

	// Collect results
	var durations []time.Duration
	for duration := range results {
		durations = append(durations, duration)
	}

	// Check for errors
	for err := range errors {
		assert.NoError(t, err, "Concurrent agent execution should not produce errors")
	}

	// Performance analysis
	avgDuration := averageDuration(durations)
	maxDuration := maxDuration(durations)
	minDuration := minDuration(durations)
	totalRuns := len(durations)

	t.Logf("Concurrent execution performance:")
	t.Logf("  Total goroutines: %d", numGoroutines)
	t.Logf("  Runs per goroutine: %d", numRunsPerGoroutine)
	t.Logf("  Total runs: %d", totalRuns)
	t.Logf("  Total wall time: %v", totalDuration)
	t.Logf("  Average run time: %v", avgDuration)
	t.Logf("  Min run time: %v", minDuration)
	t.Logf("  Max run time: %v", maxDuration)
	t.Logf("  Throughput: %.2f runs/second", float64(totalRuns)/totalDuration.Seconds())

	// Performance assertions
	assert.Less(t, avgDuration, 5*time.Second, "Average run time should be reasonable")
	assert.Less(t, maxDuration, 10*time.Second, "Max run time should be reasonable")
	assert.Greater(t, float64(totalRuns)/totalDuration.Seconds(), 1.0,
		"Should achieve at least 1 run/second throughput")
}

// TestAgentOutputSize tests agent output size consistency
func TestAgentOutputSize(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	const numRuns = 5
	var sizes []int

	for i := 0; i < numRuns; i++ {
		cmd := exec.Command(agentPath, "detect")
		output, err := cmd.CombinedOutput()
		require.NoError(t, err, "Agent execution %d should succeed", i+1)

		size := len(output)
		sizes = append(sizes, size)

		t.Logf("Run %d output size: %d bytes", i+1, size)
	}

	// Analyze size consistency
	avgSize := averageInt(sizes)
	maxSize := maxInt(sizes)
	minSize := minInt(sizes)

	t.Logf("Output size analysis:")
	t.Logf("  Average size: %d bytes", avgSize)
	t.Logf("  Min size: %d bytes", minSize)
	t.Logf("  Max size: %d bytes", maxSize)
	t.Logf("  Size variance: %d bytes", maxSize-minSize)

	// Size should be consistent (variance less than 10% of average)
	sizeVariance := maxSize - minSize
	sizeVariancePercent := float64(sizeVariance) / float64(avgSize) * 100

	assert.Less(t, sizeVariancePercent, 10.0,
		"Output size variance should be less than 10%%, got %.2f%%", sizeVariancePercent)

	// Size should be reasonable (less than 100KB)
	assert.Less(t, avgSize, 100*1024,
		"Average output size should be less than 100KB, got %d bytes", avgSize)
}

// TestAgentStartupTime tests agent startup time
func TestAgentStartupTime(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	const numRuns = 3
	var startupTimes []time.Duration

	for i := 0; i < numRuns; i++ {
		start := time.Now()

		cmd := exec.Command(agentPath, "detect")
		err := cmd.Start()
		require.NoError(t, err, "Agent should start successfully")

		startupTime := time.Since(start)
		startupTimes = append(startupTimes, startupTime)

		// Wait for completion
		err = cmd.Wait()
		require.NoError(t, err, "Agent should complete successfully")

		t.Logf("Run %d startup time: %v", i+1, startupTime)
	}

	avgStartupTime := averageDuration(startupTimes)
	maxStartupTime := maxDuration(startupTimes)

	t.Logf("Startup time analysis:")
	t.Logf("  Average startup time: %v", avgStartupTime)
	t.Logf("  Max startup time: %v", maxStartupTime)

	// Startup should be fast (less than 1 second)
	assert.Less(t, avgStartupTime, time.Second,
		"Average startup time should be less than 1 second, got %v", avgStartupTime)
}

// TestAgentScalability tests agent performance with varying load
func TestAgentScalability(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	testCases := []struct {
		name string
		concurrency int
		expectedThroughput float64 // runs per second
	}{
		{"Single execution", 1, 1.0},
		{"Low concurrency", 2, 1.5},
		{"Medium concurrency", 5, 2.0},
		{"High concurrency", 10, 3.0},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			const runsPerWorker = 2

			var wg sync.WaitGroup
			results := make(chan time.Duration, tc.concurrency*runsPerWorker)

			start := time.Now()

			// Launch workers
			for i := 0; i < tc.concurrency; i++ {
				wg.Add(1)
				go func() {
					defer wg.Done()

					for j := 0; j < runsPerWorker; j++ {
						runStart := time.Now()

						cmd := exec.Command(agentPath, "detect")
						_, err := cmd.CombinedOutput()

						duration := time.Since(runStart)
						results <- duration

						require.NoError(t, err, "Agent execution should succeed")
					}
				}()
			}

			wg.Wait()
			close(results)

			totalDuration := time.Since(start)
			totalRuns := tc.concurrency * runsPerWorker
			throughput := float64(totalRuns) / totalDuration.Seconds()

			// Collect durations
			var durations []time.Duration
			for duration := range results {
				durations = append(durations, duration)
			}

			avgDuration := averageDuration(durations)

			t.Logf("Results for %s:", tc.name)
			t.Logf("  Concurrency: %d", tc.concurrency)
			t.Logf("  Total runs: %d", totalRuns)
			t.Logf("  Total duration: %v", totalDuration)
			t.Logf("  Average run time: %v", avgDuration)
			t.Logf("  Throughput: %.2f runs/second", throughput)

			// Performance assertions
			assert.GreaterOrEqual(t, throughput, tc.expectedThroughput,
				"Throughput should be at least %.2f runs/second, got %.2f",
				tc.expectedThroughput, throughput)
		})
	}
}

// Helper functions for duration calculations

func minDuration(durations []time.Duration) time.Duration {
	if len(durations) == 0 {
		return 0
	}
	min := durations[0]
	for _, d := range durations[1:] {
		if d < min {
			min = d
		}
	}
	return min
}

func maxDuration(durations []time.Duration) time.Duration {
	if len(durations) == 0 {
		return 0
	}
	max := durations[0]
	for _, d := range durations[1:] {
		if d > max {
			max = d
		}
	}
	return max
}

func averageDuration(durations []time.Duration) time.Duration {
	if len(durations) == 0 {
		return 0
	}
	var total time.Duration
	for _, d := range durations {
		total += d
	}
	return total / time.Duration(len(durations))
}

func averageInt(numbers []int) int {
	if len(numbers) == 0 {
		return 0
	}
	total := 0
	for _, n := range numbers {
		total += n
	}
	return total / len(numbers)
}

func minInt(numbers []int) int {
	if len(numbers) == 0 {
		return 0
	}
	min := numbers[0]
	for _, n := range numbers[1:] {
		if n < min {
			min = n
		}
	}
	return min
}

func maxInt(numbers []int) int {
	if len(numbers) == 0 {
		return 0
	}
	max := numbers[0]
	for _, n := range numbers[1:] {
		if n > max {
			max = n
		}
	}
	return max
}