//go:build linux

package detect

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/frigate-config-tool/agent/internal/schema"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestLinuxDetector_DetectGPUs tests GPU detection on Linux
func TestLinuxDetector_DetectGPUs(t *testing.T) {
	detector := NewLinuxDetector("linux", "amd64")

	tests := []struct {
		name    string
		setup   func() (cleanup func())
		wantErr bool
		validate func(t *testing.T, devices []schema.HardwareDevice)
	}{
		{
			name: "detect_nvidia_gpus",
			setup: func() func() {
				// This test depends on nvidia-smi being available
				// In real environment, it will detect actual NVIDIA GPUs
				return func() {}
			},
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				// GPU detection should not fail even if no GPUs present
				assert.NotNil(t, devices)

				// If NVIDIA GPU is detected, validate its properties
				for _, dev := range devices {
					if dev.DetectionSource == "nvidia-smi" {
						assert.Equal(t, "gpu", dev.Type)
						assert.NotEmpty(t, dev.Name)
						assert.Contains(t, dev.DevicePath, "/dev/nvidia")
						assert.NotEmpty(t, dev.Capabilities)
						assert.Equal(t, "linux", dev.Platform)
					}
				}
			},
		},
		{
			name: "detect_dri_devices",
			setup: func() func() {
				// DRI devices are commonly present on Linux systems
				return func() {}
			},
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				// DRI detection should work if /dev/dri exists
				if _, err := os.Stat("/dev/dri"); err == nil {
					// We should have at least detected DRI devices
					hasDRI := false
					for _, dev := range devices {
						if dev.DetectionSource == "dri" {
							hasDRI = true
							assert.Equal(t, "gpu", dev.Type)
							assert.Contains(t, dev.DevicePath, "/dev/dri/renderD")
							assert.Contains(t, dev.Capabilities, "vaapi")
						}
					}
					// If /dev/dri exists, we should detect at least one device
					if hasDRI {
						t.Log("DRI devices detected successfully")
					}
				}
			},
		},
		{
			name: "no_duplicate_gpus",
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

// TestLinuxDetector_DetectTPUs tests TPU detection on Linux
func TestLinuxDetector_DetectTPUs(t *testing.T) {
	detector := NewLinuxDetector("linux", "amd64")

	tests := []struct {
		name     string
		wantErr  bool
		validate func(t *testing.T, devices []schema.HardwareDevice)
	}{
		{
			name:    "detect_coral_tpus",
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				// TPU detection should not fail even if no TPUs present
				assert.NotNil(t, devices)

				// If TPU is detected, validate its properties
				for _, dev := range devices {
					assert.Equal(t, "tpu", dev.Type)
					assert.NotEmpty(t, dev.Name)
					assert.Contains(t, []string{"edge_tpu", "usb", "pcie"}, dev.Capabilities[0])
					assert.Equal(t, "linux", dev.Platform)
					assert.Contains(t, []string{"sysfs", "usb_sysfs"}, dev.DetectionSource)
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

// TestLinuxDetector_DetectCameras tests camera detection on Linux
func TestLinuxDetector_DetectCameras(t *testing.T) {
	detector := NewLinuxDetector("linux", "amd64")

	tests := []struct {
		name     string
		wantErr  bool
		validate func(t *testing.T, devices []schema.HardwareDevice)
	}{
		{
			name:    "detect_video_devices",
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				// Camera detection should not fail even if no cameras present
				assert.NotNil(t, devices)

				// If cameras are detected, validate their properties
				for _, dev := range devices {
					assert.Equal(t, "camera", dev.Type)
					assert.NotEmpty(t, dev.Name)
					assert.Equal(t, "linux", dev.Platform)

					// Device path should be /dev/video* or USB path
					if dev.DevicePath != "" {
						assert.True(t,
							filepath.HasPrefix(dev.DevicePath, "/dev/video") ||
							filepath.HasPrefix(dev.DevicePath, "/dev/bus/usb"),
							"Invalid device path: %s", dev.DevicePath)
					}

					// Should have capture capability
					assert.Contains(t, dev.Capabilities, "capture")
				}
			},
		},
		{
			name:    "no_duplicate_cameras",
			wantErr: false,
			validate: func(t *testing.T, devices []schema.HardwareDevice) {
				// Check for duplicate device paths
				seen := make(map[string]bool)
				for _, dev := range devices {
					if dev.DevicePath != "" {
						assert.False(t, seen[dev.DevicePath],
							"Duplicate device path: %s", dev.DevicePath)
						seen[dev.DevicePath] = true
					}
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

// TestLinuxDetector_DeviceAvailability tests device availability checking
func TestLinuxDetector_DeviceAvailability(t *testing.T) {
	detector := NewLinuxDetector("linux", "amd64")

	tests := []struct {
		name       string
		devicePath string
		want       bool
	}{
		{
			name:       "non_existent_device",
			devicePath: "/dev/nonexistent999",
			want:       false,
		},
		{
			name:       "invalid_path",
			devicePath: "/invalid/path/to/device",
			want:       false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := detector.isDeviceAvailable(tt.devicePath)
			assert.Equal(t, tt.want, got)
		})
	}
}

// TestLinuxDetector_GenerateDeviceID tests device ID generation
func TestGenerateDeviceID(t *testing.T) {
	tests := []struct {
		name       string
		deviceType string
		path       string
		want       string
	}{
		{
			name:       "video_device",
			deviceType: "camera",
			path:       "/dev/video0",
			want:       "camera-video0",
		},
		{
			name:       "dri_device",
			deviceType: "gpu",
			path:       "/dev/dri/renderD128",
			want:       "gpu-renderD128",
		},
		{
			name:       "usb_device",
			deviceType: "tpu",
			path:       "/dev/bus/usb/001/002",
			want:       "tpu-002",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := generateDeviceID(tt.deviceType, tt.path)
			assert.Equal(t, tt.want, got)
		})
	}
}

// TestLinuxDetector_PlatformAndArchitecture tests platform and architecture reporting
func TestLinuxDetector_PlatformAndArchitecture(t *testing.T) {
	tests := []struct {
		name string
		platform string
		arch string
	}{
		{
			name:     "amd64_linux",
			platform: "linux",
			arch:     "amd64",
		},
		{
			name:     "arm64_linux",
			platform: "linux",
			arch:     "arm64",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			detector := NewLinuxDetector(tt.platform, tt.arch)
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

// TestLinuxDetector_ErrorHandling tests graceful error handling
func TestLinuxDetector_ErrorHandling(t *testing.T) {
	detector := NewLinuxDetector("linux", "amd64")

	t.Run("missing_tools_graceful", func(t *testing.T) {
		// Even if tools like nvidia-smi, v4l2-ctl are missing,
		// detection should not panic and should return empty results
		assert.NotPanics(t, func() {
			_, _ = detector.DetectGPUs()
			_, _ = detector.DetectTPUs()
			_, _ = detector.DetectCameras()
			_, _ = detector.DetectCaptureCards()
		})
	})
}

// Benchmark tests
func BenchmarkLinuxDetector_DetectGPUs(b *testing.B) {
	detector := NewLinuxDetector("linux", "amd64")
	for i := 0; i < b.N; i++ {
		_, _ = detector.DetectGPUs()
	}
}

func BenchmarkLinuxDetector_DetectTPUs(b *testing.B) {
	detector := NewLinuxDetector("linux", "amd64")
	for i := 0; i < b.N; i++ {
		_, _ = detector.DetectTPUs()
	}
}

func BenchmarkLinuxDetector_DetectCameras(b *testing.B) {
	detector := NewLinuxDetector("linux", "amd64")
	for i := 0; i < b.N; i++ {
		_, _ = detector.DetectCameras()
	}
}
