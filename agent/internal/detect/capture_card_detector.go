package detect

import (
	"strings"
	"time"

	"github.com/frigate-config-tool/agent/internal/schema"
)

// CaptureCardCapabilitiesDetector provides comprehensive capture card detection
type CaptureCardCapabilitiesDetector struct {
	detector Detector
}

// NewCaptureCardCapabilitiesDetector creates a new capture card capabilities detector
func NewCaptureCardCapabilitiesDetector(detector Detector) *CaptureCardCapabilitiesDetector {
	return &CaptureCardCapabilitiesDetector{
		detector: detector,
	}
}

// DetectCaptureCardCapabilities performs comprehensive capture card capability detection
func (c *CaptureCardCapabilitiesDetector) DetectCaptureCardCapabilities() ([]schema.CaptureCardCapabilities, error) {
	var capabilities []schema.CaptureCardCapabilities

	// Get capture card devices
	cards, err := c.detector.DetectCaptureCards()
	if err != nil {
		return capabilities, err
	}

	for _, card := range cards {
		cardCaps := schema.CaptureCardCapabilities{
			DeviceID:     card.ID,
			DeviceName:   card.Name,
			DevicePath:   card.DevicePath,
			Platform:     card.Platform,
			Architecture: card.Architecture,
		}

		// Detect card type and form factor
		cardCaps.Type = c.detectCardType(card)
		cardCaps.FormFactor = c.detectFormFactor(card)
		cardCaps.Interface = c.detectInterface(card)

		// Detect input capabilities
		cardCaps.Inputs = c.detectInputCapabilities(card)

		// Detect video capabilities
		cardCaps.Video = c.detectVideoCapabilities(card)

		// Detect audio capabilities
		cardCaps.Audio = c.detectAudioCapabilities(card)

		// Detect streaming capabilities
		cardCaps.Streaming = c.detectStreamingCapabilities(card)

		// Detect supported formats
		cardCaps.Formats = c.detectSupportedFormats(card)

		// Detect hardware features
		cardCaps.Features = c.detectHardwareFeatures(card)

		// Detect performance characteristics
		cardCaps.Performance = c.detectPerformanceCharacteristics(card)

		// Detect connectivity options
		cardCaps.Connectivity = c.detectConnectivity(card)

		capabilities = append(capabilities, cardCaps)
	}

	return capabilities, nil
}

// detectCardType identifies the capture card type
func (c *CaptureCardCapabilitiesDetector) detectCardType(card schema.HardwareDevice) string {
	name := strings.ToLower(card.Name)

	// HDMI capture cards
	if strings.Contains(name, "hdmi") {
		return "HDMI Capture Card"
	}

	// SDI capture cards
	if strings.Contains(name, "sdi") {
		return "SDI Capture Card"
	}

	// Component capture cards
	if strings.Contains(name, "component") || strings.Contains(name, "ypbpr") {
		return "Component Capture Card"
	}

	// Composite capture cards
	if strings.Contains(name, "composite") || strings.Contains(name, "rca") {
		return "Composite Capture Card"
	}

	// S-Video capture cards
	if strings.Contains(name, "s-video") || strings.Contains(name, "yuv") {
		return "S-Video Capture Card"
	}

	// DVI capture cards
	if strings.Contains(name, "dvi") {
		return "DVI Capture Card"
	}

	// DisplayPort capture cards
	if strings.Contains(name, "displayport") || strings.Contains(name, "dp") {
		return "DisplayPort Capture Card"
	}

	// VGA capture cards
	if strings.Contains(name, "vga") {
		return "VGA Capture Card"
	}

	// IP-based capture
	if strings.Contains(name, "ip") || strings.Contains(name, "network") {
		return "IP Capture Card"
	}

	// USB capture devices
	if c.hasCapability(card, []string{"usb"}) {
		return "USB Capture Device"
	}

	// Generic capture card
	return "Generic Capture Card"
}

// detectFormFactor identifies the form factor
func (c *CaptureCardCapabilitiesDetector) detectFormFactor(card schema.HardwareDevice) string {
	name := strings.ToLower(card.Name)

	// PCIe cards
	if strings.Contains(name, "pcie") || strings.Contains(name, "pci-e") || strings.Contains(name, "pci") {
		return "PCIe"
	}

	// USB devices
	if c.hasCapability(card, []string{"usb"}) {
		return "USB"
	}

	// Internal cards
	if strings.Contains(name, "internal") {
		return "Internal"
	}

	// External devices
	if strings.Contains(name, "external") || c.hasCapability(card, []string{"usb"}) {
		return "External"
	}

	return "Unknown"
}

