//go:build linux

package detect

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"

	"github.com/frigate-config-tool/agent/internal/schema"
)

// LinuxDetector implements hardware detection for Linux systems
type LinuxDetector struct {
	platform     string
	architecture string
}

// NewLinuxDetector creates a new Linux hardware detector
func NewLinuxDetector(platform, arch string) *LinuxDetector {
	return &LinuxDetector{
		platform:     platform,
		architecture: arch,
	}
}

// DetectGPUs detects GPU devices on Linux
func (d *LinuxDetector) DetectGPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Detect NVIDIA GPUs using nvidia-smi
	nvidiaGPUs, err := d.detectNVIDIAGPUs()
	if err == nil {
		devices = append(devices, nvidiaGPUs...)
	}

	// Detect Intel GPUs using enhanced detection
	intelGPUs, err := d.detectIntelGPUs()
	if err == nil {
		devices = append(devices, intelGPUs...)
	}

	// Detect AMD GPUs using enhanced detection
	amdGPUs, err := d.detectAMDGPUs()
	if err == nil {
		devices = append(devices, amdGPUs...)
	}

	// Detect Intel/AMD GPUs using /dev/dri/* (fallback)
	driGPUs, err := d.detectDRIDevices()
	if err == nil {
		devices = append(devices, driGPUs...)
	}

	// Remove duplicates based on ID
	seen := make(map[string]bool)
	var uniqueDevices []schema.HardwareDevice
	for _, device := range devices {
		if !seen[device.ID] {
			seen[device.ID] = true
			uniqueDevices = append(uniqueDevices, device)
		}
	}

	return uniqueDevices, nil
}

// DetectTPUs detects TPU devices on Linux (Google Coral, Hailo NPU)
func (d *LinuxDetector) DetectTPUs() ([]schema.HardwareDevice, error) {
	var allTPUs []schema.HardwareDevice

	// Detect Google Coral TPUs
	coralTPUs, err := d.detectCoralTPUs()
	if err == nil {
		allTPUs = append(allTPUs, coralTPUs...)
	}

	// Detect Hailo NPUs
	hailoNPUs, err := d.detectHailoNPUs()
	if err == nil {
		allTPUs = append(allTPUs, hailoNPUs...)
	}

	return allTPUs, nil
}

// DetectCameras detects video capture devices on Linux
func (d *LinuxDetector) DetectCameras() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Find all /dev/video* devices
	videoDevices, err := filepath.Glob("/dev/video*")
	if err != nil {
		return devices, err
	}

	for _, devicePath := range videoDevices {
		// Try to get device info from v4l2-ctl if available
		name := d.getV4L2DeviceName(devicePath)
		if name == "" {
			name = fmt.Sprintf("Video Device %s", devicePath)
		}

		// Get more detailed capabilities
		capabilities := []string{"capture"}
		videoCaps := d.getV4L2Capabilities(devicePath)
		if len(videoCaps) > 0 {
			capabilities = append(capabilities, videoCaps...)
		}

		device := schema.HardwareDevice{
			ID:               generateDeviceID("camera", devicePath),
			Type:             "camera",
			Name:             name,
			DevicePath:       devicePath,
			Capabilities:     capabilities,
			Platform:         d.platform,
			Architecture:     d.architecture,
			DetectionSource:  "v4l2",
			Available:        d.isDeviceAvailable(devicePath),
			InUse:            d.isDeviceInUse(devicePath),
		}
		devices = append(devices, device)
	}

	// Detect USB video devices
	usbDevices, err := d.detectUSBVideoDevices()
	if err == nil {
		devices = append(devices, usbDevices...)
	}

	// Remove duplicates based on device path
	seen := make(map[string]bool)
	var uniqueDevices []schema.HardwareDevice
	for _, device := range devices {
		if !seen[device.DevicePath] {
			seen[device.DevicePath] = true
			uniqueDevices = append(uniqueDevices, device)
		}
	}

	return uniqueDevices, nil
}

