#!/usr/bin/env python3
"""
Test script for the enhanced static typing support in GradeLib.

This script demonstrates the use of the improved CloneStatus and CloneTask classes
and verifies that the typing works correctly with mypy or other static type checkers.
"""

import asyncio
import os
import sys
from typing import Dict, List, Optional, cast

# Add the parent directory to path for local testing
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

# Import with type annotations
from gradelib import (
    setup_async,
    RepoManager,
    CloneStatus,
    CloneTask,
    CloneStatusType  # Import the Literal type
)


async def demonstrate_typed_api() -> None:
    """Demonstrate the use of typed API."""

    # Initialize the async runtime
    setup_async()

    # Get GitHub token from environment
    github_token = os.environ.get("GITHUB_TOKEN", "")
    if not github_token:
        print("Warning: GITHUB_TOKEN environment variable not set. Using empty token.")

    # Create RepoManager with strongly typed parameters
    repo_url = "https://github.com/PyO3/pyo3.git"
    manager = RepoManager(urls=[repo_url], github_username="", github_token=github_token)

    # Start the clone process
    print(f"Starting clone of {repo_url}...")
    await manager.clone(repo_url)

    # Fetch clone tasks with proper return type
    tasks: Dict[str, CloneTask] = await manager.fetch_clone_tasks()

    # Access the CloneTask and CloneStatus with proper typing
    task = tasks[repo_url]

    # Demonstrate status_type as a Literal type
    status_type: CloneStatusType = task.status.status_type

    # Status-specific processing using type narrowing
    if status_type == "queued":
        print("Task is queued for cloning")
    elif status_type == "cloning":
        # Safe access to progress which is Optional[int]
        progress = task.status.progress
        if progress is not None:
            print(f"Cloning in progress: {progress}%")
        else:
            print("Cloning in progress without progress information")
    elif status_type == "completed":
        # Safe access to temp_dir which is Optional[str]
        if task.temp_dir:
            print(f"Clone completed at: {task.temp_dir}")
        else:
            print("Clone completed but temp_dir is not available")
    elif status_type == "failed":
        # Safe access to error which is Optional[str]
        error = task.status.error
        print(f"Clone failed with error: {error or 'Unknown error'}")

    # For testing, cancel any ongoing clone to clean up
    print("Cleaning up...")


def print_type_documentation() -> None:
    """Print the documentation for the typed classes."""
    print("\n=== CloneStatus Documentation ===")
    print(CloneStatus.__doc__)

    print("\n=== CloneTask Documentation ===")
    print(CloneTask.__doc__)

    print("\n=== RepoManager Documentation ===")
    print(RepoManager.__doc__)


if __name__ == "__main__":
    print("Testing GradeLib with improved static typing support\n")

    # Print type documentation
    print_type_documentation()

    # Run the async demonstration
    print("\n=== Running Demo ===")
    asyncio.run(demonstrate_typed_api())

    print("\nTest completed successfully!")