// detectInterface identifies the interface type
func (c *CaptureCardCapabilitiesDetector) detectInterface(card schema.HardwareDevice) string {
	name := strings.ToLower(card.Name)

	if strings.Contains(name, "pcie") || strings.Contains(name, "pci-e") {
		return "PCIe x4"
	}

	if strings.Contains(name, "pci") {
		return "PCI"
	}

	if strings.Contains(name, "usb 3.0") || strings.Contains(name, "usb3") {
		return "USB 3.0"
	}

	if strings.Contains(name, "usb") {
		return "USB 2.0"
	}

	if strings.Contains(name, "thunderbolt") || strings.Contains(name, "tb") {
		return "Thunderbolt"
	}

	return "Unknown"
}

// detectInputCapabilities detects input capabilities
func (c *CaptureCardCapabilitiesDetector) detectInputCapabilities(card schema.HardwareDevice) schema.InputCapabilities {
	inputs := schema.InputCapabilities{
		MaxInputs:        c.getMaxInputs(card),
		SupportedSignals: c.getSupportedSignals(card),
		SupportedResolutions: c.getSupportedResolutions(card),
		SupportedFrameRates: c.getSupportedFrameRates(card),
		HDCPSupport:      c.checkHDCPSupport(card),
		AnalogInputs:     c.getAnalogInputs(card),
		DigitalInputs:    c.getDigitalInputs(card),
		AudioInputs:      c.getAudioInputs(card),
	}

	return inputs
}

// detectVideoCapabilities detects video capture capabilities
func (c *CaptureCardCapabilitiesDetector) detectVideoCapabilities(card schema.HardwareDevice) schema.VideoCaptureCapabilities {
	video := schema.VideoCaptureCapabilities{
		MaxResolution:    c.getMaxVideoResolution(card),
		MinResolution:    c.getMinVideoResolution(card),
		MaxFrameRate:     c.getMaxFrameRate(card),
		SupportedCodecs:  c.getSupportedVideoCodecs(card),
		ColorSpaces:      c.getSupportedColorSpaces(card),
		BitDepth:         c.getBitDepth(card),
		ChromaSubsampling: c.getChromaSubsampling(card),
		ColorRange:       c.getColorRange(card),
		VideoStandards:   c.getVideoStandards(card),
		Deinterlacing:    c.checkDeinterlacingSupport(card),
		NoiseReduction:   c.checkNoiseReduction(card),
		ColorCorrection:  c.checkColorCorrection(card),
		GammaCorrection:  c.checkGammaCorrection(card),
		Sharpness:        c.getSharpnessRange(card),
		Brightness:       c.getBrightnessRange(card),
		Contrast:         c.getContrastRange(card),
		Saturation:       c.getSaturationRange(card),
		Hue:              c.getHueRange(card),
	}

	return video
}

// detectAudioCapabilities detects audio capture capabilities
func (c *CaptureCardCapabilitiesDetector) detectAudioCapabilities(card schema.HardwareDevice) schema.AudioCaptureCapabilities {
	audio := schema.AudioCaptureCapabilities{
		HasAudioInput:    c.hasAudioInput(card),
		SupportedCodecs:  c.getSupportedAudioCodecs(card),
		SampleRateRange:  c.getSampleRateRange(card),
		BitDepthRange:    c.getBitDepthRange(card),
		ChannelCount:     c.getChannelCount(card),
		AudioInputs:      c.getAudioInputTypes(card),
		AudioStandards:   c.getAudioStandards(card),
		NoiseReduction:   c.checkAudioNoiseReduction(card),
		AGC:              c.checkAGC(card),
		AudioDelay:       c.getAudioDelayRange(card),
		VolumeControl:    c.checkVolumeControl(card),
		MuteControl:      c.checkMuteControl(card),
	}

	return audio
}

// detectStreamingCapabilities detects streaming capabilities
func (c *CaptureCardCapabilitiesDetector) detectStreamingCapabilities(card schema.HardwareDevice) schema.StreamingCapabilities {
	streaming := schema.StreamingCapabilities{
		RTSPSupport:      c.hasRTSPSupport(card),
		HTTPSupport:      c.hasHTTPSupport(card),
		WebRTCSupport:    c.hasWebRTCSupport(card),
		HLSsupport:       c.hasHLSSupport(card),
		MPEGDASHSupport:  c.hasMPEGDASHSupport(card),
		MaxStreams:       c.getMaxSimultaneousStreams(card),
		Latency:          c.getLatency(card),
		Buffering:        c.getBufferingSupport(card),
		RealTime:         c.checkRealTimeSupport(card),
		Compression:      c.getCompressionSupport(card),
		Multiplexing:     c.checkMultiplexingSupport(card),
		Timestamping:     c.checkTimestampingSupport(card),
	}

	return streaming
}

// detectSupportedFormats detects supported media formats
func (c *CaptureCardCapabilitiesDetector) detectSupportedFormats(card schema.HardwareDevice) []schema.MediaFormat {
	var formats []schema.MediaFormat

	// Get supported video formats
	videoFormats := c.getVideoFormats(card)
	formats = append(formats, videoFormats...)

	// Get supported audio formats
	audioFormats := c.getAudioFormats(card)
	formats = append(formats, audioFormats...)

	return formats
}

