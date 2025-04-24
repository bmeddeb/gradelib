#!/usr/bin/env python3
"""
Example demonstrating the updated Taiga client that works with public projects
without requiring authentication.
"""

import asyncio
import sys
import os
from pprint import pprint

# Add the parent directory to path for local testing
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

# Import the TaigaClient
from gradelib import TaigaClient


async def main():
    """
    Demonstrate accessing a public Taiga project without authentication.
    """
    # Create a Taiga client without authentication for public projects
    # No username or auth_token is needed
    client = TaigaClient(
        base_url="https://api.taiga.io/api/v1/"
        # Note: auth_token and username are optional and not provided for public projects
    )
    
    # Define a public project to fetch (replace with an actual public project)
    project_slug = "ibarraz5-ser402-team3"
    
    print(f"Fetching public project: {project_slug}")
    print("Note: No authentication credentials provided")
    
    try:
        # Fetch the project data
        project_data = await client.fetch_project_data(project_slug)
        
        # Display basic project information
        print("\n===== Project Information =====")
        print(f"Name: {project_data['project']['name']}")
        print(f"Description: {project_data['project']['description']}")
        
        # Display counts of items
        print("\n===== Project Statistics =====")
        print(f"Members: {len(project_data['members'])}")
        print(f"Sprints: {len(project_data['sprints'])}")
        
        # Count user stories
        total_stories = sum(len(stories) for stories in project_data['user_stories'].values())
        print(f"User Stories: {total_stories}")
        
        # Count tasks
        total_tasks = sum(len(tasks) for tasks in project_data['tasks'].values())
        print(f"Tasks: {total_tasks}")
        
        print("\nSuccess! Fetched public project data without authentication.")
        
    except Exception as e:
        print(f"Error: {e}")
        print("Note: If this is a private project, you will need to provide auth_token and username.")


if __name__ == "__main__":
    asyncio.run(main())
