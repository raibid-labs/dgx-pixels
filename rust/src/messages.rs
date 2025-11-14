//! Message protocol definitions for ZeroMQ IPC
//!
//! Version: 1.0
//! Serialization: MessagePack (MsgPack)
//! Transport: ZeroMQ (REQ-REP + PUB-SUB)

use serde::{Deserialize, Serialize};

// ============================================================================
// Request Messages (TUI → Backend)
// ============================================================================

/// Request message from TUI to backend
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    /// Generate a single sprite
    Generate {
        id: String,
        prompt: String,
        model: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        lora: Option<String>,
        size: (u32, u32),
        steps: u32,
        cfg_scale: f32,
    },

    /// Cancel a running job
    Cancel { job_id: String },

    /// List available models
    ListModels,

    /// Get backend status
    Status,

    /// Ping for health check
    Ping,
}

// ============================================================================
// Response Messages (Backend → TUI)
// ============================================================================

/// Response message from backend to TUI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    /// Job accepted and queued
    JobAccepted {
        job_id: String,
        estimated_time_s: f32,
    },

    /// Job completed successfully
    JobComplete {
        job_id: String,
        image_path: String,
        duration_s: f32,
    },

    /// Job failed with error
    JobError { job_id: String, error: String },

    /// Job cancelled
    JobCancelled { job_id: String },

    /// List of available models
    ModelList { models: Vec<ModelInfo> },

    /// Backend status
    StatusInfo {
        version: String,
        queue_size: u32,
        active_jobs: u32,
        uptime_s: u64,
    },

    /// Pong response
    Pong,

    /// Generic error
    Error { message: String },
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelInfo {
    pub name: String,
    pub path: String,
    pub model_type: ModelType,
    pub size_mb: u64,
}

/// Model type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    Checkpoint,
    Lora,
    Vae,
}

// ============================================================================
// Progress Updates (Backend → TUI via PUB-SUB)
// ============================================================================

/// Progress update message published by backend
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProgressUpdate {
    /// Job started
    JobStarted { job_id: String, timestamp: u64 },

    /// Generation progress
    Progress {
        job_id: String,
        stage: GenerationStage,
        step: u32,
        total_steps: u32,
        percent: f32,
        eta_s: f32,
    },

    /// Preview image available (for progressive generation)
    Preview {
        job_id: String,
        image_path: String,
        step: u32,
    },

    /// Job finished (also sent via response)
    JobFinished {
        job_id: String,
        success: bool,
        duration_s: f32,
    },

    /// Job complete (sent by backend as progress update)
    JobComplete {
        job_id: String,
        image_path: String,
        duration_s: f32,
    },
}

/// Generation stage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GenerationStage {
    Initializing,
    LoadingModels,
    Encoding,
    Sampling,
    Decoding,
    PostProcessing,
}

// ============================================================================
// Protocol Metadata
// ============================================================================

/// Protocol version information
#[allow(dead_code)]
pub const PROTOCOL_VERSION: &str = "1.0.0";

/// Default ports
#[allow(dead_code)]
pub const DEFAULT_REQ_REP_PORT: u16 = 5555;
#[allow(dead_code)]
pub const DEFAULT_PUB_SUB_PORT: u16 = 5556;

/// Default addresses
pub const DEFAULT_REQ_REP_ADDR: &str = "tcp://127.0.0.1:5555";
pub const DEFAULT_PUB_SUB_ADDR: &str = "tcp://127.0.0.1:5556";

// ============================================================================
// Serialization Helpers
// ============================================================================

/// Serialize a message to MessagePack format
pub fn serialize<T: Serialize>(msg: &T) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec_named(msg)
}

