//go:build darwin

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

// DarwinDetector implements hardware detection for macOS systems
type DarwinDetector struct {
	platform     string
	architecture string
}

// NewDarwinDetector creates a new macOS hardware detector
func NewDarwinDetector(platform, arch string) *DarwinDetector {
	return &DarwinDetector{
		platform:     platform,
		architecture: arch,
	}
}

// DetectGPUs detects GPU devices on macOS
func (d *DarwinDetector) DetectGPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use system_profiler to get GPU info
	cmd := exec.Command("system_profiler", "SPDisplaysDataType")
	output, err := cmd.Output()
	if err != nil {
		return devices, err
	}

	// Parse output for GPU information
	outputStr := string(output)
	lines := strings.Split(outputStr, "\n")

	var currentGPU schema.HardwareDevice
	inDisplaysSection := false

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)

		// Look for "Chipset Model:" which indicates GPU name
		if strings.HasPrefix(trimmed, "Chipset Model:") {
			inDisplaysSection = true
			parts := strings.SplitN(trimmed, ":", 2)
			if len(parts) == 2 {
				gpuName := strings.TrimSpace(parts[1])

				currentGPU = schema.HardwareDevice{
					ID:               generateDarwinDeviceID("gpu", gpuName),
					Type:             "gpu",
					Name:             gpuName,
					DevicePath:       "/dev/gpu", // macOS doesn't expose GPU devices in /dev
					Capabilities:     d.getGPUCapabilities(gpuName),
					Platform:         d.platform,
					Architecture:     d.architecture,
					DetectionSource:  "system_profiler",
					Available:        d.isGPUAvailable(gpuName),
					InUse:            d.isGPUInUse(gpuName),
				}
			}
		}

		// Extract additional GPU details
		if inDisplaysSection {
			d.extractGPUDetails(line, &currentGPU)
		}

		// If we found a GPU and now hit Metal or end of section, save it
		if inDisplaysSection && (strings.Contains(trimmed, "Metal:") || trimmed == "") {
			if currentGPU.Name != "" {
				devices = append(devices, currentGPU)
				currentGPU = schema.HardwareDevice{}
				inDisplaysSection = false
			}
		}
	}

	// Add last GPU if exists
	if currentGPU.Name != "" {
		devices = append(devices, currentGPU)
	}

	// Use GPURegistry for additional detection
	registryGPUs, err := d.detectGPURegistryGPUs()
	if err == nil {
		// Merge with existing devices, avoiding duplicates
		for _, registryGPU := range registryGPUs {
			found := false
			for _, existingGPU := range devices {
				if existingGPU.ID == registryGPU.ID {
					found = true
					break
				}
			}
			if !found {
				devices = append(devices, registryGPU)
			}
		}
	}

	return devices, nil
}

// DetectTPUs detects TPU devices on macOS
func (d *DarwinDetector) DetectTPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Check for Google Coral USB TPUs
	coralDevices, err := d.detectCoralUSBTPUs()
	if err == nil {
		devices = append(devices, coralDevices...)
	}

	// Check for other ML accelerators
	mlDevices, err := d.detectMLAccelerators()
	if err == nil {
		devices = append(devices, mlDevices...)
	}

	return devices, nil
}

// DetectCameras detects video capture devices on macOS
func (d *DarwinDetector) DetectCameras() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use system_profiler to get camera info with optimized approach
	cmd := exec.Command("system_profiler", "SPCameraDataType", "-json")
	output, err := cmd.Output()
	if err != nil {
		return devices, err
	}

	// Parse JSON camera information for better performance
	var cameraData map[string]interface{}
	if err := json.Unmarshal(output, &cameraData); err == nil {
		if spCameras, ok := cameraData["SPCameraDataType"]; ok {
			if camerasArray, ok := spCameras.([]interface{}); ok {
				for _, camera := range camerasArray {
					if cameraMap, ok := camera.(map[string]interface{}); ok {
						// Extract camera name
						cameraName := ""
						if name, ok := cameraMap["_name"].(string); ok {
							cameraName = name
						}

						// Skip invalid entries
						if cameraName == "" || strings.Contains(cameraName, "Model") || strings.Contains(cameraName, "Unique ID") {
							continue
						}

						// Get detailed camera capabilities
						capabilities := d.getCameraCapabilities(cameraName)

						device := schema.HardwareDevice{
							ID:               generateDarwinDeviceID("camera", cameraName),
							Type:             "camera",
							Name:             cameraName,
							DevicePath:       "/dev/video0", // macOS abstracts camera access
							Capabilities:     capabilities,
							Platform:         d.platform,
							Architecture:     d.architecture,
							DetectionSource:  "system_profiler_json",
							Available:        true, // Assume available for better performance
							InUse:            false,  // Assume not in use for better performance
						}
						devices = append(devices, device)
					}
				}
			}
		}
	}

	// Limit USB camera detection to avoid timeouts
	// Only run if we didn't find cameras above
	if len(devices) == 0 {
		usbCameras, err := d.detectUSBCamerasFast()
		if err == nil {
			devices = append(devices, usbCameras...)
		}
	}

	return devices, nil
}

