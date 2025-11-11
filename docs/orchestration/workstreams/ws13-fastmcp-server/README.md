# WS-13: FastMCP Server

**ID**: WS-13
**Orchestrator**: Integration
**Milestone**: M4
**Duration**: 5-6 days
**Priority**: P0 (CRITICAL PATH)
**Dependencies**: WS-10 (Python Backend Worker)
**Agent Type**: `backend-architect`
**Status**: Not Started

---

## Objective

Build a Model Context Protocol (MCP) server using the FastMCP framework to enable Bevy game engine integration with DGX-Pixels. This server exposes generation capabilities as MCP tools that can be invoked by Bevy applications (via bevy_brp_mcp) or AI assistants. The MCP server acts as a bridge between the Bevy asset pipeline and the Python backend worker, enabling automated sprite generation and deployment.

**Importance**: This workstream is critical for achieving automated Bevy integration (M4 milestone). It blocks both Bevy plugin integration (WS-14) and asset deployment pipeline (WS-15). Without MCP, users must manually copy generated sprites to their Bevy projects.

---

## Deliverables

1. **FastMCP Server Implementation** (`/home/beengud/raibid-labs/dgx-pixels/python/mcp_server/server.py`)
   - MCP server using fastmcp library
   - Async integration with Python backend worker (WS-10)
   - Support for stdio and SSE transports
   - Error handling with MCP error format

2. **MCP Tool Definitions** (`python/mcp_server/tools.py`)
   - `generate_sprite`: Generate single sprite with prompt
   - `generate_batch`: Generate multiple sprites from prompt list
   - `deploy_to_bevy`: Generate and auto-deploy to Bevy assets directory
   - `list_models`: List available SDXL models and LoRAs
   - `get_status`: Get generation job status

3. **Server Configuration** (`/home/beengud/raibid-labs/dgx-pixels/config/mcp_config.yaml`)
   - Server settings (host, port, transport mode)
   - Backend worker connection details (ZeroMQ endpoint)
   - Default generation parameters
   - Security settings (allowed paths, rate limiting)

4. **Integration Layer** (`python/mcp_server/worker_client.py`)
   - Client for Python backend worker (WS-10)
   - Job submission and status polling
   - Result retrieval and error handling

5. **Server Launcher** (`/home/beengud/raibid-labs/dgx-pixels/scripts/start_mcp_server.sh`)
   - Launch script with environment validation
   - Logging configuration
   - Graceful shutdown handling

6. **API Documentation** (`/home/beengud/raibid-labs/dgx-pixels/docs/mcp-api.md`)
   - MCP tool schemas and examples
   - Parameter descriptions
   - Error codes and handling
   - Example MCP client code

7. **Testing Infrastructure**
   - Unit tests for tool functions
   - Integration tests with mock backend
   - End-to-end tests with real backend worker
   - MCP protocol compliance tests

8. **Example MCP Client** (`/home/beengud/raibid-labs/dgx-pixels/examples/mcp_client.py`)
   - Reference implementation for testing
   - Example usage of all tools

---

## Acceptance Criteria

**Functional**:
- ✅ MCP server starts successfully and responds to initialization
- ✅ All 5 tools registered and discoverable (`list_tools` returns correct schemas)
- ✅ `generate_sprite` tool: Generates sprite and returns image path
- ✅ `generate_batch` tool: Generates multiple sprites, returns list of paths
- ✅ `deploy_to_bevy` tool: Generates sprite and copies to Bevy assets/ directory
- ✅ `list_models` tool: Returns list of available models from backend
- ✅ `get_status` tool: Returns job status (pending/running/complete/failed)
- ✅ Integrates with Python backend worker (WS-10) via worker_client.py
- ✅ Supports stdio transport (for bevy_brp_mcp)
- ✅ Supports SSE transport (for web clients, optional)
- ✅ Error handling returns proper MCP error format
- ✅ Graceful shutdown on SIGTERM/SIGINT

