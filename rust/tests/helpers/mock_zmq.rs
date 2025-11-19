//! Mock ZMQ client for testing

use dgx_pixels_tui::messages::{ProgressUpdate, Request, Response};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

/// Mock ZMQ client that simulates backend communication
#[derive(Clone)]
pub struct MockZmqClient {
    /// Queue of responses to return
    responses: Arc<Mutex<VecDeque<Response>>>,
    /// Queue of progress updates to return
    updates: Arc<Mutex<VecDeque<ProgressUpdate>>>,
    /// Requests that were sent (for verification)
    sent_requests: Arc<Mutex<Vec<Request>>>,
}

impl MockZmqClient {
    /// Create a new mock client
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::new())),
            updates: Arc::new(Mutex::new(VecDeque::new())),
            sent_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Queue a response to be returned
    pub fn queue_response(&self, response: Response) {
        self.responses.lock().push_back(response);
    }

    /// Queue a progress update to be returned
    pub fn queue_update(&self, update: ProgressUpdate) {
        self.updates.lock().push_back(update);
    }

    /// Get all sent requests (for verification)
    pub fn sent_requests(&self) -> Vec<Request> {
        self.sent_requests.lock().clone()
    }

    /// Clear all queued responses and updates
    pub fn clear(&self) {
        self.responses.lock().clear();
        self.updates.lock().clear();
        self.sent_requests.lock().clear();
    }

    /// Simulate sending a request (stores for verification)
    pub fn send_request(&self, request: Request) -> anyhow::Result<()> {
        self.sent_requests.lock().push(request);
        Ok(())
    }

    /// Try to receive a response (non-blocking)
    pub fn try_recv_response(&self) -> Option<Response> {
        self.responses.lock().pop_front()
    }

    /// Try to receive a progress update (non-blocking)
    pub fn try_recv_update(&self) -> Option<ProgressUpdate> {
        self.updates.lock().pop_front()
    }
}

impl Default for MockZmqClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_zmq_client() {
        let client = MockZmqClient::new();

        // Queue a response
        client.queue_response(Response::JobAccepted {
            job_id: "test-job".to_string(),
            estimated_time_s: 5.0,
        });

        // Should receive the queued response
        let response = client.try_recv_response();
        assert!(response.is_some());

        // Should be empty now
        assert!(client.try_recv_response().is_none());
    }

    #[test]
    fn test_sent_requests_tracking() {
        let client = MockZmqClient::new();

        let req = Request::Generate {
            id: "test-job-id".to_string(),
            prompt: "test".to_string(),
            model: "sdxl".to_string(),
            lora: None,
            size: (512, 512),
            steps: 20,
            cfg_scale: 7.5,
        };

        client.send_request(req.clone()).unwrap();

        let sent = client.sent_requests();
        assert_eq!(sent.len(), 1);
    }
}
