#!/usr/bin/env python3
"""
Debug script to find the null value at column 3317 in the Taiga API response.
"""

import asyncio
import aiohttp
import os
import json
import sys

# Configuration
TAIGA_URL = os.getenv("TAIGA_URL", "https://api.taiga.io/api/v1/")
TAIGA_USERNAME = os.getenv("TAIGA_USERNAME", "YOUR_TAIGA_USERNAME")
TAIGA_PASSWORD = os.getenv("TAIGA_PASSWORD", "YOUR_TAIGA_PASSWORD")  # New: password for authentication
TAIGA_TOKEN = os.getenv("TAIGA_TOKEN", None)  # Can provide token directly if available

PROJECT_SLUG = os.getenv("TAIGA_PROJECT", "ibarraz5-ser402-team3")  # Replace with your project slug

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


async def fetch_project_json(auth_token, username, slug):
    """Fetch raw project JSON from Taiga API."""
    headers = {
        "Accept": "application/json",
        "Authorization": f"Bearer {auth_token}",
        "User-Agent": "gradelib-taiga-debug",
    }
    
    url = f"{TAIGA_URL.rstrip('/')}/projects/by_slug?slug={slug}"
    print(f"Fetching project data from: {url}")
    
    async with aiohttp.ClientSession() as session:
        async with session.get(url, headers=headers) as response:
            if response.status == 200:
                text = await response.text()
                
                # Find the problematic part around column 3317
                if len(text) > 3317:
                    start = max(0, 3317 - 50)
                    end = min(len(text), 3317 + 50)
                    
                    print("\n--- JSON around column 3317 ---")
                    print(f"...{text[start:3317]}[COLUMN 3317 HERE]{text[3317:end]}...")
                
                try:
                    # Try to parse the JSON
                    data = json.loads(text)
                    print("\nJSON parsed successfully!")
                    
                    # Save the full JSON to a file
                    with open(f"{slug}_debug.json", "w") as f:
                        json.dump(data, f, indent=2)
                    print(f"Full JSON saved to {slug}_debug.json")
                    
                    return data
                except json.JSONDecodeError as e:
                    print(f"JSON parse error: {e}")
                    # Print the problematic part of the JSON
                    problem_pos = e.pos
                    print(f"Error at position {problem_pos}")
                    start = max(0, problem_pos - 20)
                    end = min(len(text), problem_pos + 20)
                    print(f"Context: '{text[start:end]}'")
                    return None
            else:
                text = await response.text()
                print(f"Error fetching project data: {response.status} - {text}")
                return None

async def main():
    """Main function."""
    if not TAIGA_USERNAME or not TAIGA_PASSWORD:
        print("Please set TAIGA_USERNAME and TAIGA_PASSWORD environment variables")
        return
    
    try:
        # Get authentication token
        print(f"Authenticating as {TAIGA_USERNAME}...")
        auth_token = await get_auth_token(TAIGA_URL, TAIGA_USERNAME, TAIGA_PASSWORD)
        print(f"Authentication successful, received token: {auth_token[:10]}...")
        
        # Fetch project data
        project_data = await fetch_project_json(auth_token, TAIGA_USERNAME, PROJECT_SLUG)
        
        if project_data:
            print("\nProject data retrieved successfully")
            print(f"Project ID: {project_data.get('id')}")
            print(f"Project Name: {project_data.get('name')}")
        else:
            print("Failed to retrieve project data")
    
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    asyncio.run(main()) 