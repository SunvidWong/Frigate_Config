//go:build windows

package detect

import (
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"strings"

	"github.com/frigate-config-tool/agent/internal/schema"
)

// WindowsDetector implements hardware detection for Windows systems
type WindowsDetector struct {
	platform     string
	architecture string
}

// NewWindowsDetector creates a new Windows hardware detector
func NewWindowsDetector(platform, arch string) *WindowsDetector {
	return &WindowsDetector{
		platform:     platform,
		architecture: arch,
	}
}

// DetectGPUs detects GPU devices on Windows
func (d *WindowsDetector) DetectGPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Detect NVIDIA GPUs using nvidia-smi
	nvidiaGPUs, err := d.detectNVIDIAGPUs()
	if err == nil {
		devices = append(devices, nvidiaGPUs...)
	}

	// Detect AMD GPUs using AMD tools
	amdGPUs, err := d.detectAMDGPUs()
	if err == nil {
		devices = append(devices, amdGPUs...)
	}

	// Detect Intel GPUs
	intelGPUs, err := d.detectIntelGPUs()
	if err == nil {
		devices = append(devices, intelGPUs...)
	}

	// Use WMIC as fallback for general GPU detection
	wmicGPUs, err := d.detectWMICGPUs()
	if err == nil {
		// Merge with existing devices, avoiding duplicates
		for _, wmicGPU := range wmicGPUs {
			found := false
			for _, existingGPU := range devices {
				if existingGPU.ID == wmicGPU.ID {
					found = true
					break
				}
			}
			if !found {
				devices = append(devices, wmicGPU)
			}
		}
	}

	return devices, nil
}

// DetectTPUs detects TPU devices on Windows
func (d *WindowsDetector) DetectTPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Detect Google Coral TPUs
	coralTPUs, err := d.detectCoralTPUs()
	if err == nil {
		devices = append(devices, coralTPUs...)
	}

	// Detect Intel OpenVINO devices
	openvinoDevices, err := d.detectOpenVINODevices()
	if err == nil {
		devices = append(devices, openvinoDevices...)
	}

	return devices, nil
}

// DetectCameras detects video capture devices on Windows
func (d *WindowsDetector) DetectCameras() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use PowerShell for camera detection
	powerShellCameras, err := d.detectPowerShellCameras()
	if err == nil {
		devices = append(devices, powerShellCameras...)
	}

	// Use WMIC as fallback
	wmicCameras, err := d.detectWMICCameras()
	if err == nil {
		// Merge with existing devices, avoiding duplicates
		for _, wmicCamera := range wmicCameras {
			found := false
			for _, existingCamera := range devices {
				if existingCamera.ID == wmicCamera.ID {
					found = true
					break
				}
			}
			if !found {
				devices = append(devices, wmicCamera)
			}
		}
	}

	return devices, nil
}

// DetectCaptureCards detects video capture cards on Windows
func (d *WindowsDetector) DetectCaptureCards() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use WMI to detect video capture devices
	wmicCaptureCards, err := d.detectWMICCaptureCards()
	if err == nil {
		devices = append(devices, wmicCaptureCards...)
	}

	// Use DirectShow to detect additional devices
	directShowDevices, err := d.detectDirectShowDevices()
	if err == nil {
		// Merge with existing devices, avoiding duplicates
		for _, dsDevice := range directShowDevices {
			found := false
			for _, existingDevice := range devices {
				if existingDevice.ID == dsDevice.ID {
					found = true
					break
				}
			}
			if !found {
				devices = append(devices, dsDevice)
			}
		}
	}

	return devices, nil
}

// DetectAll detects all hardware devices on Windows
func (d *WindowsDetector) DetectAll() ([]schema.HardwareDevice, error) {
	var allDevices []schema.HardwareDevice

	// Detect GPUs
	gpus, err := d.DetectGPUs()
	if err == nil {
		allDevices = append(allDevices, gpus...)
	}

	// Detect TPUs
	tpus, err := d.DetectTPUs()
	if err == nil {
		allDevices = append(allDevices, tpus...)
	}

	// Detect Cameras
	cameras, err := d.DetectCameras()
	if err == nil {
		allDevices = append(allDevices, cameras...)
	}

	// Detect Capture Cards
	captureCards, err := d.DetectCaptureCards()
	if err == nil {
		allDevices = append(allDevices, captureCards...)
	}

	return allDevices, nil
}

