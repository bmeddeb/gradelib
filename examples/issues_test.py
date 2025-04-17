#!/usr/bin/env python3
"""
Example script demonstrating the fetch_issues functionality in gradelib.

This script:
1. Sets up the async environment
2. Creates a RepoManager with GitHub credentials
3. Fetches issue information for specified repositories
4. Displays the issue data in a DataFrame
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
    
    # Fetch issue information
    print(f"Fetching issue information for {len(repo_urls)} repositories...")
    try:
        # You can specify state=None (default), "open", "closed", or "all"
        issues = await manager.fetch_issues(repo_urls, state="all")
        
        # Process and display the issue data
        all_issues = []
        total_issue_count = 0
        total_pr_count = 0
        
        for repo_url, repo_result in issues.items():
            repo_name = '/'.join(repo_url.split('/')[-2:])
            
            if isinstance(repo_result, str):
                # This is an error message
                print(f"\nError for repository {repo_name}: {repo_result}")
                continue
                
            print(f"\nRepository: {repo_name}")
            
            # Count actual issues (not PRs)
            actual_issues = [issue for issue in repo_result if not issue['is_pull_request']]
            pull_requests = [issue for issue in repo_result if issue['is_pull_request']]
            
            print(f"Found {len(actual_issues)} issues and {len(pull_requests)} pull requests")
            total_issue_count += len(actual_issues)
            total_pr_count += len(pull_requests)
            
            # Process only non-PR issues
            for issue in actual_issues:
                # Convert dates to datetime objects for better display
                created_at = datetime.fromisoformat(issue['created_at'].replace('Z', '+00:00'))
                updated_at = datetime.fromisoformat(issue['updated_at'].replace('Z', '+00:00'))
                
                closed_at = None
                if issue['closed_at']:
                    closed_at = datetime.fromisoformat(issue['closed_at'].replace('Z', '+00:00'))
                
                # Add repository information to each issue record
                issue_data = {
                    'Repository': repo_name,
                    'Issue Number': issue['number'],
                    'Title': issue['title'],
                    'State': issue['state'],
                    'Author': issue['user_login'],
                    'Created': created_at,
                    'Updated': updated_at,
                    'Closed': closed_at,
                    'Comments': issue['comments_count'],
                    'Labels': ', '.join(issue['labels']) if issue['labels'] else '',
                    'Assignees': ', '.join(issue['assignees']) if issue['assignees'] else '',
                    'Milestone': issue['milestone'] if issue['milestone'] else '',
                    'Locked': issue['locked'],
                    'URL': issue['html_url'],
                }
                
                all_issues.append(issue_data)
        
        # Create a DataFrame from all issue records
        if all_issues:
            df = pd.DataFrame(all_issues)
            
            # Display the DataFrame
            print(f"\nTotal Issues: {total_issue_count}")
            print(f"Total Pull Requests: {total_pr_count}")
            pd.set_option('display.max_columns', None)
            pd.set_option('display.width', 1000)
            pd.set_option('display.max_colwidth', 50)  # Truncate long titles
            
            # Sort by created date (newest first)
            df = df.sort_values('Created', ascending=False)
            
            print("\nMost Recent Issues:")
            print(df.head(10))  # Show top 10 most recent issues
            
            # Some basic statistics
            print("\nIssue Statistics:")
            print(f"Open Issues: {len(df[df['State'] == 'open'])}")
            print(f"Closed Issues: {len(df[df['State'] == 'closed'])}")
            
            # Repository statistics
            print("\nIssues by Repository:")
            print(df['Repository'].value_counts())
            
            # Author statistics
            print("\nTop Issue Authors:")
            print(df['Author'].value_counts().head(10))  # Top 10 authors
            
            # Label statistics
            if 'Labels' in df.columns and not df['Labels'].empty:
                print("\nMost Common Labels:")
                # Split the comma-separated labels into individual labels
                all_labels = []
                for labels in df['Labels'].dropna():
                    if labels:  # Only process non-empty label strings
                        all_labels.extend([label.strip() for label in labels.split(',')])
                
                # Count label occurrences
                label_counts = pd.Series(all_labels).value_counts()
                print(label_counts.head(10))  # Top 10 labels
            
            # Save to CSV
            csv_path = os.path.join(os.path.dirname(__file__), 'issue_analysis.csv')
            df.to_csv(csv_path, index=False)
            print(f"\nIssue data saved to: {csv_path}")
        else:
            print("\nNo issue data found.")
            
    except Exception as e:
        print(f"Error fetching issues: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    asyncio.run(main())
