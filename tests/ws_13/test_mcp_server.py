"""Integration tests for WS-13: FastMCP Server

Tests the MCP server implementation including:
- Configuration loading
- Backend client communication
- Tool implementations
- Error handling
"""

import asyncio
import json
import tempfile
import shutil
from pathlib import Path
import sys

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from python.mcp_server.config_loader import load_config, Config
from python.mcp_server.tools import MCPTools, ValidationError


class TestConfigLoader:
    """Test configuration loading"""

    def test_load_default_config(self):
        """Test loading default configuration"""
        config = load_config()

        assert config.mcp_server.name == "dgx-pixels"
        assert config.mcp_server.version == "0.1.0"
        assert "stdio" in config.mcp_server.transports

        assert config.backend.zmq_endpoint == "tcp://localhost:5555"
        assert config.backend.timeout_s == 300

        assert config.generation.default_resolution == "1024x1024"
        assert config.generation.default_steps == 30

        print("✓ Config loader test passed")

    def test_validation_config(self):
        """Test validation configuration"""
        config = load_config()

        assert config.validation.min_prompt_length == 3
        assert config.validation.max_prompt_length == 500
        assert "1024x1024" in config.validation.allowed_resolutions
        assert "pixel_art" in config.validation.allowed_styles

        print("✓ Validation config test passed")


class TestToolsValidation:
    """Test tool parameter validation"""

    def setup(self):
        """Setup test fixtures"""
        self.config = load_config()
        self.tools = MCPTools(self.config)

    async def test_validate_prompt(self):
        """Test prompt validation"""
        self.setup()

        # Valid prompt
        try:
            self.tools._validate_prompt("valid prompt text")
            print("✓ Valid prompt accepted")
        except ValidationError:
            assert False, "Valid prompt rejected"

        # Empty prompt
        try:
            self.tools._validate_prompt("")
            assert False, "Empty prompt should raise ValidationError"
        except ValidationError as e:
            assert e.field == "prompt"
            print("✓ Empty prompt rejected")

        # Too short
        try:
            self.tools._validate_prompt("ab")
            assert False, "Too short prompt should raise ValidationError"
        except ValidationError as e:
            assert e.field == "prompt"
            print("✓ Too short prompt rejected")

        # Too long
        try:
            self.tools._validate_prompt("x" * 501)
            assert False, "Too long prompt should raise ValidationError"
        except ValidationError as e:
            assert e.field == "prompt"
            print("✓ Too long prompt rejected")

    async def test_validate_resolution(self):
        """Test resolution validation"""
        self.setup()

        # Valid resolution
        size = self.tools._validate_resolution("1024x1024")
        assert size == [1024, 1024]
        print("✓ Valid resolution accepted")

        # Invalid format
        try:
            self.tools._validate_resolution("1024")
            assert False, "Invalid format should raise ValidationError"
        except ValidationError as e:
            assert e.field == "resolution"
            print("✓ Invalid format rejected")

        # Not allowed
        try:
            self.tools._validate_resolution("256x256")
            assert False, "Not allowed resolution should raise ValidationError"
        except ValidationError as e:
            assert e.field == "resolution"
            print("✓ Not allowed resolution rejected")

    async def test_validate_steps(self):
        """Test steps validation"""
        self.setup()

        # Valid steps
        try:
            self.tools._validate_steps(30)
            print("✓ Valid steps accepted")
        except ValidationError:
            assert False, "Valid steps rejected"

        # Too low
        try:
            self.tools._validate_steps(5)
            assert False, "Too low steps should raise ValidationError"
        except ValidationError as e:
            assert e.field == "steps"
            print("✓ Too low steps rejected")

        # Too high
        try:
            self.tools._validate_steps(150)
            assert False, "Too high steps should raise ValidationError"
        except ValidationError as e:
            assert e.field == "steps"
            print("✓ Too high steps rejected")

    async def test_validate_style(self):
        """Test style validation"""
        self.setup()

        # Valid style
        try:
            self.tools._validate_style("pixel_art")
            print("✓ Valid style accepted")
        except ValidationError:
            assert False, "Valid style rejected"

        # Invalid style
        try:
            self.tools._validate_style("invalid_style")
            assert False, "Invalid style should raise ValidationError"
        except ValidationError as e:
            assert e.field == "style"
            print("✓ Invalid style rejected")