// detectHardwareFeatures detects hardware features
func (c *CaptureCardCapabilitiesDetector) detectHardwareFeatures(card schema.HardwareDevice) []string {
	var features []string

	// Check for various hardware features
	if c.hasHardwareEncoder(card) {
		features = append(features, "hardware_encoder")
	}

	if c.hasHardwareDecoder(card) {
		features = append(features, "hardware_decoder")
	}

	if c.hasVideoProcessor(card) {
		features = append(features, "video_processor")
	}

	if c.hasAudioProcessor(card) {
		features = append(features, "audio_processor")
	}

	if c.hasDMAController(card) {
		features = append(features, "dma_controller")
	}

	if c.hasFPGA(card) {
		features = append(features, "fpga_acceleration")
	}

	if c.hasASIC(card) {
		features = append(features, "asic_acceleration")
	}

	if c.hasGPUAcceleration(card) {
		features = append(features, "gpu_acceleration")
	}

	if c.hasLowLatencyMode(card) {
		features = append(features, "low_latency_mode")
	}

	if c.hasBroadcastMode(card) {
		features = append(features, "broadcast_mode")
	}

	return features
}

// detectPerformanceCharacteristics detects performance characteristics
func (c *CaptureCardCapabilitiesDetector) detectPerformanceCharacteristics(card schema.HardwareDevice) schema.PerformanceCharacteristics {
	perf := schema.PerformanceCharacteristics{
		BaseClock:        c.getBaseClock(card),
		BoostClock:       c.getBoostClock(card),
		MemoryClock:      c.getMemoryClock(card),
		TDP:              c.getTDP(card),
		PowerLimits:      c.getPowerLimits(card),
		ThermalThrottling: c.hasThermalThrottling(card),
	}

	return perf
}

// detectConnectivity detects connectivity options
func (c *CaptureCardCapabilitiesDetector) detectConnectivity(card schema.HardwareDevice) schema.ConnectivityOptions {
	connectivity := schema.ConnectivityOptions{
		PhysicalPorts:    c.getPhysicalPorts(card),
		TriggerInputs:    c.getTriggerInputs(card),
		GPIOPins:         c.getGPIOPins(card),
		SyncConnectors:   c.getSyncConnectors(card),
		TimecodeInputs:   c.getTimecodeInputs(card),
		TimecodeOutputs:  c.getTimecodeOutputs(card),
		GenlockInput:     c.hasGenlockInput(card),
		GenlockOutput:    c.hasGenlockOutput(card),
		WordClock:        c.hasWordClock(card),
		NetworkInterfaces: c.getNetworkInterfaces(card),
	}

	return connectivity
}

// Helper methods for capability detection

func (c *CaptureCardCapabilitiesDetector) hasCapability(card schema.HardwareDevice, capabilities []string) bool {
	for _, cap := range capabilities {
		for _, cardCap := range card.Capabilities {
			if strings.Contains(strings.ToLower(cardCap), strings.ToLower(cap)) {
				return true
			}
		}
	}
	return false
}

func (c *CaptureCardCapabilitiesDetector) getMaxInputs(card schema.HardwareDevice) int {
	name := strings.ToLower(card.Name)

	if strings.Contains(name, "quad") {
		return 4
	}

	if strings.Contains(name, "dual") || strings.Contains(name, "stereo") {
		return 2
	}

	if strings.Contains(name, "multi") {
		return 8
	}

	return 1 // Default to single input
}

func (c *CaptureCardCapabilitiesDetector) getSupportedSignals(card schema.HardwareDevice) []string {
	cardType := c.detectCardType(card)
	var signals []string

	switch cardType {
	case "HDMI Capture Card":
		signals = []string{"HDMI", "DVI", "VGA"}
	case "SDI Capture Card":
		signals = []string{"SDI", "HD-SDI", "3G-SDI", "6G-SDI", "12G-SDI"}
	case "Component Capture Card":
		signals = []string{"Component (YPbPr)", "RGB"}
	case "Composite Capture Card":
		signals = []string{"Composite (NTSC)", "Composite (PAL)", "Composite (SECAM)"}
	case "S-Video Capture Card":
		signals = []string{"S-Video", "YUV"}
	case "DVI Capture Card":
		signals = []string{"DVI-D", "DVI-A", "DVI-I"}
	case "DisplayPort Capture Card":
		signals = []string{"DisplayPort", "Mini DisplayPort"}
	case "VGA Capture Card":
		signals = []string{"VGA", "RGB"}
	case "USB Capture Device":
		signals = []string{"HDMI", "Component", "Composite"}
	default:
		signals = []string{"Unknown"}
	}

	return signals
}

