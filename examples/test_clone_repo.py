#!/usr/bin/env python3
"""
Example demonstrating the clone and clone_all methods of GitHubProvider.

This script shows how to clone individual repositories and how
to manage multiple repositories with the GitHubProvider.

Usage:
    python test_clone_repo.py [REPO_URL]

Environment variables:
    GITHUB_USERNAME: GitHub username for authentication
    GITHUB_TOKEN: GitHub personal access token for authentication
"""
import os
import sys
import asyncio
from pathlib import Path

# Add project root to PYTHONPATH
PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from gradelib import setup_async
from github import GitHubProvider

# Default test repository
DEFAULT_REPO = "https://github.com/bmeddeb/gradelib"

async def main():
    """Demonstrate repository cloning with GitHubProvider"""
    # Get repo URL from command line if provided
    repo_url = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_REPO
    
    # Initialize the async runtime
    setup_async()
    
    # Get GitHub credentials from environment variables
    username = os.environ.get("GITHUB_USERNAME")
    token = os.environ.get("GITHUB_TOKEN")
    
    if not username or not token:
        print("⚠️ Environment variables GITHUB_USERNAME and GITHUB_TOKEN are required")
        print("Using placeholder values for demo purposes - API calls will fail")
        username = "demo_user"
        token = "demo_token"
    
    # Create a provider
    provider = GitHubProvider(username=username, token=token)
    print(f"✓ Created GitHubProvider with username: {username}")
    
    # Clone a single repository
    print(f"\nCloning repository: {repo_url}")
    try:
        await provider.clone(repo_url)
        print(f"✓ Successfully cloned {repo_url}")
    except Exception as e:
        print(f"✗ Failed to clone repository: {e}")
    
    # Check clone tasks
    print("\nFetching clone tasks...")
    try:
        tasks = await provider.fetch_clone_tasks()
        print(f"✓ Found {len(tasks)} clone tasks:")
        
        for url, task in tasks.items():
            status = task.get("status_type", "unknown")
            progress = task.get("progress")
            error = task.get("error")
            temp_dir = task.get("temp_dir")
            
            print(f"  - {url}:")
            print(f"    Status: {status}")
            if progress:
                print(f"    Progress: {progress}")
            if error:
                print(f"    Error: {error}")
            if temp_dir:
                print(f"    Directory: {temp_dir}")
    except Exception as e:
        print(f"✗ Failed to fetch clone tasks: {e}")
    
    # Test clone_all with multiple repositories
    print("\nTesting clone_all with predefined repositories...")
    multi_repos = [
        "https://github.com/bmeddeb/gradelib",
        "https://github.com/bmeddeb/SER402-Team3"
    ]
    
    # Create a new provider with multiple repos
    multi_provider = GitHubProvider(username=username, token=token, urls=multi_repos)
    print(f"✓ Created GitHubProvider with {len(multi_repos)} repositories")
    
    try:
        # Clone all repositories
        print("Cloning all repositories...")
        await multi_provider.clone_all()
        print("✓ Successfully cloned all repositories")
        
        # Check clone tasks
        tasks = await multi_provider.fetch_clone_tasks()
        print(f"✓ Found {len(tasks)} clone tasks:")
        
        for url, task in tasks.items():
            status = task.get("status_type", "unknown")
            print(f"  - {url}: {status}")
            if task.get("temp_dir"):
                print(f"    Directory: {task.get('temp_dir')}")
    except Exception as e:
        print(f"✗ Failed to clone all repositories: {e}")

if __name__ == "__main__":
    asyncio.run(main())
