#!/usr/bin/env python3
"""
Example script demonstrating the analyze_branches functionality in gradelib.

This script:
1. Sets up the async environment
2. Creates a RepoManager with GitHub credentials
3. Clones the specified repositories
4. Analyzes branches in the repositories
5. Displays the branch information in a DataFrame
"""

import asyncio
import os
import sys
import pandas as pd
from typing import Dict, List

# Add the parent directory to path if running the script directly
if __name__ == "__main__":
    sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

# Import gradelib
from gradelib.gradelib import setup_async, RepoManager, CloneTask


async def monitor_cloning(manager, repo_urls):
    """Monitor and display clone progress until all repositories are cloned."""
    completed = set()
    all_done = False

    while not all_done:
        tasks = await manager.fetch_clone_tasks()
        all_done = True  # Assume all are done until we find one that isn't

        for url in repo_urls:
            if url in tasks:
                task = tasks[url]
                status = task.status

                # If we've already reported this repo as complete, skip it
                if url in completed:
                    continue

                if status.status_type == "completed":
                    print(f"\n✅ {url} cloned successfully")
                    completed.add(url)
                elif status.status_type == "failed":
                    print(f"\n❌ {url} failed: {status.error}")
                    completed.add(url)
                else:
                    all_done = False  # At least one task is still in progress
                    if status.status_type == "cloning" and status.progress is not None:
                        percent = status.progress
                        bar_length = 30
                        filled_length = int(bar_length * percent / 100)
                        bar = '█' * filled_length + '░' * (bar_length - filled_length)
                        print(f"\r⏳ {url} cloning: [{bar}] {percent}%", end='', flush=True)

        if not all_done:
            await asyncio.sleep(0.5)  # Poll every half-second


async def main():
    # Initialize the async runtime environment
    setup_async()
    
    # Get GitHub credentials from environment variables (for security)
    github_username = os.environ.get("GITHUB_USERNAME", "")
    github_token = os.environ.get("GITHUB_TOKEN", "")
    
    if not github_username or not github_token:
        print("Error: GITHUB_USERNAME and GITHUB_TOKEN environment variables must be set")
        print("Example: export GITHUB_USERNAME=yourusername")
        print("         export GITHUB_TOKEN=your_personal_access_token")
        sys.exit(1)
    
    # Define the repositories to analyze
    repo_urls = [
        "https://github.com/bmeddeb/gradelib",
        "https://github.com/PyO3/pyo3",
        "https://github.com/bmeddeb/SER402-Team3"
    ]
    
    # Create the repo manager
    manager = RepoManager(repo_urls, github_username, github_token)
    
    # Clone the repositories if needed
    print(f"Cloning {len(repo_urls)} repositories...")
    await manager.clone_all()
    
    # Monitor cloning progress
    await monitor_cloning(manager, repo_urls)
    
    # Analyze branches
    print("\nAnalyzing branches...")
    branches = await manager.analyze_branches(repo_urls)
    
    # Process and display the branch information
    all_branches = []
    for repo_url, repo_branches in branches.items():
        if isinstance(repo_branches, str):
            # This is an error message
            print(f"Error analyzing branches for {repo_url}: {repo_branches}")
            continue
            
        print(f"\nRepository: {repo_url}")
        print(f"Found {len(repo_branches)} branches")
        
        repo_name = '/'.join(repo_url.split('/')[-2:]).replace('.git', '')
        
        for branch in repo_branches:
            branch_data = {
                'Repository': repo_name,
                'Branch': branch['name'],
                'Is Remote': branch['is_remote'],
                'Remote Name': branch.get('remote_name', 'N/A'),
                'Commit ID': branch['commit_id'][:8],  # Short commit hash
                'Last Commit Message': branch['commit_message'].split('\n')[0],  # First line only
                'Author': branch['author_name'],
                'Author Email': branch['author_email'],
                'Last Commit Time': pd.to_datetime(branch['author_time'], unit='s'),
                'Is HEAD': branch['is_head'],
            }
            all_branches.append(branch_data)
    
    # Create a DataFrame from all branch records
    if all_branches:
        df = pd.DataFrame(all_branches)
        
        # Display the DataFrame
        print("\nAll Branches:")
        pd.set_option('display.max_columns', None)
        pd.set_option('display.width', 1000)
        pd.set_option('display.max_colwidth', 50)  # Truncate long messages
        print(df)
        
        # Optionally, save to CSV
        csv_path = os.path.join(os.path.dirname(__file__), 'branch_analysis.csv')
        df.to_csv(csv_path, index=False)
        print(f"\nBranch data saved to: {csv_path}")
    else:
        print("\nNo branch data found.")


if __name__ == "__main__":
    asyncio.run(main())