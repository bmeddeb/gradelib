#!/usr/bin/env python3
"""
Example demonstrating the Taiga provider functionality in gradelib.

This example shows how to fetch data from a Taiga project, including
projects, sprints, user stories, tasks, and task history.

Target project: https://tree.taiga.io/project/ibarraz5-ser402-team3/timeline
"""

import asyncio
import os
import json
from datetime import datetime
import aiohttp
from pprint import pprint
from gradelib import TaigaClient

# Configuration - either set environment variables or replace these values directly
TAIGA_URL = os.getenv("TAIGA_URL", "https://api.taiga.io/api/v1/")
TAIGA_USERNAME = os.getenv("TAIGA_USERNAME", "YOUR_TAIGA_USERNAME")
TAIGA_PASSWORD = os.getenv("TAIGA_PASSWORD", "YOUR_TAIGA_PASSWORD")  # New: password for authentication
TAIGA_TOKEN = os.getenv("TAIGA_TOKEN", None)  # Can provide token directly if available

# The project slug from the URL: https://tree.taiga.io/project/ibarraz5-ser402-team3/
PROJECT_SLUG = "ibarraz5-ser402-team3"


async def get_auth_token(url, username, password):
    """
    Gets an authentication token from Taiga API using username and password.

    Args:
        url: The base URL for the Taiga API
        username: Taiga username
        password: Taiga password

    Returns:
        The authentication token string if successful

    Raises:
        Exception: If authentication fails
    """
    auth_url = f"{url.rstrip('/')}/auth"
    payload = {
        "type": "normal",
        "username": username,
        "password": password
    }

    print(f"Authenticating with Taiga API at: {auth_url}")
    async with aiohttp.ClientSession() as session:
        async with session.post(auth_url, json=payload) as response:
            if response.status == 200:
                data = await response.json()
                print("Authentication successful!")
                return data.get("auth_token")
            else:
                text = await response.text()
                raise Exception(f"Failed to get auth token: {response.status} - {text}")


def display_section(title):
    """Helper function to display a section title."""
    print(f"\n{'-' * 80}")
    print(f" {title} ".center(80, '='))
    print(f"{'-' * 80}\n")


async def fetch_and_display_project_data(client):
    """Fetches and displays project data from Taiga."""
    display_section(f"Fetching Project: {PROJECT_SLUG}")

    try:
        # Fetch project data
        start_time = datetime.now()
        print(f"Starting fetch at: {start_time}")

        project_data = await client.fetch_project_data(PROJECT_SLUG)

        end_time = datetime.now()
        duration = (end_time - start_time).total_seconds()
        print(f"Completed fetch in {duration:.2f} seconds")

        # Display basic project information
        project_info = project_data.get('project', {})
        display_section("Project Information")
        print(f"ID: {project_info.get('id')}")
        print(f"Name: {project_info.get('name')}")
        print(f"Description: {project_info.get('description')}")
        print(f"Created: {project_info.get('created_date')}")

        # Display members
        members = project_data.get('members', [])
        display_section(f"Project Members ({len(members)})")
        for i, member in enumerate(members, 1):
            print(f"{i}. {member.get('full_name')} - {member.get('role_name')}")

        # Display sprints
        sprints = project_data.get('sprints', [])
        display_section(f"Sprints/Milestones ({len(sprints)})")
        for i, sprint in enumerate(sprints, 1):
            status = "Closed" if sprint.get('closed') else "Open"
            print(f"{i}. {sprint.get('name')} - {status}")
            print(f"   {sprint.get('estimated_start')} to {sprint.get('estimated_finish')}")

        # Display user stories summary
        user_stories = project_data.get('user_stories', {})
        total_stories = sum(len(stories) for stories in user_stories.values())
        display_section(f"User Stories ({total_stories})")
        for sprint_id, stories in user_stories.items():
            print(f"Sprint {sprint_id}: {len(stories)} stories")
            # Show first 3 stories as examples
            for i, story in enumerate(stories[:3], 1):
                print(f"  {i}. [{story.get('reference')}] {story.get('subject')}")
            if len(stories) > 3:
                print(f"  ... and {len(stories) - 3} more stories")

        # Display tasks summary
        tasks = project_data.get('tasks', {})
        total_tasks = sum(len(sprint_tasks) for sprint_tasks in tasks.values())
        display_section(f"Tasks ({total_tasks})")
        for sprint_id, sprint_tasks in tasks.items():
            completed = sum(1 for task in sprint_tasks if task.get('is_closed'))
            print(f"Sprint {sprint_id}: {len(sprint_tasks)} tasks, {completed} completed")

        # Display task history summary
        histories = project_data.get('task_histories', {})
        total_history_events = sum(len(events) for events in histories.values())
        display_section(f"Task History Events ({total_history_events})")
        print(f"Total tasks with history: {len(histories)}")
        if histories:
            # Show a sample of history events for one task
            sample_task_id = next(iter(histories.keys()))
            sample_events = histories[sample_task_id]
            print(f"\nSample events for Task {sample_task_id}:")
            for i, event in enumerate(sample_events[:5], 1):
                print(f"  {i}. Event {event.get('id')} - Type: {event.get('event_type')} - {event.get('created_at')}")

        # Save the data to a JSON file for further inspection
        with open(f"{PROJECT_SLUG}_data.json", "w") as f:
            json.dump(project_data, f, indent=2)
        print(f"\nComplete data saved to {PROJECT_SLUG}_data.json")

    except Exception as e:
        print(f"Error fetching project data: {e}")


