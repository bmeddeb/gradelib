#!/usr/bin/env python3
"""
Example script demonstrating the fetch_collaborators functionality in gradelib.

This script:
1. Sets up the async environment
2. Creates a RepoManager with GitHub credentials
3. Fetches collaborator information for specified repositories
4. Displays the collaborator data in a DataFrame
"""

import asyncio
import os
import sys
import pandas as pd
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
        "https://github.com/bmeddeb/SER402-Team3"
    ]
    
    # Create the repo manager
    manager = RepoManager(repo_urls, github_username, github_token)
    
    # Fetch collaborator information
    print(f"Fetching collaborator information for {len(repo_urls)} repositories...")
    try:
        collaborators = await manager.fetch_collaborators(repo_urls)
        
        # Process and display the collaborator data
        all_collaborators = []
        for repo_url, repo_collaborators in collaborators.items():
            repo_name = repo_url.split('/')[-2] + '/' + repo_url.split('/')[-1]
            
            print(f"\nRepository: {repo_name}")
            print(f"Found {len(repo_collaborators)} collaborators")
            
            for collab in repo_collaborators:
                # Add repository information to each collaborator record
                collab_data = {
                    'Repository': repo_name,
                    'Login': collab['login'],
                    'GitHub ID': collab['github_id'],
                    'Name': collab.get('full_name', 'N/A'),
                    'Email': collab.get('email', 'N/A'),
                    'Avatar URL': collab.get('avatar_url', 'N/A')
                }
                all_collaborators.append(collab_data)
        
        # Create a DataFrame from all collaborator records
        if all_collaborators:
            df = pd.DataFrame(all_collaborators)
            
            # Display the DataFrame
            print("\nAll Collaborators:")
            pd.set_option('display.max_columns', None)
            pd.set_option('display.width', 1000)
            print(df)
            
            # Optionally, save to CSV
            csv_path = os.path.join(os.path.dirname(__file__), 'collaborators.csv')
            df.to_csv(csv_path, index=False)
            print(f"\nCollaborator data saved to: {csv_path}")
        else:
            print("\nNo collaborator data found.")
            
    except Exception as e:
        print(f"Error fetching collaborators: {e}")


if __name__ == "__main__":
    asyncio.run(main())