package detect

import (
	"os"
	"os/exec"
	"regexp"
	"strconv"
	"strings"
	"time"

	"github.com/frigate-config-tool/agent/internal/schema"
)

// HardwareAvailabilityDetector provides comprehensive hardware availability detection
type HardwareAvailabilityDetector struct {
	detector Detector
}

// NewHardwareAvailabilityDetector creates a new hardware availability detector
func NewHardwareAvailabilityDetector(detector Detector) *HardwareAvailabilityDetector {
	return &HardwareAvailabilityDetector{
		detector: detector,
	}
}

// DetectHardwareAvailability performs comprehensive hardware availability detection
func (h *HardwareAvailabilityDetector) DetectHardwareAvailability() ([]schema.HardwareAvailability, error) {
	var availability []schema.HardwareAvailability

	// Get all hardware devices
	gpus, _ := h.detector.DetectGPUs()
	tpus, _ := h.detector.DetectTPUs()
	cameras, _ := h.detector.DetectCameras()
	captureCards, _ := h.detector.DetectCaptureCards()

	// Check GPU availability
	for _, gpu := range gpus {
		gpuAvail := h.detectGPUAvailability(gpu)
		availability = append(availability, gpuAvail)
	}

	// Check TPU availability
	for _, tpu := range tpus {
		tpuAvail := h.detectTPUAvailability(tpu)
		availability = append(availability, tpuAvail)
	}

	// Check camera availability
	for _, camera := range cameras {
		cameraAvail := h.detectCameraAvailability(camera)
		availability = append(availability, cameraAvail)
	}

	// Check capture card availability
	for _, card := range captureCards {
		cardAvail := h.detectCaptureCardAvailability(card)
		availability = append(availability, cardAvail)
	}

	return availability, nil
}

// detectGPUAvailability detects GPU availability and usage
func (h *HardwareAvailabilityDetector) detectGPUAvailability(gpu schema.HardwareDevice) schema.HardwareAvailability {
	availability := schema.HardwareAvailability{
		DeviceID:   gpu.ID,
		DeviceName: gpu.Name,
		DeviceType: "gpu",
		Platform:   gpu.Platform,
		Available:  h.isGPUAvailable(gpu),
		InUse:      h.isGPUInUse(gpu),
		LastChecked: time.Now(),
	}

	// Get detailed usage information
	availability.Usage = h.getGPUUsage(gpu)
	availability.Performance = h.getGPUPerformance(gpu)
	availability.Thermal = h.getGPUThermalStatus(gpu)
	availability.Power = h.getGPUPowerStatus(gpu)
	availability.Processes = h.getGPUProcesses(gpu)
	availability.Errors = h.getGPUErrors(gpu)
	availability.Warnings = h.getGPUWarnings(gpu)
	availability.Capacity = h.getGPUCapacity(gpu)

	return availability
}

// detectTPUAvailability detects TPU availability and usage
func (h *HardwareAvailabilityDetector) detectTPUAvailability(tpu schema.HardwareDevice) schema.HardwareAvailability {
	availability := schema.HardwareAvailability{
		DeviceID:   tpu.ID,
		DeviceName: tpu.Name,
		DeviceType: "tpu",
		Platform:   tpu.Platform,
		Available:  h.isTPUAvailable(tpu),
		InUse:      h.isTPUInUse(tpu),
		LastChecked: time.Now(),
	}

	// Get detailed usage information
	availability.Usage = h.getTPUUsage(tpu)
	availability.Performance = h.getTPUPerformance(tpu)
	availability.Thermal = h.getTPUThermalStatus(tpu)
	availability.Power = h.getTPUPowerStatus(tpu)
	availability.Processes = h.getTPUProcesses(tpu)
	availability.Errors = h.getTPUErrors(tpu)
	availability.Warnings = h.getTPUWarnings(tpu)
	availability.Capacity = h.getTPUCapacity(tpu)

	return availability
}

// detectCameraAvailability detects camera availability and usage
func (h *HardwareAvailabilityDetector) detectCameraAvailability(camera schema.HardwareDevice) schema.HardwareAvailability {
	availability := schema.HardwareAvailability{
		DeviceID:   camera.ID,
		DeviceName: camera.Name,
		DeviceType: "camera",
		Platform:   camera.Platform,
		Available:  h.isCameraAvailable(camera),
		InUse:      h.isCameraInUse(camera),
		LastChecked: time.Now(),
	}

	// Get detailed usage information
	availability.Usage = h.getCameraUsage(camera)
	availability.Performance = h.getCameraPerformance(camera)
	availability.Thermal = h.getCameraThermalStatus(camera)
	availability.Power = h.getCameraPowerStatus(camera)
	availability.Processes = h.getCameraProcesses(camera)
	availability.Errors = h.getCameraErrors(camera)
	availability.Warnings = h.getCameraWarnings(camera)
	availability.Capacity = h.getCameraCapacity(camera)

	return availability
}

