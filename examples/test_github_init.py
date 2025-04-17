#!/usr/bin/env python3
"""
Example demonstrating GitHubProvider initialization.

This script shows how to initialize a GitHubProvider instance
with or without initial repository URLs.

Usage:
    python test_github_init.py

Environment variables:
    GITHUB_USERNAME: GitHub username for authentication
    GITHUB_TOKEN: GitHub personal access token for authentication
"""
import os
import sys
from pathlib import Path

# Add project root to PYTHONPATH
PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from gradelib import setup_async
from github import GitHubProvider

# Test repositories
TEST_REPOS = [
    "https://github.com/bmeddeb/gradelib",
    "https://github.com/bmeddeb/SER402-Team3",
    "https://github.com/amehlhase316/survivors-spring24C"
]

def main():
    """Demonstrate GitHubProvider initialization"""
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
    
    print("\n1. Basic initialization with just credentials:")
    provider1 = GitHubProvider(username=username, token=token)
    print(f"✓ Created GitHubProvider with username: {username}")
    print("  No repositories are initially configured")
    
    print("\n2. Initialization with repositories:")
    provider2 = GitHubProvider(username=username, token=token, urls=TEST_REPOS)
    print(f"✓ Created GitHubProvider with username: {username}")
    print(f"  Initialized with {len(TEST_REPOS)} repositories:")
    for i, repo in enumerate(TEST_REPOS, 1):
        print(f"    {i}. {repo}")
    
    print("\nGitHubProvider is now ready to use for GitHub operations")
    print("The provider maintains authentication and state across API calls")

if __name__ == "__main__":
    main()