// detectNVIDIAGPUs detects NVIDIA GPUs using nvidia-smi
func (d *WindowsDetector) detectNVIDIAGPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Try to run nvidia-smi
	cmd := exec.Command("nvidia-smi", "--query-gpu=name,driver_version,memory.total", "--format=csv,noheader,nounits")
	output, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("nvidia-smi not available: %v", err)
	}

	lines := strings.Split(string(output), "\n")
	for i, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}

		parts := strings.Split(line, ",")
		if len(parts) >= 3 {
			name := strings.TrimSpace(parts[0])
			driver := strings.TrimSpace(parts[1])
			memory := strings.TrimSpace(parts[2])

			capabilities := d.getNVIDIACapabilities(name)

			device := schema.HardwareDevice{
				ID:             fmt.Sprintf("nvidia-gpu-%d", i),
				Type:           "gpu",
				Name:           name,
				DevicePath:     fmt.Sprintf("/dev/nvidia%d", i),
				Capabilities:   capabilities,
				Driver:         &driver,
				Platform:       d.platform,
				Architecture:   d.architecture,
				VendorID:       stringPtr("10de"), // NVIDIA PCI ID
				DetectionSource: "nvidia-smi",
				Available:      true,
				InUse:          d.isNVIDIAGPUInUse(i),
			}

			// Add memory info to capabilities
			if memMB, err := strconv.Atoi(memory); err == nil {
				device.Capabilities = append(device.Capabilities, fmt.Sprintf("memory_%dmb", memMB))
			}

			devices = append(devices, device)
		}
	}

	return devices, nil
}

// detectAMDGPUs detects AMD GPUs
func (d *WindowsDetector) detectAMDGPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use WMIC to detect AMD GPUs
	cmd := exec.Command("wmic", "path", "win32_VideoController", "get", "name,AdapterRAM,DriverVersion", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("WMIC GPU detection failed: %v", err)
	}

	lines := strings.Split(string(output), "\n")
	for i, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.Contains(line, "Node,") {
			continue
		}

		parts := strings.Split(line, ",")
		if len(parts) >= 4 {
			name := strings.TrimSpace(parts[1])
			if strings.Contains(strings.ToLower(name), "amd") || strings.Contains(strings.ToLower(name), "radeon") {
				memory := strings.TrimSpace(parts[2])
				driver := strings.TrimSpace(parts[3])

				capabilities := d.getAMDCapabilities(name)

				device := schema.HardwareDevice{
					ID:             fmt.Sprintf("amd-gpu-%d", i),
					Type:           "gpu",
					Name:           name,
					DevicePath:     fmt.Sprintf("/dev/dri/card%d", i),
					Capabilities:   capabilities,
					Driver:         &driver,
					Platform:       d.platform,
					Architecture:   d.architecture,
					VendorID:       stringPtr("1002"), // AMD PCI ID
					DetectionSource: "wmic",
					Available:      true,
					InUse:          false,
				}

				// Add memory info
				if memory != "" && memory != "0" {
					device.Capabilities = append(device.Capabilities, "memory_"+memory)
				}

				devices = append(devices, device)
			}
		}
	}

	return devices, nil
}

// detectIntelGPUs detects Intel GPUs
func (d *WindowsDetector) detectIntelGPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use WMIC to detect Intel GPUs
	cmd := exec.Command("wmic", "path", "win32_VideoController", "get", "name,AdapterRAM,DriverVersion", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("WMIC GPU detection failed: %v", err)
	}

	lines := strings.Split(string(output), "\n")
	for i, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.Contains(line, "Node,") {
			continue
		}

		parts := strings.Split(line, ",")
		if len(parts) >= 4 {
			name := strings.TrimSpace(parts[1])
			if strings.Contains(strings.ToLower(name), "intel") &&
			   (strings.Contains(strings.ToLower(name), "hd graphics") ||
			    strings.Contains(strings.ToLower(name), "iris") ||
			    strings.Contains(strings.ToLower(name), "uhd") ||
			    strings.Contains(strings.ToLower(name), "arc")) {

				memory := strings.TrimSpace(parts[2])
				driver := strings.TrimSpace(parts[3])

				capabilities := d.getIntelCapabilities(name)

				device := schema.HardwareDevice{
					ID:             fmt.Sprintf("intel-gpu-%d", i),
					Type:           "gpu",
					Name:           name,
					DevicePath:     fmt.Sprintf("/dev/dri/card%d", i),
					Capabilities:   capabilities,
					Driver:         &driver,
					Platform:       d.platform,
					Architecture:   d.architecture,
					VendorID:       stringPtr("8086"), // Intel PCI ID
					DetectionSource: "wmic",
					Available:      true,
					InUse:          false,
				}

				// Add memory info
				if memory != "" && memory != "0" {
					device.Capabilities = append(device.Capabilities, "memory_"+memory)
				}

				devices = append(devices, device)
			}
		}
	}

	return devices, nil
}

