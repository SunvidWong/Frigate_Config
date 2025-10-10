package detect

import (
	"encoding/json"
	"fmt"
	"os/exec"
	"regexp"
	"strconv"
	"strings"
	"time"

	"github.com/frigate-config-tool/agent/internal/schema"
)

// CameraCapabilitiesDetector provides cross-platform camera capability detection
type CameraCapabilitiesDetector struct {
	detector Detector
}

// NewCameraCapabilitiesDetector creates a new camera capabilities detector
func NewCameraCapabilitiesDetector(detector Detector) *CameraCapabilitiesDetector {
	return &CameraCapabilitiesDetector{
		detector: detector,
	}
}

// DetectCameraCapabilities performs comprehensive camera capability detection
func (c *CameraCapabilitiesDetector) DetectCameraCapabilities() ([]schema.CameraCapabilities, error) {
	var capabilities []schema.CameraCapabilities

	// Get camera devices
	cameras, err := c.detector.DetectCameras()
	if err != nil {
		return capabilities, err
	}

	for _, camera := range cameras {
		cameraCaps := schema.CameraCapabilities{
			DeviceID:     camera.ID,
			DeviceName:   camera.Name,
			DevicePath:   camera.DevicePath,
			Platform:     camera.Platform,
			Architecture: camera.Architecture,
		}

		// Detect video capabilities
		cameraCaps.Video = c.detectVideoCapabilities(camera)

		// Detect audio capabilities
		cameraCaps.Audio = c.detectAudioCapabilities(camera)

		// Detect physical characteristics
		cameraCaps.Physical = c.detectPhysicalCharacteristics(camera)

		// Detect connection type
		cameraCaps.Connection = c.detectConnectionType(camera)

		// Detect supported formats
		cameraCaps.Formats = c.detectSupportedFormats(camera)

		// Detect streaming capabilities
		cameraCaps.Streaming = c.detectStreamingCapabilities(camera)

		// Detect special features
		cameraCaps.Features = c.detectSpecialFeatures(camera)

		capabilities = append(capabilities, cameraCaps)
	}

	return capabilities, nil
}

// detectVideoCapabilities detects video recording capabilities
func (c *CameraCapabilitiesDetector) detectVideoCapabilities(camera schema.HardwareDevice) schema.VideoCapabilities {
	video := schema.VideoCapabilities{
		MaxResolution:    c.getMaxVideoResolution(camera),
		MinResolution:    c.getMinVideoResolution(camera),
		MaxFrameRate:     c.getMaxFrameRate(camera),
		SupportedCodecs:  c.getSupportedVideoCodecs(camera),
		ColorSpaces:      c.getSupportedColorSpaces(camera),
		BitrateRange:     c.getBitrateRange(camera),
		HDRSupport:       c.checkHDRSupport(camera),
		LowLightSupport:  c.checkLowLightSupport(camera),
		WDRSupport:       c.checkWDRSupport(camera),
		EISTSupport:      c.checkEISTSupport(camera),
		DNRSupport3D:     c.check3DNRSupport(camera),
		DigitalZoom:      c.getDigitalZoomRange(camera),
		OpticalZoom:      c.getOpticalZoomRange(camera),
	}

	return video
}

// detectAudioCapabilities detects audio recording capabilities
func (c *CameraCapabilitiesDetector) detectAudioCapabilities(camera schema.HardwareDevice) schema.AudioCapabilities {
	audio := schema.AudioCapabilities{
		HasMicrophone:    c.hasBuiltInMicrophone(camera),
		SupportedCodecs:  c.getSupportedAudioCodecs(camera),
		SampleRateRange:  c.getSampleRateRange(camera),
		BitDepthRange:    c.getBitDepthRange(camera),
		ChannelCount:     c.getChannelCount(camera),
		NoiseReduction:   c.checkNoiseReduction(camera),
	}

	return audio
}

