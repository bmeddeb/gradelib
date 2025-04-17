#!/usr/bin/env python3
"""
Example demonstrating the analyze_branches method of GitHubProvider.

This script shows how to analyze branch information in Git repositories,
including details about local and remote branches.

Usage:
    python test_analyze_branches.py [REPO_URL...]

Environment variables:
    GITHUB_USERNAME: GitHub username for authentication
    GITHUB_TOKEN: GitHub personal access token for authentication
"""
import os
import sys
import asyncio
import datetime
from pathlib import Path

# Add project root to PYTHONPATH
PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from gradelib import setup_async
from github import GitHubProvider

# Default test repositories
DEFAULT_REPOS = [
    "https://github.com/bmeddeb/gradelib",
    "https://github.com/bmeddeb/SER402-Team3",
    "https://github.com/amehlhase316/survivors-spring24C"
]

async def main():
    """Demonstrate branch analysis with GitHubProvider"""
    # Get repo URLs from command line if provided
    repo_urls = sys.argv[1:] if len(sys.argv) > 1 else DEFAULT_REPOS
    
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
    
    # Clone all repositories first
    print(f"\nCloning {len(repo_urls)} repositories...")
    for repo_url in repo_urls:
        try:
            print(f"  Cloning {repo_url}...")
            await provider.clone(repo_url)
            print(f"  ✓ Cloned {repo_url}")
        except Exception as e:
            print(f"  ✗ Failed to clone {repo_url}: {e}")
    
    # Analyze branches
    print(f"\nAnalyzing branches in {len(repo_urls)} repositories...")
    try:
        branch_results = await provider.analyze_branches(repo_urls)
        print(f"✓ Completed branch analysis for {len(branch_results)} repositories")
        
        # Process each repository
        for repo_url, branches in branch_results.items():
            repo_name = repo_url.split('/')[-1]
            print(f"\nRepository: {repo_name} ({repo_url})")
            
            if isinstance(branches, list):
                # Count different types of branches
                total_branches = len(branches)
                local_branches = sum(1 for b in branches if not b['is_remote'])
                remote_branches = sum(1 for b in branches if b['is_remote'])
                
                print(f"  Total branches: {total_branches}")
                print(f"  Local branches: {local_branches}")
                print(f"  Remote branches: {remote_branches}")
                
                # Find HEAD branch
                head_branch = next((b for b in branches if b['is_head']), None)
                if head_branch:
                    print(f"  HEAD points to: {head_branch['name']}")
                    
                    # Show HEAD commit details
                    commit_time = datetime.datetime.fromtimestamp(head_branch['author_time'])
                    commit_time_str = commit_time.strftime('%Y-%m-%d %H:%M:%S')
                    
                    print(f"  Latest commit: {head_branch['commit_id'][:7]}")
                    print(f"  Commit author: {head_branch['author_name']} <{head_branch['author_email']}>")
                    print(f"  Commit time: {commit_time_str}")
                    print(f"  Commit message: {head_branch['commit_message'].splitlines()[0]}")
                
                # List all branches
                print("\n  Branch list:")
                print("  Name                 | Type   | Latest Commit       | Author")
                print("  ---------------------|--------|---------------------|------------------")
                
                # Sort branches: HEAD first, then local, then remote
                sorted_branches = sorted(branches, key=lambda b: (
                    0 if b['is_head'] else (1 if not b['is_remote'] else 2),
                    b['name']
                ))
                
                for branch in sorted_branches:
                    # Format for display
                    name = branch['name'][:19].ljust(19)
                    branch_type = "HEAD" if branch['is_head'] else ("local" if not branch['is_remote'] else "remote")
                    branch_type = branch_type.ljust(6)
                    
                    # Format commit time
                    commit_time = datetime.datetime.fromtimestamp(branch['author_time'])
                    commit_time_str = commit_time.strftime('%Y-%m-%d').ljust(19)
                    
                    author = branch['author_name'][:18].ljust(18)
                    
                    print(f"  {name} | {branch_type} | {commit_time_str} | {author}")
                
                # Branch activity analysis
                if branches:
                    # Find most recently updated branch
                    most_recent = max(branches, key=lambda b: b['author_time'])
                    recent_time = datetime.datetime.fromtimestamp(most_recent['author_time'])
                    recent_time_str = recent_time.strftime('%Y-%m-%d %H:%M:%S')
                    
                    print(f"\n  Most recently updated: {most_recent['name']} ({recent_time_str})")
                    
                    # Find oldest branch
                    oldest = min(branches, key=lambda b: b['author_time'])
                    oldest_time = datetime.datetime.fromtimestamp(oldest['author_time'])
                    oldest_time_str = oldest_time.strftime('%Y-%m-%d %H:%M:%S')
                    
                    print(f"  Oldest updated: {oldest['name']} ({oldest_time_str})")
                    
                    # Calculate time span
                    time_span = recent_time - oldest_time
                    print(f"  Repository time span: {time_span.days} days")
            else:
                # Error occurred for this repository
                print(f"  ✗ Error analyzing branches: {branches}")
        
    except Exception as e:
        print(f"✗ Failed to analyze branches: {e}")

if __name__ == "__main__":
    asyncio.run(main())
