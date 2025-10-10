//go:build windows

package detect

import (
	"encoding/json"
	"fmt"
	"os/exec"
	"regexp"
	"strconv"
	"strings"

	"github.com/frigate-config-tool/agent/internal/schema"
)

// detectNVIDIAGPUs uses nvidia-smi to detect NVIDIA GPUs on Windows
func (d *WindowsDetector) detectNVIDIAGPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Check if nvidia-smi is available
	cmd := exec.Command("nvidia-smi", "--query-gpu=index,name,uuid,driver_version,memory.total", "--format=csv,noheader")
	output, err := cmd.Output()
	if err != nil {
		return devices, err
	}

	lines := strings.Split(strings.TrimSpace(string(output)), "\n")
	for _, line := range lines {
		if line == "" {
			continue
		}

		parts := strings.Split(line, ",")
		if len(parts) < 4 {
			continue
		}

		index := strings.TrimSpace(parts[0])
		name := strings.TrimSpace(parts[1])
		uuid := strings.TrimSpace(parts[2])
		driverVersion := strings.TrimSpace(parts[3])
		memoryInfo := ""
		if len(parts) >= 5 {
			memoryInfo = strings.TrimSpace(parts[4])
		}

		devicePath := fmt.Sprintf("\\Device\\Video%d", index)

		// Enhanced capabilities detection
		capabilities := []string{"cuda", "d3d11", "d3d12", "dxva", "h264_decode", "h264_encode", "hevc_decode", "hevc_encode"}

		// Add specific capabilities based on GPU architecture
		if d.hasNVENC() {
			capabilities = append(capabilities, "nvenc", "nvdec")
		}

		// Add TensorRT if available
		if d.hasTensorRT() {
			capabilities = append(capabilities, "tensorrt")
		}

		// Add memory capability if available
		if memoryInfo != "" {
			capabilities = append(capabilities, "memory_"+memoryInfo)
		}

		device := schema.HardwareDevice{
			ID:               uuid,
			Type:             "gpu",
			Name:             name,
			DevicePath:       devicePath,
			Capabilities:     capabilities,
			Driver:           &driverVersion,
			VendorID:         stringPtr("10de"), // NVIDIA vendor ID
			Platform:         d.platform,
			Architecture:     d.architecture,
			DetectionSource:  "nvidia-smi",
			Available:        d.isNVIDIAGPUAvailable(uuid),
			InUse:            d.isNVIDIAGPUInUse(uuid),
		}
		devices = append(devices, device)
	}

	return devices, nil
}

// detectAMDGPUs detects AMD GPUs using AMD tools
func (d *WindowsDetector) detectAMDGPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use WMIC to detect AMD GPUs
	cmd := exec.Command("wmic", "path", "win32_VideoController", "where", "Name like '%AMD%'", "get", "Name,AdapterRAM,DriverVersion", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return devices, err
	}

	lines := strings.Split(strings.TrimSpace(string(output)), "\n")
	for _, line := range lines {
		if strings.Contains(line, "AMD") || strings.Contains(line, "Radeon") {
			fields := strings.Split(line, ",")
			if len(fields) >= 4 {
				name := strings.TrimSpace(fields[3])
				adapterRAM := strings.TrimSpace(fields[1])
				driverVersion := strings.TrimSpace(fields[2])

				if name != "" {
					capabilities := []string{"d3d11", "d3d12", "dxva", "h264_decode", "h264_encode", "hevc_decode", "hevc_encode"}

					// Add AMD-specific capabilities
					if d.hasVCE() {
						capabilities = append(capabilities, "vce", "vcn")
					}

					// Add memory capability if available
					if adapterRAM != "" && adapterRAM != " " {
						capabilities = append(capabilities, "memory_"+adapterRAM)
					}

					device := schema.HardwareDevice{
						ID:               generateWindowsDeviceID("gpu", name),
						Type:             "gpu",
						Name:             name,
						DevicePath:       "\\Device\\VideoController0",
						Capabilities:     capabilities,
						Driver:           &driverVersion,
						VendorID:         stringPtr("1002"), // AMD vendor ID
						Platform:         d.platform,
						Architecture:     d.architecture,
						DetectionSource:  "wmic",
						Available:        true,
						InUse:            false,
					}
					devices = append(devices, device)
				}
			}
		}
	}

	return devices, nil
}

