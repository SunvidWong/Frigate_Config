package detect

import (
	"strings"
	"time"

	"github.com/frigate-config-tool/agent/internal/schema"
)

// TPUCapabilitiesDetector provides comprehensive TPU capability detection
type TPUCapabilitiesDetector struct {
	detector Detector
}

// NewTPUCapabilitiesDetector creates a new TPU capabilities detector
func NewTPUCapabilitiesDetector(detector Detector) *TPUCapabilitiesDetector {
	return &TPUCapabilitiesDetector{
		detector: detector,
	}
}

// DetectTPUCapabilities performs comprehensive TPU capability detection
func (t *TPUCapabilitiesDetector) DetectTPUCapabilities() ([]schema.TPUCapabilities, error) {
	var capabilities []schema.TPUCapabilities

	// Get TPU devices
	tpus, err := t.detector.DetectTPUs()
	if err != nil {
		return capabilities, err
	}

	for _, tpu := range tpus {
		tpuCaps := schema.TPUCapabilities{
			DeviceID:     tpu.ID,
			DeviceName:   tpu.Name,
			DevicePath:   tpu.DevicePath,
			Platform:     tpu.Platform,
			Architecture: tpu.Architecture,
		}

		// Detect TPU type and generation
		tpuCaps.Type = t.detectTPUType(tpu)
		tpuCaps.Generation = t.detectGeneration(tpu)

		// Detect compute capabilities
		tpuCaps.Compute = t.detectComputeCapabilities(tpu)

		// Detect memory capabilities
		tpuCaps.Memory = t.detectMemoryCapabilities(tpu)

		// Detect supported models
		tpuCaps.SupportedModels = t.getSupportedModels(tpu)

		// Detect performance characteristics
		tpuCaps.Performance = t.detectPerformanceCharacteristics(tpu)

		// Detect power characteristics
		tpuCaps.Power = t.detectPowerCharacteristics(tpu)

		// Detect supported frameworks
		tpuCaps.Frameworks = t.getSupportedFrameworks(tpu)

		// Detect thermal characteristics
		tpuCaps.Thermal = t.detectThermalCharacteristics(tpu)

		capabilities = append(capabilities, tpuCaps)
	}

	return capabilities, nil
}

// detectTPUType identifies the TPU type
func (t *TPUCapabilitiesDetector) detectTPUType(tpu schema.HardwareDevice) string {
	name := strings.ToLower(tpu.Name)

	// Google Coral devices
	if strings.Contains(name, "coral") || strings.Contains(name, "edge") {
		if strings.Contains(name, "usb") {
			return "Coral USB"
		}
		if strings.Contains(name, "pcie") || strings.Contains(name, "m.2") {
			return "Coral PCIe"
		}
		if strings.Contains(name, "dev") || strings.Contains(name, "mini") {
			return "Coral Dev Board"
		}
		return "Coral"
	}

	// Apple Neural Engine
	if strings.Contains(name, "neural") || strings.Contains(name, "apple") {
		return "Apple Neural Engine"
	}

	// Intel OpenVINO
	if strings.Contains(name, "openvino") || strings.Contains(name, "intel") {
		return "Intel OpenVINO"
	}

	// Google Edge TPU
	if strings.Contains(name, "edge tpu") {
		return "Google Edge TPU"
	}

	// Generic TPU
	return "Generic TPU"
}

// detectGeneration identifies the TPU generation
func (t *TPUCapabilitiesDetector) detectGeneration(tpu schema.HardwareDevice) string {
	name := strings.ToLower(tpu.Name)

	// Coral generations
	if strings.Contains(name, "coral") {
		if strings.Contains(name, "m.2") {
			return "v2" // M.2 Coral uses v2
		}
		if strings.Contains(name, "usb") {
			return "v2" // USB Coral uses v2
		}
		if strings.Contains(name, "dev") {
			return "v1" // Dev boards use v1
		}
		return "v2"
	}

	// Apple Neural Engine generations
	if strings.Contains(name, "neural") {
		if strings.Contains(name, "m3") {
			return "4th Gen"
		}
		if strings.Contains(name, "m2") {
			return "3rd Gen"
		}
		if strings.Contains(name, "m1") {
			return "2nd Gen"
		}
		return "1st Gen"
	}

	return "Unknown"
}