// DetectCaptureCards detects video capture cards on Linux
func (d *LinuxDetector) DetectCaptureCards() ([]schema.HardwareDevice, error) {
	// On Linux, capture cards typically show up as /dev/video* devices
	// They are similar to cameras but might have different capabilities
	// For now, we can return empty as cameras cover most cases
	return []schema.HardwareDevice{}, nil
}

// detectNVIDIAGPUs uses nvidia-smi to detect NVIDIA GPUs
func (d *LinuxDetector) detectNVIDIAGPUs() ([]schema.HardwareDevice, error) {
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

		devicePath := fmt.Sprintf("/dev/nvidia%s", index)

		// Enhanced capabilities detection
		capabilities := []string{"h264_decode", "h264_encode", "hevc_decode", "hevc_encode", "cuda"}

		// Add specific capabilities based on GPU architecture
		if d.hasNVENC() {
			capabilities = append(capabilities, "nvenc", "nvdec")
		}

		// Add TensorRT if available
		if d.hasTensorRT() {
			capabilities = append(capabilities, "tensorrt")
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

// detectDRIDevices detects Intel/AMD GPUs via /dev/dri/*
func (d *LinuxDetector) detectDRIDevices() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Find renderD* devices (preferred for headless systems)
	renderDevices, err := filepath.Glob("/dev/dri/renderD*")
	if err != nil {
		return devices, err
	}

	for _, devicePath := range renderDevices {
		// Try to determine vendor/model from lspci or sysfs
		name, vendor := d.getDRIDeviceInfo(devicePath)
		if name == "" {
			name = "Generic GPU"
		}

		capabilities := []string{"vaapi"}
		if vendor == "intel" {
			capabilities = append(capabilities, "qsv")
		}

		device := schema.HardwareDevice{
			ID:               generateDeviceID("gpu", devicePath),
			Type:             "gpu",
			Name:             name,
			DevicePath:       devicePath,
			Capabilities:     capabilities,
			Platform:         d.platform,
			Architecture:     d.architecture,
			DetectionSource:  "dri",
			Available:        true,
			InUse:            false,
		}
		devices = append(devices, device)
	}

	return devices, nil
}

// getV4L2DeviceName gets the friendly name of a V4L2 device
func (d *LinuxDetector) getV4L2DeviceName(devicePath string) string {
	cmd := exec.Command("v4l2-ctl", "--device="+devicePath, "--info")
	output, err := cmd.Output()
	if err != nil {
		return ""
	}

	// Parse output for card name
	scanner := bufio.NewScanner(strings.NewReader(string(output)))
	for scanner.Scan() {
		line := scanner.Text()
		if strings.HasPrefix(line, "Card type") {
			parts := strings.SplitN(line, ":", 2)
			if len(parts) == 2 {
				return strings.TrimSpace(parts[1])
			}
		}
	}

	return ""
}

// getDRIDeviceInfo gets GPU info from sysfs or lspci
func (d *LinuxDetector) getDRIDeviceInfo(devicePath string) (name string, vendor string) {
	// Try lspci first
	cmd := exec.Command("lspci", "-v")
	output, err := cmd.Output()
	if err != nil {
		return "", ""
	}

	// Look for VGA or 3D controller entries
	scanner := bufio.NewScanner(strings.NewReader(string(output)))
	vgaRegex := regexp.MustCompile(`(?i)(VGA|3D controller).*:\s*(.*)`)

	for scanner.Scan() {
		line := scanner.Text()
		matches := vgaRegex.FindStringSubmatch(line)
		if len(matches) >= 3 {
			deviceInfo := matches[2]

			// Determine vendor
			lowerInfo := strings.ToLower(deviceInfo)
			if strings.Contains(lowerInfo, "intel") {
				vendor = "intel"
				name = deviceInfo
				return
			} else if strings.Contains(lowerInfo, "amd") || strings.Contains(lowerInfo, "radeon") {
				vendor = "amd"
				name = deviceInfo
				return
			}

			// If we found a GPU but couldn't determine vendor, still return the name
			if name == "" {
				name = deviceInfo
			}
		}
	}

	return name, vendor
}

// isDeviceInUse checks if a device file is currently in use
func (d *LinuxDetector) isDeviceInUse(devicePath string) bool {
	// Try to open the device exclusively (this is a rough check)
	// In production, we might want a more sophisticated method
	file, err := os.OpenFile(devicePath, os.O_RDONLY, 0)
	if err != nil {
		// If we can't open it, assume it's in use or inaccessible
		return true
	}
	file.Close()
	return false
}

// Helper functions

func generateDeviceID(deviceType, path string) string {
	// Generate a simple ID based on device type and path
	return fmt.Sprintf("%s-%s", deviceType, filepath.Base(path))
}

// hasNVENC checks if NVENC is available for hardware encoding
func (d *LinuxDetector) hasNVENC() bool {
	cmd := exec.Command("nvidia-smi", "--query-gpu=encoder.version", "--format=csv,noheader")
	output, err := cmd.Output()
	if err != nil {
		return false
	}
	return strings.Contains(string(output), "NVENC")
}

// hasTensorRT checks if TensorRT is available
func (d *LinuxDetector) hasTensorRT() bool {
	cmd := exec.Command("nvidia-smi", "--query-gpu=driver_version", "--format=csv,noheader")
	output, err := cmd.Output()
	if err != nil {
		return false
	}
	// Check if driver version supports TensorRT (generally newer drivers)
	version := strings.TrimSpace(string(output))
	return strings.HasPrefix(version, "470.") || strings.HasPrefix(version, "510.") ||
		   strings.HasPrefix(version, "515.") || strings.HasPrefix(version, "525.") ||
		   strings.HasPrefix(version, "530.") || strings.HasPrefix(version, "535.")
}

// isNVIDIAGPUAvailable checks if NVIDIA GPU is available and not in exclusive use
func (d *LinuxDetector) isNVIDIAGPUAvailable(uuid string) bool {
	// Check if GPU has processes using it
	cmd := exec.Command("nvidia-smi", "--query-compute-apps=pid,process_name", "--format=csv,noheader", "--id="+uuid)
	output, err := cmd.Output()
	if err != nil {
		return false
	}

	// If no output, no processes are using the GPU
	return len(strings.TrimSpace(string(output))) == 0
}

// isNVIDIAGPUInUse checks if NVIDIA GPU is currently being used
func (d *LinuxDetector) isNVIDIAGPUInUse(uuid string) bool {
	// Check if GPU has processes using it
	cmd := exec.Command("nvidia-smi", "--query-compute-apps=pid,process_name", "--format=csv,noheader", "--id="+uuid)
	output, err := cmd.Output()
	if err != nil {
		return false
	}

	// If there's output, processes are using the GPU
	return len(strings.TrimSpace(string(output))) > 0
}

// detectAMDGPUs detects AMD GPUs using additional methods
func (d *LinuxDetector) detectAMDGPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use amdgpu-pro info if available
	cmd := exec.Command("amdgpu-pro-info", "--asic")
	output, err := cmd.Output()
	if err == nil {
		// Parse amdgpu-pro output for GPU information
		lines := strings.Split(string(output), "\n")
		for _, line := range lines {
			if strings.Contains(line, "ASIC name:") {
				name := strings.TrimSpace(strings.Split(line, ":")[1])
				if name != "" {
					device := schema.HardwareDevice{
						ID:               generateDeviceID("gpu", name),
						Type:             "gpu",
						Name:             name,
						DevicePath:       "/dev/dri/card0", // Default path
						Capabilities:     []string{"vaapi", "h264_decode", "h264_encode", "vce", "vcn"},
						Driver:           stringPtr("amdgpu"),
						VendorID:         stringPtr("1002"), // AMD vendor ID
						Platform:         d.platform,
						Architecture:     d.architecture,
						DetectionSource:  "amdgpu-pro",
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

// detectIntelGPUs detects Intel GPUs with enhanced detection
func (d *LinuxDetector) detectIntelGPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use intel-gpu-tools if available
	cmd := exec.Command("intel_gpu_top", "-J")
	output, err := cmd.Output()
	if err == nil {
		// Parse JSON output for GPU information
		var gpuInfo map[string]interface{}
		if err := json.Unmarshal(output, &gpuInfo); err == nil {
			if name, ok := gpuInfo["name"].(string); ok && name != "" {
				capabilities := []string{"vaapi", "qsv", "h264_decode", "h264_encode"}

				// Check for AV1 encode/decode support
				if d.hasIntelAV1Support() {
					capabilities = append(capabilities, "av1_decode", "av1_encode")
				}

				device := schema.HardwareDevice{
					ID:               generateDeviceID("gpu", name),
					Type:             "gpu",
					Name:             name,
					DevicePath:       "/dev/dri/renderD128", // Common path
					Capabilities:     capabilities,
					Driver:           stringPtr("i915"),
					VendorID:         stringPtr("8086"), // Intel vendor ID
					Platform:         d.platform,
					Architecture:     d.architecture,
					DetectionSource:  "intel-gpu-tools",
					Available:        true,
					InUse:            false,
				}
				devices = append(devices, device)
			}
		}
	}

	return devices, nil
}

// hasIntelAV1Support checks if Intel GPU supports AV1
func (d *LinuxDetector) hasIntelAV1Support() bool {
	// Check i915 parameters for AV1 support
	cmd := exec.Command("cat", "/sys/module/i915/parameters/enable_guc")
	output, err := cmd.Output()
	if err != nil {
		return false
	}
	// Newer GPUs with GuC support typically have AV1
	return strings.TrimSpace(string(output)) == "Y"
}

// detectUSBVideoDevices detects USB video devices with enhanced info
func (d *LinuxDetector) detectUSBVideoDevices() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use lsusb to find USB video devices
	cmd := exec.Command("lsusb")
	output, err := cmd.Output()
	if err != nil {
		return devices, err
	}

	lines := strings.Split(string(output), "\n")
	for _, line := range lines {
		if strings.Contains(strings.ToLower(line), "camera") ||
		   strings.Contains(strings.ToLower(line), "video") ||
		   strings.Contains(line, "13d3:") || // Many webcams use this vendor ID
		   strings.Contains(line, "046d:") || // Logitech
		   strings.Contains(line, "0bda:") || // Realtek
		   strings.Contains(line, "1b71:") { // Razer

			parts := strings.Split(line, " ")
			if len(parts) >= 6 {
				description := strings.Join(parts[6:], " ")
				deviceID := strings.TrimSpace(parts[5])

				device := schema.HardwareDevice{
					ID:               fmt.Sprintf("usb-camera-%s", deviceID),
					Type:             "camera",
					Name:             description,
					DevicePath:       "/dev/video0", // Will be mapped later
					Capabilities:     []string{"capture", "usb"},
					Platform:         d.platform,
					Architecture:     d.architecture,
					DetectionSource:  "lsusb",
					Available:        true,
					InUse:            false,
				}
				devices = append(devices, device)
			}
		}
	}

	return devices, nil
}

// getV4L2Capabilities gets detailed capabilities from v4l2-ctl
func (d *LinuxDetector) getV4L2Capabilities(devicePath string) []string {
	var capabilities []string

	cmd := exec.Command("v4l2-ctl", "--device="+devicePath, "--list-formats")
	output, err := cmd.Output()
	if err != nil {
		return capabilities
	}

	outputStr := string(output)
	lines := strings.Split(outputStr, "\n")

	for _, line := range lines {
		line = strings.TrimSpace(line)
		if strings.Contains(line, "Video Capture") {
			capabilities = append(capabilities, "video_capture")
		}
		if strings.Contains(line, "Streaming") {
			capabilities = append(capabilities, "streaming")
		}
		if strings.Contains(line, "H.264") || strings.Contains(line, "264") {
			capabilities = append(capabilities, "h264")
		}
		if strings.Contains(line, "H.265") || strings.Contains(line, "265") || strings.Contains(line, "HEVC") {
			capabilities = append(capabilities, "h265")
		}
		if strings.Contains(line, "MJPEG") {
			capabilities = append(capabilities, "mjpeg")
		}
		if strings.Contains(line, "YUYV") {
			capabilities = append(capabilities, "yuyv")
		}
	}

	return capabilities
}

// isDeviceAvailable checks if a device is available for use
func (d *LinuxDetector) isDeviceAvailable(devicePath string) bool {
	// Check if device file exists and is readable
	if _, err := os.Stat(devicePath); err != nil {
		return false
	}

	// Try to open the device for reading
	file, err := os.OpenFile(devicePath, os.O_RDONLY, 0)
	if err != nil {
		return false
	}
	file.Close()

	return true
}

// detectCoralTPUs detects Google Coral TPUs with enhanced info
func (d *LinuxDetector) detectCoralTPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Check for Coral PCIe devices
	pcieDevices, err := filepath.Glob("/sys/bus/pci/devices/*-1b96*")
	if err == nil {
		for _, devicePath := range pcieDevices {
			// Get device information from sysfs
			name := d.getPCIEDeviceName(devicePath)
			if name == "" {
				name = "Google Coral PCIe TPU"
			}

			device := schema.HardwareDevice{
				ID:               generateDeviceID("tpu", devicePath),
				Type:             "tpu",
				Name:             name,
				DevicePath:       "/dev/apex_0", // Coral default device
				Capabilities:     []string{"edge_tpu", "pcie"},
				Platform:         d.platform,
				Architecture:     d.architecture,
				DetectionSource:  "sysfs",
				Available:        d.isCoralTPUAvailable(),
				InUse:            false,
			}
			devices = append(devices, device)
		}
	}

	// Check for Coral USB devices
	usbDevices, err := filepath.Glob("/dev/bus/usb/*/*-1b96*")
	if err == nil {
		for _, devicePath := range usbDevices {
			device := schema.HardwareDevice{
				ID:               generateDeviceID("tpu", devicePath),
				Type:             "tpu",
				Name:             "Google Coral USB TPU",
				DevicePath:       devicePath,
				Capabilities:     []string{"edge_tpu", "usb"},
				Platform:         d.platform,
				Architecture:     d.architecture,
				DetectionSource:  "usb_sysfs",
				Available:        d.isCoralTPUAvailable(),
				InUse:            false,
			}
			devices = append(devices, device)
		}
	}

	return devices, nil
}

// getPCIEDeviceName gets device name from PCI sysfs
func (d *LinuxDetector) getPCIEDeviceName(devicePath string) string {
	// Read device class name
	classPath := devicePath + "/class"
	if classData, err := os.ReadFile(classPath); err == nil {
		class := strings.TrimSpace(string(classData))
		if class == "accelerator" {
			return "Google Coral PCIe Accelerator"
		}
	}

	// Try to get vendor information
	vendorPath := devicePath + "/vendor"
	if vendorData, err := os.ReadFile(vendorPath); err == nil {
		vendorID := strings.TrimSpace(string(vendorData))
		if vendorID == "0x1b96" { // Google's vendor ID
			return "Google Coral TPU"
		}
	}

	return ""
}

// isCoralTPUAvailable checks if Coral TPU is available
func (d *LinuxDetector) isCoralTPUAvailable() bool {
	// Check if Apex runtime is available
	cmd := exec.Command("lsmod")
	output, err := cmd.Output()
	if err != nil {
		return false
	}

	// Check for Apex kernel module
	return strings.Contains(string(output), "apex")
}

// detectHailoNPUs detects Hailo AI accelerators (NPUs)
func (d *LinuxDetector) detectHailoNPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Check for Hailo PCIe devices via sysfs
	// Hailo vendor ID: 0x1e60
	pcieDevices, err := filepath.Glob("/sys/bus/pci/devices/*")
	if err != nil {
		return devices, err
	}

	for _, devicePath := range pcieDevices {
		// Read vendor ID
		vendorPath := filepath.Join(devicePath, "vendor")
		vendorData, err := os.ReadFile(vendorPath)
		if err != nil {
			continue
		}

		vendorID := strings.TrimSpace(string(vendorData))

		// Check if this is a Hailo device (vendor ID 0x1e60)
		if vendorID == "0x1e60" {
			// Read device ID to determine model
			deviceIDPath := filepath.Join(devicePath, "device")
			deviceIDData, err := os.ReadFile(deviceIDPath)
			var deviceID string
			if err == nil {
				deviceID = strings.TrimSpace(string(deviceIDData))
			}

			// Determine Hailo model based on device ID
			modelName := d.getHailoModelName(deviceID)

			// Get device index from path
			deviceName := filepath.Base(devicePath)
			deviceIndex := strings.TrimPrefix(deviceName, "0000:")

			device := schema.HardwareDevice{
				ID:               fmt.Sprintf("hailo-npu-%s", deviceIndex),
				Type:             "tpu",
				Name:             modelName,
				DevicePath:       fmt.Sprintf("/dev/hailo%s", deviceIndex),
				Capabilities:     d.getHailoCapabilities(modelName),
				Driver:           stringPtr("hailo_pci"),
				VendorID:         stringPtr("1e60"),
				Platform:         d.platform,
				Architecture:     d.architecture,
				DetectionSource:  "sysfs",
				Available:        d.isHailoNPUAvailable(),
				InUse:            false,
				Metadata: map[string]interface{}{
					"vendor":     "Hailo",
					"device_id":  deviceID,
					"pcie_slot":  deviceIndex,
				},
			}
			devices = append(devices, device)
		}
	}

	// Also check for Hailo devices via /dev/hailo*
	hailoDevices, err := filepath.Glob("/dev/hailo*")
	if err == nil && len(hailoDevices) > 0 && len(devices) == 0 {
		// Found Hailo devices but couldn't detect via PCIe
		// Add a generic Hailo device
		for idx, devicePath := range hailoDevices {
			device := schema.HardwareDevice{
				ID:               fmt.Sprintf("hailo-npu-dev-%d", idx),
				Type:             "tpu",
				Name:             "Hailo AI Accelerator",
				DevicePath:       devicePath,
				Capabilities:     []string{"ai_inference", "hailo", "yolo", "object_detection"},
				Platform:         d.platform,
				Architecture:     d.architecture,
				DetectionSource:  "dev",
				Available:        true,
				InUse:            false,
			}
			devices = append(devices, device)
		}
	}

	return devices, nil
}

// getHailoModelName determines the Hailo model name from device ID
func (d *LinuxDetector) getHailoModelName(deviceID string) string {
	// Map device IDs to model names
	// These are example mappings - actual values would need to be confirmed
	switch deviceID {
	case "0x001c":
		return "Hailo-8"
	case "0x001d":
		return "Hailo-8L"
	case "0x001e":
		return "Hailo-15"
	case "0x001f":
		return "Hailo-15H"
	default:
		return "Hailo AI Accelerator"
	}
}

// getHailoCapabilities returns capabilities for Hailo NPUs
func (d *LinuxDetector) getHailoCapabilities(modelName string) []string {
	capabilities := []string{"ai_inference", "hailo", "yolo", "object_detection", "pcie"}

	// Add model-specific capabilities
	if strings.Contains(strings.ToLower(modelName), "hailo-8") {
		capabilities = append(capabilities, "26tops", "resnet", "mobilenet", "ssd")
	} else if strings.Contains(strings.ToLower(modelName), "hailo-15") {
		capabilities = append(capabilities, "high_performance", "multi_stream")
	}

	// Hailo excels at video analytics
	capabilities = append(capabilities, "video_analytics", "real_time_inference")

	return capabilities
}

// isHailoNPUAvailable checks if Hailo NPU is available
func (d *LinuxDetector) isHailoNPUAvailable() bool {
	// Check if Hailo driver is loaded
	output, err := SafeCommandExecution("lsmod")
	if err != nil {
		return false
	}

	// Check for Hailo kernel module
	return strings.Contains(output, "hailo_pci") || strings.Contains(output, "hailo")
}

func stringPtr(s string) *string {
	return &s
}
