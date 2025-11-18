package metadata

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"strings"
)

// VideoStream represents a video stream in a media file
type VideoStream struct {
	Index      int
	CodecName  string
	Width      int
	Height     int
	BitDepth   int
	PixelFormat string
	FrameRate  string
	Disposition map[string]int
}

// AudioStream represents an audio stream
type AudioStream struct {
	Index       int
	CodecName   string
	Language    string
	Disposition map[string]int
}

// SubtitleStream represents a subtitle stream
type SubtitleStream struct {
	Index       int
	CodecName   string
	Language    string
	Disposition map[string]int
}

// FileMetadata contains all metadata about a media file
type FileMetadata struct {
	FilePath        string
	Size            int64
	VideoStreams    []VideoStream
	AudioStreams    []AudioStream
	SubtitleStreams []SubtitleStream
}

// Probe runs ffprobe on a file and returns metadata
func Probe(ffprobePath, filePath string) (*FileMetadata, error) {
	// Get file size
	info, err := os.Stat(filePath)
	if err != nil {
		return nil, fmt.Errorf("failed to stat file: %w", err)
	}

	// Run ffprobe
	cmd := exec.Command(ffprobePath,
		"-v", "quiet",
		"-print_format", "json",
		"-show_streams",
		"-show_format",
		filePath,
	)

	output, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("ffprobe failed: %w", err)
	}

	// Parse JSON output
	var result struct {
		Streams []struct {
			Index          int               `json:"index"`
			CodecType      string            `json:"codec_type"`
			CodecName      string            `json:"codec_name"`
			Width          int               `json:"width"`
			Height         int               `json:"height"`
			PixFmt         string            `json:"pix_fmt"`
			RFrameRate     string            `json:"r_frame_rate"`
			BitsPerRawSample string          `json:"bits_per_raw_sample"`
			Disposition    map[string]int    `json:"disposition"`
			Tags           map[string]string `json:"tags"`
		} `json:"streams"`
	}

	if err := json.Unmarshal(output, &result); err != nil {
		return nil, fmt.Errorf("failed to parse ffprobe output: %w", err)
	}

	metadata := &FileMetadata{
		FilePath: filePath,
		Size:     info.Size(),
	}

	// Parse streams
	for _, stream := range result.Streams {
		switch stream.CodecType {
		case "video":
			bitDepth := 8
			if stream.BitsPerRawSample != "" {
				if bd, err := strconv.Atoi(stream.BitsPerRawSample); err == nil {
					bitDepth = bd
				}
			}

			metadata.VideoStreams = append(metadata.VideoStreams, VideoStream{
				Index:       stream.Index,
				CodecName:   stream.CodecName,
				Width:       stream.Width,
				Height:      stream.Height,
				BitDepth:    bitDepth,
				PixelFormat: stream.PixFmt,
				FrameRate:   stream.RFrameRate,
				Disposition: stream.Disposition,
			})

		case "audio":
			lang := stream.Tags["language"]
			metadata.AudioStreams = append(metadata.AudioStreams, AudioStream{
				Index:       stream.Index,
				CodecName:   stream.CodecName,
				Language:    lang,
				Disposition: stream.Disposition,
			})

		case "subtitle":
			lang := stream.Tags["language"]
			metadata.SubtitleStreams = append(metadata.SubtitleStreams, SubtitleStream{
				Index:       stream.Index,
				CodecName:   stream.CodecName,
				Language:    lang,
				Disposition: stream.Disposition,
			})
		}
	}

	return metadata, nil
}

// DefaultVideoStream returns the default or first video stream
func (m *FileMetadata) DefaultVideoStream() *VideoStream {
	// Look for stream with default disposition
	for _, vs := range m.VideoStreams {
		if vs.Disposition["default"] == 1 {
			return &vs
		}
	}

	// Return first video stream if available
	if len(m.VideoStreams) > 0 {
		return &m.VideoStreams[0]
	}

	return nil
}

// IsWebRipLike checks if the file looks like a web rip (needs special handling)
func (m *FileMetadata) IsWebRipLike() bool {
	// Check if filename contains common web rip indicators
	lower := strings.ToLower(m.FilePath)
	webRipIndicators := []string{"webrip", "web-rip", "web.rip", "webdl", "web-dl", "web.dl"}
	
	for _, indicator := range webRipIndicators {
		if strings.Contains(lower, indicator) {
			return true
		}
	}

	return false
}

