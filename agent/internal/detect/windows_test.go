//go:build windows

package detect

import (
	"testing"

	"github.com/frigate-config-tool/agent/internal/schema"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestWindowsDetector_DetectGPUs tests GPU detection on Windows
func TestWindowsDetector_DetectGPUs(t *testing.T) {
	detector := NewWindowsDetector("windows", "amd64")

	tests := []struct {
		name     string
		setup    func() (cleanup func())
		wantErr  bool
		validate func(t *testing.T, devices []schema.HardwareDevice)
	}{
		{
			name: "detect_gpus_no_error",
			setup: func() func() {
				// GPU detection should work even if no GPUs or tools available
				return func() {}
			},
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				// Should not panic or error
				assert.NotNil(t, devices)

				// Validate each detected GPU
				for _, dev := range devices {
					assert.Equal(t, "gpu", dev.Type)
					assert.NotEmpty(t, dev.Name)
					assert.NotEmpty(t, dev.Capabilities)
					assert.Equal(t, "windows", dev.Platform)
					assert.Contains(t, []string{"nvidia-smi", "wmic", "amdgpu-pro", "intel-gpu-tools"}, dev.DetectionSource)
				}
			},
		},
		{
			name: "no_duplicate_gpu_ids",
			setup: func() func() {
				return func() {}
			},
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				// Check for duplicate device IDs
				seen := make(map[string]bool)
				for _, dev := range devices {
					assert.False(t, seen[dev.ID], "Duplicate device ID: %s", dev.ID)
					seen[dev.ID] = true
				}
			},
		},
		{
			name: "gpu_platform_architecture_set",
			setup: func() func() {
				return func() {}
			},
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				for _, dev := range devices {
					assert.Equal(t, "windows", dev.Platform)
					assert.Equal(t, "amd64", dev.Architecture)
				}
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cleanup := tt.setup()
			defer cleanup()

			devices, err := detector.DetectGPUs()
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
			}

			if tt.validate != nil {
				tt.validate(t, devices)
			}
		})
	}
}

// TestWindowsDetector_DetectTPUs tests TPU detection on Windows
func TestWindowsDetector_DetectTPUs(t *testing.T) {
	detector := NewWindowsDetector("windows", "amd64")

	tests := []struct {
		name     string
		wantErr  bool
		validate func(t *testing.T, devices []schema.HardwareDevice)
	}{
		{
			name:    "detect_tpus_no_error",
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				// TPU detection should not fail even if no TPUs present
				assert.NotNil(t, devices)

				// If TPUs are detected, validate their properties
				for _, dev := range devices {
					assert.Equal(t, "tpu", dev.Type)
					assert.NotEmpty(t, dev.Name)
					assert.Equal(t, "windows", dev.Platform)
					assert.NotEmpty(t, dev.Capabilities)
					assert.Contains(t, []string{"wmic", "openvino_check"}, dev.DetectionSource)
				}
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			devices, err := detector.DetectTPUs()
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
			}

			if tt.validate != nil {
				tt.validate(t, devices)
			}
		})
	}
}

// TestWindowsDetector_DetectCameras tests camera detection on Windows
func TestWindowsDetector_DetectCameras(t *testing.T) {
	detector := NewWindowsDetector("windows", "amd64")

	tests := []struct {
		name     string
		wantErr  bool
		validate func(t *testing.T, devices []schema.HardwareDevice)
	}{
		{
			name:    "detect_cameras_no_error",
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				// Camera detection should not fail
				assert.NotNil(t, devices)

				// Validate detected cameras
				for _, dev := range devices {
					assert.Equal(t, "camera", dev.Type)
					assert.NotEmpty(t, dev.Name)
					assert.Equal(t, "windows", dev.Platform)
					assert.Contains(t, []string{"powershell", "wmic"}, dev.DetectionSource)
					assert.Contains(t, dev.Capabilities, "capture")
				}
			},
		},
		{
			name:    "no_duplicate_camera_ids",
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				// Check for duplicate device IDs
				seen := make(map[string]bool)
				for _, dev := range devices {
					assert.False(t, seen[dev.ID], "Duplicate device ID: %s", dev.ID)
					seen[dev.ID] = true
				}
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			devices, err := detector.DetectCameras()
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
			}

			if tt.validate != nil {
				tt.validate(t, devices)
			}
		})
	}
}

// TestWindowsDetector_DetectCaptureCards tests capture card detection on Windows
func TestWindowsDetector_DetectCaptureCards(t *testing.T) {
	detector := NewWindowsDetector("windows", "amd64")

	tests := []struct {
		name     string
		wantErr  bool
		validate func(t *testing.T, devices []schema.HardwareDevice)
	}{
		{
			name:    "detect_capture_cards_no_error",
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				// Capture card detection should not fail
				assert.NotNil(t, devices)

				// Validate detected capture cards
				for _, dev := range devices {
					assert.Equal(t, "capture_card", dev.Type)
					assert.NotEmpty(t, dev.Name)
					assert.Equal(t, "windows", dev.Platform)
					assert.Contains(t, []string{"wmic", "directx"}, dev.DetectionSource)
					assert.NotEmpty(t, dev.Capabilities)
				}
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			devices, err := detector.DetectCaptureCards()
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
			}

			if tt.validate != nil {
				tt.validate(t, devices)
			}
		})
	}
}

