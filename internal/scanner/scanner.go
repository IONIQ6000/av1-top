package scanner

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// ScanDirectories scans all watched directories for media files
// Returns a list of file paths that match the criteria
// maxDepth: -1 for unlimited, 0 for current directory only, 1 for one level deep, etc.
func ScanDirectories(watchedDirs []string, extensions []string, minSizeBytes uint64, maxDepth int) ([]string, error) {
	var files []string
	
	for _, rootDir := range watchedDirs {
		if _, err := os.Stat(rootDir); os.IsNotExist(err) {
			fmt.Printf("Warning: Directory does not exist: %s\n", rootDir)
			continue
		}
		
		fmt.Printf("Scanning directory: %s\n", rootDir)
		found, err := scanDirectoryRecursive(rootDir, extensions, minSizeBytes, maxDepth, 0)
		if err != nil {
			fmt.Printf("Error scanning %s: %v\n", rootDir, err)
			continue
		}
		
		files = append(files, found...)
		fmt.Printf("  Found %d media files in %s\n", len(found), rootDir)
	}
	
	return files, nil
}

// scanDirectoryRecursive recursively scans a directory for media files
// maxDepth: -1 for unlimited, 0 for current directory only, 1 for one level deep, etc.
// currentDepth: current recursion depth (starts at 0)
func scanDirectoryRecursive(dir string, extensions []string, minSizeBytes uint64, maxDepth int, currentDepth int) ([]string, error) {
	var files []string
	
	// Check depth limit
	if maxDepth >= 0 && currentDepth > maxDepth {
		return files, nil // Reached max depth
	}
	
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, fmt.Errorf("cannot read directory %s: %w", dir, err)
	}
	
	var subdirs []string
	filesFoundHere := 0
	
	// First pass: collect files from current directory and list subdirectories
	for _, entry := range entries {
		path := filepath.Join(dir, entry.Name())
		
		if entry.IsDir() {
			subdirs = append(subdirs, path)
		} else {
			// Check if it's a media file
			ext := strings.ToLower(strings.TrimPrefix(filepath.Ext(path), "."))
			for _, allowedExt := range extensions {
				if ext == strings.ToLower(allowedExt) {
					// Check file size
					info, err := entry.Info()
					if err != nil {
						continue
					}
					
					if info.Size() >= int64(minSizeBytes) {
						files = append(files, path)
						filesFoundHere++
					}
					break
				}
			}
		}
	}
	
	// Second pass: recurse into subdirectories if we haven't hit max depth
	if maxDepth < 0 || currentDepth < maxDepth {
		for _, subdir := range subdirs {
			subFiles, err := scanDirectoryRecursive(subdir, extensions, minSizeBytes, maxDepth, currentDepth+1)
			if err != nil {
				// Log error but continue with other directories
				fmt.Printf("Warning: Error scanning subdirectory %s: %v\n", subdir, err)
				continue
			}
			files = append(files, subFiles...)
		}
	}
	
	return files, nil
}

// hasMediaFilesAtTopLevel checks if a directory has any media files at its top level (non-recursive)
// This is used for smart depth limiting - we only recurse into subdirectories that have files
func hasMediaFilesAtTopLevel(dir string, extensions []string) bool {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return false // Can't read directory, skip it
	}
	
	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		
		ext := strings.ToLower(strings.TrimPrefix(filepath.Ext(entry.Name()), "."))
		for _, allowedExt := range extensions {
			if ext == strings.ToLower(allowedExt) {
				return true
			}
		}
	}
	
	return false
}

// IsMediaFile checks if a file path matches the media extensions
func IsMediaFile(path string, extensions []string) bool {
	ext := strings.ToLower(strings.TrimPrefix(filepath.Ext(path), "."))
	for _, allowedExt := range extensions {
		if ext == strings.ToLower(allowedExt) {
			return true
		}
	}
	return false
}