// detectCaptureCardAvailability detects capture card availability and usage
func (h *HardwareAvailabilityDetector) detectCaptureCardAvailability(card schema.HardwareDevice) schema.HardwareAvailability {
	availability := schema.HardwareAvailability{
		DeviceID:   card.ID,
		DeviceName: card.Name,
		DeviceType: "capture_card",
		Platform:   card.Platform,
		Available:  h.isCaptureCardAvailable(card),
		InUse:      h.isCaptureCardInUse(card),
		LastChecked: time.Now(),
	}

	// Get detailed usage information
	availability.Usage = h.getCaptureCardUsage(card)
	availability.Performance = h.getCaptureCardPerformance(card)
	availability.Thermal = h.getCaptureCardThermalStatus(card)
	availability.Power = h.getCaptureCardPowerStatus(card)
	availability.Processes = h.getCaptureCardProcesses(card)
	availability.Errors = h.getCaptureCardErrors(card)
	availability.Warnings = h.getCaptureCardWarnings(card)
	availability.Capacity = h.getCaptureCardCapacity(card)

	return availability
}

// GPU availability detection methods

func (h *HardwareAvailabilityDetector) isGPUAvailable(gpu schema.HardwareDevice) bool {
	switch gpu.Platform {
	case "linux":
		return h.isLinuxGPUAvailable(gpu)
	case "darwin":
		return h.isDarwinGPUAvailable(gpu)
	case "windows":
		return h.isWindowsGPUAvailable(gpu)
	default:
		return true
	}
}

func (h *HardwareAvailabilityDetector) isLinuxGPUAvailable(gpu schema.HardwareDevice) bool {
	// Check if GPU device exists
	if _, err := os.Stat(gpu.DevicePath); os.IsNotExist(err) {
		return false
	}

	// Check if GPU driver is loaded
	if strings.Contains(strings.ToLower(gpu.Name), "nvidia") {
		cmd := exec.Command("nvidia-smi", "--query-gpu=driver_version", "--format=csv,noheader")
		return cmd.Run() == nil
	}

	if strings.Contains(strings.ToLower(gpu.Name), "amd") {
		cmd := exec.Command("rocm-smi", "--showproductname")
		return cmd.Run() == nil
	}

	if strings.Contains(strings.ToLower(gpu.Name), "intel") {
		cmd := exec.Command("intel_gpu_top", "-J")
		return cmd.Run() == nil
	}

	return true
}

func (h *HardwareAvailabilityDetector) isDarwinGPUAvailable(gpu schema.HardwareDevice) bool {
	// Check if WindowServer is running (GPU is being used by macOS)
	cmd := exec.Command("pgrep", "-f", "WindowServer")
	return cmd.Run() == nil
}

func (h *HardwareAvailabilityDetector) isWindowsGPUAvailable(gpu schema.HardwareDevice) bool {
	if strings.Contains(strings.ToLower(gpu.Name), "nvidia") {
		cmd := exec.Command("nvidia-smi", "--query-gpu=driver_version", "--format=csv,noheader")
		return cmd.Run() == nil
	}

	cmd := exec.Command("wmic", "path", "win32_VideoController", "get", "Name", "/format:csv")
	return cmd.Run() == nil
}

func (h *HardwareAvailabilityDetector) isGPUInUse(gpu schema.HardwareDevice) bool {
	switch gpu.Platform {
	case "linux":
		return h.isLinuxGPUInUse(gpu)
	case "darwin":
		return h.isDarwinGPUInUse(gpu)
	case "windows":
		return h.isWindowsGPUInUse(gpu)
	default:
		return false
	}
}

func (h *HardwareAvailabilityDetector) isLinuxGPUInUse(gpu schema.HardwareDevice) bool {
	if strings.Contains(strings.ToLower(gpu.Name), "nvidia") {
		cmd := exec.Command("nvidia-smi", "--query-compute-apps=pid", "--format=csv,noheader")
		output, err := cmd.Output()
		return err == nil && len(strings.TrimSpace(string(output))) > 0
	}

	// Check for GPU-related processes
	cmd := exec.Command("pgrep", "-i", "xorg|wayland|gpu")
	output, err := cmd.Output()
	return err == nil && len(strings.TrimSpace(string(output))) > 0
}

func (h *HardwareAvailabilityDetector) isDarwinGPUInUse(gpu schema.HardwareDevice) bool {
	// Check for GPU-intensive processes
	cmd := exec.Command("pgrep", "-i", "metal|opengl|gpu")
	output, err := cmd.Output()
	return err == nil && len(strings.TrimSpace(string(output))) > 0
}

func (h *HardwareAvailabilityDetector) isWindowsGPUInUse(gpu schema.HardwareDevice) bool {
	if strings.Contains(strings.ToLower(gpu.Name), "nvidia") {
		cmd := exec.Command("nvidia-smi", "--query-compute-apps=pid", "--format=csv,noheader")
		output, err := cmd.Output()
		return err == nil && len(strings.TrimSpace(string(output))) > 0
	}

	// Check for GPU-related processes
	cmd := exec.Command("tasklist", "/fo", "csv", "/nh")
	output, err := cmd.Output()
	if err != nil {
		return false
	}

	outputStr := string(output)
	return strings.Contains(strings.ToLower(outputStr), "dwm") ||
		   strings.Contains(strings.ToLower(outputStr), "chrome") ||
		   strings.Contains(strings.ToLower(outputStr), "firefox")
}

