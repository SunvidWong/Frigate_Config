//go:build darwin

package detect

import (
	"testing"

	"github.com/frigate-config-tool/agent/internal/schema"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestDarwinDetector_DetectGPUs tests GPU detection on macOS
func TestDarwinDetector_DetectGPUs(t *testing.T) {
	tests := []struct {
		name     string
		arch     string
		setup    func() (cleanup func())
		wantErr  bool
		validate func(t *testing.T, devices []schema.HardwareDevice)
	}{
		{
			name:    "detect_gpus_no_error",
			arch:    "amd64",
			setup:   func() func() { return func() {} },
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				// GPU detection should work even if system_profiler fails
				assert.NotNil(t, devices)

				// Validate each detected GPU
				for _, dev := range devices {
					assert.Equal(t, "gpu", dev.Type)
					assert.NotEmpty(t, dev.Name)
					assert.NotEmpty(t, dev.Capabilities)
					assert.Equal(t, "darwin", dev.Platform)
					assert.Contains(t, []string{"system_profiler", "gpu_registry"}, dev.DetectionSource)
					assert.Contains(t, dev.Capabilities, "metal") // All macOS GPUs support Metal
				}
			},
		},
		{
			name:    "detect_gpus_arm64",
			arch:    "arm64",
			setup:   func() func() { return func() {} },
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				assert.NotNil(t, devices)

				// Apple Silicon should be detected
				for _, dev := range devices {
					if dev.Architecture == "arm64" {
						assert.Equal(t, "darwin", dev.Platform)
						assert.Equal(t, "arm64", dev.Architecture)
					}
				}
			},
		},
		{
			name:    "no_duplicate_gpu_ids",
			arch:    "amd64",
			setup:   func() func() { return func() {} },
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
			detector := NewDarwinDetector("darwin", tt.arch)
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

// TestDarwinDetector_DetectTPUs tests TPU detection on macOS
func TestDarwinDetector_DetectTPUs(t *testing.T) {
	detector := NewDarwinDetector("darwin", "arm64")

	tests := []struct {
		name     string
		wantErr  bool
		validate func(t *testing.T, devices []schema.HardwareDevice)
	}{
		{
			name:    "detect_tpus_no_error",
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				// TPU detection should not fail
				assert.NotNil(t, devices)

				// Validate detected TPUs
				for _, dev := range devices {
					assert.Equal(t, "tpu", dev.Type)
					assert.NotEmpty(t, dev.Name)
					assert.Equal(t, "darwin", dev.Platform)
					assert.NotEmpty(t, dev.Capabilities)
				}
			},
		},
		{
			name:    "detect_neural_engine_arm64",
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				// On Apple Silicon, we might detect Neural Engine
				for _, dev := range devices {
					if dev.Name == "Apple Neural Engine" {
						assert.Contains(t, dev.Capabilities, "neural_engine")
						assert.Contains(t, dev.Capabilities, "coreml")
						assert.Equal(t, "system_check", dev.DetectionSource)
					}
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

// TestDarwinDetector_DetectCameras tests camera detection on macOS
func TestDarwinDetector_DetectCameras(t *testing.T) {
	detector := NewDarwinDetector("darwin", "amd64")

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
					assert.Equal(t, "darwin", dev.Platform)
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

// TestDarwinDetector_GPUCapabilityDetection tests GPU capability detection
func TestDarwinDetector_GPUCapabilityDetection(t *testing.T) {
	detector := NewDarwinDetector("darwin", "arm64")

	tests := []struct {
		name              string
		gpuName           string
		expectedCapabilities []string
	}{
		{
			name:              "apple_m1",
			gpuName:           "Apple M1",
			expectedCapabilities: []string{"metal", "videotoolbox", "h264_decode", "h264_encode", "hevc_decode", "hevc_encode"},
		},
		{
			name:              "apple_m2_pro",
			gpuName:           "Apple M2 Pro",
			expectedCapabilities: []string{"metal", "videotoolbox", "h264_decode", "hevc_encode"},
		},
		{
			name:              "apple_m3_max",
			gpuName:           "Apple M3 Max",
			expectedCapabilities: []string{"metal", "videotoolbox", "h264_decode"},
		},
		{
			name:              "intel_iris",
			gpuName:           "Intel Iris Plus Graphics",
			expectedCapabilities: []string{"metal", "videotoolbox", "h264_decode"},
		},
		{
			name:              "amd_radeon",
			gpuName:           "AMD Radeon Pro 5500M",
			expectedCapabilities: []string{"metal", "h264_decode", "h264_encode"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			capabilities := detector.getGPUCapabilities(tt.gpuName)

			// Check that expected capabilities are present
			for _, expected := range tt.expectedCapabilities {
				assert.Contains(t, capabilities, expected,
					"GPU %s should have capability: %s", tt.gpuName, expected)
			}
		})
	}
}

// TestDarwinDetector_CameraCapabilityDetection tests camera capability detection
func TestDarwinDetector_CameraCapabilityDetection(t *testing.T) {
	detector := NewDarwinDetector("darwin", "amd64")

	tests := []struct {
		name              string
		cameraName        string
		expectedCapabilities []string
	}{
		{
			name:              "facetime_hd",
			cameraName:        "FaceTime HD Camera",
			expectedCapabilities: []string{"capture", "h264", "face_time"},
		},
		{
			name:              "facetime_1080p",
			cameraName:        "FaceTime HD Camera (Built-in) 1080p",
			expectedCapabilities: []string{"capture", "h264", "face_time", "high_resolution"},
		},
		{
			name:              "continuity_camera",
			cameraName:        "iPhone Wide Camera",
			expectedCapabilities: []string{"capture", "wide_angle"},
		},
		{
			name:              "external_4k",
			cameraName:        "USB Camera 4K",
			expectedCapabilities: []string{"capture", "high_resolution"},
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

// TestDarwinDetector_DeviceIDGeneration tests device ID generation
func TestDarwinDetector_DeviceIDGeneration(t *testing.T) {
	tests := []struct {
		name       string
		deviceType string
		deviceName string
		expected   string
	}{
		{
			name:       "apple_gpu",
			deviceType: "gpu",
			deviceName: "Apple M1 Pro",
			expected:   "gpu-apple-m1-pro",
		},
		{
			name:       "facetime_camera",
			deviceType: "camera",
			deviceName: "FaceTime HD Camera",
			expected:   "camera-facetime-hd-camera",
		},
		{
			name:       "neural_engine",
			deviceType: "tpu",
			deviceName: "Apple Neural Engine",
			expected:   "tpu-apple-neural-engine",
		},
		{
			name:       "special_characters",
			deviceType: "camera",
			deviceName: "USB Camera (1080p)",
			expected:   "camera-usb-camera-1080p",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := generateDarwinDeviceID(tt.deviceType, tt.deviceName)
			assert.Equal(t, tt.expected, got)
		})
	}
}

// TestDarwinDetector_PlatformAndArchitecture tests platform and architecture reporting
func TestDarwinDetector_PlatformAndArchitecture(t *testing.T) {
	tests := []struct {
		name     string
		platform string
		arch     string
	}{
		{
			name:     "intel_mac",
			platform: "darwin",
			arch:     "amd64",
		},
		{
			name:     "apple_silicon",
			platform: "darwin",
			arch:     "arm64",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			detector := NewDarwinDetector(tt.platform, tt.arch)
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

// TestDarwinDetector_ErrorHandling tests graceful error handling
func TestDarwinDetector_ErrorHandling(t *testing.T) {
	detector := NewDarwinDetector("darwin", "amd64")

	t.Run("missing_tools_graceful", func(t *testing.T) {
		// Even if system_profiler, ioreg fail,
		// detection should not panic and should return empty results
		assert.NotPanics(t, func() {
			_, _ = detector.DetectGPUs()
			_, _ = detector.DetectTPUs()
			_, _ = detector.DetectCameras()
			_, _ = detector.DetectCaptureCards()
		})
	})
}

// TestDarwinDetector_AppleSiliconFeatures tests Apple Silicon specific features
func TestDarwinDetector_AppleSiliconFeatures(t *testing.T) {
	detector := NewDarwinDetector("darwin", "arm64")

	t.Run("neural_engine_detection", func(t *testing.T) {
		// Neural Engine should be detectable on Apple Silicon
		hasNE := detector.hasNeuralEngine()
		// This will depend on the macOS version and hardware
		t.Logf("Neural Engine available: %v", hasNE)
	})

	t.Run("av1_support_detection", func(t *testing.T) {
		hasAV1 := detector.hasAV1Support()
		t.Logf("AV1 support available: %v", hasAV1)
	})

	t.Run("prores_support_detection", func(t *testing.T) {
		hasProRes := detector.hasProResSupport()
		t.Logf("ProRes support available: %v", hasProRes)
	})
}

// TestDarwinDetector_IntelFeatures tests Intel-specific features
func TestDarwinDetector_IntelFeatures(t *testing.T) {
	detector := NewDarwinDetector("darwin", "amd64")

	t.Run("quicksync_detection", func(t *testing.T) {
		hasQS := detector.hasQuickSync()
		t.Logf("Intel Quick Sync available: %v", hasQS)
	})
}

// TestDarwinDetector_AMDFeatures tests AMD-specific features
func TestDarwinDetector_AMDFeatures(t *testing.T) {
	detector := NewDarwinDetector("darwin", "amd64")

	t.Run("vce_detection", func(t *testing.T) {
		hasVCE := detector.hasVCE()
		t.Logf("AMD VCE available: %v", hasVCE)
	})
}

// Test comprehensive hardware detection
func TestDarwinDetector_ComprehensiveDetection(t *testing.T) {
	tests := []struct {
		name string
		arch string
	}{
		{
			name: "intel_mac_detection",
			arch: "amd64",
		},
		{
			name: "apple_silicon_detection",
			arch: "arm64",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			detector := NewDarwinDetector("darwin", tt.arch)

			// Test all detection methods
			gpus, err := detector.DetectGPUs()
			assert.NoError(t, err)
			t.Logf("%s - GPUs detected: %d", tt.name, len(gpus))

			tpus, err := detector.DetectTPUs()
			assert.NoError(t, err)
			t.Logf("%s - TPUs detected: %d", tt.name, len(tpus))

			cameras, err := detector.DetectCameras()
			assert.NoError(t, err)
			t.Logf("%s - Cameras detected: %d", tt.name, len(cameras))

			captureCards, err := detector.DetectCaptureCards()
			assert.NoError(t, err)
			t.Logf("%s - Capture cards detected: %d", tt.name, len(captureCards))
		})
	}
}

// Benchmark tests
func BenchmarkDarwinDetector_DetectGPUs(b *testing.B) {
	detector := NewDarwinDetector("darwin", "arm64")
	for i := 0; i < b.N; i++ {
		_, _ = detector.DetectGPUs()
	}
}

func BenchmarkDarwinDetector_DetectTPUs(b *testing.B) {
	detector := NewDarwinDetector("darwin", "arm64")
	for i := 0; i < b.N; i++ {
		_, _ = detector.DetectTPUs()
	}
}

func BenchmarkDarwinDetector_DetectCameras(b *testing.B) {
	detector := NewDarwinDetector("darwin", "arm64")
	for i := 0; i < b.N; i++ {
		_, _ = detector.DetectCameras()
	}
}

func BenchmarkDarwinDetector_DetectCaptureCards(b *testing.B) {
	detector := NewDarwinDetector("darwin", "arm64")
	for i := 0; i < b.N; i++ {
		_, _ = detector.DetectCaptureCards()
	}
}