func (c *CaptureCardCapabilitiesDetector) getSupportedResolutions(card schema.HardwareDevice) []string {
	cardType := c.detectCardType(card)
	var resolutions []string

	switch cardType {
	case "HDMI Capture Card", "DisplayPort Capture Card", "SDI Capture Card":
		resolutions = []string{
			"3840x2160", // 4K
			"2560x1440", // 1440p
			"1920x1080", // 1080p
			"1280x720",  // 720p
			"1024x768",  // XGA
			"800x600",   // SVGA
			"640x480",   // VGA
		}
	case "Component Capture Card", "DVI Capture Card", "VGA Capture Card":
		resolutions = []string{
			"1920x1080", // 1080p
			"1280x720",  // 720p
			"1024x768",  // XGA
			"800x600",   // SVGA
			"640x480",   // VGA
		}
	case "Composite Capture Card", "S-Video Capture Card":
		resolutions = []string{
			"720x576",   // PAL
			"720x480",   // NTSC
		}
	case "USB Capture Device":
		resolutions = []string{
			"1920x1080", // 1080p
			"1280x720",  // 720p
			"640x480",   // VGA
		}
	default:
		resolutions = []string{
			"1920x1080", // 1080p
			"1280x720",  // 720p
		}
	}

	return resolutions
}

func (c *CaptureCardCapabilitiesDetector) getSupportedFrameRates(card schema.HardwareDevice) []int {
	cardType := c.detectCardType(card)
	var frameRates []int

	switch cardType {
	case "HDMI Capture Card", "DisplayPort Capture Card", "SDI Capture Card":
		frameRates = []int{24, 25, 30, 50, 60}
	case "Component Capture Card", "DVI Capture Card":
		frameRates = []int{24, 25, 30, 50, 60}
	case "Composite Capture Card", "S-Video Capture Card":
		frameRates = []int{25, 30} // PAL and NTSC
	case "USB Capture Device":
		frameRates = []int{25, 30, 50, 60}
	default:
		frameRates = []int{25, 30, 60}
	}

	return frameRates
}

func (c *CaptureCardCapabilitiesDetector) checkHDCPSupport(card schema.HardwareDevice) bool {
	cardType := c.detectCardType(card)
	return cardType == "HDMI Capture Card" || cardType == "DisplayPort Capture Card"
}

func (c *CaptureCardCapabilitiesDetector) getAnalogInputs(card schema.HardwareDevice) int {
	cardType := c.detectCardType(card)

	switch cardType {
	case "Component Capture Card":
		return 3 // Y, Pb, Pr
	case "Composite Capture Card":
		return 1 // Composite
	case "S-Video Capture Card":
		return 2 // Y, C
	case "VGA Capture Card":
		return 3 // R, G, B
	default:
		return 0
	}
}

func (c *CaptureCardCapabilitiesDetector) getDigitalInputs(card schema.HardwareDevice) int {
	cardType := c.detectCardType(card)

	switch cardType {
	case "HDMI Capture Card", "DisplayPort Capture Card", "DVI Capture Card":
		return 1
	case "SDI Capture Card":
		return c.getMaxInputs(card)
	default:
		return 0
	}
}

func (c *CaptureCardCapabilitiesDetector) getAudioInputs(card schema.HardwareDevice) int {
	if c.hasAudioInput(card) {
		return 2 // Stereo
	}
	return 0
}

func (c *CaptureCardCapabilitiesDetector) getMaxVideoResolution(card schema.HardwareDevice) string {
	resolutions := c.getSupportedResolutions(card)
	if len(resolutions) > 0 {
		return resolutions[0]
	}
	return "1920x1080"
}

func (c *CaptureCardCapabilitiesDetector) getMinVideoResolution(card schema.HardwareDevice) string {
	resolutions := c.getSupportedResolutions(card)
	if len(resolutions) > 0 {
		return resolutions[len(resolutions)-1]
	}
	return "640x480"
}

func (c *CaptureCardCapabilitiesDetector) getMaxFrameRate(card schema.HardwareDevice) int {
	frameRates := c.getSupportedFrameRates(card)
	max := 0
	for _, rate := range frameRates {
		if rate > max {
			max = rate
		}
	}
	if max == 0 {
		max = 60
	}
	return max
}

func (c *CaptureCardCapabilitiesDetector) getSupportedVideoCodecs(card schema.HardwareDevice) []string {
	var codecs []string

	// Basic codecs supported by most capture cards
	codecs = append(codecs, "raw", "yuv", "rgb")

	// Check for hardware codec support
	if c.hasHardwareEncoder(card) {
		codecs = append(codecs, "h264", "h265")
	}

	// Check for advanced codec support
	cardType := c.detectCardType(card)
	if cardType == "HDMI Capture Card" || cardType == "SDI Capture Card" {
		codecs = append(codecs, "prores", "dnxhd")
	}

	return codecs
}

