#!/usr/bin/env python3
"""
Example demonstrating the analyze_commits method of GitHubProvider.

This script shows how to analyze commit history in a Git repository,
extracting useful information about each commit.

Usage:
    python test_analyze_commits.py [REPO_URL]

Environment variables:
    GITHUB_USERNAME: GitHub username for authentication
    GITHUB_TOKEN: GitHub personal access token for authentication
"""
import os
import sys
import asyncio
import datetime
from pathlib import Path
from collections import Counter

# Add project root to PYTHONPATH
PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from gradelib import setup_async
from github import GitHubProvider

# Default test repository
DEFAULT_REPO = "https://github.com/bmeddeb/gradelib"

async def main():
    """Demonstrate commit analysis with GitHubProvider"""
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
    
    # Create a provider and clone the repository
    provider = GitHubProvider(username=username, token=token)
    print(f"✓ Created GitHubProvider with username: {username}")
    
    # Clone the repository first
    print(f"\nCloning repository: {repo_url}")
    try:
        await provider.clone(repo_url)
        print(f"✓ Successfully cloned {repo_url}")
    except Exception as e:
        print(f"✗ Failed to clone repository: {e}")
        return
    
    # Analyze commits
    print(f"\nAnalyzing commits in repository: {repo_url}")
    try:
        commits = await provider.analyze_commits(repo_url)
        print(f"✓ Found {len(commits)} commits")
        
        if not commits:
            print("No commits found in the repository")
            return
        
        # Display basic commit info
        print("\nMost recent commits:")
        for i, commit in enumerate(commits[:5], 1):
            # Format the commit date
            timestamp = commit['author_timestamp']
            date = datetime.datetime.fromtimestamp(timestamp).strftime('%Y-%m-%d %H:%M:%S')
            
            # Print commit details
            print(f"  {i}. {commit['sha'][:7]} - {date} by {commit['author_name']}")
            print(f"     {commit['message'].splitlines()[0]}")
            print(f"     +{commit['additions']} -{commit['deletions']} lines")
        
        # Analyze commit patterns
        print("\nCommit analysis:")
        
        # Count commits by author
        authors = Counter(commit['author_name'] for commit in commits)
        print(f"  Authors: {len(authors)} contributors")
        
        # Top 3 contributors
        top_authors = authors.most_common(3)
        print("  Top contributors:")
        for author, count in top_authors:
            percentage = (count / len(commits)) * 100
            print(f"    - {author}: {count} commits ({percentage:.1f}%)")
        
        # Analyze commit times
        weekdays = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday']
        days = Counter()
        hours = Counter()
        
        for commit in commits:
            dt = datetime.datetime.fromtimestamp(commit['author_timestamp'])
            days[weekdays[dt.weekday()]] += 1
            hours[dt.hour] += 1
        
        # Most active day
        most_active_day, day_count = days.most_common(1)[0]
        day_percentage = (day_count / len(commits)) * 100
        print(f"  Most active day: {most_active_day} ({day_percentage:.1f}% of commits)")
        
        # Most active time
        active_hours = sorted(hours.most_common(3))
        hour_ranges = []
        for hour, count in active_hours:
            hour_str = f"{hour:02d}:00-{hour:02d}:59"
            percentage = (count / len(commits)) * 100
            hour_ranges.append(f"{hour_str} ({percentage:.1f}%)")
        
        print(f"  Most active hours: {', '.join(hour_ranges)}")
        
        # Analyze merge commits
        merge_count = sum(1 for commit in commits if commit['is_merge'])
        merge_percentage = (merge_count / len(commits)) * 100
        print(f"  Merge commits: {merge_count} ({merge_percentage:.1f}%)")
        
        # Analyze additions/deletions
        total_additions = sum(commit['additions'] for commit in commits)
        total_deletions = sum(commit['deletions'] for commit in commits)
        net_changes = total_additions - total_deletions
        
        print(f"  Total lines: +{total_additions} added, -{total_deletions} removed, {net_changes:+} net")
        
        # Average commit size
        avg_additions = total_additions / len(commits)
        avg_deletions = total_deletions / len(commits)
        print(f"  Average commit size: +{avg_additions:.1f} added, -{avg_deletions:.1f} removed")
    
    except Exception as e:
        print(f"✗ Failed to analyze commits: {e}")

if __name__ == "__main__":
    asyncio.run(main())