// detectComputeCapabilities detects compute capabilities
func (t *TPUCapabilitiesDetector) detectComputeCapabilities(tpu schema.HardwareDevice) schema.TPUComputeCapabilities {
	compute := schema.TPUComputeCapabilities{
		MaxOPS:            t.getMaxOPS(tpu),
		MaxTOPS:           t.getMaxTOPS(tpu),
		SupportedOps:      t.getSupportedOps(tpu),
		Precision:         t.getSupportedPrecision(tpu),
		DataTypes:         t.getSupportedDataTypes(tpu),
		BatchSize:         t.getBatchSizeRange(tpu),
		Latency:           t.getLatency(tpu),
		Throughput:        t.getThroughput(tpu),
		PowerEfficiency:   t.getPowerEfficiency(tpu),
		Quantization:      t.getQuantizationSupport(tpu),
		Pruning:           t.getPruningSupport(tpu),
		Sparsity:          t.getSparsitySupport(tpu),
	}

	return compute
}

// detectMemoryCapabilities detects memory capabilities
func (t *TPUCapabilitiesDetector) detectMemoryCapabilities(tpu schema.HardwareDevice) schema.TPUMemoryCapabilities {
	memory := schema.TPUMemoryCapabilities{
		Total:     t.getTotalMemory(tpu),
		Available: t.getAvailableMemory(tpu),
		Bandwidth: t.getMemoryBandwidth(tpu),
		Type:      t.getMemoryType(tpu),
		Cache:     t.getCacheSize(tpu),
	}

	return memory
}

// getSupportedModels gets list of supported AI models
func (t *TPUCapabilitiesDetector) getSupportedModels(tpu schema.HardwareDevice) []string {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe", "Coral Dev Board", "Google Edge TPU":
		return []string{
			"MobileNet v1/v2",
			"EfficientNet",
			"Inception v1/v3",
			"ResNet v1/v2",
			"SSD MobileNet",
			"YOLO v5/v8",
			"DeepLab v3+",
			"PoseNet",
			"BERT (Quantized)",
			"GPT-2 (Quantized)",
			"Custom TensorFlow Lite models",
		}
	case "Apple Neural Engine":
		return []string{
			"CoreML models",
			"TensorFlow Lite models",
			"ONNX models",
			"PyTorch models (converted)",
			"Custom neural networks",
		}
	case "Intel OpenVINO":
		return []string{
			"OpenVINO IR models",
			"TensorFlow models",
			"ONNX models",
			"Caffe models",
			"MXNet models",
			"Keras models",
		}
	default:
		return []string{
			"Generic neural networks",
		}
	}
}

// detectPerformanceCharacteristics detects performance characteristics
func (t *TPUCapabilitiesDetector) detectPerformanceCharacteristics(tpu schema.HardwareDevice) schema.TPUPerformanceCharacteristics {
	perf := schema.TPUPerformanceCharacteristics{
		MaxFrequency:  t.getMaxFrequency(tpu),
		BaseFrequency: t.getBaseFrequency(tpu),
		BoostFrequency: t.getBoostFrequency(tpu),
		ClockSpeed:    t.getClockSpeed(tpu),
		InstructionsPerCycle: t.getInstructionsPerCycle(tpu),
		PipelineDepth: t.getPipelineDepth(tpu),
		Cores:         t.getCores(tpu),
		ExecutionUnits: t.getExecutionUnits(tpu),
	}

	return perf
}

// detectPowerCharacteristics detects power characteristics
func (t *TPUCapabilitiesDetector) detectPowerCharacteristics(tpu schema.HardwareDevice) schema.TPUPowerCharacteristics {
	power := schema.TPUPowerCharacteristics{
		TDP:           t.getTDP(tpu),
		BasePower:     t.getBasePower(tpu),
		MaxPower:      t.getMaxPower(tpu),
		IdlePower:     t.getIdlePower(tpu),
		PowerStates:   t.getPowerStates(tpu),
		VoltageRange:  t.getVoltageRange(tpu),
		CurrentRange:  t.getCurrentRange(tpu),
		Efficiency:    t.getPowerEfficiency(tpu),
	}

	return power
}

