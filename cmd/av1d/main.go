package main

import (
	"flag"
	"log"
	"os"
	"os/signal"
	"path/filepath"
	"strings"
	"sync"
	"syscall"
	"time"

	"github.com/IONIQ6000/av1-top/internal/config"
	"github.com/IONIQ6000/av1-top/internal/ffmpeg"
	"github.com/IONIQ6000/av1-top/internal/scanner"
	"github.com/IONIQ6000/av1-top/internal/transcode"
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
		cfg.MaxScanDepth,
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
	
	// Setup job tracking
	jobsDir := "/var/lib/av1janitor/jobs"
	if err := os.MkdirAll(jobsDir, 0755); err != nil {
		log.Printf("Warning: Failed to create jobs directory: %v", err)
		// Try fallback
		home, _ := os.UserHomeDir()
		jobsDir = filepath.Join(home, ".local/share/av1janitor/jobs")
		os.MkdirAll(jobsDir, 0755)
	}
	
	// Track processed files
	processed := make(map[string]bool)
	var mu sync.Mutex
	
	// Semaphore for concurrency control
	sem := make(chan struct{}, *concurrent)
	
	// Process initial files if not dry run
	if !*dryRun && len(files) > 0 {
		log.Printf("Processing %d files with %d concurrent workers...", len(files), *concurrent)
		
		for _, file := range files {
			// Check if already processed or being processed
			mu.Lock()
			if processed[file] {
				mu.Unlock()
				continue
			}
			processed[file] = true
			mu.Unlock()
			
			// Acquire semaphore
			sem <- struct{}{}
			
			go func(f string) {
				defer func() { <-sem }() // Release semaphore
				
				log.Printf("Starting transcode: %s", f)
				err := transcode.Transcode(
					fm.Path(),
					strings.Replace(fm.Path(), "ffmpeg", "ffprobe", 1),
					f,
					jobsDir,
					cfg.SizeGateFactor,
				)
				
				if err != nil {
					log.Printf("Transcode failed for %s: %v", f, err)
				} else {
					log.Printf("Transcode completed: %s", f)
				}
			}(file)
		}
	}
	
	// Periodic scanning loop
	ticker := time.NewTicker(time.Duration(cfg.ScanIntervalSec) * time.Second)
	defer ticker.Stop()
	
	for {
		select {
		case <-sigChan:
			log.Println("Shutdown requested, waiting for active jobs...")
			
			// Wait for all jobs to complete
			for i := 0; i < *concurrent; i++ {
				sem <- struct{}{}
			}
			
			log.Println("All jobs completed, exiting gracefully...")
			return
			
		case <-ticker.C:
			if *dryRun {
				continue
			}
			
			// Periodic scan
			if *verbose {
				log.Println("Periodic directory scan...")
			}
			
			newFiles, err := scanner.ScanDirectories(
				cfg.WatchedDirectories,
				cfg.MediaExtensions,
				cfg.MinFileSizeBytes,
				cfg.MaxScanDepth,
			)
			
			if err != nil {
				log.Printf("Error scanning: %v", err)
				continue
			}
			
			// Process any new files
			for _, file := range newFiles {
				mu.Lock()
				if processed[file] {
					mu.Unlock()
					continue
				}
				processed[file] = true
				mu.Unlock()
				
				// Acquire semaphore
				sem <- struct{}{}
				
				go func(f string) {
					defer func() { <-sem }()
					
					log.Printf("Starting transcode: %s", f)
					err := transcode.Transcode(
						fm.Path(),
						strings.Replace(fm.Path(), "ffmpeg", "ffprobe", 1),
						f,
						jobsDir,
						cfg.SizeGateFactor,
					)
					
					if err != nil {
						log.Printf("Transcode failed for %s: %v", f, err)
					} else {
						log.Printf("Transcode completed: %s", f)
					}
				}(file)
			}
			
			if *verbose && len(newFiles) > 0 {
				log.Printf("Found %d media files", len(newFiles))
			}
		}
	}
}