func (c *CaptureCardCapabilitiesDetector) getSupportedColorSpaces(card schema.HardwareDevice) []string {
	var colorSpaces []string

	// Standard color spaces
	colorSpaces = append(colorSpaces, "RGB", "YUV", "BT.601", "BT.709")

	// Check for wide color gamut support
	cardType := c.detectCardType(card)
	if cardType == "HDMI Capture Card" || cardType == "DisplayPort Capture Card" {
		colorSpaces = append(colorSpaces, "BT.2020", "DCI-P3")
	}

	return colorSpaces
}

func (c *CaptureCardCapabilitiesDetector) getBitDepth(card schema.HardwareDevice) []int {
	var bitDepths []int

	// Standard bit depths
	bitDepths = append(bitDepths, 8, 16, 24)

	// Check for high bit depth support
	cardType := c.detectCardType(card)
	if cardType == "HDMI Capture Card" || cardType == "SDI Capture Card" || cardType == "DisplayPort Capture Card" {
		bitDepths = append(bitDepths, 10, 12)
	}

	return bitDepths
}

func (c *CaptureCardCapabilitiesDetector) getChromaSubsampling(card schema.HardwareDevice) []string {
	return []string{"4:4:4", "4:2:2", "4:2:0", "4:1:1", "4:1:0"}
}

func (c *CaptureCardCapabilitiesDetector) getColorRange(card schema.HardwareDevice) []string {
	return []string{"Limited", "Full"}
}

func (c *CaptureCardCapabilitiesDetector) getVideoStandards(card schema.HardwareDevice) []string {
	var standards []string

	cardType := c.detectCardType(card)
	if cardType == "Composite Capture Card" || cardType == "S-Video Capture Card" {
		standards = append(standards, "NTSC", "PAL", "SECAM")
	}

	// Digital standards
	standards = append(standards, "ATSC", "DVB", "ISDB")

	return standards
}

func (c *CaptureCardCapabilitiesDetector) checkDeinterlacingSupport(card schema.HardwareDevice) bool {
	return true // Most capture cards support deinterlacing
}

func (c *CaptureCardCapabilitiesDetector) checkNoiseReduction(card schema.HardwareDevice) bool {
	return c.hasVideoProcessor(card)
}

func (c *CaptureCardCapabilitiesDetector) checkColorCorrection(card schema.HardwareDevice) bool {
	return c.hasVideoProcessor(card)
}

func (c *CaptureCardCapabilitiesDetector) checkGammaCorrection(card schema.HardwareDevice) bool {
	return c.hasVideoProcessor(card)
}

func (c *CaptureCardCapabilitiesDetector) getSharpnessRange(card schema.HardwareDevice) schema.Range {
	return schema.Range{Min: -10, Max: 10}
}

func (c *CaptureCardCapabilitiesDetector) getBrightnessRange(card schema.HardwareDevice) schema.Range {
	return schema.Range{Min: -100, Max: 100}
}

func (c *CaptureCardCapabilitiesDetector) getContrastRange(card schema.HardwareDevice) schema.Range {
	return schema.Range{Min: -100, Max: 100}
}

func (c *CaptureCardCapabilitiesDetector) getSaturationRange(card schema.HardwareDevice) schema.Range {
	return schema.Range{Min: -100, Max: 100}
}

func (c *CaptureCardCapabilitiesDetector) getHueRange(card schema.HardwareDevice) schema.Range {
	return schema.Range{Min: -180, Max: 180}
}

func (c *CaptureCardCapabilitiesDetector) hasAudioInput(card schema.HardwareDevice) bool {
	cardType := c.detectCardType(card)
	return cardType == "HDMI Capture Card" || cardType == "SDI Capture Card" || c.hasCapability(card, []string{"audio"})
}

func (c *CaptureCardCapabilitiesDetector) getSupportedAudioCodecs(card schema.HardwareDevice) []string {
	var codecs []string

	// Basic audio codecs
	codecs = append(codecs, "PCM", "LPCM")

	// Compressed audio codecs
	if c.hasAudioProcessor(card) {
		codecs = append(codecs, "AAC", "AC3", "DTS")
	}

	return codecs
}

func (c *CaptureCardCapabilitiesDetector) getSampleRateRange(card schema.HardwareDevice) schema.SampleRateRange {
	return schema.SampleRateRange{Min: 32000, Max: 192000}
}

func (c *CaptureCardCapabilitiesDetector) getBitDepthRange(card schema.HardwareDevice) schema.BitDepthRange {
	return schema.BitDepthRange{Min: 16, Max: 24}
}

func (c *CaptureCardCapabilitiesDetector) getChannelCount(card schema.HardwareDevice) int {
	if c.hasAudioInput(card) {
		return 8 // Up to 8 channels
	}
	return 0
}

