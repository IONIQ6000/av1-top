package transcode

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"github.com/IONIQ6000/av1-top/internal/metadata"
	"github.com/IONIQ6000/av1-top/internal/persistence"
)

// Quality presets based on resolution
const (
	QualityBelow1080p = 25
	Quality1080p      = 24
	Quality1440pPlus  = 23
)

// ChooseQuality selects encoding quality based on video height
func ChooseQuality(height int) int {
	if height >= 1440 {
		return Quality1440pPlus
	} else if height >= 1080 {
		return Quality1080p
	}
	return QualityBelow1080p
}

// ChooseSurface selects pixel format based on bit depth
func ChooseSurface(bitDepth int) string {
	if bitDepth > 8 {
		return "p010le" // 10-bit
	}
	return "nv12" // 8-bit
}

// BuildFFmpegCommand creates the FFmpeg command for transcoding
func BuildFFmpegCommand(ffmpegPath string, meta *metadata.FileMetadata, outputPath string, quality int) []string {
	args := []string{
		"-y", // Overwrite output
		"-v", "verbose",
		"-stats",
		"-benchmark",
		"-benchmark_all",
		"-hwaccel", "none", // Don't use hwaccel for input
	}

	// Initialize VAAPI device
	renderDevice := findRenderDevice()
	if renderDevice != "" {
		// Try using device path directly in init_hw_device
		// Format: vaapi=name:/dev/dri/renderD128
		args = append(args, "-init_hw_device", fmt.Sprintf("vaapi=va:%s", renderDevice))
		args = append(args, "-filter_hw_device", "va")
	} else {
		// Fallback to QSV
		args = append(args, "-init_hw_device", "qsv=hw")
		args = append(args, "-filter_hw_device", "hw")
	}

	// Input file
	args = append(args, "-analyzeduration", "50M", "-probesize", "50M")
	
	if meta.IsWebRipLike() {
		args = append(args, "-fflags", "+genpts", "-copyts", "-start_at_zero")
	}
	
	args = append(args, "-i", meta.FilePath)

	// Stream mapping
	args = append(args,
		"-map", "0",      // Map all streams
		"-map", "-0:v",   // Remove all video
		"-map", "-0:t",   // Remove text tracks
	)

	// Map the default video stream
	vs := meta.DefaultVideoStream()
	if vs != nil {
		args = append(args, "-map", fmt.Sprintf("0:v:%d", vs.Index))
	}

	// Map audio (excluding Russian)
	args = append(args,
		"-map", "0:a?",
		"-map", "-0:a:m:language:rus",
		"-map", "-0:a:m:language:ru",
	)

	// Map subtitles (excluding Russian)
	args = append(args,
		"-map", "0:s?",
		"-map", "-0:s:m:language:rus",
		"-map", "-0:s:m:language:ru",
	)

	// Map chapters
	args = append(args, "-map_chapters", "0")

	// WebRip sync flags
	if meta.IsWebRipLike() {
		args = append(args, "-vsync", "0", "-avoid_negative_ts", "make_zero")
	}

	// Video filter and encoding
	useVAAPI := renderDevice != ""
	
	if useVAAPI {
		// VAAPI encoding
		args = append(args,
			"-vf:v:0", "pad=ceil(iw/2)*2:ceil(ih/2)*2,setsar=1,format=nv12,hwupload",
			"-c:v:0", "av1_vaapi",
		)
		
		// Map quality to compression level
		compressionLevel := "5" // Default
		switch quality {
		case 23:
			compressionLevel = "4"
		case 24:
			compressionLevel = "5"
		case 25:
			compressionLevel = "6"
		}
		args = append(args, "-compression_level:v:0", compressionLevel)
	} else {
		// QSV encoding (fallback)
		surface := "nv12"
		if vs != nil {
			surface = ChooseSurface(vs.BitDepth)
		}
		
		args = append(args,
			"-vf:v:0", fmt.Sprintf("pad=ceil(iw/2)*2:ceil(ih/2)*2,setsar=1,format=%s,hwupload=extra_hw_frames=64", surface),
			"-c:v:0", "av1_qsv",
			"-global_quality:v:0", fmt.Sprintf("%d", quality),
			"-preset:v:0", "medium",
			"-look_ahead", "1",
		)
	}

	// Copy audio and subtitles
	args = append(args,
		"-c:a", "copy",
		"-c:s", "copy",
		"-max_muxing_queue_size", "2048",
		"-map_metadata", "0",
		"-f", "matroska",
		"-movflags", "+faststart",
		outputPath,
	)

	return args
}

// findRenderDevice finds the first available DRM render device
func findRenderDevice() string {
	entries, err := os.ReadDir("/dev/dri")
	if err != nil {
		return ""
	}

	for _, entry := range entries {
		if strings.HasPrefix(entry.Name(), "renderD") {
			return fmt.Sprintf("/dev/dri/%s", entry.Name())
		}
	}

	return ""
}

