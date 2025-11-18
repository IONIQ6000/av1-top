package config

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/pelletier/go-toml/v2"
)

type Config struct {
	WatchedDirectories []string `toml:"watched_directories"`
	MinFileSizeBytes   uint64   `toml:"min_file_size_bytes"`
	SizeGateFactor     float64  `toml:"size_gate_factor"`
	MediaExtensions    []string `toml:"media_extensions"`
	ScanIntervalSec    uint64   `toml:"scan_interval_seconds"`
	MaxScanDepth       int      `toml:"max_scan_depth"` // -1 for unlimited, 0 for current dir only, 1 for one level deep
	QsvQuality         QsvQualitySettings `toml:"qsv_quality"`
}

type QsvQualitySettings struct {
	Below1080p        uint8 `toml:"below_1080p"`
	At1080p           uint8 `toml:"at_1080p"`
	At1440pAndAbove   uint8 `toml:"at_1440p_and_above"`
}

func Load(path string) (*Config, error) {
	// Default config path
	if path == "" {
		home, _ := os.UserHomeDir()
		path = filepath.Join(home, ".config", "av1janitor", "config.toml")
	}

	// Check if file exists
	if _, err := os.Stat(path); os.IsNotExist(err) {
		return Default(), nil
	}

	// Read and parse config
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read config: %w", err)
	}

	var cfg Config
	if err := toml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}

	// Resolve relative paths to absolute paths
	for i, dir := range cfg.WatchedDirectories {
		if !filepath.IsAbs(dir) {
			// Try to resolve relative path
			absPath, err := filepath.Abs(dir)
			if err == nil {
				cfg.WatchedDirectories[i] = absPath
			}
		}
	}

	// Set default max scan depth if not specified
	if cfg.MaxScanDepth == 0 && len(cfg.WatchedDirectories) > 0 {
		cfg.MaxScanDepth = 1 // Default to one level deep
	}

	// Validate
	if err := cfg.Validate(); err != nil {
		return nil, fmt.Errorf("config validation failed: %w", err)
	}

	return &cfg, nil
}

func Default() *Config {
	return &Config{
		WatchedDirectories: []string{"/media"},
		MinFileSizeBytes:    2147483648, // 2 GiB
		SizeGateFactor:      0.9,
		MediaExtensions:     []string{"mkv", "mp4", "avi"},
		ScanIntervalSec:     60,
		MaxScanDepth:        1, // Default: scan one level of subdirectories
		QsvQuality: QsvQualitySettings{
			Below1080p:      25,
			At1080p:         24,
			At1440pAndAbove: 23,
		},
	}
}

func (c *Config) Validate() error {
	if len(c.WatchedDirectories) == 0 {
		return fmt.Errorf("watched_directories cannot be empty")
	}

	if c.SizeGateFactor <= 0 || c.SizeGateFactor > 1 {
		return fmt.Errorf("size_gate_factor must be between 0 and 1")
	}

	if c.MinFileSizeBytes == 0 {
		return fmt.Errorf("min_file_size_bytes cannot be 0")
	}

	return nil
}

