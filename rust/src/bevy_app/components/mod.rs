//! Entity components for the Bevy ECS architecture.
//!
//! Components are attached to entities and represent per-entity data,
//! as opposed to Resources which represent global singleton state.

pub mod job;
pub mod preview;

pub use job::{Job, JobStatus};
pub use preview::PreviewImage;
