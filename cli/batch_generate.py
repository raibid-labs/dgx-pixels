#!/usr/bin/env python3
"""
Batch Generation CLI

Command-line tool for submitting batch generation jobs.

Usage:
    # Single batch
    python batch_generate.py --prompt "pixel art warrior" --count 10

    # From file
    python batch_generate.py --prompts-file prompts.txt --batch-size 4

    # Custom workflow
    python batch_generate.py --prompts-file prompts.txt --workflow custom.json

    # Priority and model selection
    python batch_generate.py --prompt "urgent sprite" --priority urgent --model sdxl_custom.safetensors
"""

import argparse
import sys
import time
from pathlib import Path
from typing import List

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from python.batch import BatchProcessor, JobPriority, BatchJobStatus


def load_prompts_from_file(file_path: Path) -> List[str]:
    """Load prompts from a text file (one per line)"""
    prompts = []

    with open(file_path) as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith("#"):
                prompts.append(line)

    return prompts


def main():
    parser = argparse.ArgumentParser(
        description="Batch sprite generation CLI",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Generate 10 sprites from a single prompt
  %(prog)s --prompt "pixel art warrior sprite" --count 10

  # Generate from prompts file
  %(prog)s --prompts-file prompts.txt

  # High-priority batch with custom batch size
  %(prog)s --prompts-file prompts.txt --batch-size 8 --priority high

  # Custom workflow and model
  %(prog)s --prompt "sprite" --workflow custom.json --model my_lora.safetensors
        """,
    )

    # Input options
    input_group = parser.add_mutually_exclusive_group(required=True)
    input_group.add_argument(
        "--prompt",
        type=str,
        help="Single prompt to generate (use with --count)",
    )
    input_group.add_argument(
        "--prompts-file",
        type=Path,
        help="File containing prompts (one per line)",
    )

    parser.add_argument(
        "--count",
        type=int,
        default=1,
        help="Number of times to repeat --prompt (default: 1)",
    )

    # Generation options
    parser.add_argument(
        "--workflow",
        type=Path,
        default=Path("workflows/batch_optimized.json"),
        help="ComfyUI workflow JSON (default: workflows/batch_optimized.json)",
    )

    parser.add_argument(
        "--batch-size",
        type=int,
        default=1,
        choices=[1, 4, 8],
        help="Images per batch (default: 1)",
    )

    parser.add_argument(
        "--steps",
        type=int,
        default=20,
        help="Sampling steps (default: 20)",
    )

    parser.add_argument(
        "--cfg-scale",
        type=float,
        default=8.0,
        help="CFG scale (default: 8.0)",
    )

    parser.add_argument(
        "--model",
        type=str,
        help="Model checkpoint name (optional)",
    )

    parser.add_argument(
        "--lora",
        type=str,
        help="LoRA model name (optional)",
    )

    parser.add_argument(
        "--seed",
        type=int,
        help="Base seed (default: random)",
    )

    # Priority
    parser.add_argument(
        "--priority",
        type=str,
        choices=["urgent", "high", "normal", "low"],
        default="normal",
        help="Job priority (default: normal)",
    )

    # Server options
    parser.add_argument(
        "--host",
        type=str,
        default="localhost",
        help="ComfyUI host (default: localhost)",
    )

    parser.add_argument(
        "--port",
        type=int,
        default=8188,
        help="ComfyUI port (default: 8188)",
    )

    # Output options
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("outputs/batches"),
        help="Output directory (default: outputs/batches)",
    )

    parser.add_argument(
        "--wait",
        action="store_true",
        help="Wait for batch to complete",
    )

    parser.add_argument(
        "--monitor",
        action="store_true",
        help="Monitor progress (implies --wait)",
    )

    args = parser.parse_args()

    # Validate inputs
    if not args.workflow.exists():
        print(f"Error: Workflow not found: {args.workflow}", file=sys.stderr)
        return 1

    # Load prompts
    if args.prompt:
        prompts = [args.prompt] * args.count
    else:
        if not args.prompts_file.exists():
            print(f"Error: Prompts file not found: {args.prompts_file}", file=sys.stderr)
            return 1

        prompts = load_prompts_from_file(args.prompts_file)

        if not prompts:
            print("Error: No prompts found in file", file=sys.stderr)
            return 1

    print(f"Loaded {len(prompts)} prompts")

    # Map priority string to enum
    priority_map = {
        "urgent": JobPriority.URGENT,
        "high": JobPriority.HIGH,
        "normal": JobPriority.NORMAL,
        "low": JobPriority.LOW,
    }
    priority = priority_map[args.priority]

    # Initialize processor
    print(f"Connecting to ComfyUI at {args.host}:{args.port}...")
    processor = BatchProcessor(
        comfyui_host=args.host,
        comfyui_port=args.port,
        output_base_dir=args.output_dir,
    )

    try:
        processor.start()
    except Exception as e:
        print(f"Error: Failed to start processor: {e}", file=sys.stderr)
        return 1

    # Submit job
    print(f"\nSubmitting batch job:")
    print(f"  Prompts: {len(prompts)}")
    print(f"  Batch size: {args.batch_size}")
    print(f"  Steps: {args.steps}")
    print(f"  CFG scale: {args.cfg_scale}")
    print(f"  Priority: {args.priority}")
    if args.model:
        print(f"  Model: {args.model}")
    if args.lora:
        print(f"  LoRA: {args.lora}")

    try:
        job_id = processor.submit_job(
            prompts=prompts,
            workflow_path=args.workflow,
            batch_size=args.batch_size,
            priority=priority,
            model=args.model,
            lora=args.lora,
            steps=args.steps,
            cfg_scale=args.cfg_scale,
            seed_base=args.seed,
        )

        print(f"\n✅ Job submitted: {job_id}")

        # Wait for completion if requested
        if args.wait or args.monitor:
            print("\nWaiting for completion...")

            if args.monitor:
                # Monitor with progress updates
                last_progress = -1

                while True:
                    job = processor.get_job(job_id)

                    if job.status in (
                        BatchJobStatus.COMPLETED,
                        BatchJobStatus.FAILED,
                        BatchJobStatus.CANCELLED,
                    ):
                        break

                    progress = job.progress() * 100
                    if abs(progress - last_progress) >= 1.0:
                        stats = processor.get_statistics()
                        print(
                            f"\rProgress: {progress:5.1f}% "
                            f"({job.completed_prompts}/{job.total_prompts}) | "
                            f"Queue: {stats['queue_size']} | "
                            f"Active: {stats['active_jobs']}",
                            end="",
                            flush=True,
                        )
                        last_progress = progress

                    time.sleep(1)

                print()  # New line after progress

            else:
                # Wait without monitoring
                while True:
                    job = processor.get_job(job_id)

                    if job.status in (
                        BatchJobStatus.COMPLETED,
                        BatchJobStatus.FAILED,
                        BatchJobStatus.CANCELLED,
                    ):
                        break

                    time.sleep(2)

            # Show results
            job = processor.get_job(job_id)

            if job.status == BatchJobStatus.COMPLETED:
                print(f"\n✅ Job completed successfully!")
                print(f"   Generated: {len(job.generated_images)} images")
                print(f"   Duration: {job.completed_at - job.started_at:.1f}s")
                print(f"   Output: {job.output_dir}")

                # Calculate throughput
                if job.completed_at and job.started_at:
                    duration_min = (job.completed_at - job.started_at) / 60
                    throughput = len(job.generated_images) / duration_min
                    print(f"   Throughput: {throughput:.1f} images/min")

                return 0

            elif job.status == BatchJobStatus.FAILED:
                print(f"\n❌ Job failed: {job.error}", file=sys.stderr)
                return 1

            elif job.status == BatchJobStatus.CANCELLED:
                print(f"\n⚠️  Job was cancelled")
                return 1

        else:
            print("\nJob queued. Use batch_status.py to monitor progress.")
            return 0

    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
        return 130

    finally:
        processor.stop()


if __name__ == "__main__":
    sys.exit(main())