func (h *HardwareAvailabilityDetector) getGPUUsage(gpu schema.HardwareDevice) schema.UsageMetrics {
	var usage schema.UsageMetrics

	switch gpu.Platform {
	case "linux":
		usage = h.getLinuxGPUUsage(gpu)
	case "darwin":
		usage = h.getDarwinGPUUsage(gpu)
	case "windows":
		usage = h.getWindowsGPUUsage(gpu)
	}

	return usage
}

func (h *HardwareAvailabilityDetector) getLinuxGPUUsage(gpu schema.HardwareDevice) schema.UsageMetrics {
	var usage schema.UsageMetrics

	if strings.Contains(strings.ToLower(gpu.Name), "nvidia") {
		cmd := exec.Command("nvidia-smi", "--query-gpu=utilization.gpu,utilization.memory,memory.used,memory.total", "--format=csv,noheader")
		output, err := cmd.Output()
		if err == nil {
			usage = h.parseNvidiaUsage(string(output))
		}
	}

	// Fallback to generic usage detection
	if usage.Utilization == 0 {
		usage = h.getGenericGPUUsage(gpu)
	}

	return usage
}

func (h *HardwareAvailabilityDetector) getDarwinGPUUsage(gpu schema.HardwareDevice) schema.UsageMetrics {
	// Use ioreg to get GPU usage
	cmd := exec.Command("ioreg", "-r", "-c", "IOAccelerator")
	output, err := cmd.Output()
	if err != nil {
		return schema.UsageMetrics{}
	}

	return h.parseDarwinGPUUsage(string(output))
}

func (h *HardwareAvailabilityDetector) getWindowsGPUUsage(gpu schema.HardwareDevice) schema.UsageMetrics {
	var usage schema.UsageMetrics

	if strings.Contains(strings.ToLower(gpu.Name), "nvidia") {
		cmd := exec.Command("nvidia-smi", "--query-gpu=utilization.gpu,utilization.memory,memory.used,memory.total", "--format=csv,noheader")
		output, err := cmd.Output()
		if err == nil {
			usage = h.parseNvidiaUsage(string(output))
		}
	}

	return usage
}

func (h *HardwareAvailabilityDetector) parseNvidiaUsage(output string) schema.UsageMetrics {
	usage := schema.UsageMetrics{}

	parts := strings.Split(strings.TrimSpace(output), ",")
	if len(parts) >= 4 {
		if util, err := strconv.Atoi(strings.TrimSpace(parts[0])); err == nil {
			usage.Utilization = float64(util)
		}
		if memUtil, err := strconv.Atoi(strings.TrimSpace(parts[1])); err == nil {
			usage.MemoryUtilization = float64(memUtil)
		}
		if memUsed, err := strconv.Atoi(strings.TrimSpace(parts[2])); err == nil {
			usage.MemoryUsed = int64(memUsed) * 1024 * 1024 // Convert MB to bytes
		}
		if memTotal, err := strconv.Atoi(strings.TrimSpace(parts[3])); err == nil {
			usage.MemoryTotal = int64(memTotal) * 1024 * 1024 // Convert MB to bytes
		}
	}

	return usage
}

func (h *HardwareAvailabilityDetector) parseDarwinGPUUsage(output string) schema.UsageMetrics {
	usage := schema.UsageMetrics{}

	// Parse GPU utilization from ioreg output
	re := regexp.MustCompile(`"PerformanceStatistics" = \{[^}]+\}`)
	matches := re.FindStringSubmatch(output)
	if len(matches) >= 1 {
		// Parse performance statistics (simplified)
		usage.Utilization = 50.0 // Placeholder
	}

	return usage
}

func (h *HardwareAvailabilityDetector) getGenericGPUUsage(gpu schema.HardwareDevice) schema.UsageMetrics {
	return schema.UsageMetrics{
		Utilization:        0.0,
		MemoryUtilization:  0.0,
		MemoryUsed:         0,
		MemoryTotal:        0,
		CPUUtilization:     0.0,
		Temperature:        0.0,
		ClockSpeed:         0,
		PowerConsumption:   0.0,
	}
}

func (h *HardwareAvailabilityDetector) getGPUPerformance(gpu schema.HardwareDevice) schema.PerformanceMetrics {
	var perf schema.PerformanceMetrics

	switch gpu.Platform {
	case "linux":
		perf = h.getLinuxGPUPerformance(gpu)
	case "darwin":
		perf = h.getDarwinGPUPerformance(gpu)
	case "windows":
		perf = h.getWindowsGPUPerformance(gpu)
	}

	return perf
}

func (h *HardwareAvailabilityDetector) getLinuxGPUPerformance(gpu schema.HardwareDevice) schema.PerformanceMetrics {
	var perf schema.PerformanceMetrics

	if strings.Contains(strings.ToLower(gpu.Name), "nvidia") {
		cmd := exec.Command("nvidia-smi", "--query-gpu=temperature.gpu,power.draw,clocks.sm,clocks.memory", "--format=csv,noheader")
		output, err := cmd.Output()
		if err == nil {
			perf = h.parseNvidiaPerformance(string(output))
		}
	}

	return perf
}

