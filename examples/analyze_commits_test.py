#!/usr/bin/env python3
"""
Example script demonstrating the analyze_commits functionality in gradelib.

This script:
1. Sets up the async environment
2. Creates a RepoManager with GitHub credentials
3. Clones a sample repository
4. Waits for the clone to complete
5. Analyzes commit history with high-performance parallel processing
6. Displays the commit analysis results
"""

import asyncio
import os
from typing import Dict, List
from datetime import datetime, timezone, timedelta
import sys
from pprint import pprint

# Add the parent directory to path if running the script directly
if __name__ == "__main__":
    sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

# Import gradelib
from gradelib.gradelib import setup_async, RepoManager, CloneTask


async def main():
    # Initialize the async runtime environment
    setup_async()
    
    # Get GitHub credentials from environment variables (for security)
    # You can also hardcode these for testing, but don't commit credentials to version control
    github_username = os.environ.get("GITHUB_USERNAME", "")
    github_token = os.environ.get("GITHUB_TOKEN", "")
    
    if not github_username or not github_token:
        print("Error: GITHUB_USERNAME and GITHUB_TOKEN environment variables must be set")
        print("Example: export GITHUB_USERNAME=yourusername")
        print("         export GITHUB_TOKEN=your_personal_access_token")
        sys.exit(1)
    
    # Define a sample repository to analyze
    # Using a small open-source repo as an example
    repo_url = "https://github.com/python/peps.git"
    
    # Create the repo manager with the target repository
    manager = RepoManager([repo_url], github_username, github_token)
    
    # Start the clone process
    print(f"Cloning repository: {repo_url}...")
    await manager.clone(repo_url)
    
    # Wait for the clone to complete
    completed = False
    while not completed:
        tasks = await manager.fetch_clone_tasks()
        task: CloneTask = tasks[repo_url]
        
        if task.status.status_type == "completed":
            completed = True
            print("Clone completed successfully!")
        elif task.status.status_type == "cloning":
            progress = task.status.progress or 0
            print(f"Cloning in progress: {progress}%")
        elif task.status.status_type == "failed":
            print(f"Clone failed: {task.status.error}")
            sys.exit(1)
        
        if not completed:
            await asyncio.sleep(2)  # Wait 2 seconds before checking again
    
    # Analyze commit history
    print("Analyzing commit history (using parallel processing)...")
    try:
        commits = await manager.analyze_commits(repo_url)
        print(f"Found {len(commits)} commits in the repository")
        
        # Display some summary statistics
        authors = {}
        dates = []
        total_additions = 0
        total_deletions = 0
        merge_commits = 0
        
        for commit in commits:
            # Count unique authors
            author = f"{commit['author_name']} <{commit['author_email']}>"
            authors[author] = authors.get(author, 0) + 1
            
            # Convert timestamp to datetime
            commit_date = datetime.fromtimestamp(
                commit['author_timestamp'], 
                tz=timezone(timedelta(minutes=commit['author_offset']))
            )
            dates.append(commit_date)
            
            # Accumulate stats
            total_additions += commit['additions']
            total_deletions += commit['deletions']
            if commit['is_merge']:
                merge_commits += 1
        
        # Print summary statistics
        print("\nRepository Analysis Summary:")
        print(f"  Commit Date Range: {min(dates).date()} to {max(dates).date()}")
        print(f"  Number of Authors: {len(authors)}")
        print(f"  Top Authors:")
        for author, count in sorted(authors.items(), key=lambda x: x[1], reverse=True)[:5]:
            print(f"    - {author}: {count} commits")
        print(f"  Total Lines Added: {total_additions}")
        print(f"  Total Lines Deleted: {total_deletions}")
        print(f"  Merge Commits: {merge_commits} ({merge_commits/len(commits)*100:.1f}%)")
        
        # Print details of the most recent commit
        print("\nMost Recent Commit:")
        recent_commit = commits[0]  # Assuming commits are sorted with newest first
        print(f"  SHA: {recent_commit['sha']}")
        print(f"  Author: {recent_commit['author_name']} <{recent_commit['author_email']}>")
        print(f"  Date: {datetime.fromtimestamp(recent_commit['author_timestamp'], tz=timezone.utc)}")
        print(f"  Message: {recent_commit['message'][:100]}...")
        print(f"  Changes: +{recent_commit['additions']} -{recent_commit['deletions']}")
        
    except Exception as e:
        print(f"Error analyzing commits: {e}")
        
if __name__ == "__main__":
    # Run the async main function
    asyncio.run(main()) 