// DetectCaptureCards detects video capture cards on macOS
func (d *DarwinDetector) DetectCaptureCards() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use ioreg to find USB video capture devices
	cmd := exec.Command("ioreg", "-p", "IOUSB", "-l", "-w", "0")
	output, err := cmd.Output()
	if err != nil {
		return devices, nil // Return empty on error
	}

	outputStr := string(output)

	// Look for video-related USB devices
	lines := strings.Split(outputStr, "\n")
	for _, line := range lines {
		if strings.Contains(strings.ToLower(line), "video") ||
		   strings.Contains(strings.ToLower(line), "capture") ||
		   strings.Contains(strings.ToLower(line), "webcam") {

			// Extract device information
			deviceInfo := d.extractUSBDeviceInfo(line)
			if deviceInfo != nil && deviceInfo.name != "" {
				device := schema.HardwareDevice{
					ID:               generateDarwinDeviceID("capture_card", deviceInfo.name),
					Type:             "capture_card",
					Name:             deviceInfo.name,
					DevicePath:       "/dev/video" + deviceInfo.index, // Abstract path
					Capabilities:     []string{"capture", "usb"},
					Driver:           deviceInfo.driver,
					VendorID:         deviceInfo.vendorID,
					Platform:         d.platform,
					Architecture:     d.architecture,
					DetectionSource:  "ioreg",
					Available:        true,
					InUse:            false,
				}
				devices = append(devices, device)
			}
		}
	}

	return devices, nil
}

// extractGPUDetails extracts detailed GPU information from system_profiler output
func (d *DarwinDetector) extractGPUDetails(line string, gpu *schema.HardwareDevice) {
	trimmed := strings.TrimSpace(line)

	// Extract VRAM
	if strings.HasPrefix(trimmed, "VRAM (Total):") {
		parts := strings.SplitN(trimmed, ":", 2)
		if len(parts) == 2 {
			vramStr := strings.TrimSpace(parts[1])
			if vram, err := strconv.ParseInt(vramStr, 10, 64); err == nil {
				capability := fmt.Sprintf("vram_%dmb", vram)
				gpu.Capabilities = append(gpu.Capabilities, capability)
			}
		}
	}

	// Extract Metal version
	if strings.HasPrefix(trimmed, "Metal:") {
		parts := strings.SplitN(trimmed, ":", 2)
		if len(parts) == 2 {
			metalVersion := strings.TrimSpace(parts[1])
			if gpu.Driver == nil {
				gpu.Driver = &metalVersion
			}
			// Add Metal-specific capabilities
			if strings.Contains(metalVersion, "2.") {
				gpu.Capabilities = append(gpu.Capabilities, "metal_2")
			}
			if strings.Contains(metalVersion, "3.") {
				gpu.Capabilities = append(gpu.Capabilities, "metal_3")
			}
		}
	}
}

// detectGPURegistryGPUs uses GPURegistry to find additional GPUs
func (d *DarwinDetector) detectGPURegistryGPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use system_profiler SPDisplaysDataType with GPU information
	cmd := exec.Command("system_profiler", "SPDisplaysDataType", "-json")
	output, err := cmd.Output()
	if err != nil {
		return devices, err
	}

	// Parse JSON output
	var displaysData map[string]interface{}
	if err := json.Unmarshal(output, &displaysData); err != nil {
		return devices, err
	}

	// Look for GPUs in the parsed data
	if spDisplays, ok := displaysData["SPDisplaysDataType"]; ok {
		if displaysArray, ok := spDisplays.([]interface{}); ok {
			for _, display := range displaysArray {
				if displayMap, ok := display.(map[string]interface{}); ok {
					// Check if this is a GPU display
					if gpuType, ok := displayMap["sppci_type"].(string); ok {
						if strings.Contains(strings.ToLower(gpuType), "gpu") ||
						   strings.Contains(strings.ToLower(gpuType), "vga") {

							// Extract GPU name
							gpuName := ""
							if name, ok := displayMap["sppci_model"].(string); ok {
								gpuName = name
							}

							if gpuName != "" {
								device := schema.HardwareDevice{
									ID:               generateDarwinDeviceID("gpu", gpuName),
									Type:             "gpu",
									Name:             gpuName,
									DevicePath:       "/dev/gpu",
									Capabilities:     d.getGPUCapabilities(gpuName),
									Platform:         d.platform,
									Architecture:     d.architecture,
									DetectionSource:  "gpu_registry",
									Available:        true,
									InUse:            false,
								}
								devices = append(devices, device)
							}
						}
					}
				}
			}
		}
	}

	return devices, nil
}