// detectWMICGPUs detects GPUs using WMIC as fallback
func (d *WindowsDetector) detectWMICGPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	cmd := exec.Command("wmic", "path", "win32_VideoController", "get", "name,DeviceID", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("WMIC GPU detection failed: %v", err)
	}

	lines := strings.Split(string(output), "\n")
	for i, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.Contains(line, "Node,") {
			continue
		}

		parts := strings.Split(line, ",")
		if len(parts) >= 3 {
			name := strings.TrimSpace(parts[1])
			deviceID := strings.TrimSpace(parts[2])

			device := schema.HardwareDevice{
				ID:             fmt.Sprintf("wmic-gpu-%d", i),
				Type:           "gpu",
				Name:           name,
				DevicePath:     deviceID,
				Capabilities:   []string{"display"},
				Platform:       d.platform,
				Architecture:   d.architecture,
				DetectionSource: "wmic",
				Available:      true,
				InUse:          false,
			}

			devices = append(devices, device)
		}
	}

	return devices, nil
}

// detectCoralTPUs detects Google Coral TPUs
func (d *WindowsDetector) detectCoralTPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Check for Coral USB devices
	cmd := exec.Command("wmic", "path", "win32_PnPEntity", "where", "Description='USB Device'", "get", "Name,DeviceID", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("WMIC USB device detection failed: %v", err)
	}

	lines := strings.Split(string(output), "\n")
	for i, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.Contains(line, "Node,") {
			continue
		}

		parts := strings.Split(line, ",")
		if len(parts) >= 3 {
			name := strings.TrimSpace(parts[1])
			deviceID := strings.TrimSpace(parts[2])

			if strings.Contains(strings.ToLower(name), "coral") ||
			   strings.Contains(strings.ToLower(name), "edge") {

				device := schema.HardwareDevice{
					ID:             fmt.Sprintf("coral-tpu-%d", i),
					Type:           "tpu",
					Name:           name,
					DevicePath:     deviceID,
					Capabilities:   []string{"edge_tpu", "tensorflow_lite"},
					Platform:       d.platform,
					Architecture:   d.architecture,
					VendorID:       stringPtr("1a86"), // Google USB vendor ID
					DetectionSource: "wmic",
					Available:      true,
					InUse:          false,
				}

				devices = append(devices, device)
			}
		}
	}

	return devices, nil
}

// detectOpenVINODevices detects Intel OpenVINO devices
func (d *WindowsDetector) detectOpenVINODevices() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Check for OpenVINO runtime installation
	cmd := exec.Command("where", "benchmark_app")
	output, err := cmd.Output()
	if err != nil {
		return devices, nil // OpenVINO not installed
	}

	if len(output) > 0 {
		device := schema.HardwareDevice{
			ID:             "openvino-cpu",
			Type:           "tpu",
			Name:           "Intel OpenVINO CPU",
			DevicePath:     "/dev/cpu",
			Capabilities:   []string{"openvino", "cpu", "inference"},
			Platform:       d.platform,
			Architecture:   d.architecture,
			VendorID:       stringPtr("8086"),
			DetectionSource: "openvino_check",
			Available:      true,
			InUse:          false,
		}

		devices = append(devices, device)
	}

	return devices, nil
}