// detectPhysicalCharacteristics detects physical characteristics
func (c *CameraCapabilitiesDetector) detectPhysicalCharacteristics(camera schema.HardwareDevice) schema.PhysicalCharacteristics {
	physical := schema.PhysicalCharacteristics{
		SensorType:      c.getSensorType(camera),
		SensorSize:      c.getSensorSize(camera),
		LensMount:       c.getLensMount(camera),
		FocalLength:     c.getFocalLengthRange(camera),
		Aperture:        c.getApertureRange(camera),
		FieldOfView:     c.getFieldOfView(camera),
		MechanicalShutter: c.hasMechanicalShutter(camera),
		IRFilter:        c.hasIRFilter(camera),
		IRCutFilter:     c.hasIRCutFilter(camera),
		WeatherResistance: c.checkWeatherResistance(camera),
	}

	return physical
}

// detectConnectionType detects connection type and characteristics
func (c *CameraCapabilitiesDetector) detectConnectionType(camera schema.HardwareDevice) schema.ConnectionType {
	connection := schema.ConnectionType{
		Type:         c.getConnectionType(camera),
		Protocol:     c.getProtocol(camera),
		Bandwidth:    c.getBandwidth(camera),
		PowerSource:  c.getPowerSource(camera),
		PowerConsumption: c.getPowerConsumption(camera),
		MaxLength:    c.getMaxLength(camera),
		HotPluggable: c.isHotPluggable(camera),
	}

	return connection
}

// detectSupportedFormats detects supported media formats
func (c *CameraCapabilitiesDetector) detectSupportedFormats(camera schema.HardwareDevice) []schema.MediaFormat {
	var formats []schema.MediaFormat

	// Get supported resolutions
	resolutions := c.getSupportedResolutions(camera)
	for _, res := range resolutions {
		frameRates := c.getFrameRatesForResolution(camera, res)
		codecs := c.getSupportedVideoCodecs(camera)
		containers := c.getSupportedContainers(camera)

		for _, frameRate := range frameRates {
			for _, codec := range codecs {
				for _, container := range containers {
					formats = append(formats, schema.MediaFormat{
						Type:       "video",
						Container:  container,
						Resolution: res,
						FrameRate:  frameRate,
						Codec:      codec,
						Bitrate:    c.getBitrateForResolution(camera, res).Max,
					})
				}
			}
		}
	}

	// Get audio formats
	audioFormats := c.getSupportedAudioFormats(camera)
	formats = append(formats, audioFormats...)

	return formats
}

// detectStreamingCapabilities detects streaming capabilities
func (c *CameraCapabilitiesDetector) detectStreamingCapabilities(camera schema.HardwareDevice) schema.StreamingCapabilities {
	streaming := schema.StreamingCapabilities{
		RTSPSupport:      c.checkRTSPSupport(camera),
		HTTPSupport:      c.checkHTTPSupport(camera),
		WebRTCSupport:    c.checkWebRTCSupport(camera),
		HLSsupport:       c.checkHLSSupport(camera),
		MPEGDASHSupport:  c.checkMPEGDASHSupport(camera),
		MaxStreams:       c.getMaxSimultaneousStreams(camera),
		Latency:          c.getLatency(camera),
		Buffering:        c.getBufferingSupport(camera),
		Transport:        c.getSupportedTransports(camera),
		Authentication:   c.getSupportedAuthentication(camera),
		Encryption:       c.getSupportedEncryption(camera),
	}

	return streaming
}

// detectSpecialFeatures detects special camera features
func (c *CameraCapabilitiesDetector) detectSpecialFeatures(camera schema.HardwareDevice) []string {
	var features []string

	// Check for various special features
	if c.hasAutoFocus(camera) {
		features = append(features, "auto_focus")
	}
	if c.hasFaceDetection(camera) {
		features = append(features, "face_detection")
	}
	if c.hasMotionDetection(camera) {
		features = append(features, "motion_detection")
	}
	if c.hasPersonDetection(camera) {
		features = append(features, "person_detection")
	}
	if c.hasLicensePlateRecognition(camera) {
		features = append(features, "license_plate_recognition")
	}
	if c.hasObjectTracking(camera) {
		features = append(features, "object_tracking")
	}
	if c.hasPTZ(camera) {
		features = append(features, "ptz")
	}
	if c.hasPanoramicView(camera) {
		features = append(features, "panoramic")
	}
	if c.hasThermalImaging(camera) {
		features = append(features, "thermal")
	}
	if c.hasNightVision(camera) {
		features = append(features, "night_vision")
	}

	return features
}