func (c *CaptureCardCapabilitiesDetector) getAudioInputTypes(card schema.HardwareDevice) []string {
	var inputs []string

	cardType := c.detectCardType(card)
	switch cardType {
	case "HDMI Capture Card":
		inputs = []string{"HDMI Embedded Audio"}
	case "SDI Capture Card":
		inputs = []string{"SDI Embedded Audio", "AES/EBU", "Analog"}
	case "USB Capture Device":
		inputs = []string{"USB Audio", "Line In"}
	default:
		inputs = []string{"Analog"}
	}

	return inputs
}

func (c *CaptureCardCapabilitiesDetector) getAudioStandards(card schema.HardwareDevice) []string {
	return []string{"AES/EBU", "S/PDIF", "Analog"}
}

func (c *CaptureCardCapabilitiesDetector) checkAudioNoiseReduction(card schema.HardwareDevice) bool {
	return c.hasAudioProcessor(card)
}

func (c *CaptureCardCapabilitiesDetector) checkAGC(card schema.HardwareDevice) bool {
	return c.hasAudioProcessor(card)
}

func (c *CaptureCardCapabilitiesDetector) getAudioDelayRange(card schema.HardwareDevice) schema.Range {
	return schema.Range{Min: -1000, Max: 1000} // milliseconds
}

func (c *CaptureCardCapabilitiesDetector) checkVolumeControl(card schema.HardwareDevice) bool {
	return c.hasAudioProcessor(card)
}

func (c *CaptureCardCapabilitiesDetector) checkMuteControl(card schema.HardwareDevice) bool {
	return c.hasAudioProcessor(card)
}

func (c *CaptureCardCapabilitiesDetector) checkDirectShowSupport(card schema.HardwareDevice) bool {
	return card.Platform == "windows"
}

func (c *CaptureCardCapabilitiesDetector) checkV4L2Support(card schema.HardwareDevice) bool {
	return card.Platform == "linux"
}

func (c *CaptureCardCapabilitiesDetector) checkAVFoundationSupport(card schema.HardwareDevice) bool {
	return card.Platform == "darwin"
}

func (c *CaptureCardCapabilitiesDetector) checkFFmpegSupport(card schema.HardwareDevice) bool {
	return true // FFmpeg works across platforms
}

func (c *CaptureCardCapabilitiesDetector) getMaxSimultaneousStreams(card schema.HardwareDevice) int {
	return c.getMaxInputs(card)
}

func (c *CaptureCardCapabilitiesDetector) getLatency(card schema.HardwareDevice) time.Duration {
	cardType := c.detectCardType(card)

	switch cardType {
	case "USB Capture Device":
		return 100 * time.Millisecond
	case "PCIe Capture Card":
		return 5 * time.Millisecond
	case "HDMI Capture Card", "SDI Capture Card":
		return 2 * time.Millisecond
	default:
		return 50 * time.Millisecond
	}
}

func (c *CaptureCardCapabilitiesDetector) getBufferingSupport(card schema.HardwareDevice) bool {
	return true
}

func (c *CaptureCardCapabilitiesDetector) checkRealTimeSupport(card schema.HardwareDevice) bool {
	cardType := c.detectCardType(card)
	return cardType == "PCIe Capture Card" || cardType == "HDMI Capture Card" || cardType == "SDI Capture Card"
}

func (c *CaptureCardCapabilitiesDetector) getCompressionSupport(card schema.HardwareDevice) []string {
	var compression []string

	if c.hasHardwareEncoder(card) {
		compression = append(compression, "H.264", "H.265")
	}

	compression = append(compression, "Uncompressed", "Lossless")

	return compression
}

func (c *CaptureCardCapabilitiesDetector) checkMultiplexingSupport(card schema.HardwareDevice) bool {
	return c.getMaxInputs(card) > 1
}

func (c *CaptureCardCapabilitiesDetector) checkTimestampingSupport(card schema.HardwareDevice) bool {
	return true
}

func (c *CaptureCardCapabilitiesDetector) getVideoFormats(card schema.HardwareDevice) []schema.MediaFormat {
	var formats []schema.MediaFormat

	resolutions := c.getSupportedResolutions(card)
	codecs := c.getSupportedVideoCodecs(card)

	for _, res := range resolutions {
		for _, codec := range codecs {
			format := schema.MediaFormat{
				Type:       "video",
				Resolution: res,
				Codec:      codec,
				FrameRate:  c.getMaxFrameRate(card),
			}
			formats = append(formats, format)
		}
	}

	return formats
}

func (c *CaptureCardCapabilitiesDetector) getAudioFormats(card schema.HardwareDevice) []schema.MediaFormat {
	if !c.hasAudioInput(card) {
		return []schema.MediaFormat{}
	}

	codecs := c.getSupportedAudioCodecs(card)
	var formats []schema.MediaFormat

	for _, codec := range codecs {
		format := schema.MediaFormat{
			Type:       "audio",
			Codec:      codec,
			SampleRate: 48000,
			BitDepth:   16,
			Channels:   2,
		}
		formats = append(formats, format)
	}

	return formats
}

