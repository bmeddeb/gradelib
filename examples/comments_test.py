#!/usr/bin/env python3
"""
Example script demonstrating the fetch_comments functionality in gradelib.

This script:
1. Sets up the async environment
2. Creates a RepoManager with GitHub credentials
3. Fetches comment information of various types for specified repositories
4. Displays the comment data in a DataFrame with visualization options
"""

import asyncio
import os
import sys
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from datetime import datetime, timedelta
from typing import Dict, List
from collections import defaultdict

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
        "https://github.com/bmeddeb/SER402-Team3",
    ]
    
    # Create the repo manager
    manager = RepoManager(repo_urls, github_username, github_token)
    
    # Fetch comment information - you can specify comment types if needed
    # Example: comment_types=["issue", "pull_request"]
    # Available types: "issue", "commit", "pull_request", "review_comment"
    print(f"Fetching comments for {len(repo_urls)} repositories...")
    try:
        # Fetch all comment types by default
        comments = await manager.fetch_comments(repo_urls, comment_types=None)
        # comments = await manager.fetch_comments(repo_urls, comment_types=["issue", "pull_request"])
        
        # Process and display the comment data
        all_comments = []
        total_comment_count = 0
        comment_types_count = defaultdict(int)
        comment_timeline = defaultdict(int)
        comment_by_user = defaultdict(int)
        
        for repo_url, repo_result in comments.items():
            repo_name = '/'.join(repo_url.split('/')[-2:])
            
            if isinstance(repo_result, str):
                # This is an error message
                print(f"\nError for repository {repo_name}: {repo_result}")
                continue
                
            print(f"\nRepository: {repo_name}")
            
            # Count total comments and types
            comment_count = len(repo_result)
            print(f"Found {comment_count} comments")
            total_comment_count += comment_count
            
            # Process each comment
            for comment in repo_result:
                # Increment type counter
                comment_types_count[comment['comment_type']] += 1
                
                # Convert dates for analysis
                created_at = datetime.fromisoformat(comment['created_at'].replace('Z', '+00:00'))
                updated_at = datetime.fromisoformat(comment['updated_at'].replace('Z', '+00:00'))
                
                # Update timeline data (by day)
                comment_day = created_at.date()
                comment_timeline[comment_day] += 1
                
                # Update user activity
                comment_by_user[comment['user_login']] += 1
                
                # Extract and format text snippet
                text_snippet = comment['body']
                if len(text_snippet) > 50:
                    text_snippet = text_snippet[:47] + "..."
                
                # Build comment data dictionary
                comment_data = {
                    'Repository': repo_name,
                    'Comment ID': comment['id'],
                    'Type': comment['comment_type'],
                    'Author': comment['user_login'],
                    'Created At': created_at,
                    'Updated At': updated_at,
                    'Text Snippet': text_snippet,
                    'Issue Number': comment.get('issue_number'),
                    'PR Number': comment.get('pull_request_number'),
                    'Commit SHA': comment.get('commit_sha'),
                    'File Path': comment.get('path'),
                    'Line': comment.get('line'),
                    'URL': comment['html_url'],
                }
                
                all_comments.append(comment_data)
        
        # Create a DataFrame from all comment records
        if all_comments:
            df = pd.DataFrame(all_comments)
            
            # Display the DataFrame
            print(f"\nTotal Comments: {total_comment_count}")
            pd.set_option('display.max_columns', None)
            pd.set_option('display.width', 1000)
            pd.set_option('display.max_colwidth', 50)  # Truncate long text
            
            # Sort by creation date (newest first)
            df = df.sort_values('Created At', ascending=False)
            
            print("\nMost Recent Comments:")
            print(df.head(10))  # Show top 10 most recent comments
            
            # Basic statistics
            print("\nComment Type Distribution:")
            for comment_type, count in comment_types_count.items():
                print(f"  {comment_type}: {count} ({count/total_comment_count:.1%})")
            
            # Repository statistics
            print("\nComments by Repository:")
            print(df['Repository'].value_counts())
            
            # Author statistics
            print("\nTop Commenters:")
            print(df['Author'].value_counts().head(10))  # Top 10 most active users
            
            # Time-based analysis
            print("\nComment Activity by Day of Week:")
            day_of_week_counts = df['Created At'].dt.day_name().value_counts()
            day_order = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday']
            day_of_week_counts = day_of_week_counts.reindex(day_order)
            print(day_of_week_counts)
            
            # Save to CSV
            csv_path = os.path.join(os.path.dirname(__file__), 'comment_analysis.csv')
            df.to_csv(csv_path, index=False)
            print(f"\nComment data saved to: {csv_path}")
            
            # Generate visualizations
            try:
                output_dir = os.path.join(os.path.dirname(__file__), 'visualizations')
                os.makedirs(output_dir, exist_ok=True)
                
                print("\nGenerating visualizations...")
                
                # Set up the style
                plt.style.use('ggplot')
                
                # 1. Comment type distribution
                plt.figure(figsize=(10, 6))
                type_counts = pd.Series(comment_types_count)
                type_counts.plot(kind='bar', color='skyblue')
                plt.title('Comment Type Distribution')
                plt.xlabel('Comment Type')
                plt.ylabel('Count')
                plt.tight_layout()
                plt.savefig(os.path.join(output_dir, 'comment_types.png'))
                
                # 2. Comment timeline
                plt.figure(figsize=(12, 6))
                # Convert defaultdict to Series and sort by date
                timeline_series = pd.Series(dict(comment_timeline)).sort_index()
                # Fill in missing dates with zeros
                if len(timeline_series) > 1:
                    date_range = pd.date_range(start=timeline_series.index.min(), end=timeline_series.index.max())
                    timeline_series = timeline_series.reindex(date_range, fill_value=0)
                timeline_series.plot(marker='o', linestyle='-')
                plt.title('Comment Activity Over Time')
                plt.xlabel('Date')
                plt.ylabel('Number of Comments')
                plt.tight_layout()
                plt.savefig(os.path.join(output_dir, 'comment_timeline.png'))
                
                # 3. Top commenters
                plt.figure(figsize=(12, 8))
                # Get top 10 commenters
                top_users = dict(sorted(comment_by_user.items(), key=lambda x: x[1], reverse=True)[:10])
                plt.barh(list(reversed(list(top_users.keys()))), list(reversed(list(top_users.values()))))
                plt.title('Top 10 Commenters')
                plt.xlabel('Number of Comments')
                plt.tight_layout()
                plt.savefig(os.path.join(output_dir, 'top_commenters.png'))
                
                # 4. Comments by repository
                plt.figure(figsize=(10, 6))
                df['Repository'].value_counts().plot(kind='pie', autopct='%1.1f%%')
                plt.title('Comments by Repository')
                plt.axis('equal')
                plt.tight_layout()
                plt.savefig(os.path.join(output_dir, 'comments_by_repo.png'))
                
                # 5. Day of week activity
                plt.figure(figsize=(10, 6))
                day_of_week_counts.plot(kind='bar')
                plt.title('Comment Activity by Day of Week')
                plt.xlabel('Day of Week')
                plt.ylabel('Number of Comments')
                plt.tight_layout()
                plt.savefig(os.path.join(output_dir, 'day_of_week_activity.png'))
                
                print(f"Visualizations saved to: {output_dir}")
                
            except Exception as viz_error:
                print(f"Error generating visualizations: {viz_error}")
                import traceback
                traceback.print_exc()
                
        else:
            print("\nNo comment data found.")
            
    except Exception as e:
        print(f"Error fetching comments: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    asyncio.run(main()) 