**Performance**:
- ✅ Tool invocation overhead ≤ 200ms (server processing time, excluding generation)
- ✅ Server startup time ≤ 2 seconds
- ✅ Can handle 10 concurrent tool calls (async queue)
- ✅ Memory usage ≤ 200MB (server only, not including backend worker)

**Quality**:
- ✅ Test coverage ≥ 80% (pytest with coverage)
- ✅ All tools validate against MCP specification
- ✅ Comprehensive error handling (network, backend, filesystem)
- ✅ Documentation complete (API docs, examples, troubleshooting)
- ✅ Logging with structured output (JSON logs for observability)

---

## Technical Requirements

### Environment
- **Hardware**: DGX-Spark GB10 (ARM64) - MCP server is CPU-only
- **OS**: Ubuntu 22.04 (ARM64)
- **Python**: 3.10+ (same as backend worker)
- **Backend**: Requires WS-10 (Python Backend Worker) running

### Dependencies

**Python Packages** (`python/mcp_server/requirements.txt`):
```
# MCP framework
fastmcp>=0.2.0

# Async I/O
aiohttp>=3.9.0
asyncio>=3.4.3

# Backend communication (same as WS-10)
pyzmq>=26.0.0
msgpack>=1.0.0

# Configuration
pyyaml>=6.0

# Validation
pydantic>=2.0

# Logging
structlog>=24.1.0

# Testing
pytest>=7.4.0
pytest-asyncio>=0.21.0
pytest-cov>=4.1.0
```

**System Dependencies**:
```bash
# Python virtual environment
sudo apt install -y python3-venv python3-pip

# For testing MCP stdio transport
sudo apt install -y socat
```

