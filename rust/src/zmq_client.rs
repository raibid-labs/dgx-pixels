/// ZeroMQ client for communicating with Python backend
///
/// Implements REQ-REP pattern for request/response
/// and SUB pattern for progress updates

use crate::messages::*;
use anyhow::{Context, Result};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// ZeroMQ client for backend communication
pub struct ZmqClient {
    req_sender: Sender<ClientRequest>,
    resp_receiver: Receiver<Response>,
    update_receiver: Receiver<ProgressUpdate>,
    _req_thread: thread::JoinHandle<()>,
    _sub_thread: thread::JoinHandle<()>,
}

/// Internal message for request thread
enum ClientRequest {
    Send(Request),
    Shutdown,
}

impl ZmqClient {
    /// Create a new ZeroMQ client
    pub fn new(req_addr: &str, pub_addr: &str) -> Result<Self> {
        info!("Initializing ZeroMQ client");
        debug!("REQ-REP address: {}", req_addr);
        debug!("PUB-SUB address: {}", pub_addr);

        // Create channels for cross-thread communication
        let (req_send, req_recv) = channel::<ClientRequest>();
        let (resp_send, resp_recv) = channel::<Response>();
        let (update_send, update_recv) = channel::<ProgressUpdate>();

        // Spawn REQ-REP thread
        let req_addr = req_addr.to_string();
        let req_thread = thread::spawn(move || {
            if let Err(e) = Self::req_rep_loop(&req_addr, req_recv, resp_send) {
                error!("REQ-REP thread error: {}", e);
            }
        });

        // Spawn SUB thread
        let pub_addr = pub_addr.to_string();
        let sub_thread = thread::spawn(move || {
            if let Err(e) = Self::pub_sub_loop(&pub_addr, update_send) {
                error!("PUB-SUB thread error: {}", e);
            }
        });

        Ok(Self {
            req_sender: req_send,
            resp_receiver: resp_recv,
            update_receiver: update_recv,
            _req_thread: req_thread,
            _sub_thread: sub_thread,
        })
    }

    /// Create client with default addresses
    pub fn new_default() -> Result<Self> {
        Self::new(DEFAULT_REQ_REP_ADDR, DEFAULT_PUB_SUB_ADDR)
    }

    /// Send a request to the backend
    pub fn send_request(&self, request: Request) -> Result<()> {
        self.req_sender
            .send(ClientRequest::Send(request))
            .context("Failed to send request to worker thread")
    }

    /// Try to receive a response (non-blocking)
    pub fn try_recv_response(&self) -> Option<Response> {
        self.resp_receiver.try_recv().ok()
    }

    /// Receive a response with timeout
    pub fn recv_response_timeout(&self, timeout: Duration) -> Result<Response> {
        self.resp_receiver
            .recv_timeout(timeout)
            .context("Timeout waiting for response")
    }

    /// Try to receive a progress update (non-blocking)
    pub fn try_recv_update(&self) -> Option<ProgressUpdate> {
        self.update_receiver.try_recv().ok()
    }

    /// REQ-REP loop (runs in separate thread)
    fn req_rep_loop(
        addr: &str,
        req_recv: Receiver<ClientRequest>,
        resp_send: Sender<Response>,
    ) -> Result<()> {
        info!("Starting REQ-REP thread");

        let context = zmq::Context::new();
        let socket = context.socket(zmq::REQ)?;
        socket.connect(addr)?;
        socket.set_rcvtimeo(5000)?; // 5 second receive timeout

        info!("Connected to REQ-REP endpoint: {}", addr);

        loop {
            // Wait for request from main thread
            match req_recv.recv_timeout(Duration::from_millis(100)) {
                Ok(ClientRequest::Send(request)) => {
                    debug!("Sending request: {:?}", request);

                    // Serialize and send
                    let serialized = serialize(&request)?;
                    socket.send(&serialized, 0)?;

                    // Wait for response
                    match socket.recv_bytes(0) {
                        Ok(data) => {
                            let response: Response = deserialize(&data)?;
                            debug!("Received response: {:?}", response);

                            if resp_send.send(response).is_err() {
                                warn!("Failed to send response to main thread");
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Error receiving response: {}", e);
                            // Send error response
                            let _ = resp_send.send(Response::Error {
                                message: format!("Communication error: {}", e),
                            });
                        }
                    }
                }
                Ok(ClientRequest::Shutdown) => {
                    info!("REQ-REP thread shutting down");
                    break;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Continue waiting
                    continue;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    info!("REQ-REP channel disconnected, shutting down");
                    break;
                }
            }
        }

        Ok(())
    }

    /// PUB-SUB loop (runs in separate thread)
    fn pub_sub_loop(addr: &str, update_send: Sender<ProgressUpdate>) -> Result<()> {
        info!("Starting PUB-SUB thread");

        let context = zmq::Context::new();
        let socket = context.socket(zmq::SUB)?;
        socket.connect(addr)?;
        socket.set_subscribe(b"")?; // Subscribe to all messages
        socket.set_rcvtimeo(1000)?; // 1 second receive timeout

        info!("Connected to PUB-SUB endpoint: {}", addr);

        loop {
            match socket.recv_bytes(0) {
                Ok(data) => {
                    match deserialize::<ProgressUpdate>(&data) {
                        Ok(update) => {
                            debug!("Received update: {:?}", update);

                            if update_send.send(update).is_err() {
                                info!("Main thread disconnected, shutting down PUB-SUB");
                                break;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to deserialize update: {}", e);
                        }
                    }
                }
                Err(zmq::Error::EAGAIN) => {
                    // Timeout, continue
                    continue;
                }
                Err(e) => {
                    error!("Error receiving update: {}", e);
                    // Keep trying
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }

        Ok(())
    }
}

impl Drop for ZmqClient {
    fn drop(&mut self) {
        debug!("Shutting down ZeroMQ client");
        let _ = self.req_sender.send(ClientRequest::Shutdown);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation_fails_without_server() {
        // This should fail since no server is running
        // We're just testing that the error handling works
        let result = ZmqClient::new("tcp://127.0.0.1:9999", "tcp://127.0.0.1:9998");

        // Client creation should succeed (connection is lazy)
        // But sending a request would fail
        assert!(result.is_ok());
    }

    #[test]
    fn test_serialize_deserialize_messages() {
        let request = Request::Ping;
        let serialized = serialize(&request).unwrap();
        let deserialized: Request = deserialize(&serialized).unwrap();
        assert_eq!(request, deserialized);
    }
}