class TestDeployment:
    """Test deployment functionality"""

    async def test_deploy_to_bevy(self):
        """Test deploying sprite to Bevy assets"""
        config = load_config()
        # Disable validation for testing
        config.deployment.validate_bevy_structure = False

        tools = MCPTools(config)

        # Create temporary files
        with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as tmp:
            tmp.write(b"fake png data")
            sprite_path = tmp.name

        temp_assets_dir = tempfile.mkdtemp()

        try:
            # Deploy sprite
            result = await tools.deploy_to_bevy(
                sprite_path=sprite_path,
                bevy_assets_dir=temp_assets_dir,
                sprite_name="test_sprite",
                update_manifest=True,
            )

            assert result["status"] == "success"
            assert result["manifest_updated"] is True

            # Check deployed file exists
            deployed_path = Path(result["deployed_path"])
            assert deployed_path.exists()
            print("✓ Sprite deployed successfully")

            # Check manifest
            manifest_path = Path(temp_assets_dir) / "asset_manifest.json"
            assert manifest_path.exists()

            with open(manifest_path) as f:
                manifest = json.load(f)

            assert "sprites" in manifest
            assert len(manifest["sprites"]) == 1
            assert manifest["sprites"][0]["name"] == "test_sprite"
            print("✓ Manifest updated correctly")

            # Deploy another sprite (update manifest)
            result2 = await tools.deploy_to_bevy(
                sprite_path=sprite_path,
                bevy_assets_dir=temp_assets_dir,
                sprite_name="test_sprite2",
                update_manifest=True,
            )

            assert result2["status"] == "success"

            with open(manifest_path) as f:
                manifest = json.load(f)

            assert len(manifest["sprites"]) == 2
            print("✓ Manifest append works correctly")

        finally:
            # Cleanup
            Path(sprite_path).unlink(missing_ok=True)
            shutil.rmtree(temp_assets_dir, ignore_errors=True)

    async def test_deploy_nonexistent_file(self):
        """Test deploying nonexistent file"""
        config = load_config()
        tools = MCPTools(config)

        temp_assets_dir = tempfile.mkdtemp()

        try:
            result = await tools.deploy_to_bevy(
                sprite_path="/nonexistent/file.png",
                bevy_assets_dir=temp_assets_dir,
                sprite_name="test",
            )

            assert result["status"] == "error"
            assert "not found" in result["error"].lower()
            print("✓ Nonexistent file error handled correctly")

        finally:
            shutil.rmtree(temp_assets_dir, ignore_errors=True)


async def run_tests():
    """Run all tests"""
    print("=" * 60)
    print("WS-13: FastMCP Server Integration Tests")
    print("=" * 60)

    # Config loader tests
    print("\n[Test Suite] Configuration Loading")
    config_tests = TestConfigLoader()
    config_tests.test_load_default_config()
    config_tests.test_validation_config()

    # Validation tests
    print("\n[Test Suite] Parameter Validation")
    validation_tests = TestToolsValidation()
    await validation_tests.test_validate_prompt()
    await validation_tests.test_validate_resolution()
    await validation_tests.test_validate_steps()
    await validation_tests.test_validate_style()

    # Deployment tests
    print("\n[Test Suite] Deployment Functionality")
    deployment_tests = TestDeployment()
    await deployment_tests.test_deploy_to_bevy()
    await deployment_tests.test_deploy_nonexistent_file()

    print("\n" + "=" * 60)
    print("ALL TESTS PASSED ✓")
    print("=" * 60)


if __name__ == "__main__":
    asyncio.run(run_tests())
