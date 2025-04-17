#!/usr/bin/env python3
"""
Example demonstrating the fetch_pull_requests method of GitHubProvider.

This script shows how to retrieve and analyze pull requests from GitHub repositories,
including filtering by state (open, closed, all).

Usage:
    python test_fetch_pull_requests.py [STATE] [REPO_URL...]
    
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
    """Demonstrate pull request fetching with GitHubProvider"""
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
    
    # Fetch pull requests
    print(f"\nFetching {state} pull requests for {len(repo_urls)} repositories...")
    try:
        pr_results = await provider.fetch_pull_requests(repo_urls, state=state)
        print(f"✓ Fetched pull request information for {len(pr_results)} repositories")
        
        # Process each repository
        for repo_url, prs in pr_results.items():
            repo_name = repo_url.split('/')[-1]
            print(f"\nRepository: {repo_name} ({repo_url})")
            
            if isinstance(prs, list):
                # Count PRs by state
                open_prs = sum(1 for pr in prs if pr['state'] == 'open')
                closed_unmerged = sum(1 for pr in prs if pr['state'] == 'closed' and not pr['merged'])
                merged_prs = sum(1 for pr in prs if pr['merged'])
                
                print(f"  Total pull requests: {len(prs)}")
                print(f"  Open: {open_prs}")
                print(f"  Closed (unmerged): {closed_unmerged}")
                print(f"  Merged: {merged_prs}")
                
                if prs:
                    # Calculate merge rate
                    closed_prs = closed_unmerged + merged_prs
                    if closed_prs > 0:
                        merge_rate = (merged_prs / closed_prs) * 100
                        print(f"  Merge rate: {merge_rate:.1f}%")
                    
                    # Code churn analysis
                    total_additions = sum(pr['additions'] for pr in prs)
                    total_deletions = sum(pr['deletions'] for pr in prs)
                    total_files = sum(pr['changed_files'] for pr in prs)
                    
                    print(f"\n  Code changes across all PRs:")
                    print(f"    Files changed: {total_files}")
                    print(f"    Lines added: {total_additions}")
                    print(f"    Lines removed: {total_deletions}")
                    
                    # Calculate average PR size
                    avg_files = total_files / len(prs)
                    avg_additions = total_additions / len(prs)
                    avg_deletions = total_deletions / len(prs)
                    
                    print(f"\n  Average PR size:")
                    print(f"    {avg_files:.1f} files changed")
                    print(f"    {avg_additions:.1f} lines added")
                    print(f"    {avg_deletions:.1f} lines removed")
                    
                    # PR authors analysis
                    authors = Counter(pr['user_login'] for pr in prs)
                    print(f"\n  PR authors: {len(authors)} unique contributors")
                    
                    # Top PR authors
                    if authors:
                        top_authors = authors.most_common(3)
                        print("  Top contributors:")
                        for author, count in top_authors:
                            percentage = (count / len(prs)) * 100
                            
                            # Calculate merge rate for this author
                            author_prs = [pr for pr in prs if pr['user_login'] == author]
                            author_merged = sum(1 for pr in author_prs if pr['merged'])
                            author_closed = sum(1 for pr in author_prs 
                                              if pr['state'] == 'closed' or pr['merged'])
                            
                            if author_closed > 0:
                                author_merge_rate = (author_merged / author_closed) * 100
                                merge_rate_str = f", {author_merge_rate:.1f}% merge rate"
                            else:
                                merge_rate_str = ""
                            
                            print(f"    - {author}: {count} PRs ({percentage:.1f}%){merge_rate_str}")
                    
                    # PR by time analysis
                    months = defaultdict(int)
                    for pr in prs:
                        # Parse created_at timestamp
                        created_date = datetime.datetime.fromisoformat(pr['created_at'].replace('Z', '+00:00'))
                        month_key = f"{created_date.year}-{created_date.month:02d}"
                        months[month_key] += 1
                    
                    if months:
                        print("\n  PRs by month (recent first):")
                        for month, count in sorted(months.items(), reverse=True)[:6]:
                            percentage = (count / len(prs)) * 100
                            print(f"    - {month}: {count} PRs ({percentage:.1f}%)")
                    
                    # Recent PRs
                    recent_prs = sorted(prs, key=lambda pr: pr['created_at'], reverse=True)[:5]
                    print("\n  Most recent pull requests:")
                    for i, pr in enumerate(recent_prs, 1):
                        created_date = datetime.datetime.fromisoformat(pr['created_at'].replace('Z', '+00:00'))
                        date_str = created_date.strftime('%Y-%m-%d')
                        
                        # Format PR status
                        if pr['state'] == 'open':
                            status = "Open"
                        elif pr['merged']:
                            status = "Merged"
                        else:
                            status = "Closed (unmerged)"
                        
                        print(f"    {i}. #{pr['number']} - {pr['title']} ({date_str})")
                        print(f"       Status: {status}, Author: {pr['user_login']}")
                        print(f"       Changes: +{pr['additions']} -{pr['deletions']} in {pr['changed_files']} files")
                        
                        if pr['labels']:
                            labels_str = ", ".join(pr['labels'][:3])
                            if len(pr['labels']) > 3:
                                labels_str += f" and {len(pr['labels']) - 3} more"
                            print(f"       Labels: {labels_str}")
                        
                        if pr['is_draft']:
                            print(f"       Draft: Yes")
            else:
                # Error occurred for this repository
                print(f"  ✗ Error fetching pull requests: {prs}")
        
        # Cross-repository analysis
        all_prs = []
        for repo_url, prs in pr_results.items():
            if isinstance(prs, list):
                repo_name = repo_url.split('/')[-1]
                
                for pr in prs:
                    pr_with_repo = pr.copy()
                    pr_with_repo['repo_name'] = repo_name
                    pr_with_repo['repo_url'] = repo_url
                    all_prs.append(pr_with_repo)
        
        if len(all_prs) > 0 and len(pr_results) > 1:
            print("\nCross-repository analysis:")
            print(f"  Total PRs across all repositories: {len(all_prs)}")
            
            # PRs by repository
            repo_counts = Counter(pr['repo_name'] for pr in all_prs)
            print("\n  PRs by repository:")
            for repo, count in repo_counts.most_common():
                percentage = (count / len(all_prs)) * 100
                print(f"    - {repo}: {count} PRs ({percentage:.1f}%)")
            
            # Identify users who have created PRs in multiple repositories
            user_repos = defaultdict(set)
            for pr in all_prs:
                user_repos[pr['user_login']].add(pr['repo_name'])
            
            multi_repo_users = {user: repos for user, repos in user_repos.items() if len(repos) > 1}
            if multi_repo_users:
                print("\n  Users with PRs in multiple repositories:")
                for user, repos in sorted(multi_repo_users.items(), 
                                        key=lambda x: len(x[1]), reverse=True):
                    repos_str = ", ".join(repos)
                    pr_count = sum(1 for pr in all_prs if pr['user_login'] == user)
                    
                    # Calculate user's merge rate across all repos
                    user_prs = [pr for pr in all_prs if pr['user_login'] == user]
                    user_merged = sum(1 for pr in user_prs if pr['merged'])
                    user_closed = sum(1 for pr in user_prs 
                                    if pr['state'] == 'closed' or pr['merged'])
                    
                    if user_closed > 0:
                        user_merge_rate = (user_merged / user_closed) * 100
                        merge_rate_str = f", {user_merge_rate:.1f}% merge rate"
                    else:
                        merge_rate_str = ""
                    
                    print(f"    - {user}: {pr_count} PRs across {len(repos)} repositories{merge_rate_str}")
                    print(f"      Repositories: {repos_str}")
    
    except Exception as e:
        print(f"✗ Failed to fetch pull requests: {e}")

if __name__ == "__main__":
    asyncio.run(main())