// detectCoralUSBTPUs detects Google Coral USB TPUs on macOS
func (d *DarwinDetector) detectCoralUSBTPUs() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use system_profiler to find USB devices
	cmd := exec.Command("system_profiler", "SPUSBDataType")
	output, err := cmd.Output()
	if err != nil {
		return devices, err
	}

	outputStr := string(output)
	lines := strings.Split(outputStr, "\n")

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)

		// Look for Google Coral devices
		if strings.Contains(trimmed, "Coral") ||
		   strings.Contains(trimmed, "1b96:") { // Google's vendor ID

			// Extract device information
			deviceInfo := d.extractUSBDeviceInfo(trimmed)
			if deviceInfo != nil && strings.Contains(strings.ToLower(deviceInfo.name), "coral") {
				device := schema.HardwareDevice{
					ID:               generateDarwinDeviceID("tpu", deviceInfo.name),
					Type:             "tpu",
					Name:             deviceInfo.name,
					DevicePath:       "/dev/apex_0",
					Capabilities:     []string{"edge_tpu", "usb"},
					Driver:           deviceInfo.driver,
					VendorID:         deviceInfo.vendorID,
					Platform:         d.platform,
					Architecture:     d.architecture,
					DetectionSource:  "system_profiler",
					Available:        d.isTPUAvailable(),
					InUse:            false,
				}
				devices = append(devices, device)
			}
		}
	}

	return devices, nil
}

// detectMLAccelerators detects other ML accelerators
func (d *DarwinDetector) detectMLAccelerators() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Check for Neural Engine
	if d.hasNeuralEngine() {
		device := schema.HardwareDevice{
			ID:               "neural-engine",
			Type:             "tpu",
			Name:             "Apple Neural Engine",
			DevicePath:       "/dev/neural-engine",
			Capabilities:     []string{"neural_engine", "coreml"},
			Driver:           stringPtr("Apple"),
			VendorID:         stringPtr("05ac"), // Apple vendor ID
			Platform:         d.platform,
			Architecture:     d.architecture,
			DetectionSource:  "system_check",
			Available:        true,
			InUse:            false,
		}
		devices = append(devices, device)
	}

	return devices, nil
}

// getCameraCapabilities determines camera capabilities based on name
func (d *DarwinDetector) getCameraCapabilities(cameraName string) []string {
	capabilities := []string{"capture"}
	lowerName := strings.ToLower(cameraName)

	// Check for FaceTime HD capabilities
	if strings.Contains(lowerName, "facetime") {
		capabilities = append(capabilities, "h264", "face_time")
	}

	// Check for high-resolution capabilities
	if strings.Contains(lowerName, "1080") || strings.Contains(lowerName, "4k") {
		capabilities = append(capabilities, "high_resolution")
	}

	// Check for wide-angle capabilities
	if strings.Contains(lowerName, "wide") || strings.Contains(lowerName, "ultra") {
		capabilities = append(capabilities, "wide_angle")
	}

	return capabilities
}

// detectUSBCameras detects USB cameras using ioreg
func (d *DarwinDetector) detectUSBCameras() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	cmd := exec.Command("ioreg", "-p", "IOUSB", "-l", "-w", "0")
	output, err := cmd.Output()
	if err != nil {
		return devices, nil
	}

	outputStr := string(output)
	lines := strings.Split(outputStr, "\n")

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)

		// Look for USB video devices
		if strings.Contains(strings.ToLower(line), "camera") ||
		   strings.Contains(strings.ToLower(line), "webcam") ||
		   strings.Contains(line, "13d3:") || // Many webcams
		   strings.Contains(line, "046d:") || // Logitech
		   strings.Contains(line, "0bda:") { // Realtek

			deviceInfo := d.extractUSBDeviceInfo(trimmed)
			if deviceInfo != nil {
				device := schema.HardwareDevice{
					ID:               generateDarwinDeviceID("camera", deviceInfo.name),
					Type:             "camera",
					Name:             deviceInfo.name,
					DevicePath:       "/dev/video0", // Will be mapped by the system
					Capabilities:     []string{"capture", "usb"},
					Driver:           deviceInfo.driver,
					VendorID:         deviceInfo.vendorID,
					Platform:         d.platform,
					Architecture:     d.architecture,
					DetectionSource:  "ioreg",
					Available:        true,
					InUse:            false,
				}
				devices = append(devices, device)
			}
		}
	}

	return devices, nil
}

