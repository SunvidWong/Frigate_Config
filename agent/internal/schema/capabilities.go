package schema

import (
	"time"
)

// GPUCapabilities represents detailed GPU capabilities
type GPUCapabilities struct {
	DeviceID    string                 `json:"device_id"`
	DeviceName  string                 `json:"device_name"`
	Vendor      string                 `json:"vendor"`
	Driver      *string                `json:"driver,omitempty"`
	Platform    string                 `json:"platform"`
	Architecture string                `json:"architecture"`
	Encode      EncodeCapabilities     `json:"encode"`
	Decode      DecodeCapabilities     `json:"decode"`
	Compute     ComputeCapabilities    `json:"compute"`
	Memory      MemoryCapabilities     `json:"memory"`
	APIs        APISupport             `json:"apis"`
	Performance PerformanceCharacteristics `json:"performance"`
}

// CameraCapabilities represents detailed camera capabilities
type CameraCapabilities struct {
	DeviceID     string                  `json:"device_id"`
	DeviceName   string                  `json:"device_name"`
	DevicePath   string                  `json:"device_path"`
	Platform     string                  `json:"platform"`
	Architecture string                  `json:"architecture"`
	Video        VideoCapabilities       `json:"video"`
	Audio        AudioCapabilities       `json:"audio"`
	Physical     PhysicalCharacteristics  `json:"physical"`
	Connection   ConnectionType          `json:"connection"`
	Formats      []MediaFormat           `json:"formats"`
	Streaming    StreamingCapabilities   `json:"streaming"`
	Features     []string                `json:"features"`
}

// TPUCapabilities represents detailed TPU capabilities
type TPUCapabilities struct {
	DeviceID     string                     `json:"device_id"`
	DeviceName   string                     `json:"device_name"`
	DevicePath   string                     `json:"device_path"`
	Platform     string                     `json:"platform"`
	Architecture string                     `json:"architecture"`
	Type         string                     `json:"type"`
	Generation   string                     `json:"generation"`
	Compute      TPUComputeCapabilities     `json:"compute"`
	Memory       TPUMemoryCapabilities      `json:"memory"`
	SupportedModels []string                 `json:"supported_models"`
	Performance  TPUPerformanceCharacteristics `json:"performance"`
	Power        TPUPowerCharacteristics    `json:"power"`
	Frameworks   []string                   `json:"frameworks"`
	Thermal      TPUThermalCharacteristics  `json:"thermal"`
}

// CaptureCardCapabilities represents detailed capture card capabilities
type CaptureCardCapabilities struct {
	DeviceID     string                       `json:"device_id"`
	DeviceName   string                       `json:"device_name"`
	DevicePath   string                       `json:"device_path"`
	Platform     string                       `json:"platform"`
	Architecture string                       `json:"architecture"`
	Type         string                       `json:"type"`
	FormFactor   string                       `json:"form_factor"`
	Interface    string                       `json:"interface"`
	Inputs       InputCapabilities            `json:"inputs"`
	Video        VideoCaptureCapabilities      `json:"video"`
	Audio        AudioCaptureCapabilities      `json:"audio"`
	Streaming    StreamingCapabilities         `json:"streaming"`
	Formats      []MediaFormat                `json:"formats"`
	Features     []string                     `json:"features"`
	Performance  PerformanceCharacteristics    `json:"performance"`
	Connectivity ConnectivityOptions          `json:"connectivity"`
}

// HardwareAvailability represents hardware availability and usage status
type HardwareAvailability struct {
	DeviceID     string                 `json:"device_id"`
	DeviceName   string                 `json:"device_name"`
	DeviceType   string                 `json:"device_type"`
	Platform     string                 `json:"platform"`
	Available    bool                   `json:"available"`
	InUse        bool                   `json:"in_use"`
	LastChecked  time.Time              `json:"last_checked"`
	Usage        UsageMetrics           `json:"usage"`
	Performance  PerformanceMetrics     `json:"performance"`
	Thermal      ThermalStatus          `json:"thermal"`
	Power        PowerStatus            `json:"power"`
	Processes    []ProcessInfo          `json:"processes"`
	Errors       []string               `json:"errors"`
	Warnings     []string               `json:"warnings"`
	Capacity     CapacityInfo           `json:"capacity"`
}

