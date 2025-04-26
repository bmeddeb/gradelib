#!/usr/bin/env python3
import asyncio
import os
import sys
import time
from typing import Dict, List, Union
import pandas as pd
import gradelib.gradelib as gd

PANDAS_AVAILABLE = True

# Type alias for clarity based on stub file
BlameResult = Dict[str, Union[List[Dict[str, Union[str, int]]], str]]

# --- Configuration ---
# Get credentials from environment variables for security
GITHUB_USERNAME = os.environ.get("GITHUB_USERNAME")
GITHUB_TOKEN = os.environ.get("GITHUB_TOKEN")

# Repositories to clone for the test
REPOS_TO_CLONE = [
    "https://github.com/octocat/Spoon-Knife.git",
    "https://github.com/pallets/flask.git",
    "https://github.com/nonexistent-user-abc/nosuchrepo-xyz.git"  # Invalid URL
]

# Files to blame in the 'flask' repo (relative paths)
# Use paths known to exist in the flask repo, plus some error cases
FILES_TO_BLAME_IN_FLASK = [
    "README.rst",               # Exists
    "src/flask/app.py",         # Exists
    "non_existent_file.txt",    # Does not exist in repo filesystem
    ".gitattributes"            # Exists, check blame on config-like file
]
# URL to identify repo for blame
FLASK_REPO_URL = "https://github.com/pallets/flask.git"

# How often to poll for status updates (in seconds)
POLL_INTERVAL = 2

# --- Helper Functions ---


async def monitor_cloning(manager: gd.RepoManager) -> bool:
    """Monitors cloning progress until all tasks are finished. Returns True if FLASK_REPO_URL completed."""
    print("\n--- Monitoring Cloning Progress ---")
    all_tasks_finished = False
    final_task_states: dict[str, gd.CloneTask] = {}
    flask_repo_completed = False

    while not all_tasks_finished:
        try:
            # Use type hint from stub file if available, otherwise rely on runtime check
            current_tasks: dict[str, gd.CloneTask] = await manager.fetch_clone_tasks()
            final_task_states = current_tasks
            print(f"\n[{time.strftime('%H:%M:%S')}] Checking clone status...")
            all_tasks_finished = True  # Assume finished until proven otherwise

            for url, task in current_tasks.items():
                # Ensure we have the status object (should always be present)
                if not hasattr(task, 'status'):
                    print(
                        f"  - {url}: Error - Task object missing 'status' attribute.")
                    continue  # Skip this task for status check

                status_obj = task.status
                status_line = f"  - {url}: Status={status_obj.status_type}"

                if status_obj.status_type == "cloning" and status_obj.progress is not None:
                    status_line += f" ({status_obj.progress}%)"
                    all_tasks_finished = False  # Still working
                elif status_obj.status_type == "queued":
                    status_line += " (Waiting...)"
                    all_tasks_finished = False  # Still working
                elif status_obj.status_type == "failed":
                    status_line += f" - Error: {status_obj.error}"
                    # Failed is considered a final state
                elif status_obj.status_type == "completed":
                    status_line += f" - Path: {task.temp_dir}"
                    # Completed is a final state
                    if url == FLASK_REPO_URL:
                        flask_repo_completed = True  # Mark our target repo as OK
                else:
                    # Any other unknown state means we keep polling
                    all_tasks_finished = False
                    status_line += " (Unknown State)"

                print(status_line)

            if all_tasks_finished:
                print("\nCloning polling complete for all tasks (reached final state).")
                break

            await asyncio.sleep(POLL_INTERVAL)

        except Exception as e:
            print(f"\nError fetching clone tasks: {e}")
            print(f"Will retry after {POLL_INTERVAL * 2} seconds...")
            # Potentially add a retry limit here
            await asyncio.sleep(POLL_INTERVAL * 2)

    return flask_repo_completed