// Platform-specific detection methods

func (c *CameraCapabilitiesDetector) getLinuxCameraInfo(camera schema.HardwareDevice) schema.CameraCapabilities {
	var caps schema.CameraCapabilities

	// Use v4l2-ctl to get detailed camera information
	cmd := exec.Command("v4l2-ctl", "--device="+camera.DevicePath, "--list-formats")
	if output, err := cmd.Output(); err == nil {
		caps.Formats = c.parseV4L2Formats(string(output))
	}

	// Get supported resolutions
	cmd = exec.Command("v4l2-ctl", "--device="+camera.DevicePath, "--list-formats-ext")
	if output, err := cmd.Output(); err == nil {
		caps.Video.MaxResolution = c.parseMaxResolution(string(output))
		caps.Video.SupportedCodecs = c.parseV4L2Codecs(string(output))
	}

	// Check for specific capabilities
	cmd = exec.Command("v4l2-ctl", "--device="+camera.DevicePath, "--list-ctrls")
	if output, err := cmd.Output(); err == nil {
		caps.Features = c.parseV4L2Features(string(output))
	}

	return caps
}

func (c *CameraCapabilitiesDetector) getDarwinCameraInfo(camera schema.HardwareDevice) schema.CameraCapabilities {
	var caps schema.CameraCapabilities

	// Use system_profiler to get camera details
	cmd := exec.Command("system_profiler", "SPCameraDataType", "-json")
	if output, err := cmd.Output(); err == nil {
		var cameraData map[string]interface{}
		if err := json.Unmarshal(output, &cameraData); err == nil {
			caps = c.parseSystemProfilerCamera(cameraData, camera.Name)
		}
	}

	// Check for FaceTime capabilities
	if strings.Contains(strings.ToLower(camera.Name), "facetime") {
		caps.Video.HDRSupport = true
		caps.Video.LowLightSupport = true
		caps.Features = append(caps.Features, "face_time")
	}

	return caps
}

func (c *CameraCapabilitiesDetector) getWindowsCameraInfo(camera schema.HardwareDevice) schema.CameraCapabilities {
	var caps schema.CameraCapabilities

	// Use PowerShell to get camera capabilities
	cmd := exec.Command("powershell", "-Command", "Get-CimInstance -ClassName Win32_VideoController | Select-Object Name, AdapterRAM, DriverVersion | ConvertTo-Json")
	if output, err := cmd.Output(); err == nil {
		var cameraData []map[string]interface{}
		if err := json.Unmarshal(output, &cameraData); err == nil {
			caps = c.parseWindowsCameraInfo(cameraData, camera.Name)
		}
	}

	// Check DirectShow capabilities
	cmd = exec.Command("ffmpeg", "-f", "dshow", "-list_options", "true", "-i", "video="+camera.Name)
	if output, err := cmd.CombinedOutput(); err == nil {
		caps.Formats = c.parseFFmpegFormats(string(output))
	}

	return caps
}

// Helper methods for capability detection

func (c *CameraCapabilitiesDetector) getMaxVideoResolution(camera schema.HardwareDevice) string {
	switch camera.Platform {
	case "linux":
		return c.getLinuxMaxResolution(camera)
	case "darwin":
		return c.getDarwinMaxResolution(camera)
	case "windows":
		return c.getWindowsMaxResolution(camera)
	}
	return "1920x1080"
}

func (c *CameraCapabilitiesDetector) getMinVideoResolution(camera schema.HardwareDevice) string {
	return "320x240"
}

func (c *CameraCapabilitiesDetector) getMaxFrameRate(camera schema.HardwareDevice) int {
	// Check for high frame rate capabilities
	if strings.Contains(strings.ToLower(camera.Name), "4k") ||
	   strings.Contains(strings.ToLower(camera.Name), "high_speed") {
		return 120
	}
	if strings.Contains(strings.ToLower(camera.Name), "hd") ||
	   strings.Contains(strings.ToLower(camera.Name), "1080") {
		return 60
	}
	return 30
}