// EncodeCapabilities represents video encoding capabilities
type EncodeCapabilities struct {
	H264               bool     `json:"h264"`
	HEVC               bool     `json:"hevc"`
	AV1                bool     `json:"av1"`
	VP9                bool     `json:"vp9"`
	VP8                bool     `json:"vp8"`
	MPEG2              bool     `json:"mpeg2"`
	H264Quality        string   `json:"h264_quality"`
	HEVCQuality        string   `json:"hevc_quality"`
	AV1Quality         string   `json:"av1_quality"`
	MaxSimultaneousStreams int    `json:"max_simultaneous_streams"`
}

// DecodeCapabilities represents video decoding capabilities
type DecodeCapabilities struct {
	H264               bool     `json:"h264"`
	HEVC               bool     `json:"hevc"`
	AV1                bool     `json:"av1"`
	VP9                bool     `json:"vp9"`
	VP8                bool     `json:"vp8"`
	MPEG2              bool     `json:"mpeg2"`
	MaxResolution      string   `json:"max_resolution"`
	MaxFrameRate       int      `json:"max_frame_rate"`
	MaxSimultaneousStreams int    `json:"max_simultaneous_streams"`
}

// ComputeCapabilities represents compute capabilities
type ComputeCapabilities struct {
	CUDA       bool     `json:"cuda"`
	OpenCL     bool     `json:"opencl"`
	Vulkan     bool     `json:"vulkan"`
	DirectX    bool     `json:"directx"`
	Metal      bool     `json:"metal"`
	OpenGL     bool     `json:"opengl"`
	ComputeUnits int    `json:"compute_units"`
	Cores      int      `json:"cores"`
	MemoryBandwidth int64 `json:"memory_bandwidth"`
	TensorOps  bool     `json:"tensor_ops"`
}

// MemoryCapabilities represents memory capabilities
type MemoryCapabilities struct {
	Total     int64    `json:"total"`
	Available int64    `json:"available"`
	Used      int64    `json:"used"`
	Bandwidth int64    `json:"bandwidth"`
	Types     []string `json:"types"`
	Cache     int64    `json:"cache"`
}

// APISupport represents API support
type APISupport struct {
	CUDA       *string `json:"cuda,omitempty"`
	OpenCL     *string `json:"opencl,omitempty"`
	Vulkan     *string `json:"vulkan,omitempty"`
	Metal      *string `json:"metal,omitempty"`
	OpenGL     *string `json:"opengl,omitempty"`
	DirectX    *string `json:"directx,omitempty"`
	Direct3D   *string `json:"direct3d,omitempty"`
	Direct3D12 *string `json:"direct3d12,omitempty"`
}

// PerformanceCharacteristics represents performance characteristics
type PerformanceCharacteristics struct {
	BaseClock      int64   `json:"base_clock"`
	BoostClock     int64   `json:"boost_clock"`
	MemoryClock    int64   `json:"memory_clock"`
	TDP            int64   `json:"tdp"`
	PowerLimits    []int64 `json:"power_limits"`
	ThermalThrottling bool  `json:"thermal_throttling"`
}

// VideoCapabilities represents video capabilities
type VideoCapabilities struct {
	MaxResolution     string        `json:"max_resolution"`
	MinResolution     string        `json:"min_resolution"`
	MaxFrameRate      int           `json:"max_frame_rate"`
	SupportedCodecs   []string      `json:"supported_codecs"`
	ColorSpaces       []string      `json:"color_spaces"`
	BitrateRange      BitrateRange  `json:"bitrate_range"`
	HDRSupport        bool          `json:"hdr_support"`
	LowLightSupport   bool          `json:"low_light_support"`
	WDRSupport        bool          `json:"wdr_support"`
	EISTSupport       bool          `json:"eist_support"`
	DNRSupport3D      bool          `json:"dnr_support_3d"`
	DigitalZoom       ZoomRange     `json:"digital_zoom"`
	OpticalZoom       ZoomRange     `json:"optical_zoom"`
}

