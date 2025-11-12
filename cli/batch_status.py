#!/usr/bin/env python3
"""
Batch Status Monitor CLI

Command-line tool for monitoring batch job status and statistics.

Usage:
    # Show all jobs
    python batch_status.py

    # Show specific job
    python batch_status.py <job_id>

    # Watch mode (auto-refresh)
    python batch_status.py --watch

    # Show statistics
    python batch_status.py --stats
"""

import argparse
import sys
import time
from pathlib import Path
from datetime import datetime

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from python.batch import BatchProcessor, BatchJobStatus


def format_duration(seconds: float) -> str:
    """Format duration in human-readable form"""
    if seconds < 60:
        return f"{seconds:.1f}s"
    elif seconds < 3600:
        minutes = seconds / 60
        return f"{minutes:.1f}m"
    else:
        hours = seconds / 3600
        return f"{hours:.1f}h"


def format_timestamp(timestamp: float) -> str:
    """Format timestamp as human-readable"""
    dt = datetime.fromtimestamp(timestamp)
    return dt.strftime("%Y-%m-%d %H:%M:%S")


def print_job_summary(job):
    """Print summary of a single job"""
    status_emoji = {
        BatchJobStatus.QUEUED: "⏳",
        BatchJobStatus.RUNNING: "▶️ ",
        BatchJobStatus.COMPLETED: "✅",
        BatchJobStatus.FAILED: "❌",
        BatchJobStatus.CANCELLED: "🚫",
    }

    emoji = status_emoji.get(job.status, "❓")

    print(f"\n{emoji} Job: {job.job_id}")
    print(f"   Status: {job.status.value}")
    print(f"   Prompts: {job.completed_prompts}/{job.total_prompts} ({job.progress()*100:.1f}%)")
    print(f"   Batch Size: {job.batch_size}")
    print(f"   Priority: {job.priority.name}")

    if job.created_at:
        print(f"   Created: {format_timestamp(job.created_at)}")

    if job.started_at:
        print(f"   Started: {format_timestamp(job.started_at)}")

    if job.completed_at:
        print(f"   Completed: {format_timestamp(job.completed_at)}")
        duration = job.completed_at - job.started_at
        print(f"   Duration: {format_duration(duration)}")

    if job.status == BatchJobStatus.RUNNING and job.started_at:
        elapsed = time.time() - job.started_at
        print(f"   Elapsed: {format_duration(elapsed)}")

    if job.output_dir:
        print(f"   Output: {job.output_dir}")

    if job.generated_images:
        print(f"   Images: {len(job.generated_images)}")

    if job.error:
        print(f"   Error: {job.error}")


def print_jobs_table(jobs):
    """Print jobs in table format"""
    if not jobs:
        print("No jobs found")
        return

    print(f"\n{'Job ID':40s} | {'Status':12s} | {'Progress':10s} | {'Batch':5s} | {'Priority':8s}")
    print("-" * 85)

    for job in jobs:
        status_emoji = {
            BatchJobStatus.QUEUED: "⏳",
            BatchJobStatus.RUNNING: "▶️",
            BatchJobStatus.COMPLETED: "✅",
            BatchJobStatus.FAILED: "❌",
            BatchJobStatus.CANCELLED: "🚫",
        }

        emoji = status_emoji.get(job.status, "❓")
        progress_str = f"{job.completed_prompts}/{job.total_prompts}"

        print(
            f"{job.job_id:40s} | "
            f"{emoji} {job.status.value:10s} | "
            f"{progress_str:10s} | "
            f"{job.batch_size:5d} | "
            f"{job.priority.name:8s}"
        )


def print_statistics(stats):
    """Print processor statistics"""
    print("\n=== Batch Processor Statistics ===")
    print(f"Uptime: {format_duration(stats['uptime_s'])}")
    print(f"Queue Size: {stats['queue_size']}")
    print(f"Active Jobs: {stats['active_jobs']}")
    print(f"Total Processed: {stats['total_processed']}")
    print(f"Total Failed: {stats['total_failed']}")

    if stats['avg_generation_time_s'] > 0:
        print(f"Avg Generation Time: {stats['avg_generation_time_s']:.2f}s")
        print(f"Throughput: {stats['throughput_per_minute']:.1f} images/min")


def main():
    parser = argparse.ArgumentParser(
        description="Monitor batch job status",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument(
        "job_id",
        nargs="?",
        help="Specific job ID to monitor",
    )

    parser.add_argument(
        "--watch",
        "-w",
        action="store_true",
        help="Watch mode (auto-refresh every 2 seconds)",
    )

    parser.add_argument(
        "--stats",
        "-s",
        action="store_true",
        help="Show processor statistics",
    )

    parser.add_argument(
        "--all",
        "-a",
        action="store_true",
        help="Show all jobs (including completed)",
    )

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

    args = parser.parse_args()

    # Initialize processor (read-only mode)
    processor = BatchProcessor(
        comfyui_host=args.host,
        comfyui_port=args.port,
    )

    try:
        processor.start()
    except Exception as e:
        print(f"Error: Failed to start processor: {e}", file=sys.stderr)
        return 1

    try:
        if args.watch:
            # Watch mode
            print("Watch mode - Press Ctrl+C to exit\n")

            while True:
                # Clear screen (ANSI escape code)
                print("\033[2J\033[H", end="")

                print(f"=== Batch Status Monitor === {datetime.now().strftime('%H:%M:%S')}")

                if args.stats:
                    stats = processor.get_statistics()
                    print_statistics(stats)

                if args.job_id:
                    # Monitor specific job
                    job = processor.get_job(args.job_id)
                    if job:
                        print_job_summary(job)
                    else:
                        print(f"\nJob not found: {args.job_id}")
                else:
                    # Show all active jobs
                    all_jobs = processor.jobs.values()

                    if args.all:
                        jobs = list(all_jobs)
                    else:
                        # Only show active/queued jobs
                        jobs = [
                            j
                            for j in all_jobs
                            if j.status
                            in (BatchJobStatus.QUEUED, BatchJobStatus.RUNNING)
                        ]

                    print_jobs_table(sorted(jobs, key=lambda j: j.created_at, reverse=True))

                time.sleep(2)

        else:
            # One-time display
            if args.stats:
                stats = processor.get_statistics()
                print_statistics(stats)

            if args.job_id:
                # Show specific job
                job = processor.get_job(args.job_id)
                if job:
                    print_job_summary(job)
                else:
                    print(f"Job not found: {args.job_id}", file=sys.stderr)
                    return 1
            else:
                # Show all jobs
                all_jobs = processor.jobs.values()

                if args.all:
                    jobs = list(all_jobs)
                else:
                    # Only show active/queued jobs
                    jobs = [
                        j
                        for j in all_jobs
                        if j.status in (BatchJobStatus.QUEUED, BatchJobStatus.RUNNING)
                    ]

                print_jobs_table(sorted(jobs, key=lambda j: j.created_at, reverse=True))

    except KeyboardInterrupt:
        print("\n\nExiting...")
        return 0

    finally:
        processor.stop()

    return 0


if __name__ == "__main__":
    sys.exit(main())