func (c *CameraCapabilitiesDetector) getSupportedVideoCodecs(camera schema.HardwareDevice) []string {
	var codecs []string

	// Standard codecs that most cameras support
	codecs = append(codecs, "h264", "mjpeg")

	// Check for advanced codec support
	if strings.Contains(strings.ToLower(camera.Name), "4k") ||
	   strings.Contains(strings.ToLower(camera.Name), "ip_camera") {
		codecs = append(codecs, "h265")
	}

	return codecs
}

func (c *CameraCapabilitiesDetector) getSupportedColorSpaces(camera schema.HardwareDevice) []string {
	return []string{"YUV420", "YUYV", "RGB24"}
}

func (c *CameraCapabilitiesDetector) getBitrateRange(camera schema.HardwareDevice) schema.BitrateRange {
	return schema.BitrateRange{
		Min: 100,   // 100 kbps
		Max: 20000, // 20 Mbps
	}
}

func (c *CameraCapabilitiesDetector) checkHDRSupport(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "hdr") ||
		   strings.Contains(strings.ToLower(camera.Name), "wide_dynamic")
}

func (c *CameraCapabilitiesDetector) checkLowLightSupport(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "low_light") ||
		   strings.Contains(strings.ToLower(camera.Name), "night") ||
		   strings.Contains(strings.ToLower(camera.Name), "infrared")
}

func (c *CameraCapabilitiesDetector) checkWDRSupport(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "wdr") ||
		   strings.Contains(strings.ToLower(camera.Name), "wide_dynamic")
}

func (c *CameraCapabilitiesDetector) checkEISTSupport(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "eis") ||
		   strings.Contains(strings.ToLower(camera.Name), "electronic_stabilization")
}

func (c *CameraCapabilitiesDetector) check3DNRSupport(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "3d_nr") ||
		   strings.Contains(strings.ToLower(camera.Name), "noise_reduction")
}

func (c *CameraCapabilitiesDetector) getDigitalZoomRange(camera schema.HardwareDevice) schema.ZoomRange {
	return schema.ZoomRange{
		Min: 1.0,
		Max: c.getDigitalZoomMax(camera),
	}
}

func (c *CameraCapabilitiesDetector) getDigitalZoomMax(camera schema.HardwareDevice) float64 {
	if strings.Contains(strings.ToLower(camera.Name), "4k") {
		return 16.0
	}
	if strings.Contains(strings.ToLower(camera.Name), "1080") {
		return 8.0
	}
	return 4.0
}

func (c *CameraCapabilitiesDetector) getOpticalZoomRange(camera schema.HardwareDevice) schema.ZoomRange {
	if strings.Contains(strings.ToLower(camera.Name), "optical_zoom") ||
	   strings.Contains(strings.ToLower(camera.Name), "ptz") {
		return schema.ZoomRange{
			Min: 1.0,
			Max: c.getOpticalZoomMax(camera),
		}
	}
	return schema.ZoomRange{Min: 1.0, Max: 1.0}
}

func (c *CameraCapabilitiesDetector) getOpticalZoomMax(camera schema.HardwareDevice) float64 {
	if strings.Contains(strings.ToLower(camera.Name), "10x") {
		return 10.0
	}
	if strings.Contains(strings.ToLower(camera.Name), "5x") {
		return 5.0
	}
	if strings.Contains(strings.ToLower(camera.Name), "3x") {
		return 3.0
	}
	return 1.0
}

func (c *CameraCapabilitiesDetector) hasBuiltInMicrophone(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "microphone") ||
		   strings.Contains(strings.ToLower(camera.Name), "mic") ||
		   strings.Contains(strings.ToLower(camera.Name), "webcam")
}

func (c *CameraCapabilitiesDetector) getSupportedAudioCodecs(camera schema.HardwareDevice) []string {
	return []string{"PCM", "AAC", "G.711"}
}

func (c *CameraCapabilitiesDetector) getSampleRateRange(camera schema.HardwareDevice) schema.SampleRateRange {
	return schema.SampleRateRange{
		Min: 8000,
		Max: 48000,
	}
}