func (c *CaptureCardCapabilitiesDetector) hasHardwareEncoder(card schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(card.Name), "encoder") ||
		   strings.Contains(strings.ToLower(card.Name), "hardware")
}

func (c *CaptureCardCapabilitiesDetector) hasHardwareDecoder(card schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(card.Name), "decoder") ||
		   strings.Contains(strings.ToLower(card.Name), "hardware")
}

func (c *CaptureCardCapabilitiesDetector) hasVideoProcessor(card schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(card.Name), "processor") ||
		   strings.Contains(strings.ToLower(card.Name), "dsp")
}

func (c *CaptureCardCapabilitiesDetector) hasAudioProcessor(card schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(card.Name), "audio") ||
		   c.hasVideoProcessor(card)
}

func (c *CaptureCardCapabilitiesDetector) hasDMAController(card schema.HardwareDevice) bool {
	return c.detectFormFactor(card) == "PCIe"
}

func (c *CaptureCardCapabilitiesDetector) hasFPGA(card schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(card.Name), "fpga")
}

func (c *CaptureCardCapabilitiesDetector) hasASIC(card schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(card.Name), "asic")
}

func (c *CaptureCardCapabilitiesDetector) hasGPUAcceleration(card schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(card.Name), "gpu") ||
		   strings.Contains(strings.ToLower(card.Name), "cuda")
}

func (c *CaptureCardCapabilitiesDetector) hasLowLatencyMode(card schema.HardwareDevice) bool {
	return c.detectFormFactor(card) == "PCIe"
}

func (c *CaptureCardCapabilitiesDetector) hasBroadcastMode(card schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(card.Name), "broadcast") ||
		   c.detectCardType(card) == "SDI Capture Card"
}

func (c *CaptureCardCapabilitiesDetector) getMaxThroughput(card schema.HardwareDevice) int64 {
	maxRes := c.getMaxVideoResolution(card)
	maxFPS := c.getMaxFrameRate(card)

	// Calculate approximate throughput
	if maxRes == "3840x2160" && maxFPS >= 60 {
		return 12000000000 // 12 Gbps
	}
	if maxRes == "1920x1080" && maxFPS >= 60 {
		return 3000000000 // 3 Gbps
	}
	if maxRes == "1280x720" && maxFPS >= 60 {
		return 1500000000 // 1.5 Gbps
	}

	return 750000000 // 750 Mbps default
}

func (c *CaptureCardCapabilitiesDetector) getMaxBitrate(card schema.HardwareDevice) int64 {
	throughput := c.getMaxThroughput(card)
	return int64(float64(throughput) * 0.8) // 80% of throughput
}

func (c *CaptureCardCapabilitiesDetector) getCPUUsage(card schema.HardwareDevice) float64 {
	if c.hasHardwareEncoder(card) && c.hasHardwareDecoder(card) {
		return 5.0 // 5% CPU usage
	}
	return 20.0 // 20% CPU usage
}

func (c *CaptureCardCapabilitiesDetector) getMemoryUsage(card schema.HardwareDevice) int64 {
	maxRes := c.getMaxVideoResolution(card)

	switch maxRes {
	case "3840x2160":
		return 512 * 1024 * 1024 // 512MB
	case "1920x1080":
		return 256 * 1024 * 1024 // 256MB
	case "1280x720":
		return 128 * 1024 * 1024 // 128MB
	default:
		return 64 * 1024 * 1024 // 64MB
	}
}

func (c *CaptureCardCapabilitiesDetector) getPowerConsumption(card schema.HardwareDevice) float64 {
	formFactor := c.detectFormFactor(card)

	switch formFactor {
	case "PCIe":
		return 15.0 // 15W
	case "USB":
		return 2.5 // 2.5W
	case "External":
		return 10.0 // 10W
	default:
		return 5.0 // 5W
	}
}

func (c *CaptureCardCapabilitiesDetector) getThermalDesignPower(card schema.HardwareDevice) float64 {
	return c.getPowerConsumption(card)
}

func (c *CaptureCardCapabilitiesDetector) getCoolingSolution(card schema.HardwareDevice) string {
	formFactor := c.detectFormFactor(card)

	switch formFactor {
	case "PCIe":
		return "Passive Heatsink"
	case "USB":
		return "Passive"
	case "External":
		return "Active Fan"
	default:
		return "Passive"
	}
}

func (c *CaptureCardCapabilitiesDetector) getReliability(card schema.HardwareDevice) float64 {
	return 99.9 // 99.9% reliability
}

func (c *CaptureCardCapabilitiesDetector) getMTBF(card schema.HardwareDevice) time.Duration {
	return 87600 * time.Hour // 10 years
}