// AudioCapabilities represents audio capabilities
type AudioCapabilities struct {
	HasMicrophone    bool           `json:"has_microphone"`
	SupportedCodecs  []string       `json:"supported_codecs"`
	SampleRateRange  SampleRateRange `json:"sample_rate_range"`
	BitDepthRange    BitDepthRange  `json:"bit_depth_range"`
	ChannelCount     int            `json:"channel_count"`
	AudioInputs      []string       `json:"audio_inputs"`
	AudioStandards   []string       `json:"audio_standards"`
	NoiseReduction   bool           `json:"noise_reduction"`
	AGC              bool           `json:"agc"`
	AudioDelay       Range          `json:"audio_delay"`
	VolumeControl    bool           `json:"volume_control"`
	MuteControl      bool           `json:"mute_control"`
}

// PhysicalCharacteristics represents physical characteristics
type PhysicalCharacteristics struct {
	SensorType        string          `json:"sensor_type"`
	SensorSize        string          `json:"sensor_size"`
	LensMount         string          `json:"lens_mount"`
	FocalLength       FocalLengthRange `json:"focal_length"`
	Aperture          ApertureRange   `json:"aperture"`
	FieldOfView       FieldOfView     `json:"field_of_view"`
	MechanicalShutter bool            `json:"mechanical_shutter"`
	IRFilter          bool            `json:"ir_filter"`
	IRCutFilter       bool            `json:"ircut_filter"`
	WeatherResistance bool            `json:"weather_resistance"`
}

// ConnectionType represents connection type
type ConnectionType struct {
	Type            string  `json:"type"`
	Protocol        string  `json:"protocol"`
	Bandwidth       int64   `json:"bandwidth"`
	PowerSource     string  `json:"power_source"`
	PowerConsumption int64  `json:"power_consumption"`
	MaxLength       int64   `json:"max_length"`
	HotPluggable    bool    `json:"hot_pluggable"`
}

// MediaFormat represents media format
type MediaFormat struct {
	Type       string `json:"type"`
	Container  string `json:"container"`
	Resolution string `json:"resolution"`
	FrameRate  int    `json:"frame_rate"`
	Codec      string `json:"codec"`
	Bitrate    int    `json:"bitrate"`
	SampleRate int    `json:"sample_rate"`
	BitDepth   int    `json:"bit_depth"`
	Channels   int    `json:"channels"`
}

// StreamingCapabilities represents streaming capabilities
type StreamingCapabilities struct {
	RTSPSupport      bool     `json:"rtsp_support"`
	HTTPSupport      bool     `json:"http_support"`
	WebRTCSupport    bool     `json:"webrtc_support"`
	HLSsupport       bool     `json:"hls_support"`
	MPEGDASHSupport  bool     `json:"mpegdash_support"`
	MaxStreams       int      `json:"max_streams"`
	Latency          time.Duration `json:"latency"`
	Buffering        bool     `json:"buffering"`
	RealTime         bool     `json:"real_time"`
	Compression      []string `json:"compression"`
	Multiplexing     bool     `json:"multiplexing"`
	Timestamping     bool     `json:"timestamping"`
	Transport        []string `json:"transport"`
	Authentication   []string `json:"authentication"`
	Encryption       []string `json:"encryption"`
}

// TPUComputeCapabilities represents TPU compute capabilities
type TPUComputeCapabilities struct {
	MaxOPS         int64     `json:"max_ops"`
	MaxTOPS        float64   `json:"max_tops"`
	SupportedOps   []string  `json:"supported_ops"`
	Precision      []string  `json:"precision"`
	DataTypes      []string  `json:"data_types"`
	BatchSize      BatchSizeRange `json:"batch_size"`
	Latency        time.Duration `json:"latency"`
	Throughput     float64   `json:"throughput"`
	PowerEfficiency float64  `json:"power_efficiency"`
	Quantization   bool      `json:"quantization"`
	Pruning        bool      `json:"pruning"`
	Sparsity       bool      `json:"sparsity"`
}

// TPUMemoryCapabilities represents TPU memory capabilities
type TPUMemoryCapabilities struct {
	Total     int64    `json:"total"`
	Available int64    `json:"available"`
	Bandwidth int64    `json:"bandwidth"`
	Type      string   `json:"type"`
	Cache     int64    `json:"cache"`
}