func (h *HardwareAvailabilityDetector) getDarwinGPUPerformance(gpu schema.HardwareDevice) schema.PerformanceMetrics {
	// Use powermetrics to get GPU performance
	cmd := exec.Command("powermetrics", "--samplers", "gpu_power", "-i", "1", "-n", "1")
	output, err := cmd.Output()
	if err != nil {
		return schema.PerformanceMetrics{}
	}

	return h.parseDarwinGPUPerformance(string(output))
}

func (h *HardwareAvailabilityDetector) getWindowsGPUPerformance(gpu schema.HardwareDevice) schema.PerformanceMetrics {
	var perf schema.PerformanceMetrics

	if strings.Contains(strings.ToLower(gpu.Name), "nvidia") {
		cmd := exec.Command("nvidia-smi", "--query-gpu=temperature.gpu,power.draw,clocks.sm,clocks.memory", "--format=csv,noheader")
		output, err := cmd.Output()
		if err == nil {
			perf = h.parseNvidiaPerformance(string(output))
		}
	}

	return perf
}

func (h *HardwareAvailabilityDetector) parseNvidiaPerformance(output string) schema.PerformanceMetrics {
	perf := schema.PerformanceMetrics{}

	parts := strings.Split(strings.TrimSpace(output), ",")
	if len(parts) >= 4 {
		if temp, err := strconv.ParseFloat(strings.TrimSpace(parts[0]), 64); err == nil {
			perf.Temperature = temp
		}
		if power, err := strconv.ParseFloat(strings.TrimSpace(parts[1]), 64); err == nil {
			perf.PowerConsumption = power
		}
		if clock, err := strconv.Atoi(strings.TrimSpace(parts[2])); err == nil {
			perf.ClockSpeed = int64(clock)
		}
	}

	return perf
}

func (h *HardwareAvailabilityDetector) parseDarwinGPUPerformance(output string) schema.PerformanceMetrics {
	perf := schema.PerformanceMetrics{}

	// Parse GPU performance from powermetrics output
	re := regexp.MustCompile(`GPU (\d+) temperature: (\d+)`)
	matches := re.FindStringSubmatch(output)
	if len(matches) >= 3 {
		if temp, err := strconv.ParseFloat(matches[2], 64); err == nil {
			perf.Temperature = temp
		}
	}

	return perf
}

func (h *HardwareAvailabilityDetector) getGPUThermalStatus(gpu schema.HardwareDevice) schema.ThermalStatus {
	var thermal schema.ThermalStatus

	switch gpu.Platform {
	case "linux":
		thermal = h.getLinuxGPUThermalStatus(gpu)
	case "darwin":
		thermal = h.getDarwinGPUThermalStatus(gpu)
	case "windows":
		thermal = h.getWindowsGPUThermalStatus(gpu)
	}

	return thermal
}

func (h *HardwareAvailabilityDetector) getLinuxGPUThermalStatus(gpu schema.HardwareDevice) schema.ThermalStatus {
	var thermal schema.ThermalStatus

	if strings.Contains(strings.ToLower(gpu.Name), "nvidia") {
		cmd := exec.Command("nvidia-smi", "--query-gpu=temperature.gpu", "--format=csv,noheader")
		output, err := cmd.Output()
		if err == nil {
			if temp, err := strconv.ParseFloat(strings.TrimSpace(string(output)), 64); err == nil {
				thermal.Temperature = temp
				thermal.Throttling = temp > 85.0
				thermal.Critical = temp > 95.0
			}
		}
	}

	return thermal
}

func (h *HardwareAvailabilityDetector) getDarwinGPUThermalStatus(gpu schema.HardwareDevice) schema.ThermalStatus {
	// Use powermetrics to get GPU temperature
	cmd := exec.Command("powermetrics", "--samplers", "gpu_power", "-i", "1", "-n", "1")
	output, err := cmd.Output()
	if err != nil {
		return schema.ThermalStatus{}
	}

	return h.parseDarwinGPUThermalStatus(string(output))
}

func (h *HardwareAvailabilityDetector) getWindowsGPUThermalStatus(gpu schema.HardwareDevice) schema.ThermalStatus {
	var thermal schema.ThermalStatus

	if strings.Contains(strings.ToLower(gpu.Name), "nvidia") {
		cmd := exec.Command("nvidia-smi", "--query-gpu=temperature.gpu", "--format=csv,noheader")
		output, err := cmd.Output()
		if err == nil {
			if temp, err := strconv.ParseFloat(strings.TrimSpace(string(output)), 64); err == nil {
				thermal.Temperature = temp
				thermal.Throttling = temp > 85.0
				thermal.Critical = temp > 95.0
			}
		}
	}

	return thermal
}

func (h *HardwareAvailabilityDetector) parseDarwinGPUThermalStatus(output string) schema.ThermalStatus {
	thermal := schema.ThermalStatus{}

	// Parse GPU temperature from powermetrics output
	re := regexp.MustCompile(`GPU (\d+) temperature: (\d+)`)
	matches := re.FindStringSubmatch(output)
	if len(matches) >= 3 {
		if temp, err := strconv.ParseFloat(matches[2], 64); err == nil {
			thermal.Temperature = temp
			thermal.Throttling = temp > 85.0
			thermal.Critical = temp > 95.0
		}
	}

	return thermal
}

