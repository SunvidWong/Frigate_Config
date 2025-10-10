//go:build windows

package main

import (
	"github.com/frigate-config-tool/agent/internal/detect"
	"github.com/frigate-config-tool/agent/internal/schema"
)

func platformDetectHardware(platform, arch string) ([]schema.HardwareDevice, error) {
	detector := detect.NewWindowsDetector(platform, arch)
	return detect.DetectAll(detector)
}
