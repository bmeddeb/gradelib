#!/usr/bin/env python3
"""
Example script demonstrating the analyze_commits functionality in gradelib.

This script:
1. Sets up the async environment
2. Creates a RepoManager with GitHub credentials
3. Clones the specified repository
4. Waits for the clone to complete
5. Analyzes commit history with high-performance parallel processing
6. Displays the full commit history in a DataFrame
"""

import asyncio
import os
from typing import Dict, List
from datetime import datetime, timezone, timedelta
import sys
import pandas as pd  # For DataFrame display

# Add the parent directory to path if running the script directly
if __name__ == "__main__":
    sys.path.insert(0, os.path.abspath(
        os.path.join(os.path.dirname(__file__), '..')))

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

    # Define the repository to analyze
    repo_url = "https://github.com/bmeddeb/SER402-Team3"

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
        # Fetch the local path for the repo from the clone tasks
        tasks = await manager.fetch_clone_tasks()
        task = tasks[repo_url]
        if not task.temp_dir:
            print(f"Error: Could not determine local path for {repo_url}")
            sys.exit(1)
        repo_path = task.temp_dir
        commits = await manager.analyze_commits(repo_path)
        print(f"Found {len(commits)} commits in the repository")

        # Process commits for DataFrame display
        processed_commits = []
        for commit in commits:
            # Convert timestamps to readable datetime objects
            author_date = datetime.fromtimestamp(
                commit['author_timestamp'],
                tz=timezone(timedelta(minutes=commit['author_offset']))
            )
            committer_date = datetime.fromtimestamp(
                commit['committer_timestamp'],
                tz=timezone(timedelta(minutes=commit['committer_offset']))
            )

            # Create a processed commit entry
            processed_commit = {
                'SHA': commit['sha'][:8],  # Short SHA
                'Author': f"{commit['author_name']}",
                'Author Email': commit['author_email'],
                'Date': author_date.strftime('%Y-%m-%d %H:%M:%S'),
                # First line of message
                'Message': commit['message'].split('\n')[0],
                'Additions': commit['additions'],
                'Deletions': commit['deletions'],
                'Is Merge': 'Yes' if commit['is_merge'] else 'No'
            }
            processed_commits.append(processed_commit)

        # Create and display DataFrame
        df = pd.DataFrame(processed_commits)

        # Print summary statistics first
        print("\nRepository Analysis Summary:")
        print(f"  Total Commits: {len(commits)}")
        print(f"  Total Authors: {df['Author'].nunique()}")
        print(f"  Total Lines Added: {df['Additions'].sum()}")
        print(f"  Total Lines Deleted: {df['Deletions'].sum()}")
        print(
            f"  Merge Commits: {df['Is Merge'].value_counts().get('Yes', 0)}")

        # Display the full commit history dataframe
        print("\nFull Commit History:")
        # Set display options to show all rows and reasonable column width
        pd.set_option('display.max_rows', None)
        pd.set_option('display.max_columns', None)
        # Adjust based on your terminal width
        pd.set_option('display.width', 1000)
        pd.set_option('display.max_colwidth', 50)  # Truncate long messages

        print(df)

        # Optionally, save to CSV
        csv_path = os.path.join(os.path.dirname(
            __file__), 'commit_history.csv')
        df.to_csv(csv_path, index=False)
        print(f"\nCommit history saved to: {csv_path}")

    except Exception as e:
        print(f"Error analyzing commits: {e}")

if __name__ == "__main__":
    # Run the async main function
    asyncio.run(main())