// Transcode executes the transcoding process
func Transcode(ffmpegPath, ffprobePath string, filePath string, jobsDir string, sizeGateFactor float64) error {
	// Create job
	job := &persistence.Job{
		ID:        fmt.Sprintf("%d", time.Now().UnixNano()),
		FilePath:  filePath,
		Status:    persistence.StatusPending,
		CreatedAt: time.Now().Format(time.RFC3339),
		UpdatedAt: time.Now().Format(time.RFC3339),
	}

	// Save initial job
	if err := persistence.SaveJob(job, jobsDir); err != nil {
		return fmt.Errorf("failed to save job: %w", err)
	}

	// Update to running
	job.Status = persistence.StatusRunning
	job.UpdatedAt = time.Now().Format(time.RFC3339)
	persistence.SaveJob(job, jobsDir)

	// Get metadata
	meta, err := metadata.Probe(ffprobePath, filePath)
	if err != nil {
		job.Status = persistence.StatusFailed
		job.UpdatedAt = time.Now().Format(time.RFC3339)
		persistence.SaveJob(job, jobsDir)
		return fmt.Errorf("failed to probe file: %w", err)
	}

	// Check for video stream
	vs := meta.DefaultVideoStream()
	if vs == nil {
		job.Status = persistence.StatusSkipped
		job.UpdatedAt = time.Now().Format(time.RFC3339)
		persistence.SaveJob(job, jobsDir)
		return fmt.Errorf("no video stream found")
	}

	// Build output path
	dir := filepath.Dir(filePath)
	base := strings.TrimSuffix(filepath.Base(filePath), filepath.Ext(filePath))
	tempOutput := filepath.Join(dir, base+".av1-tmp.mkv")

	// Choose quality
	quality := ChooseQuality(vs.Height)

	// Build FFmpeg command
	args := BuildFFmpegCommand(ffmpegPath, meta, tempOutput, quality)

	// Execute FFmpeg
	cmd := exec.Command(ffmpegPath, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		job.Status = persistence.StatusFailed
		job.UpdatedAt = time.Now().Format(time.RFC3339)
		persistence.SaveJob(job, jobsDir)
		
		// Clean up temp file
		os.Remove(tempOutput)
		
		return fmt.Errorf("ffmpeg failed: %w", err)
	}

	// Check output size (size gate)
	outputInfo, err := os.Stat(tempOutput)
	if err != nil {
		job.Status = persistence.StatusFailed
		job.UpdatedAt = time.Now().Format(time.RFC3339)
		persistence.SaveJob(job, jobsDir)
		os.Remove(tempOutput)
		return fmt.Errorf("failed to stat output: %w", err)
	}

	originalSize := meta.Size
	outputSize := outputInfo.Size()
	ratio := float64(outputSize) / float64(originalSize)

	if ratio > sizeGateFactor {
		// Output too large, skip
		job.Status = persistence.StatusSkipped
		job.UpdatedAt = time.Now().Format(time.RFC3339)
		persistence.SaveJob(job, jobsDir)
		os.Remove(tempOutput)
		
		// Create .av1skip marker
		skipMarker := filepath.Join(dir, base+".av1skip")
		os.WriteFile(skipMarker, []byte(fmt.Sprintf("Output size %.1f%% of original (threshold: %.1f%%)", ratio*100, sizeGateFactor*100)), 0644)
		
		return fmt.Errorf("output too large: %.1f%% of original", ratio*100)
	}

	// Replace original with transcoded file
	finalOutput := filepath.Join(dir, base+".mkv")
	
	// Backup original
	backupPath := filePath + ".av1backup"
	if err := os.Rename(filePath, backupPath); err != nil {
		job.Status = persistence.StatusFailed
		job.UpdatedAt = time.Now().Format(time.RFC3339)
		persistence.SaveJob(job, jobsDir)
		os.Remove(tempOutput)
		return fmt.Errorf("failed to backup original: %w", err)
	}

	// Move temp to final
	if err := os.Rename(tempOutput, finalOutput); err != nil {
		// Restore backup
		os.Rename(backupPath, filePath)
		job.Status = persistence.StatusFailed
		job.UpdatedAt = time.Now().Format(time.RFC3339)
		persistence.SaveJob(job, jobsDir)
		os.Remove(tempOutput)
		return fmt.Errorf("failed to move output: %w", err)
	}

	// Remove backup
	os.Remove(backupPath)

	// Success!
	job.Status = persistence.StatusComplete
	job.UpdatedAt = time.Now().Format(time.RFC3339)
	persistence.SaveJob(job, jobsDir)

	return nil
}