// getSupportedFrameworks gets list of supported ML frameworks
func (t *TPUCapabilitiesDetector) getSupportedFrameworks(tpu schema.HardwareDevice) []string {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe", "Coral Dev Board", "Google Edge TPU":
		return []string{
			"TensorFlow Lite",
			"PyCoral",
			"C++ Coral SDK",
			"Edge TPU Compiler",
		}
	case "Apple Neural Engine":
		return []string{
			"CoreML",
			"TensorFlow Lite",
			"PyTorch",
			"ONNX Runtime",
			"CreateML",
		}
	case "Intel OpenVINO":
		return []string{
			"OpenVINO Toolkit",
			"TensorFlow",
			"PyTorch",
			"ONNX",
			"Caffe",
		}
	default:
		return []string{
			"Generic ML frameworks",
		}
	}
}

// detectThermalCharacteristics detects thermal characteristics
func (t *TPUCapabilitiesDetector) detectThermalCharacteristics(tpu schema.HardwareDevice) schema.TPUThermalCharacteristics {
	thermal := schema.TPUThermalCharacteristics{
		MaxTemperature:     t.getMaxTemperature(tpu),
		OptimalTemperature: t.getOptimalTemperature(tpu),
		ThrottlingTemperature: t.getThrottlingTemperature(tpu),
		CriticalTemperature:  t.getCriticalTemperature(tpu),
		CoolingSolution:      t.getCoolingSolution(tpu),
		ThermalManagement:    t.getThermalManagement(tpu),
	}

	return thermal
}

// Helper methods for capability detection

func (t *TPUCapabilitiesDetector) getMaxOPS(tpu schema.HardwareDevice) int64 {
	tpuType := t.detectTPUType(tpu)
	generation := t.detectGeneration(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe":
		if generation == "v2" {
			return 4000000000000 // 4 TOPS
		}
		return 4000000000000 // 4 TOPS for v1 as well
	case "Apple Neural Engine":
		if generation == "4th Gen" {
			return 38000000000000 // 38 TOPS
		}
		if generation == "3rd Gen" {
			return 15800000000000 // 15.8 TOPS
		}
		if generation == "2nd Gen" {
			return 11000000000000 // 11 TOPS
		}
		return 5000000000000 // 5 TOPS for 1st gen
	case "Intel OpenVINO":
		return 10000000000000 // 10 TOPS estimated
	default:
		return 1000000000000 // 1 TOPS default
	}
}

func (t *TPUCapabilitiesDetector) getMaxTOPS(tpu schema.HardwareDevice) float64 {
	ops := t.getMaxOPS(tpu)
	return float64(ops) / 1000000000000 // Convert to TOPS
}

func (t *TPUCapabilitiesDetector) getSupportedOps(tpu schema.HardwareDevice) []string {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe", "Google Edge TPU":
		return []string{
			"Convolution",
			"Depthwise Convolution",
			"Average Pooling",
			"Max Pooling",
			"Fully Connected",
			"Softmax",
			"Sigmoid",
			"Tanh",
			"ReLU",
			"Add",
			"Concat",
			"Resize",
			"Transpose",
		}
	case "Apple Neural Engine":
		return []string{
			"Convolution",
			"Batch Normalization",
			"Pooling",
			"Fully Connected",
			"Activation Functions",
			"Attention Mechanisms",
			"Transformer Operations",
			"Recurrent Operations",
			"Custom Operations",
		}
	case "Intel OpenVINO":
		return []string{
			"Convolution",
			"Deconvolution",
			"Pooling",
			"Fully Connected",
			"Activation",
			"Normalization",
			"Permutation",
			"Reshape",
			"Concat",
			"Split",
			"Eltwise",
		}
	default:
		return []string{
			"Basic neural network operations",
		}
	}
}

