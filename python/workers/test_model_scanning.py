#!/usr/bin/env python3
"""Test script for model directory scanning functionality

This script tests the new _scan_model_directory and _handle_list_models methods
in zmq_server.py without starting the full ZeroMQ server.
"""

import sys
from pathlib import Path
from typing import List

# Import the module
try:
    from message_protocol import ModelType, ModelInfo
    from zmq_server import ZmqServer
except ImportError as e:
    print(f"Error importing modules: {e}")
    sys.exit(1)


def test_model_scanning():
    """Test the model scanning functionality"""
    print("=" * 70)
    print("Testing Model Directory Scanning")
    print("=" * 70)
    print()

    # Create a server instance (without starting it)
    server = ZmqServer()

    # Display configured directories
    print("Model Directories:")
    print(f"  Base:        {server.COMFYUI_BASE}")
    print(f"  Checkpoints: {server.CHECKPOINT_DIR}")
    print(f"  LoRAs:       {server.LORA_DIR}")
    print(f"  VAEs:        {server.VAE_DIR}")
    print()

    # Check directory existence
    print("Directory Status:")
    for name, path in [
        ("Checkpoints", server.CHECKPOINT_DIR),
        ("LoRAs", server.LORA_DIR),
        ("VAEs", server.VAE_DIR),
    ]:
        exists = "✓" if path.exists() else "✗"
        print(f"  {exists} {name}: {path}")
    print()

    # Test scanning each directory
    print("Scanning Directories:")
    print("-" * 70)

    checkpoints = server._scan_model_directory(
        server.CHECKPOINT_DIR, ModelType.CHECKPOINT
    )
    print(f"Checkpoints: {len(checkpoints)} found")
    for model in checkpoints[:5]:  # Show first 5
        print(f"  - {model.name} ({model.size_mb} MB)")
    if len(checkpoints) > 5:
        print(f"  ... and {len(checkpoints) - 5} more")

    loras = server._scan_model_directory(server.LORA_DIR, ModelType.LORA)
    print(f"\nLoRAs: {len(loras)} found")
    for model in loras[:5]:
        print(f"  - {model.name} ({model.size_mb} MB)")
    if len(loras) > 5:
        print(f"  ... and {len(loras) - 5} more")

    vaes = server._scan_model_directory(server.VAE_DIR, ModelType.VAE)
    print(f"\nVAEs: {len(vaes)} found")
    for model in vaes[:5]:
        print(f"  - {model.name} ({model.size_mb} MB)")
    if len(vaes) > 5:
        print(f"  ... and {len(vaes) - 5} more")

    print()
    print("-" * 70)

    # Test the full list_models handler
    print("Testing _handle_list_models():")
    response = server._handle_list_models()
    print(f"Total models: {len(response.models)}")
    print()

    # Group by type
    by_type = {}
    for model in response.models:
        type_name = model.model_type.value
        by_type.setdefault(type_name, []).append(model)

    for type_name, models in sorted(by_type.items()):
        print(f"{type_name.upper()}: {len(models)}")
        for model in models[:3]:
            print(f"  - {model.name} ({model.size_mb} MB)")
        if len(models) > 3:
            print(f"  ... and {len(models) - 3} more")
        print()

    # Verify sorting
    print("Verifying alphabetical sorting:")
    sorted_names = [m.name.lower() for m in response.models]
    is_sorted = sorted_names == sorted(sorted_names)
    print(f"  {'✓' if is_sorted else '✗'} Models are sorted alphabetically")
    print()

    # Summary
    print("=" * 70)
    print("Test Complete")
    print("=" * 70)
    print(f"Total models found: {len(response.models)}")
    print(f"  Checkpoints: {len(checkpoints)}")
    print(f"  LoRAs: {len(loras)}")
    print(f"  VAEs: {len(vaes)}")
    print()

    if len(response.models) == 0:
        print("⚠️  WARNING: No models found!")
        print("   Make sure ComfyUI is installed and has models in:")
        print(f"   {server.COMFYUI_BASE}/models/")
        return False
    else:
        print("✓ Model scanning working correctly!")
        return True


if __name__ == "__main__":
    success = test_model_scanning()
    sys.exit(0 if success else 1)