// detectIntelGPUs detects Intel GPUs
func (d *WindowsDetector) detectIntelGPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use WMIC to detect Intel GPUs
	cmd := exec.Command("wmic", "path", "win32_VideoController", "where", "Name like '%Intel%'", "get", "Name,AdapterRAM,DriverVersion", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return devices, err
	}

	lines := strings.Split(strings.TrimSpace(string(output)), "\n")
	for _, line := range lines {
		if strings.Contains(line, "Intel") {
			fields := strings.Split(line, ",")
			if len(fields) >= 4 {
				name := strings.TrimSpace(fields[3])
				adapterRAM := strings.TrimSpace(fields[1])
				driverVersion := strings.TrimSpace(fields[2])

				if name != "" {
					capabilities := []string{"d3d11", "d3d12", "dxva", "h264_decode", "hevc_decode"}

					// Add Intel-specific capabilities
					if d.hasQuickSync() {
						capabilities = append(capabilities, "qsv", "quick_sync")
					}

					// Add newer capabilities based on GPU generation
					if d.hasIntelAV1Support() {
						capabilities = append(capabilities, "av1_decode")
					}

					// Add memory capability if available
					if adapterRAM != "" && adapterRAM != " " {
						capabilities = append(capabilities, "memory_"+adapterRAM)
					}

					device := schema.HardwareDevice{
						ID:               generateWindowsDeviceID("gpu", name),
						Type:             "gpu",
						Name:             name,
						DevicePath:       "\\Device\\VideoController0",
						Capabilities:     capabilities,
						Driver:           &driverVersion,
						VendorID:         stringPtr("8086"), // Intel vendor ID
						Platform:         d.platform,
						Architecture:     d.architecture,
						DetectionSource:  "wmic",
						Available:        true,
						InUse:            false,
					}
					devices = append(devices, device)
				}
			}
		}
	}

	return devices, nil
}

// detectWMICGPUs uses WMIC as fallback for general GPU detection
func (d *WindowsDetector) detectWMICGPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	cmd := exec.Command("wmic", "path", "win32_VideoController", "get", "Name,AdapterRAM,DriverVersion", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return devices, err
	}

	lines := strings.Split(strings.TrimSpace(string(output)), "\n")
	for _, line := range lines {
		fields := strings.Split(line, ",")
		if len(fields) >= 4 {
			name := strings.TrimSpace(fields[3])
			adapterRAM := strings.TrimSpace(fields[1])
			driverVersion := strings.TrimSpace(fields[2])

			if name != "" && name != "Name" {
				capabilities := []string{"d3d11", "dxva", "h264_decode"}

				// Add memory capability if available
				if adapterRAM != "" && adapterRAM != " " {
					capabilities = append(capabilities, "memory_"+adapterRAM)
				}

				device := schema.HardwareDevice{
					ID:               generateWindowsDeviceID("gpu", name),
					Type:             "gpu",
					Name:             name,
					DevicePath:       "\\Device\\VideoController0",
					Capabilities:     capabilities,
					Driver:           &driverVersion,
					Platform:         d.platform,
					Architecture:     d.architecture,
					DetectionSource:  "wmic_fallback",
					Available:        true,
					InUse:            false,
				}
				devices = append(devices, device)
			}
		}
	}

	return devices, nil
}

// detectCoralTPUs detects Google Coral TPUs on Windows
func (d *WindowsDetector) detectCoralTPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use WMI to detect USB devices with Google's vendor ID
	cmd := exec.Command("wmic", "path", "Win32_PnPEntity", "where", "DeviceID like '%USB%VID_1B96%'", "get", "Name,DeviceID", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return devices, err
	}

	lines := strings.Split(strings.TrimSpace(string(output)), "\n")
	for _, line := range lines {
		if strings.Contains(line, "Coral") || strings.Contains(line, "1B96") {
			fields := strings.Split(line, ",")
			if len(fields) >= 3 {
				name := strings.TrimSpace(fields[2])
				deviceID := strings.TrimSpace(fields[1])

				if name != "" && name != "Name" {
					device := schema.HardwareDevice{
						ID:               generateWindowsDeviceID("tpu", deviceID),
						Type:             "tpu",
						Name:             name,
						DevicePath:       deviceID,
						Capabilities:     []string{"edge_tpu", "usb"},
						VendorID:         stringPtr("1b96"), // Google vendor ID
						Platform:         d.platform,
						Architecture:     d.architecture,
						DetectionSource:  "wmic",
						Available:        true,
						InUse:            false,
					}
					devices = append(devices, device)
				}
			}
		}
	}

	return devices, nil
}

