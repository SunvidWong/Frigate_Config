package detect

import (
	"os/exec"
	"regexp"
	"strconv"
	"strings"

	"github.com/frigate-config-tool/agent/internal/schema"
)

// GPUCapabilitiesDetector provides cross-platform GPU capability detection
type GPUCapabilitiesDetector struct {
	detector Detector
}

// NewGPUCapabilitiesDetector creates a new GPU capabilities detector
func NewGPUCapabilitiesDetector(detector Detector) *GPUCapabilitiesDetector {
	return &GPUCapabilitiesDetector{
		detector: detector,
	}
}

// DetectGPUCapabilities performs comprehensive GPU capability detection
func (g *GPUCapabilitiesDetector) DetectGPUCapabilities() ([]schema.GPUCapabilities, error) {
	var capabilities []schema.GPUCapabilities

	// Get GPU devices
	gpus, err := g.detector.DetectGPUs()
	if err != nil {
		return capabilities, err
	}

	for _, gpu := range gpus {
		gpuCaps := schema.GPUCapabilities{
			DeviceID:    gpu.ID,
			DeviceName:  gpu.Name,
			Vendor:      g.getVendorFromID(gpu.VendorID),
			Driver:      g.getDriverInfo(gpu),
			Platform:    gpu.Platform,
			Architecture: gpu.Architecture,
		}

		// Detect encoding capabilities
		gpuCaps.Encode = g.detectEncodeCapabilities(gpu)

		// Detect decoding capabilities
		gpuCaps.Decode = g.detectDecodeCapabilities(gpu)

		// Detect compute capabilities
		gpuCaps.Compute = g.detectComputeCapabilities(gpu)

		// Detect memory information
		gpuCaps.Memory = g.detectMemoryCapabilities(gpu)

		// Detect API support
		gpuCaps.APIs = g.detectAPISupport(gpu)

		// Detect performance characteristics
		gpuCaps.Performance = g.detectPerformanceCharacteristics(gpu)

		capabilities = append(capabilities, gpuCaps)
	}

	return capabilities, nil
}

// detectEncodeCapabilities detects GPU encoding capabilities
func (g *GPUCapabilitiesDetector) detectEncodeCapabilities(gpu schema.HardwareDevice) schema.EncodeCapabilities {
	encode := schema.EncodeCapabilities{
		H264:  g.checkH264EncodeSupport(gpu),
		HEVC:  g.checkHEVCEncodeSupport(gpu),
		AV1:   g.checkAV1EncodeSupport(gpu),
		VP9:   g.checkVP9EncodeSupport(gpu),
		VP8:   g.checkVP8EncodeSupport(gpu),
		MPEG2: g.checkMPEG2EncodeSupport(gpu),
	}

	// Detect encoding quality levels
	encode.H264Quality = g.getEncodingQuality(gpu, "h264")
	encode.HEVCQuality = g.getEncodingQuality(gpu, "hevc")
	encode.AV1Quality = g.getEncodingQuality(gpu, "av1")

	// Detect simultaneous streams
	encode.MaxSimultaneousStreams = g.getMaxSimultaneousStreams(gpu)

	return encode
}

// detectDecodeCapabilities detects GPU decoding capabilities
func (g *GPUCapabilitiesDetector) detectDecodeCapabilities(gpu schema.HardwareDevice) schema.DecodeCapabilities {
	decode := schema.DecodeCapabilities{
		H264:  g.checkH264DecodeSupport(gpu),
		HEVC:  g.checkHEVCDecodeSupport(gpu),
		AV1:   g.checkAV1DecodeSupport(gpu),
		VP9:   g.checkVP9DecodeSupport(gpu),
		VP8:   g.checkVP8DecodeSupport(gpu),
		MPEG2: g.checkMPEG2DecodeSupport(gpu),
	}

	// Detect maximum resolution
	decode.MaxResolution = g.getMaxDecodeResolution(gpu)

	// Detect maximum frame rate
	decode.MaxFrameRate = g.getMaxDecodeFrameRate(gpu)

	// Detect simultaneous streams
	decode.MaxSimultaneousStreams = g.getMaxSimultaneousDecodeStreams(gpu)

	return decode
}

