# WS-17: MCP Integration

**Orchestrator**: Integration
**Duration**: 4-5 days
**Risk**: Medium
**Dependencies**: WS-06 (Image Assets), All screens (WS-09 through WS-16)

## Summary

Integrate with Bevy's asset hot-reloading system. Implement MCP server for Bevy asset control. Enable live asset updates from external tools and asset injection from Bevy game projects.

## Files Created
```
rust/src/bevy_app/mcp/
├── mod.rs
├── server.rs            # MCP server implementation
├── asset_sync.rs        # Asset synchronization
└── hot_reload.rs        # Hot reload handlers
```

## Key Implementation

```rust
#[derive(Resource)]
struct McpServer {
    listener: TcpListener,
    connections: Vec<McpConnection>,
}

fn handle_mcp_requests(
    mut server: ResMut<McpServer>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for request in server.poll_requests() {
        match request {
            McpRequest::LoadAsset { path } => {
                let handle = asset_server.load(path);
                // Trigger hot reload
            }
            McpRequest::UpdateAsset { path, data } => {
                // Write asset, trigger reload
            }
            _ => {}
        }
    }
}
```

## Security Considerations

- [ ] Authentication required (API keys)
- [ ] Rate limiting implemented
- [ ] Localhost-only default
- [ ] Input validation on all requests
- [ ] Security audit passed

## Acceptance Criteria
- [ ] MCP server runs alongside TUI
- [ ] Assets hot-reload when updated externally
- [ ] Bevy projects can inject assets via MCP
- [ ] Generation results auto-deploy to connected Bevy apps
- [ ] Protocol documented

## Documentation Requirements

Create:
1. `docs/mcp-protocol.md` - API specification
2. `docs/bevy-integration.md` - Integration guide
3. `docs/mcp-security.md` - Security best practices

**Branch**: `tui-modernization/ws17-mcp-integration`
