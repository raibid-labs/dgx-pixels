#!/usr/bin/env python3
"""
Batch Job Cancellation CLI

Command-line tool for cancelling batch jobs.

Usage:
    # Cancel specific job
    python batch_cancel.py <job_id>

    # Cancel all queued jobs
    python batch_cancel.py --all-queued

    # Cancel all jobs
    python batch_cancel.py --all
"""

import argparse
import sys
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from python.batch import BatchProcessor, BatchJobStatus


def main():
    parser = argparse.ArgumentParser(
        description="Cancel batch jobs",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument(
        "job_id",
        nargs="?",
        help="Job ID to cancel",
    )

    parser.add_argument(
        "--all-queued",
        action="store_true",
        help="Cancel all queued jobs",
    )

    parser.add_argument(
        "--all",
        action="store_true",
        help="Cancel ALL jobs (queued and running)",
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

    # Validate arguments
    if not args.job_id and not args.all_queued and not args.all:
        parser.error("Must specify job_id, --all-queued, or --all")

    if args.job_id and (args.all_queued or args.all):
        parser.error("Cannot specify job_id with --all-queued or --all")

    # Initialize processor
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
        if args.job_id:
            # Cancel specific job
            job = processor.get_job(args.job_id)
            if not job:
                print(f"Error: Job not found: {args.job_id}", file=sys.stderr)
                return 1

            print(f"Cancelling job {args.job_id}...")

            if processor.cancel_job(args.job_id):
                print(f"✅ Job cancelled: {args.job_id}")
                return 0
            else:
                print(f"❌ Failed to cancel job (may be already completed)", file=sys.stderr)
                return 1

        elif args.all_queued or args.all:
            # Cancel multiple jobs
            all_jobs = list(processor.jobs.values())

            if args.all_queued:
                to_cancel = [j for j in all_jobs if j.status == BatchJobStatus.QUEUED]
                print(f"Cancelling {len(to_cancel)} queued jobs...")
            else:
                to_cancel = [
                    j
                    for j in all_jobs
                    if j.status in (BatchJobStatus.QUEUED, BatchJobStatus.RUNNING)
                ]
                print(f"Cancelling {len(to_cancel)} jobs...")

            cancelled_count = 0
            for job in to_cancel:
                if processor.cancel_job(job.job_id):
                    cancelled_count += 1
                    print(f"  ✅ Cancelled: {job.job_id}")

            print(f"\n✅ Cancelled {cancelled_count} jobs")
            return 0

    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
        return 130

    finally:
        processor.stop()


if __name__ == "__main__":
    sys.exit(main())