// detectComputeCapabilities detects GPU compute capabilities
func (g *GPUCapabilitiesDetector) detectComputeCapabilities(gpu schema.HardwareDevice) schema.ComputeCapabilities {
	compute := schema.ComputeCapabilities{
		CUDA:    g.checkCUDASupport(gpu),
		OpenCL:  g.checkOpenCLSupport(gpu),
		Vulkan:  g.checkVulkanSupport(gpu),
		DirectX: g.checkDirectXSupport(gpu),
		Metal:   g.checkMetalSupport(gpu),
		OpenGL:  g.checkOpenGLSupport(gpu),
	}

	// Detect compute units and cores
	compute.ComputeUnits = g.getComputeUnits(gpu)
	compute.Cores = g.getCores(gpu)

	// Detect memory bandwidth
	compute.MemoryBandwidth = g.getMemoryBandwidth(gpu)

	// Detect tensor operations support
	compute.TensorOps = g.checkTensorOpsSupport(gpu)

	return compute
}

// detectMemoryCapabilities detects GPU memory capabilities
func (g *GPUCapabilitiesDetector) detectMemoryCapabilities(gpu schema.HardwareDevice) schema.MemoryCapabilities {
	var memory schema.MemoryCapabilities

	// Try to get memory information from different sources
	switch gpu.Platform {
	case "linux":
		memory = g.getLinuxMemoryInfo(gpu)
	case "darwin":
		memory = g.getDarwinMemoryInfo(gpu)
	case "windows":
		memory = g.getWindowsMemoryInfo(gpu)
	}

	// Detect memory types
	memory.Types = g.getMemoryTypes(gpu)

	// Detect memory bandwidth
	memory.Bandwidth = g.getMemoryBandwidth(gpu)

	return memory
}

// detectAPISupport detects API support
func (g *GPUCapabilitiesDetector) detectAPISupport(gpu schema.HardwareDevice) schema.APISupport {
	var apis schema.APISupport

	// Check for specific API support based on capabilities
	for _, cap := range gpu.Capabilities {
		switch cap {
		case "cuda":
			apis.CUDA = g.getAPIVersion("cuda", gpu)
		case "opencl":
			apis.OpenCL = g.getAPIVersion("opencl", gpu)
		case "vulkan":
			apis.Vulkan = g.getAPIVersion("vulkan", gpu)
		case "metal":
			apis.Metal = g.getAPIVersion("metal", gpu)
		case "opengl":
			apis.OpenGL = g.getAPIVersion("opengl", gpu)
		case "directx":
			apis.DirectX = g.getAPIVersion("directx", gpu)
		case "d3d11":
			apis.Direct3D = g.getAPIVersion("d3d11", gpu)
		case "d3d12":
			apis.Direct3D12 = g.getAPIVersion("d3d12", gpu)
		}
	}

	return apis
}

// detectPerformanceCharacteristics detects performance characteristics
func (g *GPUCapabilitiesDetector) detectPerformanceCharacteristics(gpu schema.HardwareDevice) schema.PerformanceCharacteristics {
	var perf schema.PerformanceCharacteristics

	// Detect base and boost clock speeds
	perf.BaseClock = g.getBaseClock(gpu)
	perf.BoostClock = g.getBoostClock(gpu)

	// Detect memory clock
	perf.MemoryClock = g.getMemoryClock(gpu)

	// Detect thermal design power
	perf.TDP = g.getTDP(gpu)

	// Detect power limits
	perf.PowerLimits = g.getPowerLimits(gpu)

	// Detect thermal throttling
	perf.ThermalThrottling = g.checkThermalThrottling(gpu)

	return perf
}

// Helper functions for specific capability detection

func (g *GPUCapabilitiesDetector) checkH264EncodeSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"h264_encode", "nvenc", "qsv", "vce", "videotoolbox"})
}

func (g *GPUCapabilitiesDetector) checkHEVCEncodeSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"hevc_encode", "nvenc", "qsv", "vce", "videotoolbox"})
}

func (g *GPUCapabilitiesDetector) checkAV1EncodeSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"av1_encode", "nvenc", "videotoolbox"})
}

func (g *GPUCapabilitiesDetector) checkH264DecodeSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"h264_decode", "nvdec", "qsv", "vce", "videotoolbox", "dxva"})
}

func (g *GPUCapabilitiesDetector) checkHEVCDecodeSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"hevc_decode", "nvdec", "qsv", "vce", "videotoolbox", "dxva"})
}

func (g *GPUCapabilitiesDetector) checkAV1DecodeSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"av1_decode", "nvdec", "videotoolbox", "dxva"})
}

func (g *GPUCapabilitiesDetector) checkVP9EncodeSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"vp9_encode", "nvenc", "qsv"})
}

func (g *GPUCapabilitiesDetector) checkVP9DecodeSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"vp9_decode", "nvdec", "qsv", "dxva"})
}

func (g *GPUCapabilitiesDetector) checkVP8EncodeSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"vp8_encode", "nvenc"})
}