// TestWindowsDetector_DetectAll tests detecting all hardware types
func TestWindowsDetector_DetectAll(t *testing.T) {
	detector := NewWindowsDetector("windows", "amd64")

	t.Run("detect_all_hardware", func(t *testing.T) {
		devices, err := detector.DetectAll()
		assert.NoError(t, err)
		assert.NotNil(t, devices)

		// Count devices by type
		deviceTypes := make(map[string]int)
		for _, dev := range devices {
			deviceTypes[dev.Type]++
			assert.Equal(t, "windows", dev.Platform)
			assert.NotEmpty(t, dev.Name)
		}

		// Log detected device counts
		t.Logf("Detected devices: %v", deviceTypes)
	})
}

// TestWindowsDetector_CapabilityDetection tests GPU capability detection
func TestWindowsDetector_CapabilityDetection(t *testing.T) {
	detector := NewWindowsDetector("windows", "amd64")

	tests := []struct {
		name              string
		gpuName           string
		expectedCapabilities []string
		method            string // "nvidia", "amd", or "intel"
	}{
		{
			name:             "nvidia_rtx_4090",
			gpuName:          "NVIDIA GeForce RTX 4090",
			expectedCapabilities: []string{"cuda", "nvenc", "nvdec", "tensorrt", "ray_tracing", "av1_encode"},
			method:           "nvidia",
		},
		{
			name:             "nvidia_rtx_3070",
			gpuName:          "NVIDIA GeForce RTX 3070",
			expectedCapabilities: []string{"cuda", "nvenc", "nvdec", "tensorrt", "ray_tracing"},
			method:           "nvidia",
		},
		{
			name:             "nvidia_quadro",
			gpuName:          "NVIDIA Quadro P4000",
			expectedCapabilities: []string{"cuda", "nvenc", "professional"},
			method:           "nvidia",
		},
		{
			name:             "amd_rx_7900",
			gpuName:          "AMD Radeon RX 7900 XT",
			expectedCapabilities: []string{"opencl", "vulkan", "rdna2", "ray_tracing", "av1_decode"},
			method:           "amd",
		},
		{
			name:             "amd_rx_6800",
			gpuName:          "AMD Radeon RX 6800",
			expectedCapabilities: []string{"opencl", "vulkan", "rdna2", "ray_tracing"},
			method:           "amd",
		},
		{
			name:             "intel_arc_a770",
			gpuName:          "Intel Arc A770",
			expectedCapabilities: []string{"quick_sync", "ray_tracing", "xmx", "ai_acceleration"},
			method:           "intel",
		},
		{
			name:             "intel_iris_xe",
			gpuName:          "Intel Iris Xe Graphics",
			expectedCapabilities: []string{"quick_sync", "av1_decode", "opencl"},
			method:           "intel",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var capabilities []string

			switch tt.method {
			case "nvidia":
				capabilities = detector.getNVIDIACapabilities(tt.gpuName)
			case "amd":
				capabilities = detector.getAMDCapabilities(tt.gpuName)
			case "intel":
				capabilities = detector.getIntelCapabilities(tt.gpuName)
			}

			// Check that expected capabilities are present
			for _, expected := range tt.expectedCapabilities {
				assert.Contains(t, capabilities, expected,
					"GPU %s should have capability: %s", tt.gpuName, expected)
			}
		})
	}
}

// TestWindowsDetector_CameraCapabilityDetection tests camera capability detection
func TestWindowsDetector_CameraCapabilityDetection(t *testing.T) {
	detector := NewWindowsDetector("windows", "amd64")

	tests := []struct {
		name              string
		cameraName        string
		expectedCapabilities []string
	}{
		{
			name:              "logitech_1080p",
			cameraName:        "Logitech HD Pro Webcam C920 1080p",
			expectedCapabilities: []string{"capture", "video_capture", "1080p"},
		},
		{
			name:              "microsoft_4k",
			cameraName:        "Microsoft LifeCam Studio 4K",
			expectedCapabilities: []string{"capture", "video_capture", "4k"},
		},
		{
			name:              "wide_angle_hd",
			cameraName:        "Wide Angle HD Camera 720p",
			expectedCapabilities: []string{"capture", "video_capture", "720p", "wide_angle"},
		},
		{
			name:              "infrared_camera",
			cameraName:        "Infrared Camera IR",
			expectedCapabilities: []string{"capture", "video_capture", "infrared"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			capabilities := detector.getCameraCapabilities(tt.cameraName)

			// Check that expected capabilities are present
			for _, expected := range tt.expectedCapabilities {
				assert.Contains(t, capabilities, expected,
					"Camera %s should have capability: %s", tt.cameraName, expected)
			}
		})
	}
}

