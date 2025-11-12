/// Side-by-Side Model Comparison System
///
/// This module implements the killer feature of DGX-Pixels - the ability to
/// compare multiple models (base SDXL vs custom LoRA) side-by-side to validate
/// training improvements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

// ============================================================================
// Core Data Structures
// ============================================================================

/// Configuration for a model to compare
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelConfig {
    /// Display name for the model
    pub name: String,

    /// Base model checkpoint
    pub base: String,

    /// Optional LoRA model
    pub lora: Option<String>,

    /// LoRA strength (0.0-1.0)
    pub lora_strength: f32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            name: "Unnamed Model".to_string(),
            base: "sd_xl_base_1.0.safetensors".to_string(),
            lora: None,
            lora_strength: 1.0,
        }
    }
}

/// Generation parameters for fair comparison
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenerationParams {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub seed: u64,
    pub width: u32,
    pub height: u32,
    pub steps: u32,
    pub cfg_scale: f32,
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            negative_prompt: None,
            seed: 42,
            width: 1024,
            height: 1024,
            steps: 30,
            cfg_scale: 7.5,
        }
    }
}

/// Comparison job tracking multiple model generations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonJob {
    /// Unique comparison ID
    pub comparison_id: String,

    /// Generation parameters (shared across all models)
    pub params: GenerationParams,

    /// Models being compared
    pub models: Vec<ModelConfig>,

    /// Job IDs for each model (parallel generation)
    pub job_ids: Vec<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Current status
    pub status: ComparisonStatus,
}

/// Status of a comparison job
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComparisonStatus {
    /// Jobs are being submitted
    Initializing,

    /// Waiting for generations to complete
    Running { completed: usize, total: usize },

    /// All generations completed
    Complete,

    /// One or more jobs failed
    Failed { reason: String },

    /// User cancelled
    Cancelled,
}

/// Result from a single model in comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResult {
    /// Model configuration
    pub model: ModelConfig,

    /// Job ID
    pub job_id: String,

    /// Generated image path
    pub image_path: Option<PathBuf>,

    /// Generation time in seconds
    pub duration_s: Option<f32>,

    /// Generation status
    pub status: ModelResultStatus,

    /// Error message if failed
    pub error: Option<String>,
}

/// Status of individual model result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelResultStatus {
    Pending,
    Generating { progress_percent: f32 },
    Complete,
    Failed,
    Cancelled,
}

/// Complete comparison result with all models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    /// Comparison ID
    pub comparison_id: String,

    /// Generation parameters
    pub params: GenerationParams,

    /// Results from each model
    pub results: Vec<ModelResult>,

    /// User preference (index of preferred model, if voted)
    pub user_preference: Option<usize>,

    /// Notes from user
    pub notes: Option<String>,

    /// Completion timestamp
    pub completed_at: DateTime<Utc>,
}

impl ComparisonResult {
    /// Get the winning model (if preference set)
    pub fn winner(&self) -> Option<&ModelConfig> {
        self.user_preference
            .and_then(|idx| self.results.get(idx))
            .map(|result| &result.model)
    }

    /// Check if all results succeeded
    pub fn all_succeeded(&self) -> bool {
        self.results.iter().all(|r| r.status == ModelResultStatus::Complete)
    }

    /// Get average generation time
    pub fn avg_generation_time(&self) -> Option<f32> {
        let times: Vec<f32> = self.results
            .iter()
            .filter_map(|r| r.duration_s)
            .collect();

        if times.is_empty() {
            None
        } else {
            Some(times.iter().sum::<f32>() / times.len() as f32)
        }
    }
}

// ============================================================================
// Comparison Manager
// ============================================================================

/// Manages multiple comparison jobs and tracks results
#[derive(Debug, Clone)]
pub struct ComparisonManager {
    /// Active comparison jobs
    active_comparisons: HashMap<String, ComparisonJob>,

    /// Completed comparisons
    completed_comparisons: Vec<ComparisonResult>,

    /// Model results indexed by job_id
    job_to_comparison: HashMap<String, String>,
}

impl Default for ComparisonManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ComparisonManager {
    /// Create a new comparison manager
    pub fn new() -> Self {
        Self {
            active_comparisons: HashMap::new(),
            completed_comparisons: Vec::new(),
            job_to_comparison: HashMap::new(),
        }
    }

    /// Create a new comparison job
    pub fn create_comparison(
        &mut self,
        params: GenerationParams,
        models: Vec<ModelConfig>,
    ) -> String {
        let comparison_id = format!("cmp-{}", uuid::Uuid::new_v4());

        let job = ComparisonJob {
            comparison_id: comparison_id.clone(),
            params,
            models,
            job_ids: Vec::new(),
            created_at: Utc::now(),
            status: ComparisonStatus::Initializing,
        };

        self.active_comparisons.insert(comparison_id.clone(), job);
        comparison_id
    }