func (g *GPUCapabilitiesDetector) checkVP8DecodeSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"vp8_decode", "nvdec", "dxva"})
}

func (g *GPUCapabilitiesDetector) checkMPEG2EncodeSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"mpeg2_encode", "nvenc"})
}

func (g *GPUCapabilitiesDetector) checkMPEG2DecodeSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"mpeg2_decode", "nvdec", "dxva"})
}

func (g *GPUCapabilitiesDetector) checkCUDASupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"cuda", "tensorrt"})
}

func (g *GPUCapabilitiesDetector) checkOpenCLSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"opencl"})
}

func (g *GPUCapabilitiesDetector) checkVulkanSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"vulkan"})
}

func (g *GPUCapabilitiesDetector) checkDirectXSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"directx", "d3d11", "d3d12"})
}

func (g *GPUCapabilitiesDetector) checkMetalSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"metal"})
}

func (g *GPUCapabilitiesDetector) checkOpenGLSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"opengl"})
}

func (g *GPUCapabilitiesDetector) checkTensorOpsSupport(gpu schema.HardwareDevice) bool {
	return g.hasCapability(gpu, []string{"tensorrt", "tensor_cores", "neural_engine"})
}

// Utility functions

func (g *GPUCapabilitiesDetector) hasCapability(gpu schema.HardwareDevice, capabilities []string) bool {
	for _, cap := range capabilities {
		for _, gpuCap := range gpu.Capabilities {
			if strings.Contains(strings.ToLower(gpuCap), strings.ToLower(cap)) {
				return true
			}
		}
	}
	return false
}

func (g *GPUCapabilitiesDetector) getVendorFromID(vendorID *string) string {
	if vendorID == nil {
		return "Unknown"
	}

	vendorMap := map[string]string{
		"10de": "NVIDIA",
		"1002": "AMD",
		"8086": "Intel",
		"05ac": "Apple",
		"1b96": "Google",
	}

	if vendor, exists := vendorMap[*vendorID]; exists {
		return vendor
	}

	return "Unknown"
}

func (g *GPUCapabilitiesDetector) getDriverInfo(gpu schema.HardwareDevice) *string {
	return gpu.Driver
}

// Additional helper functions for detailed capability detection
func (g *GPUCapabilitiesDetector) getEncodingQuality(gpu schema.HardwareDevice, codec string) string {
	// Simplified quality detection - in practice would test actual encoding
	if strings.Contains(strings.ToLower(gpu.Name), "rtx") ||
	   strings.Contains(strings.ToLower(gpu.Name), "radeon") ||
	   strings.Contains(strings.ToLower(gpu.Name), "apple m") {
		return "high"
	}
	return "medium"
}

func (g *GPUCapabilitiesDetector) getMaxSimultaneousStreams(gpu schema.HardwareDevice) int {
	// Simplified detection - would need actual testing
	if strings.Contains(strings.ToLower(gpu.Name), "rtx 40") ||
	   strings.Contains(strings.ToLower(gpu.Name), "radeon 7000") ||
	   strings.Contains(strings.ToLower(gpu.Name), "apple m2") ||
	   strings.Contains(strings.ToLower(gpu.Name), "apple m3") {
		return 8
	}
	if strings.Contains(strings.ToLower(gpu.Name), "rtx 30") ||
	   strings.Contains(strings.ToLower(gpu.Name), "radeon 6000") ||
	   strings.Contains(strings.ToLower(gpu.Name), "apple m1") {
		return 4
	}
	return 2
}

func (g *GPUCapabilitiesDetector) getMaxDecodeResolution(gpu schema.HardwareDevice) string {
	if strings.Contains(strings.ToLower(gpu.Name), "rtx") ||
	   strings.Contains(strings.ToLower(gpu.Name), "radeon") ||
	   strings.Contains(strings.ToLower(gpu.Name), "apple m") {
		return "8K"
	}
	return "4K"
}

func (g *GPUCapabilitiesDetector) getMaxDecodeFrameRate(gpu schema.HardwareDevice) int {
	if strings.Contains(strings.ToLower(gpu.Name), "rtx") ||
	   strings.Contains(strings.ToLower(gpu.Name), "radeon") ||
	   strings.Contains(strings.ToLower(gpu.Name), "apple m") {
		return 120
	}
	return 60
}

func (g *GPUCapabilitiesDetector) getMaxSimultaneousDecodeStreams(gpu schema.HardwareDevice) int {
	return g.getMaxSimultaneousStreams(gpu) * 2 // Usually more decode streams
}