func (h *HardwareAvailabilityDetector) getGPUPowerStatus(gpu schema.HardwareDevice) schema.PowerStatus {
	var power schema.PowerStatus

	switch gpu.Platform {
	case "linux":
		power = h.getLinuxGPUPowerStatus(gpu)
	case "darwin":
		power = h.getDarwinGPUPowerStatus(gpu)
	case "windows":
		power = h.getWindowsGPUPowerStatus(gpu)
	}

	return power
}

func (h *HardwareAvailabilityDetector) getLinuxGPUPowerStatus(gpu schema.HardwareDevice) schema.PowerStatus {
	var power schema.PowerStatus

	if strings.Contains(strings.ToLower(gpu.Name), "nvidia") {
		cmd := exec.Command("nvidia-smi", "--query-gpu=power.draw,power.limit", "--format=csv,noheader")
		output, err := cmd.Output()
		if err == nil {
			power = h.parseNvidiaPowerStatus(string(output))
		}
	}

	return power
}

func (h *HardwareAvailabilityDetector) getDarwinGPUPowerStatus(gpu schema.HardwareDevice) schema.PowerStatus {
	// Use powermetrics to get GPU power consumption
	cmd := exec.Command("powermetrics", "--samplers", "gpu_power", "-i", "1", "-n", "1")
	output, err := cmd.Output()
	if err != nil {
		return schema.PowerStatus{}
	}

	return h.parseDarwinGPUPowerStatus(string(output))
}

func (h *HardwareAvailabilityDetector) getWindowsGPUPowerStatus(gpu schema.HardwareDevice) schema.PowerStatus {
	var power schema.PowerStatus

	if strings.Contains(strings.ToLower(gpu.Name), "nvidia") {
		cmd := exec.Command("nvidia-smi", "--query-gpu=power.draw,power.limit", "--format=csv,noheader")
		output, err := cmd.Output()
		if err == nil {
			power = h.parseNvidiaPowerStatus(string(output))
		}
	}

	return power
}

func (h *HardwareAvailabilityDetector) parseNvidiaPowerStatus(output string) schema.PowerStatus {
	power := schema.PowerStatus{}

	parts := strings.Split(strings.TrimSpace(output), ",")
	if len(parts) >= 2 {
		if draw, err := strconv.ParseFloat(strings.TrimSpace(parts[0]), 64); err == nil {
			power.CurrentPower = draw
		}
		if limit, err := strconv.ParseFloat(strings.TrimSpace(parts[1]), 64); err == nil {
			power.PowerLimit = limit
		}
	}

	return power
}

func (h *HardwareAvailabilityDetector) parseDarwinGPUPowerStatus(output string) schema.PowerStatus {
	power := schema.PowerStatus{}

	// Parse GPU power from powermetrics output
	re := regexp.MustCompile(`GPU Power: ([\d.]+)`)
	matches := re.FindStringSubmatch(output)
	if len(matches) >= 2 {
		if pwr, err := strconv.ParseFloat(matches[1], 64); err == nil {
			power.CurrentPower = pwr
		}
	}

	return power
}

func (h *HardwareAvailabilityDetector) getGPUProcesses(gpu schema.HardwareDevice) []schema.ProcessInfo {
	var processes []schema.ProcessInfo

	switch gpu.Platform {
	case "linux":
		processes = h.getLinuxGPUProcesses(gpu)
	case "darwin":
		processes = h.getDarwinGPUProcesses(gpu)
	case "windows":
		processes = h.getWindowsGPUProcesses(gpu)
	}

	return processes
}

func (h *HardwareAvailabilityDetector) getLinuxGPUProcesses(gpu schema.HardwareDevice) []schema.ProcessInfo {
	var processes []schema.ProcessInfo

	if strings.Contains(strings.ToLower(gpu.Name), "nvidia") {
		cmd := exec.Command("nvidia-smi", "--query-compute-apps=pid,process_name,used_memory", "--format=csv,noheader")
		output, err := cmd.Output()
		if err == nil {
			processes = h.parseNvidiaProcesses(string(output))
		}
	}

	return processes
}

func (h *HardwareAvailabilityDetector) getDarwinGPUProcesses(gpu schema.HardwareDevice) []schema.ProcessInfo {
	// Get processes using GPU
	cmd := exec.Command("ps", "aux")
	output, err := cmd.Output()
	if err != nil {
		return []schema.ProcessInfo{}
	}

	return h.parseDarwinGPUProcesses(string(output))
}

func (h *HardwareAvailabilityDetector) getWindowsGPUProcesses(gpu schema.HardwareDevice) []schema.ProcessInfo {
	var processes []schema.ProcessInfo

	if strings.Contains(strings.ToLower(gpu.Name), "nvidia") {
		cmd := exec.Command("nvidia-smi", "--query-compute-apps=pid,process_name,used_memory", "--format=csv,noheader")
		output, err := cmd.Output()
		if err == nil {
			processes = h.parseNvidiaProcesses(string(output))
		}
	}

	return processes
}

