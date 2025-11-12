#!/usr/bin/env python3
"""
Memory Profiler for SDXL Inference on DGX-Spark GB10

Profiles VRAM usage during SDXL generation to:
- Track memory allocation per model component (UNet, VAE, CLIP)
- Identify memory bottlenecks
- Validate <60GB VRAM target
- Monitor unified memory usage (GB10 shares CPU+GPU memory)
"""

import torch
import json
import time
import psutil
from typing import Dict, List, Optional, Callable, Any
from pathlib import Path
from dataclasses import dataclass, field
from contextlib import contextmanager


@dataclass
class MemorySnapshot:
    """Single memory usage snapshot"""
    timestamp: float
    stage: str
    gpu_allocated_gb: float
    gpu_reserved_gb: float
    gpu_free_gb: float
    system_used_gb: float
    system_available_gb: float
    system_percent: float

    def to_dict(self) -> Dict[str, Any]:
        return {
            'timestamp': self.timestamp,
            'stage': self.stage,
            'gpu_allocated_gb': round(self.gpu_allocated_gb, 2),
            'gpu_reserved_gb': round(self.gpu_reserved_gb, 2),
            'gpu_free_gb': round(self.gpu_free_gb, 2),
            'system_used_gb': round(self.system_used_gb, 2),
            'system_available_gb': round(self.system_available_gb, 2),
            'system_percent': round(self.system_percent, 1),
        }


@dataclass
class MemoryProfile:
    """Complete memory profile for a generation run"""
    config_name: str
    batch_size: int
    image_resolution: tuple
    snapshots: List[MemorySnapshot] = field(default_factory=list)
    peak_gpu_allocated_gb: float = 0.0
    peak_gpu_reserved_gb: float = 0.0
    peak_system_used_gb: float = 0.0
    total_duration_s: float = 0.0

    def to_dict(self) -> Dict[str, Any]:
        return {
            'config_name': self.config_name,
            'batch_size': self.batch_size,
            'image_resolution': list(self.image_resolution),
            'snapshots': [s.to_dict() for s in self.snapshots],
            'peak_gpu_allocated_gb': round(self.peak_gpu_allocated_gb, 2),
            'peak_gpu_reserved_gb': round(self.peak_gpu_reserved_gb, 2),
            'peak_system_used_gb': round(self.peak_system_used_gb, 2),
            'total_duration_s': round(self.total_duration_s, 2),
        }