func (c *CameraCapabilitiesDetector) getBitDepthRange(camera schema.HardwareDevice) schema.BitDepthRange {
	return schema.BitDepthRange{
		Min: 16,
		Max: 24,
	}
}

func (c *CameraCapabilitiesDetector) getChannelCount(camera schema.HardwareDevice) int {
	if c.hasBuiltInMicrophone(camera) {
		return 1 // Mono
	}
	return 0 // No audio
}

func (c *CameraCapabilitiesDetector) checkNoiseReduction(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "noise_reduction")
}

func (c *CameraCapabilitiesDetector) checkAutoGainControl(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "auto_gain")
}

func (c *CameraCapabilitiesDetector) checkEchoCancellation(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "echo_cancellation")
}

func (c *CameraCapabilitiesDetector) getSensorType(camera schema.HardwareDevice) string {
	if strings.Contains(strings.ToLower(camera.Name), "cmos") {
		return "CMOS"
	}
	if strings.Contains(strings.ToLower(camera.Name), "ccd") {
		return "CCD"
	}
	return "Unknown"
}

func (c *CameraCapabilitiesDetector) getSensorSize(camera schema.HardwareDevice) string {
	// Simplified sensor size detection
	if strings.Contains(strings.ToLower(camera.Name), "4k") {
		return "1/2.3\""
	}
	if strings.Contains(strings.ToLower(camera.Name), "1080") {
		return "1/3\""
	}
	return "1/4\""
}

func (c *CameraCapabilitiesDetector) getLensMount(camera schema.HardwareDevice) string {
	return "Fixed" // Most cameras have fixed lenses
}

func (c *CameraCapabilitiesDetector) getFocalLengthRange(camera schema.HardwareDevice) schema.FocalLengthRange {
	return schema.FocalLengthRange{
		Min: 4.0,  // 4mm
		Max: 6.0,  // 6mm
	}
}

func (c *CameraCapabilitiesDetector) getApertureRange(camera schema.HardwareDevice) schema.ApertureRange {
	return schema.ApertureRange{
		Min: 1.8,  // f/1.8
		Max: 2.8,  // f/2.8
	}
}

func (c *CameraCapabilitiesDetector) getFieldOfView(camera schema.HardwareDevice) schema.FieldOfView {
	return schema.FieldOfView{
		Horizontal: 90.0, // 90 degrees
		Vertical:   67.5, // 67.5 degrees
		Diagonal:   107.0, // 107 degrees
	}
}

func (c *CameraCapabilitiesDetector) hasMechanicalShutter(camera schema.HardwareDevice) bool {
	return false // Most security cameras use electronic shutters
}

func (c *CameraCapabilitiesDetector) hasIRFilter(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "infrared") ||
		   strings.Contains(strings.ToLower(camera.Name), "ir")
}

func (c *CameraCapabilitiesDetector) hasIRCutFilter(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "ircut") ||
		   strings.Contains(strings.ToLower(camera.Name), "ir_cut")
}

func (c *CameraCapabilitiesDetector) checkWeatherResistance(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "weatherproof") ||
		   strings.Contains(strings.ToLower(camera.Name), "outdoor") ||
		   strings.Contains(strings.ToLower(camera.Name), "ip66") ||
		   strings.Contains(strings.ToLower(camera.Name), "ip67")
}

func (c *CameraCapabilitiesDetector) getConnectionType(camera schema.HardwareDevice) string {
	for _, cap := range camera.Capabilities {
		if cap == "usb" {
			return "USB"
		}
		if cap == "ip" {
			return "IP"
		}
		if cap == "wifi" {
			return "WiFi"
		}
	}
	return "Unknown"
}

func (c *CameraCapabilitiesDetector) getProtocol(camera schema.HardwareDevice) string {
	connType := c.getConnectionType(camera)
	switch connType {
	case "USB":
		return "USB 2.0/3.0"
	case "IP":
		return "RTSP/HTTP"
	case "WiFi":
		return "WiFi 802.11ac"
	default:
		return "Unknown"
	}
}

