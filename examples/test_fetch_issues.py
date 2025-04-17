#!/usr/bin/env python3
"""
Example demonstrating the fetch_issues method of GitHubProvider.

This script shows how to retrieve and analyze issues from GitHub repositories,
including filtering by state (open, closed, all).

Usage:
    python test_fetch_issues.py [STATE] [REPO_URL...]
    
    STATE: 'open', 'closed', or 'all' (default: 'all')

Environment variables:
    GITHUB_USERNAME: GitHub username for authentication
    GITHUB_TOKEN: GitHub personal access token for authentication
"""
import os
import sys
import asyncio
import datetime
from pathlib import Path
from collections import Counter, defaultdict

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
    """Demonstrate issue fetching with GitHubProvider"""
    # Parse command line arguments
    args = sys.argv[1:]
    state = None
    repo_urls = []
    
    if args:
        if args[0].lower() in ('open', 'closed', 'all'):
            state = args[0].lower()
            repo_urls = args[1:] if len(args) > 1 else DEFAULT_REPOS
        else:
            repo_urls = args
    else:
        state = 'all'
        repo_urls = DEFAULT_REPOS
    
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
    
    # Fetch issues
    print(f"\nFetching {state} issues for {len(repo_urls)} repositories...")
    try:
        issue_results = await provider.fetch_issues(repo_urls, state=state)
        print(f"✓ Fetched issue information for {len(issue_results)} repositories")
        
        # Process each repository
        for repo_url, issues in issue_results.items():
            repo_name = repo_url.split('/')[-1]
            print(f"\nRepository: {repo_name} ({repo_url})")
            
            if isinstance(issues, list):
                # Count issues by state
                open_issues = sum(1 for i in issues if i['state'] == 'open')
                closed_issues = sum(1 for i in issues if i['state'] == 'closed')
                
                print(f"  Total issues: {len(issues)}")
                print(f"  Open issues: {open_issues}")
                print(f"  Closed issues: {closed_issues}")
                
                if issues:
                    # Analyze issue statistics
                    
                    # Issues by author
                    authors = Counter(issue['user_login'] for issue in issues)
                    print(f"\n  Issue authors: {len(authors)} unique contributors")
                    
                    # Top authors
                    if authors:
                        top_authors = authors.most_common(3)
                        print("  Top issue creators:")
                        for author, count in top_authors:
                            percentage = (count / len(issues)) * 100
                            print(f"    - {author}: {count} issues ({percentage:.1f}%)")
                    
                    # Issues by labels
                    all_labels = []
                    for issue in issues:
                        all_labels.extend(issue['labels'])
                    
                    label_counts = Counter(all_labels)
                    print(f"\n  Issue labels: {len(label_counts)} unique labels")
                    
                    # Top labels
                    if label_counts:
                        top_labels = label_counts.most_common(5)
                        print("  Most common labels:")
                        for label, count in top_labels:
                            percentage = (count / len(issues)) * 100
                            print(f"    - {label}: {count} occurrences ({percentage:.1f}%)")
                    
                    # Issues by month
                    months = defaultdict(int)
                    for issue in issues:
                        # Parse created_at timestamp
                        created_date = datetime.datetime.fromisoformat(issue['created_at'].replace('Z', '+00:00'))
                        month_key = f"{created_date.year}-{created_date.month:02d}"
                        months[month_key] += 1
                    
                    print("\n  Issues by month (recent first):")
                    for month, count in sorted(months.items(), reverse=True)[:6]:
                        percentage = (count / len(issues)) * 100
                        print(f"    - {month}: {count} issues ({percentage:.1f}%)")
                    
                    # Recent issues
                    recent_issues = sorted(issues, key=lambda i: i['created_at'], reverse=True)[:5]
                    print("\n  Most recent issues:")
                    for i, issue in enumerate(recent_issues, 1):
                        created_date = datetime.datetime.fromisoformat(issue['created_at'].replace('Z', '+00:00'))
                        date_str = created_date.strftime('%Y-%m-%d')
                        print(f"    {i}. #{issue['number']} - {issue['title']} ({date_str})")
                        print(f"       State: {issue['state']}, Author: {issue['user_login']}")
                        
                        if issue['labels']:
                            labels_str = ", ".join(issue['labels'][:3])
                            if len(issue['labels']) > 3:
                                labels_str += f" and {len(issue['labels']) - 3} more"
                            print(f"       Labels: {labels_str}")
                        
                        if issue['assignees']:
                            assignees_str = ", ".join(issue['assignees'][:3])
                            if len(issue['assignees']) > 3:
                                assignees_str += f" and {len(issue['assignees']) - 3} more"
                            print(f"       Assignees: {assignees_str}")
            else:
                # Error occurred for this repository
                print(f"  ✗ Error fetching issues: {issues}")
        
        # Cross-repository analysis
        all_issues = []
        for repo_url, issues in issue_results.items():
            if isinstance(issues, list):
                repo_name = repo_url.split('/')[-1]
                
                for issue in issues:
                    issue_with_repo = issue.copy()
                    issue_with_repo['repo_name'] = repo_name
                    issue_with_repo['repo_url'] = repo_url
                    all_issues.append(issue_with_repo)
        
        if len(all_issues) > 0 and len(issue_results) > 1:
            print("\nCross-repository analysis:")
            print(f"  Total issues across all repositories: {len(all_issues)}")
            
            # Issues by repository
            repo_counts = Counter(issue['repo_name'] for issue in all_issues)
            print("\n  Issues by repository:")
            for repo, count in repo_counts.most_common():
                percentage = (count / len(all_issues)) * 100
                print(f"    - {repo}: {count} issues ({percentage:.1f}%)")
            
            # Identify users who have created issues in multiple repositories
            user_repos = defaultdict(set)
            for issue in all_issues:
                user_repos[issue['user_login']].add(issue['repo_name'])
            
            multi_repo_users = {user: repos for user, repos in user_repos.items() if len(repos) > 1}
            if multi_repo_users:
                print("\n  Users with issues in multiple repositories:")
                for user, repos in sorted(multi_repo_users.items(), 
                                        key=lambda x: len(x[1]), reverse=True):
                    repos_str = ", ".join(repos)
                    issue_count = sum(1 for i in all_issues if i['user_login'] == user)
                    print(f"    - {user}: {issue_count} issues across {len(repos)} repositories")
                    print(f"      Repositories: {repos_str}")
    
    except Exception as e:
        print(f"✗ Failed to fetch issues: {e}")

if __name__ == "__main__":
    asyncio.run(main())