class MemoryProfiler:
    """
    Profile memory usage during SDXL inference

    Tracks both GPU VRAM and system memory (important for unified memory on GB10).
    """

    def __init__(self):
        self.has_cuda = torch.cuda.is_available()
        self.profiles: List[MemoryProfile] = []
        self.current_profile: Optional[MemoryProfile] = None

    def _get_gpu_memory(self) -> tuple:
        """Get GPU memory statistics (allocated, reserved, free)"""
        if not self.has_cuda:
            return (0.0, 0.0, 0.0)

        allocated = torch.cuda.memory_allocated() / 1e9
        reserved = torch.cuda.memory_reserved() / 1e9

        # Get total memory from device properties
        props = torch.cuda.get_device_properties(0)
        total = props.total_memory / 1e9
        free = total - reserved

        return (allocated, reserved, free)

    def _get_system_memory(self) -> tuple:
        """Get system memory statistics (used, available, percent)"""
        mem = psutil.virtual_memory()
        used_gb = mem.used / 1e9
        available_gb = mem.available / 1e9
        percent = mem.percent
        return (used_gb, available_gb, percent)

    def take_snapshot(self, stage: str) -> MemorySnapshot:
        """
        Take a memory snapshot at current moment

        Args:
            stage: Description of current stage (e.g., "model_load", "inference", "vae_decode")

        Returns:
            MemorySnapshot
        """
        gpu_allocated, gpu_reserved, gpu_free = self._get_gpu_memory()
        sys_used, sys_available, sys_percent = self._get_system_memory()

        snapshot = MemorySnapshot(
            timestamp=time.time(),
            stage=stage,
            gpu_allocated_gb=gpu_allocated,
            gpu_reserved_gb=gpu_reserved,
            gpu_free_gb=gpu_free,
            system_used_gb=sys_used,
            system_available_gb=sys_available,
            system_percent=sys_percent,
        )

        if self.current_profile is not None:
            self.current_profile.snapshots.append(snapshot)

            # Update peak values
            self.current_profile.peak_gpu_allocated_gb = max(
                self.current_profile.peak_gpu_allocated_gb,
                gpu_allocated
            )
            self.current_profile.peak_gpu_reserved_gb = max(
                self.current_profile.peak_gpu_reserved_gb,
                gpu_reserved
            )
            self.current_profile.peak_system_used_gb = max(
                self.current_profile.peak_system_used_gb,
                sys_used
            )

        return snapshot

    @contextmanager
    def profile(
        self,
        config_name: str,
        batch_size: int = 1,
        image_resolution: tuple = (1024, 1024),
    ):
        """
        Context manager for profiling a generation run

        Usage:
            with profiler.profile("baseline", batch_size=1):
                # Run generation
                profiler.take_snapshot("start")
                model.load()
                profiler.take_snapshot("model_loaded")
                output = model.generate()
                profiler.take_snapshot("complete")
        """
        # Start new profile
        self.current_profile = MemoryProfile(
            config_name=config_name,
            batch_size=batch_size,
            image_resolution=image_resolution,
        )

        # Clear GPU cache before profiling
        if self.has_cuda:
            torch.cuda.empty_cache()
            torch.cuda.synchronize()

        start_time = time.time()

        try:
            yield self.current_profile
        finally:
            # Finalize profile
            end_time = time.time()
            self.current_profile.total_duration_s = end_time - start_time
            self.profiles.append(self.current_profile)
            self.current_profile = None

    def print_snapshot(self, snapshot: MemorySnapshot) -> None:
        """Print snapshot to console"""
        print(f"[MEMORY] {snapshot.stage:20s} | "
              f"GPU: {snapshot.gpu_allocated_gb:5.1f}GB allocated, "
              f"{snapshot.gpu_reserved_gb:5.1f}GB reserved | "
              f"System: {snapshot.system_used_gb:5.1f}GB ({snapshot.system_percent:.0f}%)")

    def print_profile_summary(self, profile: MemoryProfile) -> None:
        """Print profile summary"""
        print(f"\n=== Memory Profile: {profile.config_name} ===")
        print(f"Batch size: {profile.batch_size}")
        print(f"Resolution: {profile.image_resolution}")
        print(f"Duration: {profile.total_duration_s:.2f}s")
        print(f"Peak GPU allocated: {profile.peak_gpu_allocated_gb:.2f} GB")
        print(f"Peak GPU reserved: {profile.peak_gpu_reserved_gb:.2f} GB")
        print(f"Peak system memory: {profile.peak_system_used_gb:.2f} GB")

        print(f"\nSnapshots ({len(profile.snapshots)}):")
        for snapshot in profile.snapshots:
            self.print_snapshot(snapshot)

    def save_profile(self, profile: MemoryProfile, output_path: Path) -> None:
        """Save profile to JSON file"""
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, 'w') as f:
            json.dump(profile.to_dict(), f, indent=2)
        print(f"[INFO] Profile saved to {output_path}")

    def save_all_profiles(self, output_path: Path) -> None:
        """Save all profiles to JSON file"""
        output_path.parent.mkdir(parents=True, exist_ok=True)
        data = {
            'profiles': [p.to_dict() for p in self.profiles],
            'summary': {
                'total_profiles': len(self.profiles),
                'configs_tested': [p.config_name for p in self.profiles],
            }
        }
        with open(output_path, 'w') as f:
            json.dump(data, f, indent=2)
        print(f"[INFO] All profiles saved to {output_path}")

    def compare_profiles(
        self,
        profile_names: Optional[List[str]] = None
    ) -> None:
        """Print comparison of multiple profiles"""
        if profile_names is None:
            profiles_to_compare = self.profiles
        else:
            profiles_to_compare = [
                p for p in self.profiles if p.config_name in profile_names
            ]

        if not profiles_to_compare:
            print("[WARNING] No profiles to compare")
            return

        print("\n=== Memory Profile Comparison ===")
        print(f"{'Config':20s} | {'Peak GPU (GB)':12s} | {'Peak Sys (GB)':12s} | {'Duration (s)':12s}")
        print("-" * 70)

        for profile in profiles_to_compare:
            print(f"{profile.config_name:20s} | "
                  f"{profile.peak_gpu_allocated_gb:12.2f} | "
                  f"{profile.peak_system_used_gb:12.2f} | "
                  f"{profile.total_duration_s:12.2f}")


def create_memory_report(
    profiler: MemoryProfiler,
    output_dir: Path,
) -> None:
    """
    Create comprehensive memory report

    Generates:
    - Individual profile JSONs
    - Comparison report
    - Summary statistics
    """
    output_dir.mkdir(parents=True, exist_ok=True)

    # Save individual profiles
    for profile in profiler.profiles:
        filename = f"memory_{profile.config_name}_batch{profile.batch_size}.json"
        profiler.save_profile(profile, output_dir / filename)

    # Save comparison
    profiler.save_all_profiles(output_dir / "memory_comparison.json")

    # Print comparison
    profiler.compare_profiles()


if __name__ == "__main__":
    # Self-test: verify memory tracking works
    print("=== Memory Profiler Self-Test ===\n")

    profiler = MemoryProfiler()

    # Test snapshot
    with profiler.profile("test_profile", batch_size=1):
        snapshot1 = profiler.take_snapshot("start")
        profiler.print_snapshot(snapshot1)

        # Allocate some memory
        if torch.cuda.is_available():
            dummy_tensor = torch.randn(1024, 1024, 1024, device='cuda')

        snapshot2 = profiler.take_snapshot("allocated")
        profiler.print_snapshot(snapshot2)

        # Free memory
        if torch.cuda.is_available():
            del dummy_tensor
            torch.cuda.empty_cache()

        snapshot3 = profiler.take_snapshot("freed")
        profiler.print_snapshot(snapshot3)

    # Print summary
    if profiler.profiles:
        profiler.print_profile_summary(profiler.profiles[0])

    print("\n✅ Self-test complete")
