"""Test client for FastMCP server

This script tests all MCP tools by calling them and validating responses.
It helps verify that the MCP server is working correctly before integrating
with Bevy.

Usage:
    python -m python.mcp_server.test_client
"""

import asyncio
import sys
import json
from pathlib import Path

# Note: This is a simplified test client
# In production, you'd use the official MCP client library


class SimpleMCPClient:
    """Simplified MCP client for testing

    This is a basic implementation for testing purposes.
    For production, use the official MCP client library.
    """

    def __init__(self):
        self.tools = None

    async def connect(self):
        """Connect to MCP server"""
        # Import tools directly for testing
        from .config_loader import load_config
        from .tools import MCPTools

        config = load_config()
        self.tools = MCPTools(config)
        print("Connected to MCP server (direct import for testing)")

    async def call_tool(self, tool_name: str, params: dict):
        """Call an MCP tool

        Args:
            tool_name: Name of the tool
            params: Tool parameters

        Returns:
            Tool result
        """
        if tool_name == "generate_sprite":
            return await self.tools.generate_sprite(**params)
        elif tool_name == "generate_batch":
            return await self.tools.generate_batch(**params)
        elif tool_name == "deploy_to_bevy":
            return await self.tools.deploy_to_bevy(**params)
        else:
            raise ValueError(f"Unknown tool: {tool_name}")


async def test_generate_sprite():
    """Test generate_sprite tool"""
    print("\n" + "=" * 60)
    print("TEST: generate_sprite")
    print("=" * 60)

    client = SimpleMCPClient()
    await client.connect()

    # Test 1: Basic generation
    print("\n[Test 1] Basic sprite generation...")
    result = await client.call_tool(
        "generate_sprite",
        {
            "prompt": "medieval knight with sword and shield",
            "style": "pixel_art",
            "resolution": "1024x1024",
        },
    )

    print(f"Status: {result['status']}")
    if result["status"] == "success":
        print(f"Job ID: {result['job_id']}")
        print(f"Output path: {result['output_path']}")
        print(f"Generation time: {result['generation_time']:.2f}s")
        print("✓ Test passed")
    else:
        print(f"Error: {result.get('error')}")
        print("✗ Test failed")

    # Test 2: With custom parameters
    print("\n[Test 2] Generation with custom parameters...")
    result = await client.call_tool(
        "generate_sprite",
        {
            "prompt": "wizard casting spell",
            "style": "16bit",
            "resolution": "512x512",
            "steps": 20,
            "cfg_scale": 8.0,
        },
    )

    print(f"Status: {result['status']}")
    if result["status"] == "success":
        print(f"Job ID: {result['job_id']}")
        print("✓ Test passed")
    else:
        print(f"Error: {result.get('error')}")
        print("✗ Test failed")

    # Test 3: Validation error (empty prompt)
    print("\n[Test 3] Validation error handling...")
    result = await client.call_tool(
        "generate_sprite",
        {
            "prompt": "",  # Invalid: empty prompt
            "style": "pixel_art",
        },
    )

    print(f"Status: {result['status']}")
    if result["status"] == "error":
        print(f"Error (expected): {result.get('error')}")
        print("✓ Test passed (error correctly handled)")
    else:
        print("✗ Test failed (should have returned error)")

    # Test 4: Invalid style
    print("\n[Test 4] Invalid style handling...")
    result = await client.call_tool(
        "generate_sprite",
        {
            "prompt": "knight sprite",
            "style": "invalid_style",  # Invalid style
        },
    )

    print(f"Status: {result['status']}")
    if result["status"] == "error":
        print(f"Error (expected): {result.get('error')}")
        print("✓ Test passed (error correctly handled)")
    else:
        print("✗ Test failed (should have returned error)")