// detectOpenVINODevices detects Intel OpenVINO devices
func (d *WindowsDetector) detectOpenVINODevices() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Check if OpenVINO runtime is available
	cmd := exec.Command("where", "openvino")
	_, err := cmd.Output()
	if err == nil {
		// OpenVINO is available, add CPU and GPU inference devices
		cpuDevice := schema.HardwareDevice{
			ID:               "openvino-cpu",
			Type:             "tpu",
			Name:             "OpenVINO CPU Inference",
			DevicePath:       "cpu",
			Capabilities:     []string{"inference", "cpu", "openvino"},
			VendorID:         stringPtr("8086"), // Intel vendor ID
			Platform:         d.platform,
			Architecture:     d.architecture,
			DetectionSource:  "openvino",
			Available:        true,
			InUse:            false,
		}
		devices = append(devices, cpuDevice)

		// Add GPU device if integrated GPU is available
		if d.hasIntelGPU() {
			gpuDevice := schema.HardwareDevice{
				ID:               "openvino-gpu",
				Type:             "tpu",
				Name:             "OpenVINO GPU Inference",
				DevicePath:       "gpu",
				Capabilities:     []string{"inference", "gpu", "openvino", "d3d11"},
				VendorID:         stringPtr("8086"), // Intel vendor ID
				Platform:         d.platform,
				Architecture:     d.architecture,
				DetectionSource:  "openvino",
				Available:        true,
				InUse:            false,
			}
			devices = append(devices, gpuDevice)
		}
	}

	return devices, nil
}

// detectPowerShellCameras uses PowerShell to detect cameras
func (d *WindowsDetector) detectPowerShellCameras() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use PowerShell to get camera devices
	cmd := exec.Command("powershell", "-Command", "Get-PnpDevice -Class Camera | Select-Object FriendlyName, Status | ConvertTo-Json")
	output, err := cmd.Output()
	if err != nil {
		return devices, err
	}

	var cameras []map[string]interface{}
	if err := json.Unmarshal(output, &cameras); err == nil {
		for _, camera := range cameras {
			if friendlyName, ok := camera["FriendlyName"].(string); ok {
				if friendlyName != "" {
					capabilities := d.getCameraCapabilities(friendlyName)
					available := true
					if status, ok := camera["Status"].(string); ok {
						available = status == "OK"
					}

					device := schema.HardwareDevice{
						ID:               generateWindowsDeviceID("camera", friendlyName),
						Type:             "camera",
						Name:             friendlyName,
						DevicePath:       "\\\\?\\#camera",
						Capabilities:     capabilities,
						Platform:         d.platform,
						Architecture:     d.architecture,
						DetectionSource:  "powershell",
						Available:        available,
						InUse:            !available,
					}
					devices = append(devices, device)
				}
			}
		}
	}

	return devices, nil
}

// detectWMICCameras uses WMIC to detect cameras
func (d *WindowsDetector) detectWMICCameras() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	cmd := exec.Command("wmic", "path", "Win32_PnPEntity", "where", "PNPClass='Camera'", "get", "Name", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return devices, err
	}

	lines := strings.Split(strings.TrimSpace(string(output)), "\n")
	for _, line := range lines {
		fields := strings.Split(line, ",")
		if len(fields) >= 2 {
			name := strings.TrimSpace(fields[1])
			if name != "" && name != "Name" {
				capabilities := d.getCameraCapabilities(name)

				device := schema.HardwareDevice{
					ID:               generateWindowsDeviceID("camera", name),
					Type:             "camera",
					Name:             name,
					DevicePath:       "\\\\?\\#camera",
					Capabilities:     capabilities,
					Platform:         d.platform,
					Architecture:     d.architecture,
					DetectionSource:  "wmic",
					Available:        true,
					InUse:            false,
				}
				devices = append(devices, device)
			}
		}
	}

	return devices, nil
}

// detectWMICCaptureCards detects video capture cards using WMI
func (d *WindowsDetector) detectWMICCaptureCards() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	cmd := exec.Command("wmic", "path", "Win32_PnPEntity", "where", "PNPClass='Media' and (Name like '%capture%' or Name like '%video%')", "get", "Name,DeviceID", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return devices, err
	}

	lines := strings.Split(strings.TrimSpace(string(output)), "\n")
	for _, line := range lines {
		fields := strings.Split(line, ",")
		if len(fields) >= 3 {
			name := strings.TrimSpace(fields[2])
			deviceID := strings.TrimSpace(fields[1])

			if name != "" && name != "Name" {
				device := schema.HardwareDevice{
					ID:               generateWindowsDeviceID("capture_card", deviceID),
					Type:             "capture_card",
					Name:             name,
					DevicePath:       deviceID,
					Capabilities:     []string{"capture"},
					Platform:         d.platform,
					Architecture:     d.architecture,
					DetectionSource:  "wmic",
					Available:        true,
					InUse:            false,
				}
				devices = append(devices, device)
			}
		}
	}

	return devices, nil
}

