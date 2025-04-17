#!/usr/bin/env python3
"""
Example script demonstrating the fetch_pull_requests functionality in gradelib.

This script:
1. Sets up the async environment
2. Creates a RepoManager with GitHub credentials
3. Fetches pull request information for specified repositories
4. Displays the pull request data in a DataFrame
"""

import asyncio
import os
import sys
import pandas as pd
from datetime import datetime
from typing import Dict, List

# Add the parent directory to path if running the script directly
if __name__ == "__main__":
    sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

# Import gradelib
from gradelib.gradelib import setup_async, RepoManager


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
    ]
    
    # Create the repo manager
    manager = RepoManager(repo_urls, github_username, github_token)
    
    # Fetch pull request information
    print(f"Fetching pull request information for {len(repo_urls)} repositories...")
    try:
        # You can specify state=None (default), "open", "closed", or "all"
        pull_requests = await manager.fetch_pull_requests(repo_urls, state="all")
        
        # Process and display the pull request data
        all_prs = []
        total_pr_count = 0
        
        for repo_url, repo_result in pull_requests.items():
            repo_name = '/'.join(repo_url.split('/')[-2:])
            
            if isinstance(repo_result, str):
                # This is an error message
                print(f"\nError for repository {repo_name}: {repo_result}")
                continue
                
            print(f"\nRepository: {repo_name}")
            print(f"Found {len(repo_result)} pull requests")
            total_pr_count += len(repo_result)
            
            for pr in repo_result:
                # Convert dates to datetime objects for better display
                created_at = datetime.fromisoformat(pr['created_at'].replace('Z', '+00:00'))
                updated_at = datetime.fromisoformat(pr['updated_at'].replace('Z', '+00:00'))
                
                closed_at = None
                if pr['closed_at']:
                    closed_at = datetime.fromisoformat(pr['closed_at'].replace('Z', '+00:00'))
                    
                merged_at = None
                if pr['merged_at']:
                    merged_at = datetime.fromisoformat(pr['merged_at'].replace('Z', '+00:00'))
                
                # Add repository information to each PR record
                pr_data = {
                    'Repository': repo_name,
                    'PR Number': pr['number'],
                    'Title': pr['title'],
                    'State': pr['state'],
                    'Author': pr['user_login'],
                    'Created': created_at,
                    'Updated': updated_at,
                    'Closed': closed_at,
                    'Merged': merged_at,
                    'Comments': pr['comments'],
                    'Commits': pr['commits'],
                    'Additions': pr['additions'],
                    'Deletions': pr['deletions'],
                    'Changed Files': pr['changed_files'],
                    'Is Draft': pr['is_draft'],
                    'Is Merged': pr['merged'],
                    'Labels': ', '.join(pr['labels']) if pr['labels'] else '',
                }
                
                all_prs.append(pr_data)
        
        # Create a DataFrame from all PR records
        if all_prs:
            df = pd.DataFrame(all_prs)
            
            # Display the DataFrame
            print(f"\nTotal Pull Requests: {total_pr_count}")
            pd.set_option('display.max_columns', None)
            pd.set_option('display.width', 1000)
            pd.set_option('display.max_colwidth', 50)  # Truncate long titles
            
            # Sort by created date (newest first)
            df = df.sort_values('Created', ascending=False)
            
            print("\nMost Recent Pull Requests:")
            print(df.head(10))  # Show top 10 most recent PRs
            
            # Some basic statistics
            print("\nPull Request Statistics:")
            print(f"Open PRs: {len(df[df['State'] == 'open'])}")
            print(f"Closed PRs: {len(df[df['State'] == 'closed'])}")
            print(f"Merged PRs: {len(df[df['Is Merged'] == True])}")
            print(f"Draft PRs: {len(df[df['Is Draft'] == True])}")
            
            # Repository statistics
            print("\nPRs by Repository:")
            print(df['Repository'].value_counts())
            
            # Author statistics
            print("\nTop PR Authors:")
            print(df['Author'].value_counts().head(10))  # Top 10 authors
            
            # Code changes statistics
            print("\nCode Change Statistics:")
            print(f"Total Additions: {df['Additions'].sum()}")
            print(f"Total Deletions: {df['Deletions'].sum()}")
            print(f"Total Files Changed: {df['Changed Files'].sum()}")
            print(f"Average Commits per PR: {df['Commits'].mean():.2f}")
            
            # Save to CSV
            csv_path = os.path.join(os.path.dirname(__file__), 'pull_request_analysis.csv')
            df.to_csv(csv_path, index=False)
            print(f"\nPull request data saved to: {csv_path}")
        else:
            print("\nNo pull request data found.")
            
    except Exception as e:
        print(f"Error fetching pull requests: {e}")


if __name__ == "__main__":
    asyncio.run(main())
