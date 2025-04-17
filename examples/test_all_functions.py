#!/usr/bin/env python3
"""
A comprehensive example that demonstrates how to use all functions
exposed from Rust to Python in the gradelib module.

This script:
1. Initializes the async runtime
2. Creates a GitHubProvider
3. Clones repositories
4. Analyzes commits, branches, and blame
5. Fetches collaborators, issues, and pull requests

Usage:
    python test_all_functions.py

Environment variables:
    GITHUB_USERNAME: GitHub username for authentication
    GITHUB_TOKEN: GitHub personal access token for authentication
"""
import os
import sys
import asyncio
from pathlib import Path

# Add project root to PYTHONPATH
PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from gradelib import setup_async
from github import GitHubProvider

# Test repositories
TEST_REPOS = {
    "gradelib": "https://github.com/bmeddeb/gradelib",
    "ser402": "https://github.com/bmeddeb/SER402-Team3",
    "survivors": "https://github.com/amehlhase316/survivors-spring24C"
}

# Files to analyze with blame
TEST_FILES = [
    "README.md",
    "Cargo.toml",
    "pyproject.toml"
]

async def main():
    """Main function that demonstrates all gradelib functionality"""
    print("Testing gradelib functions...")
    
    # Step 1: Initialize the async runtime
    print("\n1. Initializing async runtime...")
    setup_async()
    print("✓ Async runtime initialized")
    
    # Get GitHub credentials
    username = os.environ.get("GITHUB_USERNAME")
    token = os.environ.get("GITHUB_TOKEN")
    
    if not username or not token:
        print("⚠️ Environment variables GITHUB_USERNAME and GITHUB_TOKEN are required")
        print("Using placeholder values for demo purposes - API calls will fail")
        username = "demo_user"
        token = "demo_token"
    
    # Step 2: Create a GitHubProvider
    print("\n2. Creating GitHubProvider...")
    provider = GitHubProvider(username=username, token=token, urls=list(TEST_REPOS.values()))
    print(f"✓ Created GitHubProvider with username: {username}")
    
    try:
        # Step 3: Clone repositories (if credentials are valid)
        print("\n3. Cloning repositories...")
        for name, url in TEST_REPOS.items():
            print(f"   Cloning {name} from {url}...")
            try:
                await provider.clone(url)
                print(f"   ✓ Cloned {name}")
            except Exception as e:
                print(f"   ✗ Failed to clone {name}: {e}")
        
        # Step 4: Fetch clone tasks
        print("\n4. Fetching clone tasks...")
        tasks = await provider.fetch_clone_tasks()
        print(f"   ✓ Fetched {len(tasks)} clone tasks")
        for url, task in tasks.items():
            print(f"   - {url}: {task.get('status_type', 'unknown')}")
        
        # Step 5: Analyze commits (use the first repo)
        repo_url = list(TEST_REPOS.values())[0]
        print(f"\n5. Analyzing commits for {repo_url}...")
        try:
            commits = await provider.analyze_commits(repo_url)
            print(f"   ✓ Found {len(commits)} commits")
            if commits:
                # Display a sample commit
                commit = commits[0]
                print(f"   Sample commit: {commit['sha'][:7]} by {commit['author_name']}")
                print(f"   Message: {commit['message'].splitlines()[0]}")
        except Exception as e:
            print(f"   ✗ Failed to analyze commits: {e}")
        
        # Step 6: Perform bulk blame
        print(f"\n6. Performing bulk blame on files in {repo_url}...")
        try:
            blame_results = await provider.bulk_blame(repo_url, TEST_FILES)
            print(f"   ✓ Blame analysis completed for {len(blame_results)} files")
            for file_path, blame_data in blame_results.items():
                if isinstance(blame_data, list):
                    print(f"   - {file_path}: {len(blame_data)} lines analyzed")
                else:
                    print(f"   - {file_path}: Error - {blame_data}")
        except Exception as e:
            print(f"   ✗ Failed to perform bulk blame: {e}")
        
        # Step 7: Analyze branches
        print("\n7. Analyzing branches in all repositories...")
        try:
            branch_results = await provider.analyze_branches(list(TEST_REPOS.values()))
            print(f"   ✓ Branch analysis completed for {len(branch_results)} repositories")
            for repo, branches in branch_results.items():
                repo_name = next((name for name, url in TEST_REPOS.items() if url == repo), repo)
                if isinstance(branches, list):
                    print(f"   - {repo_name}: {len(branches)} branches found")
                    # Show branch names
                    if branches:
                        branch_names = [b['name'] for b in branches]
                        print(f"     Branches: {', '.join(branch_names[:5])}" + 
                              (f" and {len(branch_names) - 5} more..." if len(branch_names) > 5 else ""))
                else:
                    print(f"   - {repo_name}: Error - {branches}")
        except Exception as e:
            print(f"   ✗ Failed to analyze branches: {e}")
        
        # Step 8: Fetch collaborators
        print("\n8. Fetching collaborators for all repositories...")
        try:
            collab_results = await provider.fetch_collaborators(list(TEST_REPOS.values()))
            print(f"   ✓ Fetched collaborators for {len(collab_results)} repositories")
            for repo, collaborators in collab_results.items():
                repo_name = next((name for name, url in TEST_REPOS.items() if url == repo), repo)
                if isinstance(collaborators, list):
                    print(f"   - {repo_name}: {len(collaborators)} collaborators")
                    if collaborators:
                        collab_names = [c['login'] for c in collaborators]
                        print(f"     Collaborators: {', '.join(collab_names[:5])}" + 
                              (f" and {len(collab_names) - 5} more..." if len(collab_names) > 5 else ""))
                else:
                    print(f"   - {repo_name}: Error - {collaborators}")
        except Exception as e:
            print(f"   ✗ Failed to fetch collaborators: {e}")
        
        # Step 9: Fetch issues
        print("\n9. Fetching issues for all repositories...")
        try:
            issue_results = await provider.fetch_issues(list(TEST_REPOS.values()), state="all")
            print(f"   ✓ Fetched issues for {len(issue_results)} repositories")
            for repo, issues in issue_results.items():
                repo_name = next((name for name, url in TEST_REPOS.items() if url == repo), repo)
                if isinstance(issues, list):
                    print(f"   - {repo_name}: {len(issues)} issues")
                    # Count open vs closed
                    open_issues = sum(1 for i in issues if i['state'] == 'open')
                    closed_issues = sum(1 for i in issues if i['state'] == 'closed')
                    print(f"     Open: {open_issues}, Closed: {closed_issues}")
                else:
                    print(f"   - {repo_name}: Error - {issues}")
        except Exception as e:
            print(f"   ✗ Failed to fetch issues: {e}")
        
        # Step 10: Fetch pull requests
        print("\n10. Fetching pull requests for all repositories...")
        try:
            pr_results = await provider.fetch_pull_requests(list(TEST_REPOS.values()), state="all")
            print(f"   ✓ Fetched pull requests for {len(pr_results)} repositories")
            for repo, prs in pr_results.items():
                repo_name = next((name for name, url in TEST_REPOS.items() if url == repo), repo)
                if isinstance(prs, list):
                    print(f"   - {repo_name}: {len(prs)} pull requests")
                    # Count open vs closed vs merged
                    open_prs = sum(1 for pr in prs if pr['state'] == 'open')
                    closed_prs = sum(1 for pr in prs if pr['state'] == 'closed' and not pr['merged'])
                    merged_prs = sum(1 for pr in prs if pr['merged'])
                    print(f"     Open: {open_prs}, Closed: {closed_prs}, Merged: {merged_prs}")
                else:
                    print(f"   - {repo_name}: Error - {prs}")
        except Exception as e:
            print(f"   ✗ Failed to fetch pull requests: {e}")
    
    except Exception as e:
        print(f"Error during execution: {e}")
    
    print("\nTest completed!")

if __name__ == "__main__":
    asyncio.run(main())