// detectPowerShellCameras detects cameras using PowerShell
func (d *WindowsDetector) detectPowerShellCameras() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use PowerShell to get camera devices
	psCommand := "Get-WmiObject -Class Win32_PnPEntity | Where-Object {$_.PNPClass -eq 'Camera' -or $_.PNPClass -eq 'Image'} | Select-Object Name, DeviceID | ConvertTo-Csv -NoTypeInformation"
	cmd := exec.Command("powershell", "-Command", psCommand)
	output, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("PowerShell camera detection failed: %v", err)
	}

	lines := strings.Split(string(output), "\n")
	for i, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.Contains(line, "Name") || strings.Contains(line, "\"") {
			continue
		}

		parts := strings.Split(line, ",")
		if len(parts) >= 2 {
			name := strings.Trim(strings.TrimSpace(parts[0]), "\"")
			deviceID := strings.Trim(strings.TrimSpace(parts[1]), "\"")

			if name != "" && deviceID != "" {
				capabilities := d.getCameraCapabilities(name)

				device := schema.HardwareDevice{
					ID:             fmt.Sprintf("camera-%d", i),
					Type:           "camera",
					Name:           name,
					DevicePath:     deviceID,
					Capabilities:   capabilities,
					Platform:       d.platform,
					Architecture:   d.architecture,
					DetectionSource: "powershell",
					Available:      true,
					InUse:          d.isCameraInUse(deviceID),
				}

				devices = append(devices, device)
			}
		}
	}

	return devices, nil
}

// detectWMICCameras detects cameras using WMIC
func (d *WindowsDetector) detectWMICCameras() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	cmd := exec.Command("wmic", "path", "win32_PnPEntity", "where", "PNPClass='Camera' or PNPClass='Image'", "get", "Name,DeviceID", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("WMIC camera detection failed: %v", err)
	}

	lines := strings.Split(string(output), "\n")
	for i, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.Contains(line, "Node,") {
			continue
		}

		parts := strings.Split(line, ",")
		if len(parts) >= 3 {
			name := strings.TrimSpace(parts[1])
			deviceID := strings.TrimSpace(parts[2])

			if name != "" && deviceID != "" {
				device := schema.HardwareDevice{
					ID:             fmt.Sprintf("wmic-camera-%d", i),
					Type:           "camera",
					Name:           name,
					DevicePath:     deviceID,
					Capabilities:   []string{"capture"},
					Platform:       d.platform,
					Architecture:   d.architecture,
					DetectionSource: "wmic",
					Available:      true,
					InUse:          false,
				}

				devices = append(devices, device)
			}
		}
	}

	return devices, nil
}

// detectWMICCaptureCards detects capture cards using WMI
func (d *WindowsDetector) detectWMICCaptureCards() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	cmd := exec.Command("wmic", "path", "win32_PnPEntity", "where", "PNPClass='Media' or PNPClass='Video'", "get", "Name,DeviceID,Description", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("WMIC capture card detection failed: %v", err)
	}

	lines := strings.Split(string(output), "\n")
	for i, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.Contains(line, "Node,") {
			continue
		}

		parts := strings.Split(line, ",")
		if len(parts) >= 4 {
			name := strings.TrimSpace(parts[1])
			deviceID := strings.TrimSpace(parts[2])
			description := strings.TrimSpace(parts[3])

			if d.isCaptureCard(name, description) {
				capabilities := d.getCaptureCardCapabilities(name, description)

				device := schema.HardwareDevice{
					ID:             fmt.Sprintf("capture-card-%d", i),
					Type:           "capture_card",
					Name:           name,
					DevicePath:     deviceID,
					Capabilities:   capabilities,
					Platform:       d.platform,
					Architecture:   d.architecture,
					DetectionSource: "wmic",
					Available:      true,
					InUse:          false,
				}

				devices = append(devices, device)
			}
		}
	}

	return devices, nil
}

