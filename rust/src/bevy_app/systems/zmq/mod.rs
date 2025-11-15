//! # ZeroMQ Integration Systems
//!
//! Bevy systems for polling ZMQ client and processing responses/updates.

mod poller;
mod response_handler;
mod update_handler;

pub use poller::*;
pub use response_handler::*;
pub use update_handler::*;

use bevy::prelude::*;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::zmq_client::ZmqClient;

/// Thread-safe ZeroMQ client resource for Bevy.
#[derive(Resource, Clone)]
pub struct ZmqClientResource(pub Arc<Mutex<ZmqClient>>);

impl ZmqClientResource {
    /// Create new resource from ZmqClient.
    pub fn new(client: ZmqClient) -> Self {
        Self(Arc::new(Mutex::new(client)))
    }
}

impl std::fmt::Debug for ZmqClientResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZmqClientResource").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zmq_resource_creation() {
        // Note: Can't actually create ZmqClient without backend running
        // This test just verifies the types compile
    }
}