func (g *GPUCapabilitiesDetector) getComputeUnits(gpu schema.HardwareDevice) int {
	// Simplified detection - would need actual hardware queries
	if strings.Contains(strings.ToLower(gpu.Name), "rtx 4090") {
		return 128
	}
	if strings.Contains(strings.ToLower(gpu.Name), "rtx 4080") {
		return 112
	}
	if strings.Contains(strings.ToLower(gpu.Name), "apple m3 max") {
		return 40
	}
	if strings.Contains(strings.ToLower(gpu.Name), "apple m2 max") {
		return 38
	}
	return 0 // Unknown
}

func (g *GPUCapabilitiesDetector) getCores(gpu schema.HardwareDevice) int {
	// Simplified detection - would need actual hardware queries
	if strings.Contains(strings.ToLower(gpu.Name), "rtx 4090") {
		return 16384
	}
	if strings.Contains(strings.ToLower(gpu.Name), "rtx 4080") {
		return 9728
	}
	if strings.Contains(strings.ToLower(gpu.Name), "apple m3 max") {
		return 40
	}
	return 0 // Unknown
}

func (g *GPUCapabilitiesDetector) getMemoryBandwidth(gpu schema.HardwareDevice) int64 {
	// Extract from capabilities if available
	for _, cap := range gpu.Capabilities {
		if strings.HasPrefix(strings.ToLower(cap), "vram_") {
			// Extract VRAM info and estimate bandwidth
			re := regexp.MustCompile(`vram_(\d+)`)
			matches := re.FindStringSubmatch(cap)
			if len(matches) >= 2 {
				if vram, err := strconv.Atoi(matches[1]); err == nil {
					// Rough estimate: VRAM (MB) * 8 = Bandwidth (GB/s)
					return int64(vram * 8)
				}
			}
		}
	}
	return 0
}

func (g *GPUCapabilitiesDetector) getAPIVersion(api string, gpu schema.HardwareDevice) *string {
	// Simplified API version detection
	var version string

	switch gpu.Platform {
	case "linux":
		version = g.getLinuxAPIVersion(api, gpu)
	case "darwin":
		version = g.getDarwinAPIVersion(api, gpu)
	case "windows":
		version = g.getWindowsAPIVersion(api, gpu)
	}

	if version != "" {
		return &version
	}
	return nil
}

func (g *GPUCapabilitiesDetector) getLinuxAPIVersion(api string, gpu schema.HardwareDevice) string {
	switch api {
	case "cuda":
		cmd := exec.Command("nvidia-smi", "--query-gpu=driver_version", "--format=csv,noheader")
		if output, err := cmd.Output(); err == nil {
			return strings.TrimSpace(string(output))
		}
	case "vulkan":
		cmd := exec.Command("vulkaninfo", "--summary")
		if output, err := cmd.Output(); err == nil {
			// Parse Vulkan version from output
			re := regexp.MustCompile(`Vulkan API Version: ([\d.]+)`)
			matches := re.FindStringSubmatch(string(output))
			if len(matches) >= 2 {
				return matches[1]
			}
		}
	case "opengl":
		cmd := exec.Command("glxinfo", "-B")
		if output, err := cmd.Output(); err == nil {
			re := regexp.MustCompile(`OpenGL version string: ([\d.]+)`)
			matches := re.FindStringSubmatch(string(output))
			if len(matches) >= 2 {
				return matches[1]
			}
		}
	}
	return ""
}

func (g *GPUCapabilitiesDetector) getDarwinAPIVersion(api string, gpu schema.HardwareDevice) string {
	switch api {
	case "metal":
		cmd := exec.Command("system_profiler", "SPDisplaysDataType")
		if output, err := cmd.Output(); err == nil {
			re := regexp.MustCompile(`Metal: ([\d.]+)`)
			matches := re.FindStringSubmatch(string(output))
			if len(matches) >= 2 {
				return matches[1]
			}
		}
	case "opengl":
		cmd := exec.Command("system_profiler", "SPDisplaysDataType")
		if output, err := cmd.Output(); err == nil {
			re := regexp.MustCompile(`OpenGL: ([\d.]+)`)
			matches := re.FindStringSubmatch(string(output))
			if len(matches) >= 2 {
				return matches[1]
			}
		}
	}
	return ""
}

func (g *GPUCapabilitiesDetector) getWindowsAPIVersion(api string, gpu schema.HardwareDevice) string {
	switch api {
	case "directx":
		cmd := exec.Command("dxdiag", "/t", "dxdiag_output.txt")
		cmd.Run() // Ignore error for now
		// Would need to parse dxdiag_output.txt
		return "12.0" // Simplified
	case "vulkan":
		cmd := exec.Command("vulkaninfo", "--summary")
		if output, err := cmd.Output(); err == nil {
			re := regexp.MustCompile(`Vulkan API Version: ([\d.]+)`)
			matches := re.FindStringSubmatch(string(output))
			if len(matches) >= 2 {
				return matches[1]
			}
		}
	}
	return ""
}

