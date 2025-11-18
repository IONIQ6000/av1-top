package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/IONIQ6000/av1-top/internal/config"
	"github.com/IONIQ6000/av1-top/internal/ffmpeg"
	"github.com/IONIQ6000/av1-top/internal/persistence"
)

func main() {
	var (
		configPath   = flag.String("config", "", "Path to configuration file")
		concurrent   = flag.Int("concurrent", 1, "Number of concurrent transcodes")
		once         = flag.Bool("once", false, "Run once and exit")
		dryRun       = flag.Bool("dry-run", false, "Dry run mode (analyze but don't transcode)")
		verbose      = flag.Bool("verbose", false, "Verbose logging")
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
	log.Println("Starting continuous daemon mode...")

	// TODO: Implement daemon loop
	// This will watch directories, process files, and manage transcoding jobs

	// Wait for shutdown signal
	<-sigChan
	log.Println("Shutdown requested, exiting gracefully...")
}