// detectUSBCamerasFast detects USB cameras using a faster approach
func (d *DarwinDetector) detectUSBCamerasFast() ([]schema.HardwareDevice, error) {
	var devices []schema.HardwareDevice

	// Use a simpler ioreg command for faster execution
	cmd := exec.Command("ioreg", "-p", "IOUSB")
	output, err := cmd.Output()
	if err != nil {
		return devices, nil
	}

	outputStr := string(output)
	// Look for common USB camera indicators
	if strings.Contains(outputStr, "Camera") || strings.Contains(outputStr, "Webcam") {
		// Add a generic USB camera device
		device := schema.HardwareDevice{
			ID:               "usb-camera-generic",
			Type:             "camera",
			Name:             "USB Camera",
			DevicePath:       "/dev/video0",
			Capabilities:     []string{"capture", "usb"},
			Platform:         d.platform,
			Architecture:     d.architecture,
			DetectionSource:  "ioreg_fast",
			Available:        true,
			InUse:            false,
		}
		devices = append(devices, device)
	}

	return devices, nil
}

// extractUSBDeviceInfo extracts USB device information
func (d *DarwinDetector) extractUSBDeviceInfo(line string) *struct {
	name      string
	driver    *string
	vendorID  *string
	index     string
} {
	info := &struct {
		name      string
		driver    *string
		vendorID  *string
		index     string
	}{}

	// Extract product name
	productRegex := regexp.MustCompile(`"USB Product Name" = "([^"]+)"`)
	if matches := productRegex.FindStringSubmatch(line); len(matches) >= 2 {
		info.name = matches[1]
	}

	// Extract vendor ID
	vendorRegex := regexp.MustCompile(`"Vendor ID" = 0x([0-9a-fA-F]+)`)
	if matches := vendorRegex.FindStringSubmatch(line); len(matches) >= 2 {
		vendorID := "0x" + matches[1]
		info.vendorID = &vendorID
	}

	// Extract device index
	indexRegex := regexp.MustCompile(`@(\d+)`)
	if matches := indexRegex.FindStringSubmatch(line); len(matches) >= 2 {
		info.index = matches[1]
	}

	// Extract driver if available
	driverRegex := regexp.MustCompile(`"IOUserClientClass" = "([^"]+)"`)
	if matches := driverRegex.FindStringSubmatch(line); len(matches) >= 2 {
		driver := matches[1]
		info.driver = &driver
	}

	return info
}

// isGPUAvailable checks if GPU is available
func (d *DarwinDetector) isGPUAvailable(gpuName string) bool {
	// Check if GPU process is running
	cmd := exec.Command("pgrep", "-f", "WindowServer")
	err := cmd.Run()
	return err == nil
}

// isGPUInUse checks if GPU is currently being used
func (d *DarwinDetector) isGPUInUse(gpuName string) bool {
	// Check for GPU-intensive processes
	cmd := exec.Command("pgrep", "-i", "metal|opengl|gpu")
	output, err := cmd.Output()
	if err != nil {
		return false
	}

	// If any GPU-related processes are running, consider GPU in use
	return len(strings.TrimSpace(string(output))) > 0
}

// isCameraAvailable checks if camera is available
func (d *DarwinDetector) isCameraAvailable(cameraName string) bool {
	// Check if any process is using the camera
	cmd := exec.Command("sh", "-c", "lsof | grep -i camera")
	output, err := cmd.Output()
	if err != nil {
		return true // Assume available if we can't check
	}

	// If no processes are using camera devices, it's available
	return !strings.Contains(strings.ToLower(string(output)), "video")
}

// isCameraInUse checks if camera is currently being used
func (d *DarwinDetector) isCameraInUse(cameraName string) bool {
	cmd := exec.Command("sh", "-c", "lsof | grep -i video")
	output, err := cmd.Output()
	if err != nil {
		return false
	}

	return len(strings.TrimSpace(string(output))) > 0
}