async def run_bulk_blame(manager: gd.RepoManager, repo_path: str):
    """Runs and processes the bulk blame operation."""
    print("\n--- Running Bulk Blame ---")
    print(f"Target Repo Path: {repo_path}")
    print(f"Files to Blame: {FILES_TO_BLAME_IN_FLASK}")

    try:
        blame_results: BlameResult = await manager.bulk_blame(
            repo_path=repo_path,
            file_paths=FILES_TO_BLAME_IN_FLASK
        )

        print("\nBlame Results Received:")
        for file_path, result in blame_results.items():
            print(f"\n  File: {file_path}")
            if isinstance(result, str):
                # It's an error string from the Rust side for this specific file
                print(f"    Error: {result}")
            elif isinstance(result, list):
                # It's a list of blame line dictionaries
                print(f"    Blamed lines returned: {len(result)}")
                if result:
                    # Demonstrate converting to Pandas DataFrame if pandas is available
                    if PANDAS_AVAILABLE:
                        try:
                            df = pd.DataFrame.from_records(result)
                            print(
                                "    DataFrame conversion successful. Sample (first 5 lines):")
                            print(df.head().to_string())
                            # Example analysis:
                            # print("\n    Author Counts:")
                            # print(df['author_name'].value_counts())
                        except Exception as e:
                            print(
                                f"    Error converting blame results to Pandas DataFrame: {e}")
                    else:
                        # Pandas not installed, just show first few raw dicts
                        print("    Sample blame data (first 2 lines):")
                        for i, line_data in enumerate(result[:2]):
                            print(f"      Line {i+1}: {line_data}")
                else:
                    print(
                        "    (Received empty list - file might be empty, binary, or have no blame info)")
            else:
                print(
                    f"    Error: Unexpected result type received: {type(result)}")

    except ValueError as e:
        # This catches the PyErr::new::<PyValueError, _> raised in lib.rs for overall errors
        print(
            f"\nError running bulk_blame (e.g., repo not found/completed?): {e}")
    except Exception as e:
        # Catch any other unexpected exceptions during the call
        print(
            f"\nAn unexpected Python error occurred during bulk_blame call: {e}")


# --- Main Execution ---

async def main():
    """Main async function to orchestrate tests."""
    print("--- Repository Cloning & Blame Test ---")
    if not GITHUB_USERNAME or not GITHUB_TOKEN:
        print("\nError: Please set GITHUB_USERNAME and GITHUB_TOKEN environment variables.")
        print("       (Needed for authentication during git operations)")
        return

    # 1. Initialize Runtime
    # Check if needed based on pyo3 async integration method used
    print("\nInitializing async runtime (if required by backend)...")
    try:
        gd.setup_async()
        print("Runtime setup call completed.")
    except Exception as e:
        print(
            f"Warning: Error during runtime initialization (might be optional): {e}")
        # Continue execution even if setup_async fails or is not needed

    # 2. Create Manager
    print("\nCreating RepoManager...")
    try:
        manager = gd.RepoManager(
            urls=REPOS_TO_CLONE,
            github_username=GITHUB_USERNAME,
            github_token=GITHUB_TOKEN,
        )
        print(f"Manager created for {len(REPOS_TO_CLONE)} repositories.")
    except Exception as e:
        print(f"Fatal Error creating RepoManager: {e}")
        return

    # 3. Start Cloning
    print(f"\nIssuing clone_all() command...")
    try:
        await manager.clone_all()
        print("clone_all() command issued successfully.")
    except Exception as e:
        print(f"Error issuing clone_all command: {e}")
        # Decide whether to proceed or exit if issuing command fails
        # For testing, we might still want to monitor existing tasks if any started

    # 4. Monitor Cloning
    flask_cloned_ok = await monitor_cloning(manager)

    # 5. Run Bulk Blame (if target repo cloned successfully)
    if flask_cloned_ok:
        # Fetch the local path for the flask repo from the clone tasks
        tasks = await manager.fetch_clone_tasks()
        flask_task = tasks.get(FLASK_REPO_URL)
        if flask_task and flask_task.temp_dir:
            await run_bulk_blame(manager, flask_task.temp_dir)
        else:
            print(
                f"\nCould not determine local path for '{FLASK_REPO_URL}'. Skipping bulk blame.")
    else:
        print(
            f"\nSkipping bulk blame because target repo '{FLASK_REPO_URL}' did not complete successfully.")

    print("\n--- Test Script Finished ---")
    print("Note: Temporary directories should be cleaned up automatically by Rust.")

if __name__ == "__main__":
    if sys.platform == "win32":
        # Necessary for asyncio on Windows in some environments
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())

    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nTest interrupted by user.")
    except Exception as e:
        print(f"\nAn unexpected error occurred in the main execution: {e}")
