# WS-05: ZeroMQ Integration

**Orchestrator**: Core Systems
**Duration**: 3-4 days
**Risk**: Medium
**Dependencies**: WS-02 (ECS State)

## Summary

Integrate ZeroMQ client with Bevy's async runtime. Create polling systems for ZMQ responses/updates and implement job state update systems using ECS patterns.

## Files Created
```
rust/src/bevy_app/systems/zmq/
├── mod.rs
├── poller.rs            # ZMQ polling system
├── response_handler.rs  # Response processing
└── update_handler.rs    # Progress updates
```

## Key Implementation

```rust
#[derive(Resource)]
struct ZmqClientResource(Arc<Mutex<ZmqClient>>);

fn poll_zmq_responses(
    client: Res<ZmqClientResource>,
    mut commands: Commands,
    mut job_query: Query<(Entity, &mut Job)>,
) {
    let client = client.0.lock().unwrap();
    
    while let Some(response) = client.try_recv_response() {
        match response {
            Response::JobAccepted { job_id, .. } => {
                commands.spawn(Job::new(job_id, prompt));
            }
            Response::JobComplete { job_id, image_path, duration_s } => {
                for (_, mut job) in job_query.iter_mut() {
                    if job.id == job_id {
                        job.status = JobStatus::Complete { image_path, duration_s };
                    }
                }
            }
            _ => {}
        }
    }
}
```

## Acceptance Criteria
- [ ] ZMQ client initializes in Bevy app
- [ ] Responses processed in real-time
- [ ] Job state updates trigger UI refresh
- [ ] No deadlocks or race conditions

**Branch**: `tui-modernization/ws05-zeromq-integration`
