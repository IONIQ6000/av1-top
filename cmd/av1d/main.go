package main

import (
	"flag"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/IONIQ6000/av1-top/internal/config"
	"github.com/IONIQ6000/av1-top/internal/ffmpeg"
	"github.com/IONIQ6000/av1-top/internal/scanner"
)

func main() {
	var (
		configPath = flag.String("config", "", "Path to configuration file")
		concurrent = flag.Int("concurrent", 1, "Number of concurrent transcodes")
		dryRun     = flag.Bool("dry-run", false, "Dry run mode (analyze but don't transcode)")
		verbose    = flag.Bool("verbose", false, "Verbose logging")
	)
	flag.Parse()

	// Load configuration
	cfg, err := config.Load(*configPath)
	if err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	// Validate FFmpeg
	fm, err := ffmpeg.NewManager()
	if err != nil {
		log.Fatalf("FFmpeg validation failed: %v", err)
	}
	log.Printf("✓ Found FFmpeg %s at %s", fm.Version(), fm.Path())

	// Setup signal handling
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt, syscall.SIGTERM)

	// Start daemon
	log.Println("=== AV1 Daemon ===")
	log.Println("Automated AV1 transcoding with Intel QSV")
	log.Println("Starting continuous daemon mode...")
	log.Printf("Concurrent jobs: %d", *concurrent)
	log.Printf("Dry run: %v", *dryRun)
	
	// Display configuration
	log.Println("Configuration:")
	log.Printf("  Watched directories:")
	for _, dir := range cfg.WatchedDirectories {
		log.Printf("    - %s", dir)
	}
	log.Printf("  Media extensions: %v", cfg.MediaExtensions)
	log.Printf("  Min file size: %.1f GiB", float64(cfg.MinFileSizeBytes)/(1024*1024*1024))
	log.Printf("  Size gate: %.0f%%", cfg.SizeGateFactor*100)
	
	// Perform initial directory scan
	log.Println("Performing initial directory scan...")
	files, err := scanner.ScanDirectories(
		cfg.WatchedDirectories,
		cfg.MediaExtensions,
		cfg.MinFileSizeBytes,
	)
	if err != nil {
		log.Printf("Error scanning directories: %v", err)
	} else {
		log.Printf("Found %d media files to process", len(files))
		if len(files) == 0 {
			log.Println("No media files found! Check:")
			log.Printf("  - Directory exists and is readable: %v", cfg.WatchedDirectories)
			log.Printf("  - File extensions match: %v", cfg.MediaExtensions)
			log.Printf("  - Files meet minimum size: %.1f GiB", float64(cfg.MinFileSizeBytes)/(1024*1024*1024))
		} else {
			log.Println("Sample files found:")
			for i, file := range files {
				if i >= 5 {
					break
				}
				info, err := os.Stat(file)
				if err == nil {
					sizeGB := float64(info.Size()) / (1024 * 1024 * 1024)
					log.Printf("  [%d] %s (%.2f GiB)", i+1, file, sizeGB)
				} else {
					log.Printf("  [%d] %s", i+1, file)
				}
			}
			if len(files) > 5 {
				log.Printf("  ... and %d more files", len(files)-5)
			}
		}
	}
	
	if *dryRun {
		log.Println("DRY RUN MODE - No actual transcoding will occur")
	}
	
	log.Println("Watching for new files (Ctrl+C to stop)...")
	
	// TODO: Implement file watching and transcoding
	// For now, just wait for shutdown signal
	// In the future, this will:
	// - Set up filesystem watcher (fsnotify)
	// - Process files with concurrency limit
	// - Create and manage jobs
	// - Execute FFmpeg transcoding
	
	// Simple loop that scans periodically
	ticker := time.NewTicker(time.Duration(cfg.ScanIntervalSec) * time.Second)
	defer ticker.Stop()
	
	for {
		select {
		case <-sigChan:
			log.Println("Shutdown requested, exiting gracefully...")
			return
		case <-ticker.C:
			// Periodic scan (will be replaced with filesystem watching)
			if *verbose {
				log.Println("Periodic directory scan...")
			}
			files, err := scanner.ScanDirectories(
				cfg.WatchedDirectories,
				cfg.MediaExtensions,
				cfg.MinFileSizeBytes,
			)
			if err == nil && *verbose {
				log.Printf("Found %d media files", len(files))
			}
		}
	}
}