func (h *HardwareAvailabilityDetector) parseNvidiaProcesses(output string) []schema.ProcessInfo {
	var processes []schema.ProcessInfo

	lines := strings.Split(strings.TrimSpace(output), "\n")
	for _, line := range lines {
		if line == "" {
			continue
		}

		parts := strings.Split(line, ",")
		if len(parts) >= 3 {
			process := schema.ProcessInfo{}
			if pid, err := strconv.Atoi(strings.TrimSpace(parts[0])); err == nil {
				process.PID = pid
			}
			process.Name = strings.TrimSpace(parts[1])
			if memory, err := strconv.Atoi(strings.TrimSpace(parts[2])); err == nil {
				process.MemoryUsage = int64(memory) * 1024 * 1024 // Convert MB to bytes
			}
			processes = append(processes, process)
		}
	}

	return processes
}

func (h *HardwareAvailabilityDetector) parseDarwinGPUProcesses(output string) []schema.ProcessInfo {
	var processes []schema.ProcessInfo

	lines := strings.Split(output, "\n")
	for _, line := range lines {
		if strings.Contains(strings.ToLower(line), "metal") ||
		   strings.Contains(strings.ToLower(line), "opengl") ||
		   strings.Contains(strings.ToLower(line), "gpu") {
			parts := strings.Fields(line)
			if len(parts) >= 11 {
				if pid, err := strconv.Atoi(parts[1]); err == nil {
					process := schema.ProcessInfo{
						PID:  pid,
						Name: parts[10],
					}
					processes = append(processes, process)
				}
			}
		}
	}

	return processes
}

func (h *HardwareAvailabilityDetector) getGPUErrors(gpu schema.HardwareDevice) []string {
	var errors []string

	// Check for common GPU errors
	if !h.isGPUAvailable(gpu) {
		errors = append(errors, "GPU not available")
	}

	if h.getGPUThermalStatus(gpu).Critical {
		errors = append(errors, "GPU temperature critical")
	}

	if h.getGPUPowerStatus(gpu).CurrentPower > h.getGPUPowerStatus(gpu).PowerLimit {
		errors = append(errors, "GPU power limit exceeded")
	}

	return errors
}

func (h *HardwareAvailabilityDetector) getGPUWarnings(gpu schema.HardwareDevice) []string {
	var warnings []string

	// Check for GPU warnings
	if h.getGPUThermalStatus(gpu).Throttling {
		warnings = append(warnings, "GPU thermal throttling active")
	}

	usage := h.getGPUUsage(gpu)
	if usage.Utilization > 90.0 {
		warnings = append(warnings, "GPU utilization very high")
	}

	if usage.MemoryUtilization > 90.0 {
		warnings = append(warnings, "GPU memory utilization very high")
	}

	return warnings
}

func (h *HardwareAvailabilityDetector) getGPUCapacity(gpu schema.HardwareDevice) schema.CapacityInfo {
	usage := h.getGPUUsage(gpu)
	thermal := h.getGPUThermalStatus(gpu)
	power := h.getGPUPowerStatus(gpu)

	return schema.CapacityInfo{
		MaxThroughput:    100.0, // Placeholder
		CurrentThroughput: usage.Utilization,
		MaxLoad:          100.0,
		CurrentLoad:      usage.Utilization,
		MaxTemperature:   95.0,
		CurrentTemperature: thermal.Temperature,
		MaxPower:         power.PowerLimit,
		CurrentPower:     power.CurrentPower,
	}
}

// TPU availability detection methods (simplified implementations)

func (h *HardwareAvailabilityDetector) isTPUAvailable(tpu schema.HardwareDevice) bool {
	// Check if TPU device exists
	if _, err := os.Stat(tpu.DevicePath); os.IsNotExist(err) {
		return false
	}

	// Check if TPU driver is loaded
	switch tpu.Platform {
	case "linux":
		if strings.Contains(strings.ToLower(tpu.Name), "coral") {
			cmd := exec.Command("lsmod")
			output, err := cmd.Output()
			return err == nil && strings.Contains(string(output), "apex")
		}
	case "darwin":
		if strings.Contains(strings.ToLower(tpu.Name), "neural") {
			return true // Assume available if detected
		}
	}

	return true
}

func (h *HardwareAvailabilityDetector) isTPUInUse(tpu schema.HardwareDevice) bool {
	// Check for TPU processes
	cmd := exec.Command("pgrep", "-i", "coral|edgetpu|neural")
	output, err := cmd.Output()
	return err == nil && len(strings.TrimSpace(string(output))) > 0
}

func (h *HardwareAvailabilityDetector) getTPUUsage(tpu schema.HardwareDevice) schema.UsageMetrics {
	// Simplified TPU usage detection
	return schema.UsageMetrics{
		Utilization:        0.0,
		MemoryUtilization:  0.0,
		MemoryUsed:         0,
		MemoryTotal:        0,
		CPUUtilization:     0.0,
		Temperature:        0.0,
		ClockSpeed:         0,
		PowerConsumption:   0.0,
	}
}

func (h *HardwareAvailabilityDetector) getTPUPerformance(tpu schema.HardwareDevice) schema.PerformanceMetrics {
	return schema.PerformanceMetrics{
		Temperature:      0.0,
		PowerConsumption: 0.0,
		ClockSpeed:       0,
		FanSpeed:         0,
		Throughput:       0.0,
		Latency:          0,
		Quality:          0.0,
		ErrorRate:        0.0,
	}
}