func (t *TPUCapabilitiesDetector) getSupportedPrecision(tpu schema.HardwareDevice) []string {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe", "Google Edge TPU":
		return []string{"INT8"}
	case "Apple Neural Engine":
		return []string{"FP16", "FP32", "INT8"}
	case "Intel OpenVINO":
		return []string{"FP32", "FP16", "INT8", "INT16"}
	default:
		return []string{"FP32"}
	}
}

func (t *TPUCapabilitiesDetector) getSupportedDataTypes(tpu schema.HardwareDevice) []string {
	precision := t.getSupportedPrecision(tpu)
	var dataTypes []string

	for _, prec := range precision {
		switch prec {
		case "FP32":
			dataTypes = append(dataTypes, "float32")
		case "FP16":
			dataTypes = append(dataTypes, "float16", "bfloat16")
		case "INT8":
			dataTypes = append(dataTypes, "int8", "uint8")
		case "INT16":
			dataTypes = append(dataTypes, "int16", "uint16")
		}
	}

	return dataTypes
}

func (t *TPUCapabilitiesDetector) getBatchSizeRange(tpu schema.HardwareDevice) schema.BatchSizeRange {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe":
		return schema.BatchSizeRange{Min: 1, Max: 16}
	case "Apple Neural Engine":
		return schema.BatchSizeRange{Min: 1, Max: 64}
	case "Intel OpenVINO":
		return schema.BatchSizeRange{Min: 1, Max: 128}
	default:
		return schema.BatchSizeRange{Min: 1, Max: 8}
	}
}

func (t *TPUCapabilitiesDetector) getLatency(tpu schema.HardwareDevice) time.Duration {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB":
		return 15 * time.Millisecond
	case "Coral PCIe":
		return 5 * time.Millisecond
	case "Apple Neural Engine":
		return 2 * time.Millisecond
	case "Intel OpenVINO":
		return 3 * time.Millisecond
	default:
		return 10 * time.Millisecond
	}
}

func (t *TPUCapabilitiesDetector) getThroughput(tpu schema.HardwareDevice) float64 {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB":
		return 30.0 // 30 FPS
	case "Coral PCIe":
		return 60.0 // 60 FPS
	case "Apple Neural Engine":
		return 100.0 // 100 FPS
	case "Intel OpenVINO":
		return 80.0 // 80 FPS
	default:
		return 20.0 // 20 FPS
	}
}

func (t *TPUCapabilitiesDetector) getPowerEfficiency(tpu schema.HardwareDevice) float64 {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB":
		return 2.0 // 2 TOPS/W
	case "Coral PCIe":
		return 4.0 // 4 TOPS/W
	case "Apple Neural Engine":
		return 10.0 // 10 TOPS/W
	case "Intel OpenVINO":
		return 5.0 // 5 TOPS/W
	default:
		return 1.0 // 1 TOPS/W
	}
}

func (t *TPUCapabilitiesDetector) getQuantizationSupport(tpu schema.HardwareDevice) bool {
	return true // Most TPUs support quantization
}

func (t *TPUCapabilitiesDetector) getPruningSupport(tpu schema.HardwareDevice) bool {
	tpuType := t.detectTPUType(tpu)
	return tpuType == "Apple Neural Engine" || tpuType == "Intel OpenVINO"
}

func (t *TPUCapabilitiesDetector) getSparsitySupport(tpu schema.HardwareDevice) bool {
	tpuType := t.detectTPUType(tpu)
	return tpuType == "Apple Neural Engine"
}

func (t *TPUCapabilitiesDetector) getTotalMemory(tpu schema.HardwareDevice) int64 {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB":
		return 8 * 1024 * 1024 // 8MB
	case "Coral PCIe":
		return 8 * 1024 * 1024 // 8MB
	case "Apple Neural Engine":
		return 16 * 1024 * 1024 // 16MB (shared with system memory)
	case "Intel OpenVINO":
		return 32 * 1024 * 1024 // 32MB (estimated)
	default:
		return 8 * 1024 * 1024 // 8MB default
	}
}

func (t *TPUCapabilitiesDetector) getAvailableMemory(tpu schema.HardwareDevice) int64 {
	total := t.getTotalMemory(tpu)
	return int64(float64(total) * 0.8) // Assume 80% available
}

