[Taiga-Provider](taiga-provider.md)

# Taiga Provider for gradelib

This document provides instructions for using the Taiga provider in the gradelib Python library. The Taiga provider enables you to fetch data from Taiga projects asynchronously and efficiently using Rust's performance benefits.

## Features

- Fetch comprehensive project data including:
  - Project details
  - Project members
  - Sprints/milestones (both open and closed)
  - User stories
  - Tasks
  - Task history
- Concurrent fetching for multiple projects
- Efficient async implementation with Tokio

## Authentication

To use the Taiga API, you need:

1. A Taiga account with access to the projects you want to fetch
2. An authentication token from Taiga

### Getting a Taiga Authentication Token

There are two ways to obtain a Taiga authentication token:

#### Option 1: Using the API (Recommended)

The most reliable way is to authenticate directly with the Taiga API using your username and password. Here's a Python function to help with that:

```python
async def get_auth_token(url, username, password):
    """Gets an authentication token from Taiga."""
    import aiohttp

    auth_url = f"{url.rstrip('/')}/auth"
    payload = {
        "type": "normal",
        "username": username,
        "password": password
    }

    async with aiohttp.ClientSession() as session:
        async with session.post(auth_url, json=payload) as response:
            if response.status == 200:
                data = await response.json()
                return data.get("auth_token")
            else:
                text = await response.text()
                raise Exception(f"Failed to get auth token: {response.status} - {text}")
```

You can use this function like this:

```python
token = await get_auth_token("https://api.taiga.io/api/v1/", "your_username", "your_password")
```

The response will contain the `auth_token` that you can use for subsequent API calls.

## Option 2: Using the Taiga Web Interface
Alternatively, you can extract the token from the web interface:

1. Log into your Taiga account in a web browser
2. Open your browser's developer tools (`F12` or `Ctrl+Shift+I`)
3. Go to the **Network** tab and filter for `XHR`/`Fetch` requests
4. Reload the page or perform an action that triggers an API call
5. Look for requests to the Taiga API and examine their headers
6. Find the `Authorization` header that contains `Bearer YOUR_TOKEN`
7. The token is the string after `Bearer`

## Option 3: Using cURL
You can also use cURL to get a token:
```bash
curl -X POST \
  https://api.taiga.io/api/v1/auth \
  -H 'Content-Type: application/json' \
  -d '{
    "type": "normal",
    "username": "your_username",
    "password": "your_password"
}'
```

## Example Usage
### Basic Usage

```python
import asyncio
import aiohttp
from gradelib import TaigaClient

async def get_auth_token(url, username, password):
    """Gets an authentication token from Taiga."""
    auth_url = f"{url.rstrip('/')}/auth"
    payload = {
        "type": "normal",
        "username": username,
        "password": password
    }

    async with aiohttp.ClientSession() as session:
        async with session.post(auth_url, json=payload) as response:
            if response.status == 200:
                data = await response.json()
                return data.get("auth_token")
            else:
                text = await response.text()
                raise Exception(f"Failed to get auth token: {response.status} - {text}")

async def fetch_project():
    # Authenticate and get token
    token = await get_auth_token(
        "https://api.taiga.io/api/v1/",
        "your_username",
        "your_password"
    )

    # Create a client
    client = TaigaClient(
        base_url="https://api.taiga.io/api/v1/",
        auth_token=token,
        username="your_username"
    )

    # Fetch a single project
    project_data = await client.fetch_project_data("project-slug")

    # Access the data
    print(f"Project: {project_data['project']['name']}")
    print(f"Members: {len(project_data['members'])}")
    print(f"Sprints: {len(project_data['sprints'])}")

    # Fetch multiple projects concurrently
    results = await client.fetch_multiple_projects(["project1", "project2"])

    for slug, result in results.items():
        if result is True:
            print(f"{slug}: Successfully fetched")
        else:
            print(f"{slug}: {result}")

# Run the async function
asyncio.run(fetch_project())
```

## Running the Example Script
The repository includes an example script in `examples/taiga_example.py` that demonstrates all features of the Taiga provider. The script also includes a diagnostic function to check if the Taiga provider is properly installed in your environment.

### To run it:
Set your Taiga credentials as environment variables:
```bash
export TAIGA_URL="https://api.taiga.io/api/v1/"
export TAIGA_USERNAME="your_taiga_username"
export TAIGA_PASSWORD="your_taiga_password"
```

### run the script:
Requires uv
```python
uv run examples/taiga_example.py

# Or if you do not have uv installed
python examples/taiga_example.py
```

# Data Structure
### The `fetch_project_data` method returns a dictionary with the following structure:
```json
{
  "project": {
    "id": 123,
    "name": "Project Name",
    "slug": "project-slug",
    "description": "Project description",
    "created_date": "2023-01-01T00:00:00.000Z",
    "modified_date": "2023-01-02T00:00:00.000Z"
  },
  "members": [
    {
      "id": 456,
      "user": 789,
      "role": 1,
      "role_name": "Product Owner",
      "full_name": "John Doe"
    },
    ...
  ],
  "sprints": [
    {
      "id": 101,
      "name": "Sprint 1",
      "estimated_start": "2023-01-15",
      "estimated_finish": "2023-01-31",
      "created_date": "2023-01-10T00:00:00.000Z",
      "closed": false
    },
    ...
  ],
  "user_stories": {
    "101": [  # Sprint ID
      {
        "id": 201,
        "reference": 1,
        "subject": "User Story 1",
        "status": "In progress"
      },
      ...
    ],
    ...
  },
  "tasks": {
    "101": [  # Sprint ID
      {
        "id": 301,
        "reference": 1,
        "subject": "Task 1",
        "is_closed": false,
        "assigned_to": 789
      },
      ...
    ],
    ...
  },
  "task_histories": {
    "301": [  # Task ID
      {
        "id": 401,
        "created_at": "2023-01-20T00:00:00.000Z",
        "event_type": 1
      },
      ...
    ],
    ...
  }
}
```

#  Dependencies
* Python 3.9+
* aiohttp (pip install aiohttp)
* gradelib

## Usage Example

```python
results = await client.fetch_multiple_projects(slugs)
# results is a dict: {slug: True or error string}
# For each slug, the value is either True (on success) or an error string (on failure for that slug).
# No exceptions are raised for individual failures; a single failure does not abort the batch.
```