func (c *CameraCapabilitiesDetector) getBandwidth(camera schema.HardwareDevice) int64 {
	if c.getMaxVideoResolution(camera) == "3840x2160" { // 4K
		return 20000000 // 20 Mbps
	}
	if c.getMaxVideoResolution(camera) == "1920x1080" { // 1080p
		return 8000000 // 8 Mbps
	}
	return 2000000 // 2 Mbps
}

func (c *CameraCapabilitiesDetector) getPowerSource(camera schema.HardwareDevice) string {
	if c.getConnectionType(camera) == "USB" {
		return "Bus Powered"
	}
	return "External Power"
}

func (c *CameraCapabilitiesDetector) getPowerConsumption(camera schema.HardwareDevice) int64 {
	return 2000 // 2W typical for security cameras
}

func (c *CameraCapabilitiesDetector) getMaxLength(camera schema.HardwareDevice) int64 {
	switch c.getConnectionType(camera) {
	case "USB":
		return 5 // 5 meters for USB
	case "IP":
		return 100 // 100 meters for Ethernet
	default:
		return 0
	}
}

func (c *CameraCapabilitiesDetector) isHotPluggable(camera schema.HardwareDevice) bool {
	return c.getConnectionType(camera) == "USB"
}

func (c *CameraCapabilitiesDetector) getSupportedResolutions(camera schema.HardwareDevice) []string {
	var resolutions []string

	maxRes := c.getMaxVideoResolution(camera)
	if maxRes == "3840x2160" {
		resolutions = []string{
			"3840x2160", // 4K
			"2560x1440", // 1440p
			"1920x1080", // 1080p
			"1280x720",  // 720p
			"640x480",   // VGA
		}
	} else if maxRes == "1920x1080" {
		resolutions = []string{
			"1920x1080", // 1080p
			"1280x720",  // 720p
			"640x480",   // VGA
		}
	} else {
		resolutions = []string{
			"1280x720", // 720p
			"640x480",  // VGA
		}
	}

	return resolutions
}

func (c *CameraCapabilitiesDetector) getFrameRatesForResolution(camera schema.HardwareDevice, resolution string) []int {
	maxFPS := c.getMaxFrameRate(camera)
	var fps []int

	if resolution == "3840x2160" {
		fps = []int{15, 30}
		if maxFPS >= 60 {
			fps = append(fps, 60)
		}
	} else {
		fps = []int{15, 30}
		if maxFPS >= 60 {
			fps = append(fps, 60)
		}
		if maxFPS >= 120 {
			fps = append(fps, 120)
		}
	}

	return fps
}

func (c *CameraCapabilitiesDetector) getBitrateForResolution(camera schema.HardwareDevice, resolution string) schema.BitrateRange {
	switch resolution {
	case "3840x2160":
		return schema.BitrateRange{Min: 8000, Max: 20000}
	case "2560x1440":
		return schema.BitrateRange{Min: 6000, Max: 15000}
	case "1920x1080":
		return schema.BitrateRange{Min: 2000, Max: 8000}
	case "1280x720":
		return schema.BitrateRange{Min: 1000, Max: 4000}
	default:
		return schema.BitrateRange{Min: 500, Max: 2000}
	}
}

func (c *CameraCapabilitiesDetector) getSupportedAudioFormats(camera schema.HardwareDevice) []schema.MediaFormat {
	if !c.hasBuiltInMicrophone(camera) {
		return []schema.MediaFormat{}
	}

	return []schema.MediaFormat{
		{
			Type:        "audio",
			Container:   "AAC",
			Codec:       "AAC",
			SampleRate:  48000,
			BitDepth:    16,
			Channels:    1,
		},
		{
			Type:        "audio",
			Container:   "PCM",
			Codec:       "PCM",
			SampleRate:  44100,
			BitDepth:    16,
			Channels:    1,
		},
	}
}

func (c *CameraCapabilitiesDetector) getSupportedContainers(camera schema.HardwareDevice) []string {
	return []string{"MP4", "AVI", "MKV"}
}

func (c *CameraCapabilitiesDetector) checkRTSPSupport(camera schema.HardwareDevice) bool {
	return c.getConnectionType(camera) == "IP"
}

func (c *CameraCapabilitiesDetector) checkHTTPSupport(camera schema.HardwareDevice) bool {
	return c.getConnectionType(camera) == "IP"
}