// detectDirectShowDevices detects DirectShow video devices
func (d *WindowsDetector) detectDirectShowDevices() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// This would require implementing DirectShow detection
	// For now, return empty as DirectShow detection is complex
	// and would require additional dependencies

	return devices, nil
}

// Helper functions for capability detection

func (d *WindowsDetector) hasNVENC() bool {
	cmd := exec.Command("nvidia-smi", "--query-gpu=encoder.version", "--format=csv,noheader")
	output, err := cmd.Output()
	if err != nil {
		return false
	}
	return strings.Contains(string(output), "NVENC")
}

func (d *WindowsDetector) hasTensorRT() bool {
	cmd := exec.Command("nvidia-smi", "--query-gpu=driver_version", "--format=csv,noheader")
	output, err := cmd.Output()
	if err != nil {
		return false
	}
	version := strings.TrimSpace(string(output))
	// Check if driver version supports TensorRT
	return strings.HasPrefix(version, "470.") || strings.HasPrefix(version, "510.") ||
		strings.HasPrefix(version, "515.") || strings.HasPrefix(version, "525.") ||
		strings.HasPrefix(version, "530.") || strings.HasPrefix(version, "535.")
}

func (d *WindowsDetector) hasVCE() bool {
	// Check if AMD VCE is available by looking for AMD GPU
	cmd := exec.Command("wmic", "path", "win32_VideoController", "where", "Name like '%AMD%'", "get", "Name", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return false
	}
	return strings.Contains(string(output), "AMD") || strings.Contains(string(output), "Radeon")
}

func (d *WindowsDetector) hasQuickSync() bool {
	// Check if Intel Quick Sync is available by looking for Intel GPU
	cmd := exec.Command("wmic", "path", "win32_VideoController", "where", "Name like '%Intel%'", "get", "Name", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return false
	}
	return strings.Contains(string(output), "Intel")
}

func (d *WindowsDetector) hasIntelAV1Support() bool {
	// This would require checking Intel GPU driver version
	// For now, assume newer Intel GPUs support AV1
	cmd := exec.Command("wmic", "path", "win32_VideoController", "where", "Name like '%Intel%'", "get", "DriverVersion", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return false
	}
	// Check if driver version is recent enough (simplified)
	return strings.Contains(string(output), "30.") || strings.Contains(string(output), "31.")
}

func (d *WindowsDetector) hasIntelGPU() bool {
	cmd := exec.Command("wmic", "path", "win32_VideoController", "where", "Name like '%Intel%'", "get", "Name", "/format:csv")
	output, err := cmd.Output()
	if err != nil {
		return false
	}
	return strings.Contains(string(output), "Intel")
}

func (d *WindowsDetector) isNVIDIAGPUAvailable(uuid string) bool {
	// Check if GPU has processes using it
	cmd := exec.Command("nvidia-smi", "--query-compute-apps=pid,process_name", "--format=csv,noheader", "--id="+uuid)
	output, err := cmd.Output()
	if err != nil {
		return false
	}
	return len(strings.TrimSpace(string(output))) == 0
}

func (d *WindowsDetector) isNVIDIAGPUInUse(uuid string) bool {
	// Check if GPU has processes using it
	cmd := exec.Command("nvidia-smi", "--query-compute-apps=pid,process_name", "--format=csv,noheader", "--id="+uuid)
	output, err := cmd.Output()
	if err != nil {
		return false
	}
	return len(strings.TrimSpace(string(output))) > 0
}

func (d *WindowsDetector) getCameraCapabilities(cameraName string) []string {
	capabilities := []string{"capture"}
	lowerName := strings.ToLower(cameraName)

	// Check for specific camera capabilities
	if strings.Contains(lowerName, "hd") || strings.Contains(lowerName, "1080") {
		capabilities = append(capabilities, "high_definition")
	}

	if strings.Contains(lowerName, "4k") || strings.Contains(lowerName, "2160") {
		capabilities = append(capabilities, "4k")
	}

	if strings.Contains(lowerName, "usb") {
		capabilities = append(capabilities, "usb")
	}

	if strings.Contains(lowerName, "webcam") {
		capabilities = append(capabilities, "webcam")
	}

	return capabilities
}

func generateWindowsDeviceID(deviceType, name string) string {
	// Generate a simple ID based on device type and name
	// Remove spaces and special characters
	cleanName := regexp.MustCompile(`[^a-zA-Z0-9]+`).ReplaceAllString(name, "-")
	// Remove trailing dashes
	cleanName = strings.Trim(cleanName, "-")
	return fmt.Sprintf("%s-%s", deviceType, strings.ToLower(cleanName))
}

func stringPtr(s string) *string {
	return &s
}