func (h *HardwareAvailabilityDetector) getTPUThermalStatus(tpu schema.HardwareDevice) schema.ThermalStatus {
	return schema.ThermalStatus{
		Temperature: 0.0,
		Throttling: false,
		Critical:   false,
		FanSpeed:   0,
	}
}

func (h *HardwareAvailabilityDetector) getTPUPowerStatus(tpu schema.HardwareDevice) schema.PowerStatus {
	return schema.PowerStatus{
		CurrentPower: 0.0,
		PowerLimit:   0.0,
		Voltage:      0.0,
		Current:      0.0,
		Efficiency:   0.0,
	}
}

func (h *HardwareAvailabilityDetector) getTPUProcesses(tpu schema.HardwareDevice) []schema.ProcessInfo {
	return []schema.ProcessInfo{}
}

func (h *HardwareAvailabilityDetector) getTPUErrors(tpu schema.HardwareDevice) []string {
	var errors []string

	if !h.isTPUAvailable(tpu) {
		errors = append(errors, "TPU not available")
	}

	return errors
}

func (h *HardwareAvailabilityDetector) getTPUWarnings(tpu schema.HardwareDevice) []string {
	return []string{}
}

func (h *HardwareAvailabilityDetector) getTPUCapacity(tpu schema.HardwareDevice) schema.CapacityInfo {
	return schema.CapacityInfo{
		MaxThroughput:     100.0,
		CurrentThroughput: 0.0,
		MaxLoad:           100.0,
		CurrentLoad:       0.0,
		MaxTemperature:    85.0,
		CurrentTemperature: 0.0,
		MaxPower:          10.0,
		CurrentPower:      0.0,
	}
}

// Camera availability detection methods

func (h *HardwareAvailabilityDetector) isCameraAvailable(camera schema.HardwareDevice) bool {
	// Check if camera device exists
	if _, err := os.Stat(camera.DevicePath); os.IsNotExist(err) {
		return false
	}

	// Check if camera is accessible
	switch camera.Platform {
	case "linux":
		cmd := exec.Command("v4l2-ctl", "--device="+camera.DevicePath, "--info")
		return cmd.Run() == nil
	case "darwin":
		// Check if camera is accessible via AVFoundation
		return true // Assume available if detected
	case "windows":
		// Check if camera is accessible via DirectShow
		return true // Assume available if detected
	}

	return true
}

func (h *HardwareAvailabilityDetector) isCameraInUse(camera schema.HardwareDevice) bool {
	switch camera.Platform {
	case "linux":
		cmd := exec.Command("lsof", camera.DevicePath)
		output, err := cmd.Output()
		return err == nil && len(strings.TrimSpace(string(output))) > 0
	case "darwin":
		cmd := exec.Command("lsof", "|", "grep", "-i", "camera")
		output, err := cmd.Output()
		return err == nil && len(strings.TrimSpace(string(output))) > 0
	case "windows":
		cmd := exec.Command("tasklist", "/fo", "csv", "/nh")
		output, err := cmd.Output()
		if err != nil {
			return false
		}
		outputStr := string(output)
		return strings.Contains(strings.ToLower(outputStr), "zoom") ||
			   strings.Contains(strings.ToLower(outputStr), "teams") ||
			   strings.Contains(strings.ToLower(outputStr), "skype")
	}
	return false
}

func (h *HardwareAvailabilityDetector) getCameraUsage(camera schema.HardwareDevice) schema.UsageMetrics {
	return schema.UsageMetrics{
		Utilization:        0.0,
		MemoryUtilization:  0.0,
		MemoryUsed:         0,
		MemoryTotal:        0,
		CPUUtilization:     0.0,
		Temperature:        0.0,
		ClockSpeed:         0,
		PowerConsumption:   0.0,
	}
}

func (h *HardwareAvailabilityDetector) getCameraPerformance(camera schema.HardwareDevice) schema.PerformanceMetrics {
	return schema.PerformanceMetrics{
		Temperature:      0.0,
		PowerConsumption: 0.0,
		ClockSpeed:       0,
		FanSpeed:         0,
		Throughput:       30.0, // 30 FPS typical
		Latency:          50,   // 50ms typical
		Quality:          100.0,
		ErrorRate:        0.0,
	}
}

func (h *HardwareAvailabilityDetector) getCameraThermalStatus(camera schema.HardwareDevice) schema.ThermalStatus {
	return schema.ThermalStatus{
		Temperature: 0.0,
		Throttling: false,
		Critical:   false,
		FanSpeed:   0,
	}
}

func (h *HardwareAvailabilityDetector) getCameraPowerStatus(camera schema.HardwareDevice) schema.PowerStatus {
	return schema.PowerStatus{
		CurrentPower: 0.0,
		PowerLimit:   0.0,
		Voltage:      0.0,
		Current:      0.0,
		Efficiency:   0.0,
	}
}

func (h *HardwareAvailabilityDetector) getCameraProcesses(camera schema.HardwareDevice) []schema.ProcessInfo {
	var processes []schema.ProcessInfo

	if h.isCameraInUse(camera) {
		// Add a generic process using the camera
		process := schema.ProcessInfo{
			Name: "Unknown Process",
		}
		processes = append(processes, process)
	}

	return processes
}