// detectDirectShowDevices detects DirectShow devices
func (d *WindowsDetector) detectDirectShowDevices() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use DirectX diagnostic tool to get video capture devices
	cmd := exec.Command("dxdiag", "/t", "dxdiag_output.txt")
	cmd.Run() // Ignore error, file may not be created

	// Try to read the output file
	output, err := os.ReadFile("dxdiag_output.txt")
	if err != nil {
		return devices, nil // DirectX diag not available
	}

	content := string(output)
	lines := strings.Split(content, "\n")

	i := 0
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if strings.Contains(strings.ToLower(line), "video capture") ||
		   strings.Contains(strings.ToLower(line), "capture card") ||
		   strings.Contains(strings.ToLower(line), "hDMI") {

			device := schema.HardwareDevice{
				ID:             fmt.Sprintf("directshow-%d", i),
				Type:           "capture_card",
				Name:           line,
				DevicePath:     fmt.Sprintf("/dev/video%d", i),
				Capabilities:   []string{"directshow", "video_capture"},
				Platform:       d.platform,
				Architecture:   d.architecture,
				DetectionSource: "directx",
				Available:      true,
				InUse:          false,
			}

			devices = append(devices, device)
			i++
		}
	}

	return devices, nil
}

// Helper methods for capability detection and device status

// getNVIDIACapabilities determines NVIDIA GPU capabilities based on name
func (d *WindowsDetector) getNVIDIACapabilities(name string) []string {
	var capabilities []string
	lowerName := strings.ToLower(name)

	// Basic NVIDIA capabilities
	capabilities = append(capabilities, "cuda", "nvenc", "nvdec", "opengl", "directx")

	// Codec support
	capabilities = append(capabilities, "h264_decode", "h264_encode")
	capabilities = append(capabilities, "hevc_decode", "hevc_encode")

	// Advanced features based on GPU series
	if strings.Contains(lowerName, "rtx") {
		capabilities = append(capabilities, "tensorrt", "ray_tracing", "dlss")
		if strings.Contains(lowerName, "rtx 40") || strings.Contains(lowerName, "rtx 30") {
			capabilities = append(capabilities, "av1_decode", "av1_encode")
		}
	}

	if strings.Contains(lowerName, "quadro") {
		capabilities = append(capabilities, "professional", "ecc_memory")
	}

	// Tensor cores for AI workloads
	if strings.Contains(lowerName, "20") || strings.Contains(lowerName, "30") || strings.Contains(lowerName, "40") {
		capabilities = append(capabilities, "tensor_cores")
	}

	return capabilities
}

// getAMDCapabilities determines AMD GPU capabilities based on name
func (d *WindowsDetector) getAMDCapabilities(name string) []string {
	var capabilities []string
	lowerName := strings.ToLower(name)

	// Basic AMD capabilities
	capabilities = append(capabilities, "opencl", "vulkan", "opengl", "directx")

	// AMD encoding/decoding
	capabilities = append(capabilities, "h264_decode", "h264_encode")
	capabilities = append(capabilities, "hevc_decode", "hevc_encode")

	// RDNA and later architectures
	if strings.Contains(lowerName, "rx") && (strings.Contains(lowerName, "6000") || strings.Contains(lowerName, "7000")) {
		capabilities = append(capabilities, "rdna2", "ray_tracing")
		capabilities = append(capabilities, "av1_decode", "av1_encode")
	}

	// Professional cards
	if strings.Contains(lowerName, "pro") || strings.Contains(lowerName, "instinct") {
		capabilities = append(capabilities, "professional", "compute")
	}

	return capabilities
}

// getIntelCapabilities determines Intel GPU capabilities based on name
func (d *WindowsDetector) getIntelCapabilities(name string) []string {
	var capabilities []string
	lowerName := strings.ToLower(name)

	// Basic Intel capabilities
	capabilities = append(capabilities, "opengl", "directx", "quick_sync")

	// Intel Quick Sync Video
	capabilities = append(capabilities, "h264_decode", "h264_encode")
	capabilities = append(capabilities, "hevc_decode", "hevc_encode")

	// Newer Intel GPUs
	if strings.Contains(lowerName, "iris") || strings.Contains(lowerName, "arc") {
		capabilities = append(capabilities, "av1_decode", "vp9_decode", "vp8_decode")
		capabilities = append(capabilities, "opencl", "vulkan")
	}

	// Arc GPUs
	if strings.Contains(lowerName, "arc") {
		capabilities = append(capabilities, "ray_tracing", "xmx", "ai_acceleration")
	}

	return capabilities
}