/// Deserialize a message from MessagePack format
pub fn deserialize<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Result<T, rmp_serde::decode::Error> {
    rmp_serde::from_slice(data)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_generate_request() {
        let req = Request::Generate {
            id: "job-001".to_string(),
            prompt: "16-bit knight sprite".to_string(),
            model: "sdxl-base".to_string(),
            lora: None,
            size: (1024, 1024),
            steps: 30,
            cfg_scale: 7.5,
        };

        let serialized = serialize(&req).expect("Failed to serialize");
        let deserialized: Request = deserialize(&serialized).expect("Failed to deserialize");

        assert_eq!(req, deserialized);
    }

    #[test]
    fn test_serialize_job_accepted_response() {
        let resp = Response::JobAccepted {
            job_id: "job-001".to_string(),
            estimated_time_s: 3.5,
        };

        let serialized = serialize(&resp).expect("Failed to serialize");
        let deserialized: Response = deserialize(&serialized).expect("Failed to deserialize");

        assert_eq!(resp, deserialized);
    }

    #[test]
    fn test_serialize_progress_update() {
        let update = ProgressUpdate::Progress {
            job_id: "job-001".to_string(),
            stage: GenerationStage::Sampling,
            step: 15,
            total_steps: 30,
            percent: 50.0,
            eta_s: 1.8,
        };

        let serialized = serialize(&update).expect("Failed to serialize");
        let deserialized: ProgressUpdate = deserialize(&serialized).expect("Failed to deserialize");

        assert_eq!(update, deserialized);
    }

    #[test]
    fn test_serialize_cancel_request() {
        let req = Request::Cancel {
            job_id: "job-001".to_string(),
        };

        let serialized = serialize(&req).expect("Failed to serialize");
        let deserialized: Request = deserialize(&serialized).expect("Failed to deserialize");

        assert_eq!(req, deserialized);
    }

    #[test]
    fn test_serialize_model_list_response() {
        let resp = Response::ModelList {
            models: vec![
                ModelInfo {
                    name: "SDXL Base".to_string(),
                    path: "/models/sdxl-base.safetensors".to_string(),
                    model_type: ModelType::Checkpoint,
                    size_mb: 6500,
                },
                ModelInfo {
                    name: "Pixel Art LoRA".to_string(),
                    path: "/models/loras/pixelart.safetensors".to_string(),
                    model_type: ModelType::Lora,
                    size_mb: 144,
                },
            ],
        };

        let serialized = serialize(&resp).expect("Failed to serialize");
        let deserialized: Response = deserialize(&serialized).expect("Failed to deserialize");

        assert_eq!(resp, deserialized);
    }

    #[test]
    fn test_serialize_job_error_response() {
        let resp = Response::JobError {
            job_id: "job-001".to_string(),
            error: "Model not found: sdxl-custom".to_string(),
        };

        let serialized = serialize(&resp).expect("Failed to serialize");
        let deserialized: Response = deserialize(&serialized).expect("Failed to deserialize");

        assert_eq!(resp, deserialized);
    }

    #[test]
    fn test_serialize_status_info_response() {
        let resp = Response::StatusInfo {
            version: "1.0.0".to_string(),
            queue_size: 3,
            active_jobs: 1,
            uptime_s: 3600,
        };

        let serialized = serialize(&resp).expect("Failed to serialize");
        let deserialized: Response = deserialize(&serialized).expect("Failed to deserialize");

        assert_eq!(resp, deserialized);
    }

    #[test]
    fn test_serialize_ping_pong() {
        let req = Request::Ping;
        let serialized = serialize(&req).expect("Failed to serialize");
        let deserialized: Request = deserialize(&serialized).expect("Failed to deserialize");
        assert_eq!(req, deserialized);

        let resp = Response::Pong;
        let serialized = serialize(&resp).expect("Failed to serialize");
        let deserialized: Response = deserialize(&serialized).expect("Failed to deserialize");
        assert_eq!(resp, deserialized);
    }

    #[test]
    fn test_serialize_job_started_update() {
        let update = ProgressUpdate::JobStarted {
            job_id: "job-001".to_string(),
            timestamp: 1699564800,
        };

        let serialized = serialize(&update).expect("Failed to serialize");
        let deserialized: ProgressUpdate = deserialize(&serialized).expect("Failed to deserialize");

        assert_eq!(update, deserialized);
    }

    #[test]
    fn test_serialize_preview_update() {
        let update = ProgressUpdate::Preview {
            job_id: "job-001".to_string(),
            image_path: "/tmp/preview-001.png".to_string(),
            step: 10,
        };

        let serialized = serialize(&update).expect("Failed to serialize");
        let deserialized: ProgressUpdate = deserialize(&serialized).expect("Failed to deserialize");

        assert_eq!(update, deserialized);
    }

    #[test]
    fn test_all_generation_stages() {
        let stages = vec![
            GenerationStage::Initializing,
            GenerationStage::LoadingModels,
            GenerationStage::Encoding,
            GenerationStage::Sampling,
            GenerationStage::Decoding,
            GenerationStage::PostProcessing,
        ];

        for stage in stages {
            let update = ProgressUpdate::Progress {
                job_id: "test".to_string(),
                stage: stage.clone(),
                step: 1,
                total_steps: 10,
                percent: 10.0,
                eta_s: 5.0,
            };

            let serialized = serialize(&update).expect("Failed to serialize");
            let deserialized: ProgressUpdate =
                deserialize(&serialized).expect("Failed to deserialize");

            assert_eq!(update, deserialized);
        }
    }

    #[test]
    fn test_message_size_reasonable() {
        // Ensure messages don't get too large
        let req = Request::Generate {
            id: "job-001".to_string(),
            prompt: "A".repeat(500), // 500 char prompt
            model: "sdxl-base".to_string(),
            lora: Some("pixelart".to_string()),
            size: (1024, 1024),
            steps: 30,
            cfg_scale: 7.5,
        };

        let serialized = serialize(&req).expect("Failed to serialize");
        // MessagePack should compress well, expect < 1KB for reasonable prompts
        assert!(
            serialized.len() < 1024,
            "Message too large: {} bytes",
            serialized.len()
        );
    }
}