func (h *HardwareAvailabilityDetector) getCameraErrors(camera schema.HardwareDevice) []string {
	var errors []string

	if !h.isCameraAvailable(camera) {
		errors = append(errors, "Camera not available")
	}

	return errors
}

func (h *HardwareAvailabilityDetector) getCameraWarnings(camera schema.HardwareDevice) []string {
	return []string{}
}

func (h *HardwareAvailabilityDetector) getCameraCapacity(camera schema.HardwareDevice) schema.CapacityInfo {
	return schema.CapacityInfo{
		MaxThroughput:     60.0,  // 60 FPS max
		CurrentThroughput: 0.0,
		MaxLoad:           100.0,
		CurrentLoad:       0.0,
		MaxTemperature:    60.0,
		CurrentTemperature: 0.0,
		MaxPower:          2.5,
		CurrentPower:      0.0,
	}
}

// Capture card availability detection methods (simplified implementations)

func (h *HardwareAvailabilityDetector) isCaptureCardAvailable(card schema.HardwareDevice) bool {
	// Check if capture card device exists
	if _, err := os.Stat(card.DevicePath); os.IsNotExist(err) {
		return false
	}

	// Additional platform-specific checks
	switch card.Platform {
	case "linux":
		cmd := exec.Command("v4l2-ctl", "--device="+card.DevicePath, "--info")
		return cmd.Run() == nil
	case "darwin":
		// Check if capture card is accessible
		return true
	case "windows":
		// Check if capture card is accessible via DirectShow
		return true
	}

	return true
}

func (h *HardwareAvailabilityDetector) isCaptureCardInUse(card schema.HardwareDevice) bool {
	switch card.Platform {
	case "linux":
		cmd := exec.Command("lsof", card.DevicePath)
		output, err := cmd.Output()
		return err == nil && len(strings.TrimSpace(string(output))) > 0
	case "darwin":
		cmd := exec.Command("lsof", "|", "grep", "-i", "capture")
		output, err := cmd.Output()
		return err == nil && len(strings.TrimSpace(string(output))) > 0
	case "windows":
		cmd := exec.Command("tasklist", "/fo", "csv", "/nh")
		output, err := cmd.Output()
		if err != nil {
			return false
		}
		outputStr := string(output)
		return strings.Contains(strings.ToLower(outputStr), "obs") ||
			   strings.Contains(strings.ToLower(outputStr), "vlc") ||
			   strings.Contains(strings.ToLower(outputStr), "ffmpeg")
	}
	return false
}

func (h *HardwareAvailabilityDetector) getCaptureCardUsage(card schema.HardwareDevice) schema.UsageMetrics {
	return schema.UsageMetrics{
		Utilization:        0.0,
		MemoryUtilization:  0.0,
		MemoryUsed:         0,
		MemoryTotal:        0,
		CPUUtilization:     0.0,
		Temperature:        0.0,
		ClockSpeed:         0,
		PowerConsumption:   0.0,
	}
}

func (h *HardwareAvailabilityDetector) getCaptureCardPerformance(card schema.HardwareDevice) schema.PerformanceMetrics {
	return schema.PerformanceMetrics{
		Temperature:      0.0,
		PowerConsumption: 0.0,
		ClockSpeed:       0,
		FanSpeed:         0,
		Throughput:       60.0, // 60 FPS typical
		Latency:          5,    // 5ms typical
		Quality:          100.0,
		ErrorRate:        0.0,
	}
}

func (h *HardwareAvailabilityDetector) getCaptureCardThermalStatus(card schema.HardwareDevice) schema.ThermalStatus {
	return schema.ThermalStatus{
		Temperature: 0.0,
		Throttling: false,
		Critical:   false,
		FanSpeed:   0,
	}
}

func (h *HardwareAvailabilityDetector) getCaptureCardPowerStatus(card schema.HardwareDevice) schema.PowerStatus {
	return schema.PowerStatus{
		CurrentPower: 0.0,
		PowerLimit:   0.0,
		Voltage:      0.0,
		Current:      0.0,
		Efficiency:   0.0,
	}
}

func (h *HardwareAvailabilityDetector) getCaptureCardProcesses(card schema.HardwareDevice) []schema.ProcessInfo {
	var processes []schema.ProcessInfo

	if h.isCaptureCardInUse(card) {
		// Add a generic process using the capture card
		process := schema.ProcessInfo{
			Name: "Unknown Process",
		}
		processes = append(processes, process)
	}

	return processes
}

func (h *HardwareAvailabilityDetector) getCaptureCardErrors(card schema.HardwareDevice) []string {
	var errors []string

	if !h.isCaptureCardAvailable(card) {
		errors = append(errors, "Capture card not available")
	}

	return errors
}

func (h *HardwareAvailabilityDetector) getCaptureCardWarnings(card schema.HardwareDevice) []string {
	return []string{}
}

func (h *HardwareAvailabilityDetector) getCaptureCardCapacity(card schema.HardwareDevice) schema.CapacityInfo {
	return schema.CapacityInfo{
		MaxThroughput:     60.0,  // 60 FPS max
		CurrentThroughput: 0.0,
		MaxLoad:           100.0,
		CurrentLoad:       0.0,
		MaxTemperature:    70.0,
		CurrentTemperature: 0.0,
		MaxPower:          15.0,
		CurrentPower:      0.0,
	}
}