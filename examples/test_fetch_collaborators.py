#!/usr/bin/env python3
"""
Example demonstrating the fetch_collaborators method of GitHubProvider.

This script shows how to retrieve information about collaborators
on GitHub repositories, including their roles and activity.

Usage:
    python test_fetch_collaborators.py [REPO_URL...]

Environment variables:
    GITHUB_USERNAME: GitHub username for authentication
    GITHUB_TOKEN: GitHub personal access token for authentication
"""
import os
import sys
import asyncio
from pathlib import Path
from collections import Counter

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
    """Demonstrate collaborator fetching with GitHubProvider"""
    # Get repo URLs from command line if provided
    repo_urls = sys.argv[1:] if len(sys.argv) > 1 else DEFAULT_REPOS
    
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
    
    # Fetch collaborators for all repositories
    print(f"\nFetching collaborators for {len(repo_urls)} repositories...")
    try:
        collab_results = await provider.fetch_collaborators(repo_urls)
        print(f"✓ Fetched collaborator information for {len(collab_results)} repositories")
        
        # Process each repository
        for repo_url, collaborators in collab_results.items():
            repo_name = repo_url.split('/')[-1]
            print(f"\nRepository: {repo_name} ({repo_url})")
            
            if isinstance(collaborators, list):
                # Overview of collaborators
                print(f"  Total collaborators: {len(collaborators)}")
                
                # Create a table of collaborators
                if collaborators:
                    print("\n  Collaborators:")
                    print("  Username            | Name                 | Email")
                    print("  --------------------|----------------------|----------------------")
                    
                    for collab in collaborators:
                        login = collab['login'][:19].ljust(19)
                        name = collab.get('full_name', "N/A")[:21].ljust(21)
                        email = collab.get('email', "N/A")[:21].ljust(21)
                        
                        print(f"  {login} | {name} | {email}")
                    
                    # Count unique email domains
                    domains = Counter()
                    for collab in collaborators:
                        if collab.get('email'):
                            email = collab['email']
                            domain = email.split('@')[-1] if '@' in email else "unknown"
                            domains[domain] += 1
                    
                    # Display domain statistics
                    if domains:
                        print("\n  Email domains:")
                        for domain, count in domains.most_common():
                            percentage = (count / len(collaborators)) * 100
                            print(f"    - {domain}: {count} collaborators ({percentage:.1f}%)")
            else:
                # Error occurred for this repository
                print(f"  ✗ Error fetching collaborators: {collaborators}")
        
        # Cross-repository analysis
        all_collaborators = {}
        for repo_url, collaborators in collab_results.items():
            if isinstance(collaborators, list):
                repo_name = repo_url.split('/')[-1]
                
                for collab in collaborators:
                    login = collab['login']
                    
                    if login not in all_collaborators:
                        all_collaborators[login] = {
                            'name': collab.get('full_name', "N/A"),
                            'email': collab.get('email', "N/A"),
                            'repos': []
                        }
                    
                    all_collaborators[login]['repos'].append(repo_name)
        
        # Find collaborators who work on multiple repositories
        multi_repo_collaborators = {login: data for login, data in all_collaborators.items() 
                                  if len(data['repos']) > 1}
        
        if multi_repo_collaborators:
            print("\nCollaborators working across multiple repositories:")
            for login, data in sorted(multi_repo_collaborators.items(), 
                                    key=lambda x: len(x[1]['repos']), reverse=True):
                repos = ", ".join(data['repos'])
                print(f"  - {login} ({data['name']}): {len(data['repos'])} repositories")
                print(f"    Repositories: {repos}")
        else:
            print("\nNo collaborators work across multiple repositories")
    
    except Exception as e:
        print(f"✗ Failed to fetch collaborators: {e}")

if __name__ == "__main__":
    asyncio.run(main())