// getCameraCapabilities determines camera capabilities based on name
func (d *WindowsDetector) getCameraCapabilities(name string) []string {
	var capabilities []string
	lowerName := strings.ToLower(name)

	// Basic capture capabilities
	capabilities = append(capabilities, "capture", "video_capture")

	// Resolution support based on typical webcams
	if strings.Contains(lowerName, "hd") || strings.Contains(lowerName, "720p") {
		capabilities = append(capabilities, "720p")
	}
	if strings.Contains(lowerName, "1080p") || strings.Contains(lowerName, "fullhd") {
		capabilities = append(capabilities, "1080p")
	}
	if strings.Contains(lowerName, "4k") || strings.Contains(lowerName, "2160p") {
		capabilities = append(capabilities, "4k")
	}

	// Special features
	if strings.Contains(lowerName, "wide") || strings.Contains(lowerName, "wfov") {
		capabilities = append(capabilities, "wide_angle")
	}
	if strings.Contains(lowerName, "infrared") || strings.Contains(lowerName, "ir") {
		capabilities = append(capabilities, "infrared")
	}

	return capabilities
}

// getCaptureCardCapabilities determines capture card capabilities
func (d *WindowsDetector) getCaptureCardCapabilities(name, description string) []string {
	var capabilities []string
	lowerName := strings.ToLower(name)
	lowerDesc := strings.ToLower(description)

	// Basic capture capabilities
	capabilities = append(capabilities, "video_capture", "hardware_encoding")

	// Input types
	if strings.Contains(lowerName, "hdmi") || strings.Contains(lowerDesc, "hdmi") {
		capabilities = append(capabilities, "hdmi_input")
	}
	if strings.Contains(lowerName, "sdi") || strings.Contains(lowerDesc, "sdi") {
		capabilities = append(capabilities, "sdi_input")
	}
	if strings.Contains(lowerName, "component") || strings.Contains(lowerDesc, "component") {
		capabilities = append(capabilities, "component_input")
	}
	if strings.Contains(lowerName, "composite") || strings.Contains(lowerDesc, "composite") {
		capabilities = append(capabilities, "composite_input")
	}

	// Professional features
	if strings.Contains(lowerName, "professional") || strings.Contains(lowerName, "pro") {
		capabilities = append(capabilities, "professional_grade")
	}
	if strings.Contains(lowerName, "4k") || strings.Contains(lowerDesc, "4k") {
		capabilities = append(capabilities, "4k_support")
	}

	return capabilities
}

// isNVIDIAGPUInUse checks if NVIDIA GPU is currently in use
func (d *WindowsDetector) isNVIDIAGPUInUse(gpuIndex int) bool {
	// Try to check GPU utilization using nvidia-smi
	cmd := exec.Command("nvidia-smi", "--query-gpu=utilization_gpu", "--format=csv,noheader,nounits", fmt.Sprintf("--id=%d", gpuIndex))
	output, err := cmd.Output()
	if err != nil {
		return false // Cannot determine usage, assume not in use
	}

	utilization := strings.TrimSpace(string(output))
	if utilization == "" {
		return false
	}

	// If utilization is greater than 0%, consider GPU in use
	if utilPercent, err := strconv.Atoi(utilization); err == nil && utilPercent > 0 {
		return true
	}

	return false
}

// isCameraInUse checks if camera is currently in use
func (d *WindowsDetector) isCameraInUse(deviceID string) bool {
	// Check if camera is being used by checking file handles
	cmd := exec.Command("powershell", "-Command", fmt.Sprintf("Get-WmiObject -Class Win32_Process | Where-Object {$_.CommandLine -like '*%s*'}", deviceID))
	output, err := cmd.Output()
	if err != nil {
		return false
	}

	return len(strings.TrimSpace(string(output))) > 0
}

// isCaptureCard determines if a device is a capture card based on name and description
func (d *WindowsDetector) isCaptureCard(name, description string) bool {
	lowerName := strings.ToLower(name)
	lowerDesc := strings.ToLower(description)

	// Common capture card indicators
	captureCardTerms := []string{
		"capture", "capture card", "video capture", "hDMI capture", "SDI capture",
		"av to usb", "video converter", "game capture", "streaming",
		"elgato", "hauppauge", "blackmagic", "magewell", "avermedia",
	}

	for _, term := range captureCardTerms {
		if strings.Contains(lowerName, term) || strings.Contains(lowerDesc, term) {
			return true
		}
	}

	return false
}

// stringPtr returns a pointer to a string value
func stringPtr(s string) *string {
	return &s
}