// isTPUAvailable checks if TPU is available
func (d *DarwinDetector) isTPUAvailable() bool {
	// Check for Apex kernel module
	cmd := exec.Command("sh", "-c", "kextstat | grep apex")
	output, err := cmd.Output()
	if err != nil {
		return false
	}

	return strings.Contains(string(output), "apex")
}

// hasNeuralEngine checks if Neural Engine is available
func (d *DarwinDetector) hasNeuralEngine() bool {
	// Check macOS version for Neural Engine support
	cmd := exec.Command("sw_vers", "-productVersion")
	output, err := cmd.Output()
	if err != nil {
		return false
	}

	version := strings.TrimSpace(string(output))
	// Neural Engine is available on Apple Silicon (M1/M2/M3)
	return strings.HasPrefix(version, "11.") ||
		   strings.HasPrefix(version, "12.") ||
		   strings.HasPrefix(version, "13.") ||
		   strings.HasPrefix(version, "14.") ||
		   strings.HasPrefix(version, "15.")
}

// getGPUCapabilities determines GPU capabilities based on name/type
func (d *DarwinDetector) getGPUCapabilities(gpuName string) []string {
	lowerName := strings.ToLower(gpuName)
	capabilities := []string{"metal"} // All macOS GPUs support Metal

	// Apple Silicon
	if strings.Contains(lowerName, "apple") {
		capabilities = append(capabilities, "videotoolbox", "h264_decode", "h264_encode", "hevc_decode", "hevc_encode")

		// Check for specific Apple Silicon capabilities
		if d.hasAV1Support() {
			capabilities = append(capabilities, "av1_decode", "av1_encode")
		}
		if d.hasProResSupport() {
			capabilities = append(capabilities, "prores")
		}
	}

	// Intel GPUs
	if strings.Contains(lowerName, "intel") {
		capabilities = append(capabilities, "videotoolbox", "h264_decode")
		if d.hasQuickSync() {
			capabilities = append(capabilities, "qsv")
		}
	}

	// AMD GPUs
	if strings.Contains(lowerName, "amd") || strings.Contains(lowerName, "radeon") {
		capabilities = append(capabilities, "h264_decode", "h264_encode")
		if d.hasVCE() {
			capabilities = append(capabilities, "vce")
		}
	}

	return capabilities
}

// hasAV1Support checks if GPU supports AV1
func (d *DarwinDetector) hasAV1Support() bool {
	// Check macOS version and GPU type
	cmd := exec.Command("sw_vers", "-productVersion")
	output, err := cmd.Output()
	if err != nil {
		return false
	}

	version := strings.TrimSpace(string(output))
	// AV1 support in newer macOS versions
	return strings.HasPrefix(version, "13.") ||
		   strings.HasPrefix(version, "14.") ||
		   strings.HasPrefix(version, "15.")
}

// hasProResSupport checks if ProRes RAW is supported
func (d *DarwinDetector) hasProResSupport() bool {
	// This is simplified - in practice would check specific GPU models
	return strings.Contains(strings.ToLower(d.getMacModel()), "pro")
}

// hasQuickSync checks if Intel Quick Sync is available
func (d *DarwinDetector) hasQuickSync() bool {
	// This would need to check specific Intel GPU models
	// For now, assume Intel GPUs have Quick Sync on macOS
	cmd := exec.Command("system_profiler", "SPDisplaysDataType")
	output, err := cmd.Output()
	if err != nil {
		return false
	}

	return strings.Contains(string(output), "Intel")
}

// hasVCE checks if AMD VCE is available
func (d *DarwinDetector) hasVCE() bool {
	// This would need to check specific AMD GPU models
	cmd := exec.Command("system_profiler", "SPDisplaysDataType")
	output, err := cmd.Output()
	if err != nil {
		return false
	}

	return strings.Contains(string(output), "AMD") ||
	       strings.Contains(string(output), "Radeon")
}

// getMacModel gets the Mac model
func (d *DarwinDetector) getMacModel() string {
	cmd := exec.Command("system_profiler", "SPHardwareDataType")
	output, err := cmd.Output()
	if err != nil {
		return ""
	}

	outputStr := string(output)
	lines := strings.Split(outputStr, "\n")
	for _, line := range lines {
		if strings.Contains(line, "Model Name:") {
			parts := strings.SplitN(line, ":", 2)
			if len(parts) == 2 {
				return strings.TrimSpace(parts[1])
			}
		}
	}
	return ""
}

func generateDarwinDeviceID(deviceType, name string) string {
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