func (c *CameraCapabilitiesDetector) checkWebRTCSupport(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "webrtc") ||
		   strings.Contains(strings.ToLower(camera.Name), "web_rtc")
}

func (c *CameraCapabilitiesDetector) checkHLSSupport(camera schema.HardwareDevice) bool {
	return c.getConnectionType(camera) == "IP"
}

func (c *CameraCapabilitiesDetector) checkMPEGDASHSupport(camera schema.HardwareDevice) bool {
	return c.getConnectionType(camera) == "IP"
}

func (c *CameraCapabilitiesDetector) getMaxSimultaneousStreams(camera schema.HardwareDevice) int {
	if strings.Contains(strings.ToLower(camera.Name), "4k") {
		return 2
	}
	return 3
}

func (c *CameraCapabilitiesDetector) getLatency(camera schema.HardwareDevice) time.Duration {
	if c.getConnectionType(camera) == "USB" {
		return 50 * time.Millisecond
	}
	if c.getConnectionType(camera) == "IP" {
		return 200 * time.Millisecond
	}
	return 100 * time.Millisecond
}

func (c *CameraCapabilitiesDetector) getBufferingSupport(camera schema.HardwareDevice) bool {
	return true
}

func (c *CameraCapabilitiesDetector) getSupportedTransports(camera schema.HardwareDevice) []string {
	switch c.getConnectionType(camera) {
	case "IP":
		return []string{"TCP", "UDP", "HTTP"}
	case "USB":
		return []string{"USB"}
	default:
		return []string{}
	}
}

func (c *CameraCapabilitiesDetector) getSupportedAuthentication(camera schema.HardwareDevice) []string {
	if c.getConnectionType(camera) == "IP" {
		return []string{"Basic", "Digest", "Bearer"}
	}
	return []string{}
}

func (c *CameraCapabilitiesDetector) getSupportedEncryption(camera schema.HardwareDevice) []string {
	if c.getConnectionType(camera) == "IP" {
		return []string{"TLS", "SRTP"}
	}
	return []string{}
}

// Special feature detection methods

func (c *CameraCapabilitiesDetector) hasAutoFocus(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "auto_focus") ||
		   strings.Contains(strings.ToLower(camera.Name), "af")
}

func (c *CameraCapabilitiesDetector) hasFaceDetection(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "face_detection") ||
		   strings.Contains(strings.ToLower(camera.Name), "ai_camera")
}

func (c *CameraCapabilitiesDetector) hasMotionDetection(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "motion_detection") ||
		   strings.Contains(strings.ToLower(camera.Name), "pir")
}

func (c *CameraCapabilitiesDetector) hasPersonDetection(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "person_detection") ||
		   strings.Contains(strings.ToLower(camera.Name), "ai_camera")
}

func (c *CameraCapabilitiesDetector) hasLicensePlateRecognition(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "license_plate") ||
		   strings.Contains(strings.ToLower(camera.Name), "lpr")
}

func (c *CameraCapabilitiesDetector) hasObjectTracking(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "object_tracking") ||
		   strings.Contains(strings.ToLower(camera.Name), "tracking")
}

func (c *CameraCapabilitiesDetector) hasPTZ(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "ptz") ||
		   strings.Contains(strings.ToLower(camera.Name), "pan_tilt_zoom")
}

func (c *CameraCapabilitiesDetector) hasPanoramicView(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "panoramic") ||
		   strings.Contains(strings.ToLower(camera.Name), "360") ||
		   strings.Contains(strings.ToLower(camera.Name), "fisheye")
}

func (c *CameraCapabilitiesDetector) hasThermalImaging(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "thermal") ||
		   strings.Contains(strings.ToLower(camera.Name), "infrared_camera")
}

func (c *CameraCapabilitiesDetector) hasNightVision(camera schema.HardwareDevice) bool {
	return strings.Contains(strings.ToLower(camera.Name), "night_vision") ||
		   strings.Contains(strings.ToLower(camera.Name), "infrared") ||
		   strings.Contains(strings.ToLower(camera.Name), "ir")
}

// Platform-specific helper methods