    /// Register job IDs for a comparison
    pub fn register_jobs(&mut self, comparison_id: &str, job_ids: Vec<String>) {
        if let Some(job) = self.active_comparisons.get_mut(comparison_id) {
            job.job_ids = job_ids.clone();
            job.status = ComparisonStatus::Running {
                completed: 0,
                total: job_ids.len(),
            };

            // Map job IDs to comparison ID
            for job_id in job_ids {
                self.job_to_comparison.insert(job_id, comparison_id.to_string());
            }
        }
    }

    /// Update progress for a job
    pub fn update_job_progress(&mut self, job_id: &str, progress_percent: f32) {
        if let Some(comparison_id) = self.job_to_comparison.get(job_id) {
            // Progress tracking happens at the UI level
            // This is just for potential future use
            tracing::debug!(
                "Job {} progress: {:.1}% (comparison: {})",
                job_id,
                progress_percent,
                comparison_id
            );
        }
    }

    /// Mark a job as complete
    pub fn complete_job(
        &mut self,
        job_id: &str,
        image_path: PathBuf,
        duration_s: f32,
    ) {
        if let Some(comparison_id) = self.job_to_comparison.get(job_id).cloned() {
            if let Some(job) = self.active_comparisons.get_mut(&comparison_id) {
                if let ComparisonStatus::Running { completed, total } = &mut job.status {
                    *completed += 1;

                    if *completed >= *total {
                        job.status = ComparisonStatus::Complete;
                        self.finalize_comparison(&comparison_id);
                    }
                }
            }
        }
    }

    /// Mark a job as failed
    pub fn fail_job(&mut self, job_id: &str, error: String) {
        if let Some(comparison_id) = self.job_to_comparison.get(job_id).cloned() {
            if let Some(job) = self.active_comparisons.get_mut(&comparison_id) {
                job.status = ComparisonStatus::Failed { reason: error };
            }
        }
    }

    /// Finalize a comparison and move to completed
    fn finalize_comparison(&mut self, comparison_id: &str) {
        if let Some(job) = self.active_comparisons.remove(comparison_id) {
            let results: Vec<ModelResult> = job.models
                .iter()
                .zip(job.job_ids.iter())
                .map(|(model, job_id)| ModelResult {
                    model: model.clone(),
                    job_id: job_id.clone(),
                    image_path: None, // Will be filled by UI
                    duration_s: None, // Will be filled by UI
                    status: ModelResultStatus::Complete,
                    error: None,
                })
                .collect();

            let result = ComparisonResult {
                comparison_id: comparison_id.to_string(),
                params: job.params,
                results,
                user_preference: None,
                notes: None,
                completed_at: Utc::now(),
            };

            self.completed_comparisons.push(result);

            // Clean up job mappings
            for job_id in &job.job_ids {
                self.job_to_comparison.remove(job_id);
            }
        }
    }

    /// Get active comparison by ID
    pub fn get_active(&self, comparison_id: &str) -> Option<&ComparisonJob> {
        self.active_comparisons.get(comparison_id)
    }

    /// Get completed comparison by ID
    pub fn get_completed(&self, comparison_id: &str) -> Option<&ComparisonResult> {
        self.completed_comparisons
            .iter()
            .find(|c| c.comparison_id == comparison_id)
    }

    /// Get mutable completed comparison by ID
    pub fn get_completed_mut(&mut self, comparison_id: &str) -> Option<&mut ComparisonResult> {
        self.completed_comparisons
            .iter_mut()
            .find(|c| c.comparison_id == comparison_id)
    }

    /// Get all completed comparisons
    pub fn get_all_completed(&self) -> &[ComparisonResult] {
        &self.completed_comparisons
    }

    /// Set user preference for a comparison
    pub fn set_preference(
        &mut self,
        comparison_id: &str,
        model_index: usize,
        notes: Option<String>,
    ) -> bool {
        if let Some(result) = self.get_completed_mut(comparison_id) {
            if model_index < result.results.len() {
                result.user_preference = Some(model_index);
                if let Some(n) = notes {
                    result.notes = Some(n);
                }
                return true;
            }
        }
        false
    }

    /// Get statistics across all comparisons
    pub fn get_statistics(&self) -> ComparisonStatistics {
        let total_comparisons = self.completed_comparisons.len();
        let mut model_wins: HashMap<String, usize> = HashMap::new();
        let mut total_with_preference = 0;

        for result in &self.completed_comparisons {
            if let Some(winner) = result.winner() {
                let model_name = winner.name.clone();
                *model_wins.entry(model_name).or_insert(0) += 1;
                total_with_preference += 1;
            }
        }

        ComparisonStatistics {
            total_comparisons,
            comparisons_with_preference: total_with_preference,
            model_wins,
        }
    }
}