async def test_multiple_projects(client):
    """Tests fetching multiple projects concurrently."""
    display_section("Testing Multiple Projects Fetch")

    # Add a list of project slugs to test concurrent fetching
    # Using our real project and some fake ones to demonstrate error handling
    slugs = [
        PROJECT_SLUG,
        "ibarraz5-ser402-team3",
        "ibarraz5-ser401-team3-1"
    ]

    print(f"Fetching {len(slugs)} projects concurrently...")
    start_time = datetime.now()

    try:
        results = await client.fetch_multiple_projects(slugs)

        end_time = datetime.now()
        duration = (end_time - start_time).total_seconds()
        print(f"Completed fetch in {duration:.2f} seconds")

        # Display results
        for slug, result in results.items():
            if result is True:
                print(f"✅ {slug}: Successfully fetched")
            else:
                print(f"❌ {slug}: {result}")

    except Exception as e:
        print(f"Error fetching multiple projects: {e}")


async def main_async():
    """Asynchronous main function that runs the example."""
    display_section("Taiga Provider Example")

    try:
        # Authenticate with Taiga
        auth_token = TAIGA_TOKEN
        if not auth_token and TAIGA_USERNAME != "YOUR_TAIGA_USERNAME" and TAIGA_PASSWORD != "YOUR_TAIGA_PASSWORD":
            auth_token = await get_auth_token(TAIGA_URL, TAIGA_USERNAME, TAIGA_PASSWORD)
        elif not auth_token:
            print("No authentication token provided and default credentials detected.")
            print("Please set your Taiga credentials via environment variables or edit the script.")
            return

        # Create the Taiga client
        client = TaigaClient(
            base_url=TAIGA_URL,
            auth_token=auth_token,
            username=TAIGA_USERNAME
        )

        # Run the example functions
        await fetch_and_display_project_data(client)
        await test_multiple_projects(client)

    except Exception as e:
        print(f"Error in main_async function: {e}")


def main():
    """Main execution function - ensures async execution works on all platforms."""
    print("Taiga Provider Example")
    print("======================")

    try:
        # Check if TaigaClient is available
        print("Checking for TaigaClient...")
        try:
            from gradelib import TaigaClient
            print("✅ TaigaClient found in gradelib")
        except ImportError:
            try:
                from gradelib.taiga import TaigaClient
                print("✅ TaigaClient found in gradelib.taiga")
            except ImportError:
                print("❌ TaigaClient not found. Make sure the Taiga provider is properly built and installed.")
                return

        # Run the async main function
        print("\nStarting Taiga API example...")
        asyncio.run(main_async())

    except Exception as e:
        print(f"Error in main function: {e}")


if __name__ == "__main__":
    main()