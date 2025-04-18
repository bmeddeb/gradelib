#!/usr/bin/env python3
"""
Simple script to demonstrate the TaigaClient functionality in gradelib.

This script fetches data from a specified Taiga project and prints summary information.
"""

import asyncio
import os
import json
from pprint import pprint

# Import necessary modules
from gradelib.gradelib import setup_async, TaigaClient



# Configuration - replace with your values or use environment variables
TAIGA_URL = os.getenv("TAIGA_URL", "https://api.taiga.io/api/v1/")
TAIGA_USERNAME = os.getenv("TAIGA_USERNAME", "your_username")
TAIGA_PASSWORD = os.getenv("TAIGA_PASSWORD", "your_password")
# You can provide token directly if you have it
TAIGA_TOKEN = os.getenv("TAIGA_TOKEN", None)

# The project slug from the URL (e.g., "myproject-slug")
PROJECT_SLUG = "your-project-slug"


async def get_auth_token(url, username, password):
    """Gets an authentication token from Taiga API using username and password."""
    import aiohttp

    auth_url = f"{url.rstrip('/')}/auth"
    payload = {
        "type": "normal",
        "username": username,
        "password": password
    }

    print(f"Authenticating with Taiga API...")
    async with aiohttp.ClientSession() as session:
        async with session.post(auth_url, json=payload) as response:
            if response.status == 200:
                data = await response.json()
                print("Authentication successful!")
                return data.get("auth_token")
            else:
                text = await response.text()
                raise Exception(f"Failed to get auth token: {response.status} - {text}")


async def main():
    """Main execution function that fetches and displays project data."""
    print("Taiga Project Data Example")
    print("==========================")

    try:
        # Initialize the async runtime (required for all async operations)
        setup_async()

        # Get authentication token if not provided
        auth_token = TAIGA_TOKEN
        if not auth_token and TAIGA_USERNAME != "your_username" and TAIGA_PASSWORD != "your_password":
            auth_token = await get_auth_token(TAIGA_URL, TAIGA_USERNAME, TAIGA_PASSWORD)

        if not auth_token:
            print("No authentication token available. Please set your Taiga credentials.")
            return

        # Create the Taiga client
        client = TaigaClient(
            base_url=TAIGA_URL,
            auth_token=auth_token,
            username=TAIGA_USERNAME
        )

        # Fetch project data
        print(f"\nFetching data for project: {PROJECT_SLUG}")
        project_data = await client.fetch_project_data(PROJECT_SLUG)

        # Display project summary
        print("\nProject Summary:")
        print(f"Name: {project_data['project']['name']}")
        print(f"Members: {len(project_data['members'])}")
        print(f"Sprints: {len(project_data['sprints'])}")

        total_stories = sum(len(stories) for stories in project_data['user_stories'].values())
        total_tasks = sum(len(tasks) for tasks in project_data['tasks'].values())

        print(f"User Stories: {total_stories}")
        print(f"Tasks: {total_tasks}")

        # Optionally save to file
        with open(f"{PROJECT_SLUG}_data.json", "w") as f:
            json.dump(project_data, f, indent=2)
        print(f"\nComplete data saved to {PROJECT_SLUG}_data.json")

    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    asyncio.run(main())