func (t *TPUCapabilitiesDetector) getMemoryBandwidth(tpu schema.HardwareDevice) int64 {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB":
		return 400000000 // 400 MB/s
	case "Coral PCIe":
		return 2500000000 // 2.5 GB/s
	case "Apple Neural Engine":
		return 68250000000 // 68.25 GB/s (M1)
	case "Intel OpenVINO":
		return 10000000000 // 10 GB/s (estimated)
	default:
		return 1000000000 // 1 GB/s
	}
}

func (t *TPUCapabilitiesDetector) getMemoryType(tpu schema.HardwareDevice) string {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe":
		return "On-chip SRAM"
	case "Apple Neural Engine":
		return "Unified Memory"
	case "Intel OpenVINO":
		return "DRAM"
	default:
		return "Unknown"
	}
}

func (t *TPUCapabilitiesDetector) getCacheSize(tpu schema.HardwareDevice) int64 {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe":
		return 256 * 1024 // 256KB
	case "Apple Neural Engine":
		return 1024 * 1024 // 1MB
	case "Intel OpenVINO":
		return 512 * 1024 // 512KB
	default:
		return 256 * 1024 // 256KB
	}
}

func (t *TPUCapabilitiesDetector) getMaxFrequency(tpu schema.HardwareDevice) int64 {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe":
		return 600000000 // 600 MHz
	case "Apple Neural Engine":
		if t.detectGeneration(tpu) == "4th Gen" {
			return 1200000000 // 1.2 GHz
		}
		return 1000000000 // 1 GHz
	case "Intel OpenVINO":
		return 1500000000 // 1.5 GHz
	default:
		return 500000000 // 500 MHz
	}
}

func (t *TPUCapabilitiesDetector) getBaseFrequency(tpu schema.HardwareDevice) int64 {
	max := t.getMaxFrequency(tpu)
	return int64(float64(max) * 0.7) // 70% of max
}

func (t *TPUCapabilitiesDetector) getBoostFrequency(tpu schema.HardwareDevice) int64 {
	max := t.getMaxFrequency(tpu)
	return int64(float64(max) * 1.1) // 110% of max
}

func (t *TPUCapabilitiesDetector) getClockSpeed(tpu schema.HardwareDevice) int64 {
	return t.getMaxFrequency(tpu)
}

func (t *TPUCapabilitiesDetector) getInstructionsPerCycle(tpu schema.HardwareDevice) int {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe":
		return 1024 // 1024 MACs per cycle
	case "Apple Neural Engine":
		return 4096 // 4096 operations per cycle
	case "Intel OpenVINO":
		return 2048 // 2048 operations per cycle
	default:
		return 512 // 512 operations per cycle
	}
}

func (t *TPUCapabilitiesDetector) getPipelineDepth(tpu schema.HardwareDevice) int {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe":
		return 16
	case "Apple Neural Engine":
		return 32
	case "Intel OpenVINO":
		return 24
	default:
		return 12
	}
}

func (t *TPUCapabilitiesDetector) getCores(tpu schema.HardwareDevice) int {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe":
		return 1
	case "Apple Neural Engine":
		return 16 // Apple Neural Engine has 16 cores
	case "Intel OpenVINO":
		return 8 // Estimated
	default:
		return 4
	}
}

func (t *TPUCapabilitiesDetector) getExecutionUnits(tpu schema.HardwareDevice) int {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe":
		return 8
	case "Apple Neural Engine":
		return 16
	case "Intel OpenVINO":
		return 12
	default:
		return 4
	}
}

func (t *TPUCapabilitiesDetector) getTDP(tpu schema.HardwareDevice) float64 {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB":
		return 2.5 // 2.5W
	case "Coral PCIe":
		return 2.0 // 2.0W
	case "Apple Neural Engine":
		return 1.0 // 1.0W (estimated)
	case "Intel OpenVINO":
		return 15.0 // 15W (estimated)
	default:
		return 5.0 // 5W default
	}
}

