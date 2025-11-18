package persistence

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

type JobStatus string

const (
	StatusPending  JobStatus = "pending"
	StatusRunning  JobStatus = "running"
	StatusComplete JobStatus = "complete"
	StatusFailed   JobStatus = "failed"
	StatusSkipped  JobStatus = "skipped"
)

type Job struct {
	ID        string    `json:"id"`
	FilePath  string    `json:"file_path"`
	Status    JobStatus `json:"status"`
	CreatedAt string    `json:"created_at"`
	UpdatedAt string    `json:"updated_at"`
}

func SaveJob(job *Job, jobsDir string) error {
	if err := os.MkdirAll(jobsDir, 0755); err != nil {
		return fmt.Errorf("failed to create jobs directory: %w", err)
	}

	path := filepath.Join(jobsDir, job.ID+".json")
	data, err := json.MarshalIndent(job, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal job: %w", err)
	}

	if err := os.WriteFile(path, data, 0644); err != nil {
		return fmt.Errorf("failed to write job file: %w", err)
	}

	return nil
}

func LoadJobs(jobsDir string) ([]*Job, error) {
	files, err := os.ReadDir(jobsDir)
	if err != nil {
		return nil, fmt.Errorf("failed to read jobs directory: %w", err)
	}

	var jobs []*Job
	for _, file := range files {
		if filepath.Ext(file.Name()) != ".json" {
			continue
		}

		path := filepath.Join(jobsDir, file.Name())
		data, err := os.ReadFile(path)
		if err != nil {
			continue // Skip corrupted files
		}

		var job Job
		if err := json.Unmarshal(data, &job); err != nil {
			continue // Skip invalid JSON
		}

		jobs = append(jobs, &job)
	}

	return jobs, nil
}