### Technical Constraints
- Must follow MCP specification (https://modelcontextprotocol.io/)
- Must integrate with existing Python backend worker (WS-10)
- Must not duplicate generation logic (delegate to backend)
- Must validate all file paths (security: prevent directory traversal)
- Must handle backend worker restarts gracefully
- Must be stateless (no persistent state in MCP server)
- Must support ARM64 architecture (all dependencies ARM-compatible)

### MCP Specification Compliance
- Tool schemas must include: name, description, inputSchema (JSON Schema)
- Errors must use MCP error codes: -32600 (invalid request), -32601 (method not found), -32602 (invalid params), -32603 (internal error)
- Transport: stdio (required), SSE (optional but recommended)
- Protocol version: MCP 1.0

---

## Implementation Plan

### Phase 1: Server Foundation (Days 1-2)
**Goal**: Set up FastMCP server structure and basic tool scaffolding

**Tasks**:
1. Create Python virtual environment: `python3 -m venv python/mcp_server/.venv`
2. Install dependencies: `pip install -r python/mcp_server/requirements.txt`
3. Create server.py with FastMCP initialization
4. Implement basic tool: `list_models` (simple, no backend interaction)
5. Configure logging with structlog
6. Test server startup with stdio transport
7. Write unit tests for server initialization
8. Create config/mcp_config.yaml with default settings

**Output**: MCP server that starts and responds to basic tool calls

**Verification**:
```bash
# Start MCP server (stdio transport)
cd /home/beengud/raibid-labs/dgx-pixels
python python/mcp_server/server.py --transport stdio

# Test with example client
python examples/mcp_client.py list_models
# Should return: {"tools": [{"name": "list_models", ...}]}
```

### Phase 2: Backend Integration (Days 3-4)
**Goal**: Integrate with Python backend worker (WS-10)

**Tasks**:
1. Create worker_client.py with ZeroMQ client (reuse patterns from WS-09/WS-10)
2. Implement connection management (connect, reconnect, health check)
3. Implement `generate_sprite` tool with backend integration
4. Implement `generate_batch` tool with parallel job submission
5. Implement `get_status` tool with job status polling
6. Add error handling for backend failures (timeout, connection lost, job failed)
7. Write integration tests with mock backend
8. Test with real backend worker (requires WS-10 complete)

**Output**: MCP server fully integrated with backend worker

**Verification**:
```bash
# Start backend worker (WS-10)
cd /home/beengud/raibid-labs/dgx-pixels
python python/workers/generation_worker.py &

# Start MCP server
python python/mcp_server/server.py &

# Test generate_sprite tool
python examples/mcp_client.py generate_sprite \
  --prompt "16-bit pixel art knight sprite" \
  --output /tmp/knight.png

# Check output
test -f /tmp/knight.png && echo "✅ Sprite generated"
```

### Phase 3: Bevy Integration Tools (Day 5)
**Goal**: Implement Bevy-specific tools for asset deployment

**Tasks**:
1. Implement `deploy_to_bevy` tool (generate + copy to Bevy assets/)
2. Add path validation (ensure target is valid Bevy project)
3. Implement asset naming conventions (follow Bevy best practices)
4. Add file existence checks (warn if overwriting)
5. Create launcher script: `scripts/start_mcp_server.sh`
6. Write end-to-end test with example Bevy project
7. Update documentation with Bevy integration guide

**Output**: Complete MCP server with Bevy deployment support

**Verification**:
```bash
# Create test Bevy project structure
mkdir -p /tmp/test_bevy_project/assets/sprites

# Generate and deploy sprite
python examples/mcp_client.py deploy_to_bevy \
  --prompt "pixel art tree" \
  --bevy-project /tmp/test_bevy_project \
  --asset-path sprites/tree.png

# Verify deployment
test -f /tmp/test_bevy_project/assets/sprites/tree.png && echo "✅ Asset deployed"
```

### Phase 4: Testing, Documentation & Polish (Day 6)
**Goal**: Comprehensive testing, documentation, and production readiness

**Tasks**:
1. Achieve ≥80% test coverage (pytest --cov)
2. Write API documentation (docs/mcp-api.md)
3. Add example MCP client code
4. Test error handling (backend down, invalid params, filesystem errors)
5. Add rate limiting (optional, for production)
6. Test with bevy_brp_mcp client (requires WS-14, or manual test)
7. Performance benchmarks (tool invocation latency)
8. Create completion summary

**Output**: Production-ready MCP server with full documentation

**Verification**:
```bash
# Run test suite
cd /home/beengud/raibid-labs/dgx-pixels
pytest python/mcp_server/tests/ -v --cov=python/mcp_server --cov-report=html

# Run MCP protocol compliance tests
pytest python/mcp_server/tests/test_mcp_compliance.py -v

# Check documentation
test -f docs/mcp-api.md && echo "✅ API docs exist"

# Manual test: Use MCP server with bevy_brp_mcp (if WS-14 complete)
# Otherwise, test with example client
python examples/mcp_client.py --help
```

---

## Test-Driven Development (TDD)

### Test Requirements

**Unit Tests** (`python/mcp_server/tests/test_tools.py`):
- `test_list_models`: Verify list_models returns valid model list
- `test_generate_sprite_params`: Verify parameter validation (prompt required, etc.)
- `test_generate_batch_params`: Verify batch parameters (prompt list, max size)
- `test_deploy_to_bevy_path_validation`: Verify path sanitization (no ../, absolute paths only)
- `test_get_status`: Verify status polling returns correct format
- `test_error_handling`: Verify MCP error format for various failures
- `test_config_loading`: Verify config loads from YAML correctly

**Integration Tests** (`python/mcp_server/tests/test_integration.py`):
- `test_server_startup`: Verify server starts with stdio transport
- `test_backend_connection`: Verify connection to backend worker (mock)
- `test_generate_sprite_e2e`: End-to-end sprite generation with mock backend
- `test_generate_batch_e2e`: End-to-end batch generation with mock backend
- `test_backend_failure_handling`: Verify graceful handling of backend failures
- `test_concurrent_requests`: Verify server handles 10 concurrent tool calls

**MCP Compliance Tests** (`python/mcp_server/tests/test_mcp_compliance.py`):
- `test_tool_schema_validity`: Verify all tool schemas are valid JSON Schema
- `test_error_codes`: Verify error responses use correct MCP error codes
- `test_initialization`: Verify MCP initialization handshake
- `test_stdio_transport`: Verify stdio transport works correctly

**Performance Tests** (`python/mcp_server/tests/test_performance.py`):
- `bench_tool_invocation`: Measure tool invocation overhead (target: ≤ 200ms)
- `bench_server_startup`: Measure server startup time (target: ≤ 2s)
- `bench_concurrent_load`: Measure throughput with 10 concurrent requests

### Test Commands

```bash
# Run all tests
cd /home/beengud/raibid-labs/dgx-pixels
pytest python/mcp_server/tests/ -v

# Run with coverage
pytest python/mcp_server/tests/ --cov=python/mcp_server --cov-report=html
open htmlcov/index.html

# Run specific test
pytest python/mcp_server/tests/test_tools.py::test_generate_sprite_params -v

# Run integration tests only
pytest python/mcp_server/tests/test_integration.py -v

# Run MCP compliance tests
pytest python/mcp_server/tests/test_mcp_compliance.py -v

# Run performance tests
pytest python/mcp_server/tests/test_performance.py -v --benchmark-only

# Check code quality
ruff check python/mcp_server/
mypy python/mcp_server/
```

### Expected Test Output
```
============================= test session starts ==============================
python/mcp_server/tests/test_tools.py::test_list_models PASSED          [ 10%]
python/mcp_server/tests/test_tools.py::test_generate_sprite_params PASSED [ 20%]
python/mcp_server/tests/test_tools.py::test_generate_batch_params PASSED [ 30%]
python/mcp_server/tests/test_tools.py::test_deploy_to_bevy_path_validation PASSED [ 40%]
python/mcp_server/tests/test_tools.py::test_get_status PASSED           [ 50%]
python/mcp_server/tests/test_tools.py::test_error_handling PASSED       [ 60%]
python/mcp_server/tests/test_tools.py::test_config_loading PASSED       [ 70%]

python/mcp_server/tests/test_integration.py::test_server_startup PASSED [ 80%]
python/mcp_server/tests/test_integration.py::test_backend_connection PASSED [ 85%]
python/mcp_server/tests/test_integration.py::test_generate_sprite_e2e PASSED [ 90%]
python/mcp_server/tests/test_integration.py::test_concurrent_requests PASSED [100%]

python/mcp_server/tests/test_mcp_compliance.py::test_tool_schema_validity PASSED
python/mcp_server/tests/test_mcp_compliance.py::test_error_codes PASSED
python/mcp_server/tests/test_mcp_compliance.py::test_stdio_transport PASSED

python/mcp_server/tests/test_performance.py::bench_tool_invocation: 145ms ✅
python/mcp_server/tests/test_performance.py::bench_server_startup: 1.8s ✅
python/mcp_server/tests/test_performance.py::bench_concurrent_load: 10 req/s ✅

---------- coverage: platform linux, python 3.10.12 -----------
Name                                      Stmts   Miss  Cover
-------------------------------------------------------------
python/mcp_server/server.py                 120      18    85%
python/mcp_server/tools.py                  180      25    86%
python/mcp_server/worker_client.py           95      12    87%
-------------------------------------------------------------
TOTAL                                       395      55    86%

============================= 18 passed in 12.34s ==============================

WS-13: ALL TESTS PASSING ✅
Coverage: 86% ✅
```

---

## Dependencies

### Blocked By
- **WS-10 (Python Backend Worker)**: MCP server requires backend to generate sprites

### Blocks
- **WS-14 (Bevy Plugin Integration)**: Bevy plugin needs MCP server for communication
- **WS-15 (Asset Deployment Pipeline)**: Deployment pipeline uses MCP tools

### Soft Dependencies
- **WS-04 (ComfyUI Setup)**: Helpful for end-to-end testing, but not required if WS-10 mocks ComfyUI
- **WS-05 (SDXL Optimization)**: Better performance, but not required for MCP server functionality

---

## Known Issues & Risks

### Issue 1: Backend Worker Availability
**Problem**: MCP server depends on backend worker (WS-10) being running and accessible
**Impact**: High (MCP server non-functional if backend down)
**Mitigation**:
- Implement health checks (ping backend on startup)
- Return clear error messages if backend unavailable
- Support graceful degradation (list_models works even if backend down)
- Document backend startup order in deployment guide
**Fallback**: MCP server returns "backend unavailable" error with retry instructions
**Status**: Medium risk - document dependency clearly

### Issue 2: Path Traversal Security
**Problem**: `deploy_to_bevy` tool accepts file paths, risk of directory traversal attacks
**Impact**: High (security vulnerability)
**Mitigation**:
- Validate all paths (reject ../, reject absolute paths outside allowed directories)
- Use Path.resolve() to canonicalize paths
- Whitelist allowed directories in config
- Add unit tests for path validation edge cases
**Fallback**: Reject all paths with suspicious patterns
**Status**: Must implement - critical security requirement

### Issue 3: MCP Specification Evolution
**Problem**: MCP specification is relatively new and may evolve
**Impact**: Medium (potential breaking changes)
**Mitigation**:
- Pin fastmcp version in requirements.txt
- Test against MCP specification compliance
- Monitor MCP specification updates
- Document MCP version compatibility
**Fallback**: Fork fastmcp if needed for stability
**Status**: Low risk currently - fastmcp is stable

### Issue 4: ARM Compatibility of fastmcp
**Problem**: fastmcp may not be tested on ARM64 architecture
**Impact**: High (blocking if incompatible)
**Mitigation**:
- Test fastmcp installation on DGX-Spark ARM before starting implementation
- Check fastmcp dependencies for ARM support
- Have fallback: implement MCP server manually if fastmcp doesn't work
**Fallback**: Use mcp library directly instead of fastmcp wrapper
**Status**: MUST VERIFY BEFORE STARTING - test ARM compatibility first

---

## Integration Points

### With Other Workstreams
- **WS-10 (Backend Worker)**: MCP server calls backend via worker_client.py (ZeroMQ)
- **WS-14 (Bevy Plugin)**: Bevy uses bevy_brp_mcp to call MCP server tools
- **WS-15 (Asset Deployment)**: Deployment pipeline invokes MCP tools programmatically

### With External Systems
- **Bevy Game Engine**: Via bevy_brp_mcp plugin (stdio transport)
- **AI Assistants**: Via MCP protocol (stdio or SSE transport)
- **Backend Worker**: Via ZeroMQ (REQ-REP pattern from WS-10)
- **Filesystem**: Writes generated sprites to specified paths
- **Config File**: Reads config/mcp_config.yaml for settings

---

## Verification & Validation

### Verification Steps (Agent Self-Check)

```bash
# Step 1: Verify project structure
test -f /home/beengud/raibid-labs/dgx-pixels/python/mcp_server/server.py && echo "✅ Server implementation exists"
test -f /home/beengud/raibid-labs/dgx-pixels/python/mcp_server/tools.py && echo "✅ Tool definitions exist"
test -f /home/beengud/raibid-labs/dgx-pixels/config/mcp_config.yaml && echo "✅ Config exists"

# Step 2: Verify dependencies installed
cd /home/beengud/raibid-labs/dgx-pixels
source python/mcp_server/.venv/bin/activate
python -c "import fastmcp; print('✅ fastmcp installed')"
python -c "import pyzmq; print('✅ pyzmq installed')"

# Step 3: Verify server starts
timeout 5s python python/mcp_server/server.py --transport stdio || echo "✅ Server starts (timed out waiting for input, expected)"

# Step 4: Run test suite
pytest python/mcp_server/tests/ -v && echo "✅ All tests passing"

# Step 5: Check test coverage
pytest python/mcp_server/tests/ --cov=python/mcp_server --cov-report=term | grep -E "(TOTAL.*[8-9][0-9]%|TOTAL.*100%)" && echo "✅ Coverage ≥ 80%"

# Step 6: Test MCP compliance
pytest python/mcp_server/tests/test_mcp_compliance.py -v && echo "✅ MCP compliance verified"

# Step 7: Test with example client (requires backend running)
# Skip if backend not available
python examples/mcp_client.py list_models && echo "✅ Example client works"

# Step 8: Verify documentation
test -f /home/beengud/raibid-labs/dgx-pixels/docs/mcp-api.md && echo "✅ API documentation exists"
```

### Acceptance Verification (Orchestrator)

```bash
# Run complete verification script
/home/beengud/raibid-labs/dgx-pixels/scripts/verify_ws_13.sh

# Expected output:
# ✅ MCP server implementation complete
# ✅ All 5 tools implemented (generate_sprite, generate_batch, deploy_to_bevy, list_models, get_status)
# ✅ Backend integration layer complete (worker_client.py)
# ✅ Configuration file exists and valid (mcp_config.yaml)
# ✅ Unit tests passing (7/7)
# ✅ Integration tests passing (6/6)
# ✅ MCP compliance tests passing (3/3)
# ✅ Performance tests passing (3/3)
# ✅ Test coverage ≥ 80% (actual: 86%)
# ✅ API documentation complete (docs/mcp-api.md)
# ✅ Example client working
# ✅ Path validation security tests passing
# ✅ Server starts in ≤ 2 seconds
# ✅ Tool invocation overhead ≤ 200ms
#
# Manual Verification Required:
# ⚠️  Test with bevy_brp_mcp client (WS-14 dependency)
#
# WS-13: READY FOR COMPLETION ✅
```

---

## Success Metrics

**Completion Criteria**:
- All acceptance criteria met (functional, performance, quality)
- All tests passing (≥80% coverage)
- MCP specification compliance verified
- Backend integration working
- Documentation complete
- Security validation passed (path traversal tests)
- Completion summary created

**Quality Metrics**:
- Test coverage: ≥80% (pytest --cov)
- MCP compliance: All tools validate against spec
- Security: Path validation tests passing
- Documentation: API docs complete with examples

**Performance Metrics**:
- Tool invocation overhead: ≤ 200ms
- Server startup: ≤ 2 seconds
- Concurrent requests: 10+ simultaneous
- Memory usage: ≤ 200MB

---

## Completion Checklist

Before marking WS-13 complete:

- [ ] MCP server implementation complete (`python/mcp_server/server.py`)
- [ ] All 5 tools implemented and tested (`python/mcp_server/tools.py`)
- [ ] Backend integration layer complete (`python/mcp_server/worker_client.py`)
- [ ] Configuration file created (`config/mcp_config.yaml`)
- [ ] Launcher script created (`scripts/start_mcp_server.sh`)
- [ ] Unit tests written and passing (≥7 tests)
- [ ] Integration tests written and passing (≥6 tests)
- [ ] MCP compliance tests passing (3 tests)
- [ ] Performance tests passing (3 benchmarks)
- [ ] Test coverage ≥ 80%
- [ ] Path validation security tests passing
- [ ] API documentation written (`docs/mcp-api.md`)
- [ ] Example MCP client working (`examples/mcp_client.py`)
- [ ] Tested with real backend worker (WS-10)
- [ ] Code quality checks passing (ruff, mypy)
- [ ] Completion summary created (`docs/orchestration/workstreams/ws13-fastmcp-server/COMPLETION_SUMMARY.md`)
- [ ] GitHub issue PIXELS-040 closed with summary link

---

## Example Tool Schemas

### generate_sprite Tool Schema
```json
{
  "name": "generate_sprite",
  "description": "Generate a single pixel art sprite with given prompt",
  "inputSchema": {
    "type": "object",
    "properties": {
      "prompt": {
        "type": "string",
        "description": "Text description of the sprite to generate"
      },
      "model": {
        "type": "string",
        "description": "Model name (default: sdxl-base-1.0)",
        "default": "sdxl-base-1.0"
      },
      "lora": {
        "type": "string",
        "description": "Optional LoRA model to apply",
        "default": null
      },
      "resolution": {
        "type": "string",
        "description": "Output resolution (default: 1024x1024)",
        "enum": ["512x512", "1024x1024", "2048x2048"],
        "default": "1024x1024"
      },
      "output_path": {
        "type": "string",
        "description": "Path to save generated sprite"
      }
    },
    "required": ["prompt", "output_path"]
  }
}
```

### deploy_to_bevy Tool Schema
```json
{
  "name": "deploy_to_bevy",
  "description": "Generate sprite and deploy to Bevy project assets directory",
  "inputSchema": {
    "type": "object",
    "properties": {
      "prompt": {
        "type": "string",
        "description": "Text description of the sprite to generate"
      },
      "bevy_project_path": {
        "type": "string",
        "description": "Path to Bevy project root (must contain assets/ directory)"
      },
      "asset_path": {
        "type": "string",
        "description": "Relative path within assets/ directory (e.g. sprites/knight.png)"
      },
      "model": {
        "type": "string",
        "description": "Model name (default: sdxl-base-1.0)",
        "default": "sdxl-base-1.0"
      }
    },
    "required": ["prompt", "bevy_project_path", "asset_path"]
  }
}
```

---

## Example Usage

```python
# Example MCP client usage
import asyncio
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

async def main():
    # Connect to MCP server
    server_params = StdioServerParameters(
        command="python",
        args=["python/mcp_server/server.py", "--transport", "stdio"]
    )

    async with stdio_client(server_params) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()

            # List available tools
            tools = await session.list_tools()
            print(f"Available tools: {[t.name for t in tools]}")

            # Generate a sprite
            result = await session.call_tool(
                "generate_sprite",
                {
                    "prompt": "16-bit pixel art knight sprite",
                    "output_path": "/tmp/knight.png"
                }
            )
            print(f"Generated sprite: {result}")

            # Deploy to Bevy project
            result = await session.call_tool(
                "deploy_to_bevy",
                {
                    "prompt": "pixel art tree",
                    "bevy_project_path": "/home/user/my_game",
                    "asset_path": "sprites/tree.png"
                }
            )
            print(f"Deployed to Bevy: {result}")

asyncio.run(main())
```

---

## Related Issues

- GitHub Issue: #PIXELS-040 (FastMCP Server)
- GitHub Issue: #PIXELS-041 (MCP Tool Definitions)
- GitHub Issue: #PIXELS-042 (Backend Integration)
- Related Workstreams: WS-10, WS-14, WS-15
- Related Docs: `docs/04-bevy-integration.md`, `docs/mcp-api.md`

---

## References

- Architecture: `docs/07-rust-python-architecture.md` (MCP Integration)
- Bevy Integration: `docs/04-bevy-integration.md` (MCP patterns)
- Roadmap: `docs/ROADMAP.md` (M4 - Bevy Integration)
- MCP Specification: https://modelcontextprotocol.io/
- FastMCP Documentation: https://github.com/jlowin/fastmcp
- bevy_brp_mcp: https://github.com/bevyengine/bevy/tree/main/crates/bevy_brp_mcp

---

**Status**: Ready for agent spawn
**Last Updated**: 2025-11-10
**Estimated LOC**: 500-700 (Python) + 200 (tests) + 100 (docs)