func (t *TPUCapabilitiesDetector) getBasePower(tpu schema.HardwareDevice) float64 {
	tdp := t.getTDP(tpu)
	return tdp * 0.3 // 30% of TDP
}

func (t *TPUCapabilitiesDetector) getMaxPower(tpu schema.HardwareDevice) float64 {
	tdp := t.getTDP(tpu)
	return tdp * 1.2 // 120% of TDP
}

func (t *TPUCapabilitiesDetector) getIdlePower(tpu schema.HardwareDevice) float64 {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe":
		return 0.1 // 0.1W
	case "Apple Neural Engine":
		return 0.05 // 0.05W
	case "Intel OpenVINO":
		return 1.0 // 1.0W
	default:
		return 0.5 // 0.5W
	}
}

func (t *TPUCapabilitiesDetector) getPowerStates(tpu schema.HardwareDevice) []string {
	return []string{
		"Active",
		"Idle",
		"Sleep",
		"Deep Sleep",
		"Off",
	}
}

func (t *TPUCapabilitiesDetector) getVoltageRange(tpu schema.HardwareDevice) schema.VoltageRange {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB":
		return schema.VoltageRange{Min: 3.3, Max: 5.0} // USB voltage
	case "Coral PCIe":
		return schema.VoltageRange{Min: 3.3, Max: 12.0} // PCIe voltage
	case "Apple Neural Engine":
		return schema.VoltageRange{Min: 0.8, Max: 1.2} // On-chip voltage
	case "Intel OpenVINO":
		return schema.VoltageRange{Min: 1.0, Max: 1.5} // CPU voltage range
	default:
		return schema.VoltageRange{Min: 1.0, Max: 3.3}
	}
}

func (t *TPUCapabilitiesDetector) getCurrentRange(tpu schema.HardwareDevice) schema.CurrentRange {
	power := t.getTDP(tpu)
	voltage := t.getVoltageRange(tpu)

	maxCurrent := power / voltage.Min

	return schema.CurrentRange{
		Min: maxCurrent * 0.1, // 10% of max
		Max: maxCurrent,
	}
}

func (t *TPUCapabilitiesDetector) getMaxTemperature(tpu schema.HardwareDevice) float64 {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe":
		return 85.0 // 85°C
	case "Apple Neural Engine":
		return 100.0 // 100°C
	case "Intel OpenVINO":
		return 105.0 // 105°C
	default:
		return 90.0 // 90°C
	}
}

func (t *TPUCapabilitiesDetector) getOptimalTemperature(tpu schema.HardwareDevice) float64 {
	max := t.getMaxTemperature(tpu)
	return max * 0.7 // 70% of max temperature
}

func (t *TPUCapabilitiesDetector) getThrottlingTemperature(tpu schema.HardwareDevice) float64 {
	max := t.getMaxTemperature(tpu)
	return max * 0.85 // 85% of max temperature
}

func (t *TPUCapabilitiesDetector) getCriticalTemperature(tpu schema.HardwareDevice) float64 {
	max := t.getMaxTemperature(tpu)
	return max * 0.95 // 95% of max temperature
}

func (t *TPUCapabilitiesDetector) getCoolingSolution(tpu schema.HardwareDevice) string {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB":
		return "Passive (USB connector)"
	case "Coral PCIe":
		return "Passive (PCIE slot)"
	case "Apple Neural Engine":
		return "Integrated cooling"
	case "Intel OpenVINO":
		return "Active (CPU cooling)"
	default:
		return "Passive"
	}
}

func (t *TPUCapabilitiesDetector) getThermalManagement(tpu schema.HardwareDevice) []string {
	tpuType := t.detectTPUType(tpu)

	switch tpuType {
	case "Coral USB", "Coral PCIe":
		return []string{
			"Thermal Throttling",
			"Automatic Shutdown",
		}
	case "Apple Neural Engine":
		return []string{
			"Dynamic Frequency Scaling",
			"Thermal Throttling",
			"Power Gating",
		}
	case "Intel OpenVINO":
		return []string{
			"Intel SpeedStep",
			"Thermal Monitoring",
			"Power Management",
		}
	default:
		return []string{
			"Basic Thermal Management",
		}
	}
}