async def test_generate_batch():
    """Test generate_batch tool"""
    print("\n" + "=" * 60)
    print("TEST: generate_batch")
    print("=" * 60)

    client = SimpleMCPClient()
    await client.connect()

    # Test 1: Batch generation
    print("\n[Test 1] Batch generation with 3 prompts...")
    result = await client.call_tool(
        "generate_batch",
        {
            "prompts": [
                "knight character sprite",
                "wizard character sprite",
                "archer character sprite",
            ],
            "style": "pixel_art",
            "resolution": "1024x1024",
        },
    )

    print(f"Status: {result['status']}")
    print(f"Successful: {result.get('successful', 0)}")
    print(f"Failed: {result.get('failed', 0)}")
    print(f"Total time: {result.get('total_time', 0):.2f}s")

    if result["status"] in ["success", "partial"]:
        print(f"Job IDs: {result.get('job_ids')}")
        print("✓ Test passed")
    else:
        print(f"Error: {result.get('error')}")
        print("✗ Test failed")

    # Test 2: Empty batch
    print("\n[Test 2] Empty batch handling...")
    result = await client.call_tool(
        "generate_batch",
        {
            "prompts": [],  # Invalid: empty list
            "style": "pixel_art",
        },
    )

    print(f"Status: {result['status']}")
    if result["status"] == "error":
        print(f"Error (expected): {result.get('error')}")
        print("✓ Test passed (error correctly handled)")
    else:
        print("✗ Test failed (should have returned error)")

    # Test 3: Batch too large
    print("\n[Test 3] Batch size limit handling...")
    result = await client.call_tool(
        "generate_batch",
        {
            "prompts": [f"sprite {i}" for i in range(25)],  # Exceeds max (20)
            "style": "pixel_art",
        },
    )

    print(f"Status: {result['status']}")
    if result["status"] == "error":
        print(f"Error (expected): {result.get('error')}")
        print("✓ Test passed (error correctly handled)")
    else:
        print("✗ Test failed (should have returned error)")


async def test_deploy_to_bevy():
    """Test deploy_to_bevy tool"""
    print("\n" + "=" * 60)
    print("TEST: deploy_to_bevy")
    print("=" * 60)

    client = SimpleMCPClient()
    await client.connect()

    # Create a temporary test sprite file
    import tempfile
    import shutil

    with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as tmp:
        tmp.write(b"fake png data")
        test_sprite_path = tmp.name

    # Create temporary Bevy assets directory
    temp_assets_dir = tempfile.mkdtemp()

    try:
        # Test 1: Basic deployment
        print("\n[Test 1] Basic deployment...")
        result = await client.call_tool(
            "deploy_to_bevy",
            {
                "sprite_path": test_sprite_path,
                "bevy_assets_dir": temp_assets_dir,
                "sprite_name": "test_knight",
            },
        )

        print(f"Status: {result['status']}")
        if result["status"] == "success":
            print(f"Deployed path: {result['deployed_path']}")
            print(f"Manifest updated: {result['manifest_updated']}")

            # Verify file exists
            deployed_path = Path(result["deployed_path"])
            if deployed_path.exists():
                print("✓ Test passed (file deployed)")
            else:
                print("✗ Test failed (file not found)")
        else:
            print(f"Error: {result.get('error')}")
            print("✗ Test failed")

        # Test 2: Deployment with manifest update
        print("\n[Test 2] Deployment with manifest...")
        result = await client.call_tool(
            "deploy_to_bevy",
            {
                "sprite_path": test_sprite_path,
                "bevy_assets_dir": temp_assets_dir,
                "sprite_name": "test_wizard",
                "update_manifest": True,
            },
        )

        print(f"Status: {result['status']}")
        if result["status"] == "success":
            manifest_path = Path(temp_assets_dir) / "asset_manifest.json"
            if manifest_path.exists():
                with open(manifest_path) as f:
                    manifest = json.load(f)
                print(f"Manifest entries: {len(manifest.get('sprites', []))}")
                print("✓ Test passed (manifest updated)")
            else:
                print("✗ Test failed (manifest not found)")
        else:
            print(f"Error: {result.get('error')}")
            print("✗ Test failed")

        # Test 3: File not found
        print("\n[Test 3] File not found handling...")
        result = await client.call_tool(
            "deploy_to_bevy",
            {
                "sprite_path": "/nonexistent/file.png",
                "bevy_assets_dir": temp_assets_dir,
                "sprite_name": "test",
            },
        )

        print(f"Status: {result['status']}")
        if result["status"] == "error":
            print(f"Error (expected): {result.get('error')}")
            print("✓ Test passed (error correctly handled)")
        else:
            print("✗ Test failed (should have returned error)")

    finally:
        # Cleanup
        Path(test_sprite_path).unlink(missing_ok=True)
        shutil.rmtree(temp_assets_dir, ignore_errors=True)


async def run_all_tests():
    """Run all test suites"""
    print("DGX-Pixels MCP Server Test Suite")
    print("=" * 60)

    try:
        await test_generate_sprite()
        await test_generate_batch()
        await test_deploy_to_bevy()

        print("\n" + "=" * 60)
        print("ALL TESTS COMPLETED")
        print("=" * 60)

    except Exception as e:
        print(f"\n✗ Test suite failed with exception: {e}")
        import traceback

        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(run_all_tests())