// TPUPerformanceCharacteristics represents TPU performance characteristics
type TPUPerformanceCharacteristics struct {
	MaxFrequency     int64   `json:"max_frequency"`
	BaseFrequency    int64   `json:"base_frequency"`
	BoostFrequency   int64   `json:"boost_frequency"`
	ClockSpeed       int64   `json:"clock_speed"`
	InstructionsPerCycle int  `json:"instructions_per_cycle"`
	PipelineDepth    int     `json:"pipeline_depth"`
	Cores            int     `json:"cores"`
	ExecutionUnits   int     `json:"execution_units"`
}

// TPUPowerCharacteristics represents TPU power characteristics
type TPUPowerCharacteristics struct {
	TDP             float64   `json:"tdp"`
	BasePower       float64   `json:"base_power"`
	MaxPower        float64   `json:"max_power"`
	IdlePower       float64   `json:"idle_power"`
	PowerStates     []string  `json:"power_states"`
	VoltageRange    VoltageRange `json:"voltage_range"`
	CurrentRange    CurrentRange `json:"current_range"`
	Efficiency      float64   `json:"efficiency"`
}

// TPUThermalCharacteristics represents TPU thermal characteristics
type TPUThermalCharacteristics struct {
	MaxTemperature        float64 `json:"max_temperature"`
	OptimalTemperature    float64 `json:"optimal_temperature"`
	ThrottlingTemperature float64 `json:"throttling_temperature"`
	CriticalTemperature   float64 `json:"critical_temperature"`
	CoolingSolution       string  `json:"cooling_solution"`
	ThermalManagement     []string `json:"thermal_management"`
}

// InputCapabilities represents input capabilities
type InputCapabilities struct {
	MaxInputs         int         `json:"max_inputs"`
	SupportedSignals  []string    `json:"supported_signals"`
	SupportedResolutions []string  `json:"supported_resolutions"`
	SupportedFrameRates []int     `json:"supported_frame_rates"`
	HDCPSupport       bool        `json:"hdcp_support"`
	AnalogInputs      int         `json:"analog_inputs"`
	DigitalInputs     int         `json:"digital_inputs"`
	AudioInputs       int         `json:"audio_inputs"`
}

// VideoCaptureCapabilities represents video capture capabilities
type VideoCaptureCapabilities struct {
	MaxResolution      string   `json:"max_resolution"`
	MinResolution      string   `json:"min_resolution"`
	MaxFrameRate       int      `json:"max_frame_rate"`
	SupportedCodecs    []string `json:"supported_codecs"`
	ColorSpaces        []string `json:"color_spaces"`
	BitDepth           []int    `json:"bit_depth"`
	ChromaSubsampling  []string `json:"chroma_subsampling"`
	ColorRange         []string `json:"color_range"`
	VideoStandards     []string `json:"video_standards"`
	Deinterlacing      bool     `json:"deinterlacing"`
	NoiseReduction     bool     `json:"noise_reduction"`
	ColorCorrection    bool     `json:"color_correction"`
	GammaCorrection    bool     `json:"gamma_correction"`
	Sharpness          Range    `json:"sharpness"`
	Brightness         Range    `json:"brightness"`
	Contrast           Range    `json:"contrast"`
	Saturation         Range    `json:"saturation"`
	Hue                Range    `json:"hue"`
}

// AudioCaptureCapabilities represents audio capture capabilities
type AudioCaptureCapabilities struct {
	HasAudioInput     bool           `json:"has_audio_input"`
	SupportedCodecs   []string       `json:"supported_codecs"`
	SampleRateRange   SampleRateRange `json:"sample_rate_range"`
	BitDepthRange     BitDepthRange  `json:"bit_depth_range"`
	ChannelCount      int            `json:"channel_count"`
	AudioInputs       []string       `json:"audio_inputs"`
	AudioStandards    []string       `json:"audio_standards"`
	NoiseReduction    bool           `json:"noise_reduction"`
	AGC               bool           `json:"agc"`
	AudioDelay        Range          `json:"audio_delay"`
	VolumeControl     bool           `json:"volume_control"`
	MuteControl       bool           `json:"mute_control"`
}