/// Statistics across all comparisons
#[derive(Debug, Clone)]
pub struct ComparisonStatistics {
    pub total_comparisons: usize,
    pub comparisons_with_preference: usize,
    pub model_wins: HashMap<String, usize>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison_manager_create() {
        let mut manager = ComparisonManager::new();

        let params = GenerationParams::default();
        let models = vec![
            ModelConfig {
                name: "Base SDXL".to_string(),
                base: "sdxl_base.safetensors".to_string(),
                lora: None,
                lora_strength: 1.0,
            },
            ModelConfig {
                name: "Pixel Art LoRA".to_string(),
                base: "sdxl_base.safetensors".to_string(),
                lora: Some("pixel_art.safetensors".to_string()),
                lora_strength: 1.0,
            },
        ];

        let comparison_id = manager.create_comparison(params, models);
        assert!(manager.get_active(&comparison_id).is_some());
    }

    #[test]
    fn test_comparison_job_tracking() {
        let mut manager = ComparisonManager::new();

        let params = GenerationParams::default();
        let models = vec![
            ModelConfig::default(),
            ModelConfig::default(),
        ];

        let comparison_id = manager.create_comparison(params, models);
        let job_ids = vec!["job-1".to_string(), "job-2".to_string()];

        manager.register_jobs(&comparison_id, job_ids.clone());

        let job = manager.get_active(&comparison_id).unwrap();
        assert_eq!(job.job_ids, job_ids);
        assert_eq!(
            job.status,
            ComparisonStatus::Running {
                completed: 0,
                total: 2
            }
        );
    }

    #[test]
    fn test_comparison_completion() {
        let mut manager = ComparisonManager::new();

        let params = GenerationParams::default();
        let models = vec![ModelConfig::default(), ModelConfig::default()];

        let comparison_id = manager.create_comparison(params, models);
        let job_ids = vec!["job-1".to_string(), "job-2".to_string()];
        manager.register_jobs(&comparison_id, job_ids);

        manager.complete_job("job-1", PathBuf::from("/tmp/img1.png"), 3.5);
        assert!(manager.get_active(&comparison_id).is_some());

        manager.complete_job("job-2", PathBuf::from("/tmp/img2.png"), 3.8);
        assert!(manager.get_active(&comparison_id).is_none());
        assert!(manager.get_completed(&comparison_id).is_some());
    }

    #[test]
    fn test_user_preference() {
        let mut manager = ComparisonManager::new();

        let params = GenerationParams::default();
        let models = vec![
            ModelConfig {
                name: "Model A".to_string(),
                ..Default::default()
            },
            ModelConfig {
                name: "Model B".to_string(),
                ..Default::default()
            },
        ];

        let comparison_id = manager.create_comparison(params, models);
        manager.register_jobs(&comparison_id, vec!["job-1".to_string(), "job-2".to_string()]);
        manager.complete_job("job-1", PathBuf::from("/tmp/1.png"), 3.0);
        manager.complete_job("job-2", PathBuf::from("/tmp/2.png"), 3.0);

        let success = manager.set_preference(
            &comparison_id,
            1,
            Some("Model B looks better".to_string()),
        );
        assert!(success);

        let result = manager.get_completed(&comparison_id).unwrap();
        assert_eq!(result.user_preference, Some(1));
        assert_eq!(result.winner().unwrap().name, "Model B");
    }

    #[test]
    fn test_statistics() {
        let mut manager = ComparisonManager::new();

        // Create first comparison
        let params = GenerationParams::default();
        let models = vec![
            ModelConfig {
                name: "Model A".to_string(),
                ..Default::default()
            },
            ModelConfig {
                name: "Model B".to_string(),
                ..Default::default()
            },
        ];

        let cmp1 = manager.create_comparison(params.clone(), models.clone());
        manager.register_jobs(&cmp1, vec!["j1".to_string(), "j2".to_string()]);
        manager.complete_job("j1", PathBuf::from("/tmp/1.png"), 3.0);
        manager.complete_job("j2", PathBuf::from("/tmp/2.png"), 3.0);
        manager.set_preference(&cmp1, 1, None); // Model B wins

        // Create second comparison
        let cmp2 = manager.create_comparison(params, models);
        manager.register_jobs(&cmp2, vec!["j3".to_string(), "j4".to_string()]);
        manager.complete_job("j3", PathBuf::from("/tmp/3.png"), 3.0);
        manager.complete_job("j4", PathBuf::from("/tmp/4.png"), 3.0);
        manager.set_preference(&cmp2, 1, None); // Model B wins again

        let stats = manager.get_statistics();
        assert_eq!(stats.total_comparisons, 2);
        assert_eq!(stats.comparisons_with_preference, 2);
        assert_eq!(stats.model_wins.get("Model B"), Some(&2));
    }
}