// TestWindowsDetector_CaptureCardDetection tests capture card identification
func TestWindowsDetector_CaptureCardDetection(t *testing.T) {
	detector := NewWindowsDetector("windows", "amd64")

	tests := []struct {
		name        string
		deviceName  string
		description string
		isCaptureCard bool
	}{
		{
			name:          "elgato_hd60",
			deviceName:    "Elgato Game Capture HD60 S+",
			description:   "HDMI Video Capture Device",
			isCaptureCard: true,
		},
		{
			name:          "avermedia_live_gamer",
			deviceName:    "AVerMedia Live Gamer EXTREME 2",
			description:   "Video Capture Card",
			isCaptureCard: true,
		},
		{
			name:          "blackmagic_decklink",
			deviceName:    "Blackmagic DeckLink Mini Recorder 4K",
			description:   "Professional Video Capture",
			isCaptureCard: true,
		},
		{
			name:          "regular_webcam",
			deviceName:    "Logitech HD Webcam",
			description:   "USB Video Device",
			isCaptureCard: false,
		},
		{
			name:          "display_adapter",
			deviceName:    "NVIDIA Display Adapter",
			description:   "Graphics Card",
			isCaptureCard: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := detector.isCaptureCard(tt.deviceName, tt.description)
			assert.Equal(t, tt.isCaptureCard, result,
				"Device: %s, Description: %s", tt.deviceName, tt.description)
		})
	}
}

// TestWindowsDetector_PlatformAndArchitecture tests platform and architecture reporting
func TestWindowsDetector_PlatformAndArchitecture(t *testing.T) {
	tests := []struct {
		name     string
		platform string
		arch     string
	}{
		{
			name:     "amd64_windows",
			platform: "windows",
			arch:     "amd64",
		},
		{
			name:     "arm64_windows",
			platform: "windows",
			arch:     "arm64",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			detector := NewWindowsDetector(tt.platform, tt.arch)
			require.NotNil(t, detector)

			// Test GPU detection includes platform and architecture
			gpus, err := detector.DetectGPUs()
			assert.NoError(t, err)
			for _, gpu := range gpus {
				assert.Equal(t, tt.platform, gpu.Platform)
				assert.Equal(t, tt.arch, gpu.Architecture)
			}

			// Test TPU detection includes platform and architecture
			tpus, err := detector.DetectTPUs()
			assert.NoError(t, err)
			for _, tpu := range tpus {
				assert.Equal(t, tt.platform, tpu.Platform)
				assert.Equal(t, tt.arch, tpu.Architecture)
			}

			// Test Camera detection includes platform and architecture
			cameras, err := detector.DetectCameras()
			assert.NoError(t, err)
			for _, camera := range cameras {
				assert.Equal(t, tt.platform, camera.Platform)
				assert.Equal(t, tt.arch, camera.Architecture)
			}
		})
	}
}

// TestWindowsDetector_ErrorHandling tests graceful error handling
func TestWindowsDetector_ErrorHandling(t *testing.T) {
	detector := NewWindowsDetector("windows", "amd64")

	t.Run("missing_tools_graceful", func(t *testing.T) {
		// Even if tools like nvidia-smi, wmic, PowerShell are missing/fail,
		// detection should not panic and should return empty results
		assert.NotPanics(t, func() {
			_, _ = detector.DetectGPUs()
			_, _ = detector.DetectTPUs()
			_, _ = detector.DetectCameras()
			_, _ = detector.DetectCaptureCards()
			_, _ = detector.DetectAll()
		})
	})
}

// Benchmark tests
func BenchmarkWindowsDetector_DetectGPUs(b *testing.B) {
	detector := NewWindowsDetector("windows", "amd64")
	for i := 0; i < b.N; i++ {
		_, _ = detector.DetectGPUs()
	}
}

func BenchmarkWindowsDetector_DetectTPUs(b *testing.B) {
	detector := NewWindowsDetector("windows", "amd64")
	for i := 0; i < b.N; i++ {
		_, _ = detector.DetectTPUs()
	}
}

func BenchmarkWindowsDetector_DetectCameras(b *testing.B) {
	detector := NewWindowsDetector("windows", "amd64")
	for i := 0; i < b.N; i++ {
		_, _ = detector.DetectCameras()
	}
}

func BenchmarkWindowsDetector_DetectAll(b *testing.B) {
	detector := NewWindowsDetector("windows", "amd64")
	for i := 0; i < b.N; i++ {
		_, _ = detector.DetectAll()
	}
}