// ConnectivityOptions represents connectivity options
type ConnectivityOptions struct {
	PhysicalPorts    []string `json:"physical_ports"`
	TriggerInputs    int      `json:"trigger_inputs"`
	GPIOPins         int      `json:"gpio_pins"`
	SyncConnectors   []string `json:"sync_connectors"`
	TimecodeInputs   []string `json:"timecode_inputs"`
	TimecodeOutputs  []string `json:"timecode_outputs"`
	GenlockInput     bool     `json:"genlock_input"`
	GenlockOutput    bool     `json:"genlock_output"`
	WordClock        bool     `json:"word_clock"`
	NetworkInterfaces []string `json:"network_interfaces"`
}

// UsageMetrics represents usage metrics
type UsageMetrics struct {
	Utilization        float64 `json:"utilization"`
	MemoryUtilization  float64 `json:"memory_utilization"`
	MemoryUsed         int64   `json:"memory_used"`
	MemoryTotal        int64   `json:"memory_total"`
	CPUUtilization     float64 `json:"cpu_utilization"`
	Temperature        float64 `json:"temperature"`
	ClockSpeed         int64   `json:"clock_speed"`
	PowerConsumption   float64 `json:"power_consumption"`
}

// PerformanceMetrics represents performance metrics
type PerformanceMetrics struct {
	Temperature      float64 `json:"temperature"`
	PowerConsumption float64 `json:"power_consumption"`
	ClockSpeed       int64   `json:"clock_speed"`
	FanSpeed         int     `json:"fan_speed"`
	Throughput       float64 `json:"throughput"`
	Latency          int     `json:"latency"`
	Quality          float64 `json:"quality"`
	ErrorRate        float64 `json:"error_rate"`
}

// ThermalStatus represents thermal status
type ThermalStatus struct {
	Temperature float64 `json:"temperature"`
	Throttling bool    `json:"throttling"`
	Critical   bool    `json:"critical"`
	FanSpeed   int     `json:"fan_speed"`
}

// PowerStatus represents power status
type PowerStatus struct {
	CurrentPower float64 `json:"current_power"`
	PowerLimit   float64 `json:"power_limit"`
	Voltage      float64 `json:"voltage"`
	Current      float64 `json:"current"`
	Efficiency   float64 `json:"efficiency"`
}

// ProcessInfo represents process information
type ProcessInfo struct {
	PID          int    `json:"pid"`
	Name         string `json:"name"`
	MemoryUsage  int64  `json:"memory_usage"`
	CPUUsage     float64 `json:"cpu_usage"`
}

// CapacityInfo represents capacity information
type CapacityInfo struct {
	MaxThroughput     float64 `json:"max_throughput"`
	CurrentThroughput float64 `json:"current_throughput"`
	MaxLoad           float64 `json:"max_load"`
	CurrentLoad       float64 `json:"current_load"`
	MaxTemperature    float64 `json:"max_temperature"`
	CurrentTemperature float64 `json:"current_temperature"`
	MaxPower          float64 `json:"max_power"`
	CurrentPower      float64 `json:"current_power"`
}

// Helper types
type BitrateRange struct {
	Min int `json:"min"`
	Max int `json:"max"`
}

type ZoomRange struct {
	Min float64 `json:"min"`
	Max float64 `json:"max"`
}

type SampleRateRange struct {
	Min int `json:"min"`
	Max int `json:"max"`
}

type BitDepthRange struct {
	Min int `json:"min"`
	Max int `json:"max"`
}

type Range struct {
	Min float64 `json:"min"`
	Max float64 `json:"max"`
}

type BatchSizeRange struct {
	Min int `json:"min"`
	Max int `json:"max"`
}

type FocalLengthRange struct {
	Min float64 `json:"min"`
	Max float64 `json:"max"`
}

type ApertureRange struct {
	Min float64 `json:"min"`
	Max float64 `json:"max"`
}

type FieldOfView struct {
	Horizontal float64 `json:"horizontal"`
	Vertical   float64 `json:"vertical"`
	Diagonal   float64 `json:"diagonal"`
}

type VoltageRange struct {
	Min float64 `json:"min"`
	Max float64 `json:"max"`
}

type CurrentRange struct {
	Min float64 `json:"min"`
	Max float64 `json:"max"`
}