func (c *CaptureCardCapabilitiesDetector) getPhysicalPorts(card schema.HardwareDevice) []string {
	cardType := c.detectCardType(card)

	switch cardType {
	case "HDMI Capture Card":
		return []string{"HDMI Input", "USB Output"}
	case "SDI Capture Card":
		return []string{"BNC Input", "USB Output"}
	case "Component Capture Card":
		return []string{"RCA Input", "USB Output"}
	case "USB Capture Device":
		return []string{"HDMI Input", "USB Connection"}
	default:
		return []string{"Input", "Output"}
	}
}

func (c *CaptureCardCapabilitiesDetector) getTriggerInputs(card schema.HardwareDevice) int {
	return c.getMaxInputs(card)
}

func (c *CaptureCardCapabilitiesDetector) getGPIOPins(card schema.HardwareDevice) int {
	if strings.Contains(strings.ToLower(card.Name), "gpio") {
		return 8
	}
	return 0
}

func (c *CaptureCardCapabilitiesDetector) getSyncConnectors(card schema.HardwareDevice) []string {
	if c.detectCardType(card) == "SDI Capture Card" {
		return []string{"BNC Ref Input", "BNC Loop Through"}
	}
	return []string{}
}

func (c *CaptureCardCapabilitiesDetector) getTimecodeInputs(card schema.HardwareDevice) []string {
	cardType := c.detectCardType(card)
	if cardType == "SDI Capture Card" || cardType == "HDMI Capture Card" {
		return []string{"Embedded Timecode", "LTC Input"}
	}
	return []string{}
}

func (c *CaptureCardCapabilitiesDetector) getTimecodeOutputs(card schema.HardwareDevice) []string {
	cardType := c.detectCardType(card)
	if cardType == "SDI Capture Card" || cardType == "HDMI Capture Card" {
		return []string{"Embedded Timecode", "LTC Output"}
	}
	return []string{}
}

func (c *CaptureCardCapabilitiesDetector) hasGenlockInput(card schema.HardwareDevice) bool {
	return c.detectCardType(card) == "SDI Capture Card"
}

func (c *CaptureCardCapabilitiesDetector) hasGenlockOutput(card schema.HardwareDevice) bool {
	return c.detectCardType(card) == "SDI Capture Card"
}

func (c *CaptureCardCapabilitiesDetector) hasWordClock(card schema.HardwareDevice) bool {
	return c.detectCardType(card) == "SDI Capture Card"
}

func (c *CaptureCardCapabilitiesDetector) getNetworkInterfaces(card schema.HardwareDevice) []string {
	if strings.Contains(strings.ToLower(card.Name), "ip") {
		return []string{"Ethernet", "TCP/IP"}
	}
	return []string{}
}

// Streaming capability detection methods
func (c *CaptureCardCapabilitiesDetector) hasRTSPSupport(card schema.HardwareDevice) bool {
	return true // Most capture cards support RTSP streaming
}

func (c *CaptureCardCapabilitiesDetector) hasHTTPSupport(card schema.HardwareDevice) bool {
	return true // Most capture cards support HTTP streaming
}

func (c *CaptureCardCapabilitiesDetector) hasWebRTCSupport(card schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(card.Name), "webrtc")
}

func (c *CaptureCardCapabilitiesDetector) hasHLSSupport(card schema.HardwareDevice) bool {
	return true // Most capture cards support HLS
}

func (c *CaptureCardCapabilitiesDetector) hasMPEGDASHSupport(card schema.HardwareDevice) bool {
	return true // Most capture cards support MPEG-DASH
}

// Performance characteristics methods
func (c *CaptureCardCapabilitiesDetector) getBaseClock(card schema.HardwareDevice) int64 {
	return 1000000000 // 1 GHz base clock
}

func (c *CaptureCardCapabilitiesDetector) getBoostClock(card schema.HardwareDevice) int64 {
	return 1500000000 // 1.5 GHz boost clock
}

func (c *CaptureCardCapabilitiesDetector) getMemoryClock(card schema.HardwareDevice) int64 {
	return 2000000000 // 2 GHz memory clock
}

func (c *CaptureCardCapabilitiesDetector) getTDP(card schema.HardwareDevice) int64 {
	formFactor := c.detectFormFactor(card)
	switch formFactor {
	case "PCIe":
		return 15000 // 15W
	case "USB":
		return 2500 // 2.5W
	case "External":
		return 10000 // 10W
	default:
		return 5000 // 5W
	}
}

func (c *CaptureCardCapabilitiesDetector) getPowerLimits(card schema.HardwareDevice) []int64 {
	return []int64{5000, 10000, 15000} // 5W, 10W, 15W power limits
}

func (c *CaptureCardCapabilitiesDetector) hasThermalThrottling(card schema.HardwareDevice) bool {
	return c.detectFormFactor(card) == "PCIe"
}