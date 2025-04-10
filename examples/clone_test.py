#!/usr/bin/env python3
import asyncio
import os
from gradelib import gradelib

async def monitor_progress(manager):
    """Monitor and display clone progress in real-time."""
    completed = set()
    all_done = False

    while not all_done:
        tasks = await manager.fetch_clone_tasks()
        all_done = True  # Assume all are done until we find one that isn't

        for url, task in tasks.items():
            status = task.status
            repo_name = url.split('/')[-1].replace('.git', '')

            # If we've already reported this repo as complete, skip it
            if url in completed:
                continue

            if status.status_type == "completed":
                print(f"\n✅ {repo_name} cloned at {task.temp_dir}")
                completed.add(url)
            elif status.status_type == "failed":
                print(f"\n❌ {repo_name} failed: {status.error}")
                completed.add(url)
            else:
                all_done = False  # At least one task is still in progress
                if status.status_type == "cloning" and status.progress is not None:
                    percent = status.progress
                    bar_length = 30
                    filled_length = int(bar_length * percent / 100)
                    bar = '█' * filled_length + '░' * (bar_length - filled_length)
                    print(f"\r⏳ {repo_name} cloning: [{bar}] {percent}%", end='', flush=True)

        if not all_done:
            await asyncio.sleep(0.5)  # Poll every half-second

async def main():
    # Initialize the runtime
    gradelib.setup_async()

    # Get GitHub token from environment
    github_token = os.environ.get("GITHUB_TOKEN")
    if not github_token:
        print("Please set GITHUB_TOKEN environment variable")
        return

    # List of repository URLs to clone
    repositories = [
        "https://github.com/PyO3/pyo3.git",
        "https://github.com/PyO3/pyo3-async-runtimes.git",
        "https://github.com/rust-lang/rust-analyzer.git"  # Larger repo to show progress
    ]

    print(f"Will clone {len(repositories)} repositories:")
    for repo in repositories:
        repo_name = repo.split('/')[-1].replace('.git', '')
        print(f"  - {repo_name}")

    # Create the repository manager
    # Use empty username for public repos, or provide your username if needed
    manager = gradelib.RepoManager(repositories, "", github_token)

    # Start monitoring progress in a separate task
    monitor_task = asyncio.create_task(monitor_progress(manager))

    # Start the cloning process
    print("\nStarting clone operations...")
    await manager.clone_all()

    # Wait for the monitoring to finish final updates
    await monitor_task
    print("\nAll repositories have been cloned successfully!")

if __name__ == "__main__":
    asyncio.run(main())