func (g *GPUCapabilitiesDetector) getBaseClock(gpu schema.HardwareDevice) int64 {
	// Simplified clock detection - would need vendor-specific tools
	return 0
}

func (g *GPUCapabilitiesDetector) getBoostClock(gpu schema.HardwareDevice) int64 {
	// Simplified clock detection - would need vendor-specific tools
	return 0
}

func (g *GPUCapabilitiesDetector) getMemoryClock(gpu schema.HardwareDevice) int64 {
	// Simplified clock detection - would need vendor-specific tools
	return 0
}

func (g *GPUCapabilitiesDetector) getTDP(gpu schema.HardwareDevice) int64 {
	// Simplified TDP detection - would need vendor-specific queries
	return 0
}

func (g *GPUCapabilitiesDetector) getPowerLimits(gpu schema.HardwareDevice) []int64 {
	// Simplified power limit detection
	return []int64{}
}

func (g *GPUCapabilitiesDetector) checkThermalThrottling(gpu schema.HardwareDevice) bool {
	// Simplified thermal throttling detection
	return false
}

func (g *GPUCapabilitiesDetector) getLinuxMemoryInfo(gpu schema.HardwareDevice) schema.MemoryCapabilities {
	var memory schema.MemoryCapabilities

	cmd := exec.Command("nvidia-smi", "--query-gpu=memory.total,memory.used", "--format=csv,noheader")
	if output, err := cmd.Output(); err == nil {
		// Parse memory info
		parts := strings.Split(strings.TrimSpace(string(output)), ",")
		if len(parts) >= 1 {
			if total, err := strconv.ParseInt(strings.TrimSpace(parts[0]), 10, 64); err == nil {
				memory.Total = total * 1024 * 1024 // Convert MB to bytes
			}
		}
		if len(parts) >= 2 {
			if used, err := strconv.ParseInt(strings.TrimSpace(parts[1]), 10, 64); err == nil {
				memory.Used = used * 1024 * 1024 // Convert MB to bytes
			}
		}
	}

	return memory
}

func (g *GPUCapabilitiesDetector) getDarwinMemoryInfo(gpu schema.HardwareDevice) schema.MemoryCapabilities {
	var memory schema.MemoryCapabilities

	// Apple Silicon GPUs share system memory
	cmd := exec.Command("sysctl", "hw.memsize")
	if output, err := cmd.Output(); err == nil {
		re := regexp.MustCompile(`hw.memsize: (\d+)`)
		matches := re.FindStringSubmatch(string(output))
		if len(matches) >= 2 {
			if total, err := strconv.ParseInt(matches[1], 10, 64); err == nil {
				memory.Total = total
			}
		}
	}

	return memory
}

func (g *GPUCapabilitiesDetector) getWindowsMemoryInfo(gpu schema.HardwareDevice) schema.MemoryCapabilities {
	var memory schema.MemoryCapabilities

	cmd := exec.Command("nvidia-smi", "--query-gpu=memory.total,memory.used", "--format=csv,noheader")
	if output, err := cmd.Output(); err == nil {
		parts := strings.Split(strings.TrimSpace(string(output)), ",")
		if len(parts) >= 1 {
			if total, err := strconv.ParseInt(strings.TrimSpace(parts[0]), 10, 64); err == nil {
				memory.Total = total * 1024 * 1024
			}
		}
		if len(parts) >= 2 {
			if used, err := strconv.ParseInt(strings.TrimSpace(parts[1]), 10, 64); err == nil {
				memory.Used = used * 1024 * 1024
			}
		}
	}

	return memory
}

func (g *GPUCapabilitiesDetector) getMemoryTypes(gpu schema.HardwareDevice) []string {
	var types []string

	// Detect memory types based on GPU model
	lowerName := strings.ToLower(gpu.Name)

	if strings.Contains(lowerName, "gddr6") {
		types = append(types, "GDDR6")
	}
	if strings.Contains(lowerName, "gddr5") {
		types = append(types, "GDDR5")
	}
	if strings.Contains(lowerName, "hbm") {
		types = append(types, "HBM2")
	}
	if strings.Contains(lowerName, "apple") || strings.Contains(lowerName, "m1") || strings.Contains(lowerName, "m2") || strings.Contains(lowerName, "m3") {
		types = append(types, "Unified Memory")
	}

	if len(types) == 0 {
		types = append(types, "Unknown")
	}

	return types
}