func (c *CameraCapabilitiesDetector) getLinuxMaxResolution(camera schema.HardwareDevice) string {
	cmd := exec.Command("v4l2-ctl", "--device="+camera.DevicePath, "--list-formats-ext")
	if output, err := cmd.Output(); err == nil {
		return c.parseMaxResolution(string(output))
	}
	return "1920x1080"
}

func (c *CameraCapabilitiesDetector) getDarwinMaxResolution(camera schema.HardwareDevice) string {
	if strings.Contains(strings.ToLower(camera.Name), "4k") {
		return "3840x2160"
	}
	if strings.Contains(strings.ToLower(camera.Name), "1080") {
		return "1920x1080"
	}
	return "1280x720"
}

func (c *CameraCapabilitiesDetector) getWindowsMaxResolution(camera schema.HardwareDevice) string {
	cmd := exec.Command("ffmpeg", "-f", "dshow", "-list_options", "true", "-i", "video="+camera.Name)
	if output, err := cmd.CombinedOutput(); err == nil {
		return c.parseFFmpegMaxResolution(string(output))
	}
	return "1920x1080"
}

// Parsing helper methods

func (c *CameraCapabilitiesDetector) parseV4L2Formats(output string) []schema.MediaFormat {
	var formats []schema.MediaFormat
	// Parse v4l2-ctl output to extract format information
	// This is a simplified implementation
	return formats
}

func (c *CameraCapabilitiesDetector) parseMaxResolution(output string) string {
	// Parse resolution from v4l2-ctl or ffmpeg output
	re := regexp.MustCompile(`(\d+)x(\d+)`)
	matches := re.FindAllStringSubmatch(output, -1)

	maxWidth := 0
	maxHeight := 0

	for _, match := range matches {
		if len(match) >= 3 {
			if width, err := strconv.Atoi(match[1]); err == nil {
				if height, err := strconv.Atoi(match[2]); err == nil {
					if width > maxWidth || (width == maxWidth && height > maxHeight) {
						maxWidth = width
						maxHeight = height
					}
				}
			}
		}
	}

	if maxWidth > 0 && maxHeight > 0 {
		return fmt.Sprintf("%dx%d", maxWidth, maxHeight)
	}

	return "1920x1080"
}

func (c *CameraCapabilitiesDetector) parseV4L2Codecs(output string) []string {
	var codecs []string
	// Parse v4l2-ctl output to extract codec information
	if strings.Contains(output, "H264") {
		codecs = append(codecs, "h264")
	}
	if strings.Contains(output, "MJPG") {
		codecs = append(codecs, "mjpeg")
	}
	if strings.Contains(output, "H265") || strings.Contains(output, "HEVC") {
		codecs = append(codecs, "h265")
	}
	return codecs
}

func (c *CameraCapabilitiesDetector) parseV4L2Features(output string) []string {
	var features []string
	// Parse v4l2-ctl output to extract feature information
	if strings.Contains(output, "auto_exposure") {
		features = append(features, "auto_exposure")
	}
	if strings.Contains(output, "auto_focus") {
		features = append(features, "auto_focus")
	}
	if strings.Contains(output, "white_balance") {
		features = append(features, "auto_white_balance")
	}
	return features
}

func (c *CameraCapabilitiesDetector) parseSystemProfilerCamera(data map[string]interface{}, cameraName string) schema.CameraCapabilities {
	var caps schema.CameraCapabilities
	// Parse system_profiler JSON output to extract camera capabilities
	return caps
}

func (c *CameraCapabilitiesDetector) parseWindowsCameraInfo(data []map[string]interface{}, cameraName string) schema.CameraCapabilities {
	var caps schema.CameraCapabilities
	// Parse Windows camera info from PowerShell output
	return caps
}

func (c *CameraCapabilitiesDetector) parseFFmpegFormats(output string) []schema.MediaFormat {
	var formats []schema.MediaFormat
	// Parse ffmpeg output to extract format information
	return formats
}

func (c *CameraCapabilitiesDetector) parseFFmpegMaxResolution(output string) string {
	// Parse ffmpeg output to extract maximum resolution
	return c.parseMaxResolution(output)
}