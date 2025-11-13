"""WS-13 Structure validation tests

Tests the MCP server file structure and basic imports without requiring
all dependencies to be installed.
"""

from pathlib import Path
import sys

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))


def test_directory_structure():
    """Test that all required files exist"""
    print("\n[Test] Directory Structure")

    mcp_server_dir = project_root / "python" / "mcp_server"
    config_dir = project_root / "config"

    required_files = [
        mcp_server_dir / "__init__.py",
        mcp_server_dir / "__main__.py",
        mcp_server_dir / "server.py",
        mcp_server_dir / "config_loader.py",
        mcp_server_dir / "backend_client.py",
        mcp_server_dir / "tools.py",
        mcp_server_dir / "test_client.py",
        mcp_server_dir / "README.md",
        mcp_server_dir / "QUICKSTART.md",
        mcp_server_dir / "requirements.txt",
        config_dir / "mcp_config.yaml",
    ]

    for file_path in required_files:
        if not file_path.exists():
            print(f"  ✗ Missing: {file_path}")
            return False
        print(f"  ✓ Found: {file_path.name}")

    print("  ✓ All required files present")
    return True


def test_file_sizes():
    """Test that files have content"""
    print("\n[Test] File Sizes")

    mcp_server_dir = project_root / "python" / "mcp_server"

    files_to_check = {
        "server.py": 8000,  # Main server, should be substantial
        "tools.py": 15000,  # Tool implementations
        "backend_client.py": 7000,  # Backend client
        "config_loader.py": 5000,  # Config loader
        "README.md": 9000,  # Documentation
    }

    for filename, min_size in files_to_check.items():
        file_path = mcp_server_dir / filename
        actual_size = file_path.stat().st_size

        if actual_size < min_size:
            print(
                f"  ✗ {filename}: {actual_size} bytes (expected >{min_size})"
            )
            return False

        print(f"  ✓ {filename}: {actual_size} bytes")

    print("  ✓ All files have expected content")
    return True


def test_code_structure():
    """Test that Python files have expected structure"""
    print("\n[Test] Code Structure")

    mcp_server_dir = project_root / "python" / "mcp_server"

    # Test server.py has expected functions
    server_py = (mcp_server_dir / "server.py").read_text()
    expected_server_contents = [
        "def start_server",
        "@mcp.tool()",
        "async def generate_sprite",
        "async def generate_batch",
        "async def deploy_to_bevy",
        "from fastmcp import FastMCP",
    ]

    for expected in expected_server_contents:
        if expected not in server_py:
            print(f"  ✗ server.py missing: {expected}")
            return False

    print("  ✓ server.py has expected structure")

    # Test tools.py has expected classes
    tools_py = (mcp_server_dir / "tools.py").read_text()
    expected_tools_contents = [
        "class MCPTools",
        "class ValidationError",
        "async def generate_sprite",
        "async def generate_batch",
        "async def deploy_to_bevy",
        "def _validate_prompt",
        "def _validate_resolution",
    ]

    for expected in expected_tools_contents:
        if expected not in tools_py:
            print(f"  ✗ tools.py missing: {expected}")
            return False

    print("  ✓ tools.py has expected structure")

    # Test config_loader.py has expected classes
    config_py = (mcp_server_dir / "config_loader.py").read_text()
    expected_config_contents = [
        "class MCPServerConfig",
        "class BackendConfig",
        "class GenerationConfig",
        "class DeploymentConfig",
        "class ValidationConfig",
        "def load_config",
    ]

    for expected in expected_config_contents:
        if expected not in config_py:
            print(f"  ✗ config_loader.py missing: {expected}")
            return False

    print("  ✓ config_loader.py has expected structure")

    # Test backend_client.py has expected classes
    backend_py = (mcp_server_dir / "backend_client.py").read_text()
    expected_backend_contents = [
        "class BackendClient",
        "async def connect",
        "async def ping",
        "async def get_status",
        "async def generate_sprite",
        "async def list_models",
    ]

    for expected in expected_backend_contents:
        if expected not in backend_py:
            print(f"  ✗ backend_client.py missing: {expected}")
            return False

    print("  ✓ backend_client.py has expected structure")

    print("  ✓ All code files have expected structure")
    return True


