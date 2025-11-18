//! Bevy systems for the DGX-Pixels TUI.
//!
//! Systems are functions that operate on resources and components,
//! scheduled and executed by the Bevy ECS scheduler.

pub mod assets;
pub mod input;
pub mod render;
pub mod state_init;
pub mod zmq;

pub use state_init::init_app_state;
