package schema

import (
	"encoding/json"
	"time"
)

// HardwareDevice represents a detected hardware accelerator or video device
type HardwareDevice struct {
	ID              string   `json:"id"`
	Type            string   `json:"type"` // "gpu", "tpu", "camera", "capture_card"
	Name            string   `json:"name"`
	DevicePath      string   `json:"device_path"`
	Capabilities    []string `json:"capabilities"`
	Driver          *string  `json:"driver,omitempty"`
	Platform        string   `json:"platform"`        // "linux", "windows", "darwin"
	Architecture    string   `json:"architecture"`    // "x86_64", "arm64", "arm32"
	VendorID        *string  `json:"vendor_id,omitempty"`
	DetectionSource string   `json:"detection_source"`
	Available       bool                   `json:"available"`
	InUse           bool                   `json:"in_use"`
	Error           *string                `json:"error,omitempty"`
	Metadata        map[string]interface{} `json:"metadata,omitempty"`
}

// DetectionResult is the top-level JSON response from the agent
type DetectionResult struct {
	Devices      []HardwareDevice `json:"devices"`
	Platform     string           `json:"platform"`
	Architecture string           `json:"architecture"`
	DetectedAt   string           `json:"detected_at"` // ISO 8601 timestamp
}

// NewDetectionResult creates a new detection result with current timestamp
func NewDetectionResult(platform, arch string) *DetectionResult {
	return &DetectionResult{
		Devices:      make([]HardwareDevice, 0),
		Platform:     platform,
		Architecture: arch,
		DetectedAt:   time.Now().UTC().Format(time.RFC3339),
	}
}

// AddDevice adds a hardware device to the result
func (dr *DetectionResult) AddDevice(device HardwareDevice) {
	dr.Devices = append(dr.Devices, device)
}

// ToJSON converts the detection result to JSON string
func (dr *DetectionResult) ToJSON() (string, error) {
	bytes, err := json.MarshalIndent(dr, "", "  ")
	if err != nil {
		return "", err
	}
	return string(bytes), nil
}

// DeviceTypeGPU represents a GPU device
const DeviceTypeGPU = "gpu"

// DeviceTypeTPU represents a TPU device
const DeviceTypeTPU = "tpu"

// DeviceTypeCamera represents a camera device
const DeviceTypeCamera = "camera"

// DeviceTypeCaptureCard represents a capture card
const DeviceTypeCaptureCard = "capture_card"