def test_configuration():
    """Test configuration file"""
    print("\n[Test] Configuration File")

    config_path = project_root / "config" / "mcp_config.yaml"
    config_content = config_path.read_text()

    expected_config_sections = [
        "mcp_server:",
        "backend:",
        "generation:",
        "deployment:",
        "validation:",
        "error_handling:",
        "logging:",
        "performance:",
    ]

    for section in expected_config_sections:
        if section not in config_content:
            print(f"  ✗ Missing config section: {section}")
            return False

    print(f"  ✓ Configuration file has all sections")

    # Check specific settings
    expected_settings = [
        "zmq_endpoint:",
        "comfyui_url:",
        "default_workflow:",
        "bevy_assets_base:",
        "allowed_resolutions:",
        "allowed_styles:",
    ]

    for setting in expected_settings:
        if setting not in config_content:
            print(f"  ✗ Missing setting: {setting}")
            return False

    print(f"  ✓ Configuration has all required settings")
    return True


def test_documentation():
    """Test documentation files"""
    print("\n[Test] Documentation")

    mcp_server_dir = project_root / "python" / "mcp_server"

    readme = (mcp_server_dir / "README.md").read_text()
    expected_readme_sections = [
        "# DGX-Pixels FastMCP Server",
        "## Installation",
        "## Configuration",
        "## Usage",
        "## MCP Tools",
        "### 1. generate_sprite",
        "### 2. generate_batch",
        "### 3. deploy_to_bevy",
        "## Testing",
        "## Troubleshooting",
    ]

    for section in expected_readme_sections:
        if section not in readme:
            print(f"  ✗ README missing section: {section}")
            return False

    print("  ✓ README.md has all expected sections")

    quickstart = (mcp_server_dir / "QUICKSTART.md").read_text()
    expected_quickstart_sections = [
        "# FastMCP Server - Quick Start Guide",
        "## Prerequisites",
        "## Step 1: Install Dependencies",
        "## Step 2: Configure the Server",
        "## Step 3: Start Backend Services",
        "## Common Issues",
    ]

    for section in expected_quickstart_sections:
        if section not in quickstart:
            print(f"  ✗ QUICKSTART missing section: {section}")
            return False

    print("  ✓ QUICKSTART.md has all expected sections")
    return True


def test_integration_points():
    """Test that integration points are properly referenced"""
    print("\n[Test] Integration Points")

    mcp_server_dir = project_root / "python" / "mcp_server"

    # Check backend_client imports from workers
    backend_client = (mcp_server_dir / "backend_client.py").read_text()
    expected_imports = [
        "from message_protocol import",
        "GenerateRequest",
        "CancelRequest",
        "ListModelsRequest",
        "StatusRequest",
    ]

    for expected in expected_imports:
        if expected not in backend_client:
            print(f"  ✗ backend_client missing import: {expected}")
            return False

    print("  ✓ Backend client properly imports from workers")

    # Check server imports tools and config
    server = (mcp_server_dir / "server.py").read_text()
    if "from .config_loader import load_config" not in server:
        print("  ✗ server.py doesn't import config_loader")
        return False

    if "from .tools import MCPTools" not in server:
        print("  ✗ server.py doesn't import MCPTools")
        return False

    print("  ✓ Server properly imports dependencies")
    return True


def run_all_tests():
    """Run all structure tests"""
    print("=" * 60)
    print("WS-13: FastMCP Server Structure Validation")
    print("=" * 60)

    tests = [
        test_directory_structure,
        test_file_sizes,
        test_code_structure,
        test_configuration,
        test_documentation,
        test_integration_points,
    ]

    passed = 0
    failed = 0

    for test in tests:
        try:
            if test():
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"  ✗ Test failed with exception: {e}")
            failed += 1

    print("\n" + "=" * 60)
    print(f"Results: {passed} passed, {failed} failed")
    print("=" * 60)

    if failed == 0:
        print("\n✓ ALL STRUCTURE TESTS PASSED")
        return 0
    else:
        print(f"\n✗ {failed} TESTS FAILED")
        return 1


if __name__ == "__main__":
    exit_code = run_all_tests()
    sys.exit(exit_code)
