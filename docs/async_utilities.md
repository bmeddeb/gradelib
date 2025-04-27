# Async Utilities in gradelib

This document describes the async utilities available in gradelib that make it easier to work with gradelib's asynchronous functions in various contexts.

## Why Async/Await in gradelib?

Gradelib uses asynchronous programming (`async/await`) to provide non-blocking operations for:

- Network requests (GitHub API, Taiga API)
- Git operations
- File I/O
- OAuth code exchange

This approach allows for better performance when working with multiple repositories or making parallel requests.

## Setup for Async Operations

Before using any async functionality in gradelib, you must initialize the async runtime environment:

```python
from gradelib import setup_async

# Initialize the async runtime environment
setup_async()
```

This function sets up the Tokio runtime (from Rust) to power gradelib's async operations.

## The async_handler Decorator

The `async_handler` decorator makes it easy to use async functions in synchronous contexts:

```python
from gradelib import async_handler

@async_handler
async def my_async_function(arg1, arg2):
    # Async operations here...
    return result

# Now you can call this from synchronous code:
result = my_async_function(arg1, arg2)  # No need for asyncio.run()!
```

This is particularly useful when:

- Working with web frameworks like Flask that are primarily synchronous
- Using gradelib in scripts or command-line tools
- Integrating with other synchronous libraries

### How async_handler Works

The `async_handler` decorator:

1. Creates or retrieves an event loop appropriate for the current thread
2. Runs the async function to completion in that event loop
3. Returns the result (or raises any exceptions) from the async function
4. Automatically applies `nest_asyncio` to handle nested event loops

### Handling Exceptions

Exceptions raised by your async function are properly propagated:

```python
@async_handler
async def might_fail():
    # This will raise a ValueError
    raise ValueError("Something went wrong")

try:
    might_fail()  # The ValueError will be raised here
except ValueError as e:
    print(f"Caught error: {e}")
```

### Threading Considerations

When using `async_handler` in multi-threaded applications:

- Each thread will get its own event loop
- The decorator handles creating event loops in new threads
- For web frameworks, consider disabling threading (e.g., `threaded=False` in Flask)

## Practical Examples

### Basic Example

```python
from gradelib import setup_async, async_handler
from gradelib import GitHubOAuthClient

setup_async()

@async_handler
async def get_github_token(code):
    return await GitHubOAuthClient.exchange_code_for_token(
        client_id="YOUR_CLIENT_ID",
        client_secret="YOUR_CLIENT_SECRET",
        code=code,
        redirect_uri="YOUR_REDIRECT_URI"
    )

# Use in synchronous code:
code = "authorization_code_from_github"
token = get_github_token(code)
print(f"Got token: {token}")
```

### Flask Integration

```python
from flask import Flask, request
from gradelib import setup_async, async_handler, GitHubOAuthClient

setup_async()
app = Flask(__name__)
app.config['THREADED'] = False  # Avoid threading issues with asyncio

@app.route('/callback')
def callback():
    code = request.args.get('code')
    token = exchange_code_for_token(code)
    # ...use token...
    return "Authentication successful!"

@async_handler
async def exchange_code_for_token(code):
    return await GitHubOAuthClient.exchange_code_for_token(
        client_id="YOUR_CLIENT_ID",
        client_secret="YOUR_CLIENT_SECRET",
        code=code,
        redirect_uri="YOUR_REDIRECT_URI"
    )
```

### CLI Tool Example

```python
import argparse
from gradelib import setup_async, async_handler, RepoManager

setup_async()

@async_handler
async def analyze_repo(repo_url, token):
    manager = RepoManager([repo_url], github_username="", github_token=token)
    await manager.clone(repo_url)
    status = await manager.fetch_clone_tasks()
    if status[repo_url].status.status_type == "completed":
        path = status[repo_url].temp_dir
        commits = await manager.analyze_commits(path)
        return commits
    else:
        raise RuntimeError(f"Failed to clone: {status[repo_url].status.error}")

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("repo_url", help="GitHub repository URL")
    parser.add_argument("token", help="GitHub personal access token")
    args = parser.parse_args()
    
    try:
        commits = analyze_repo(args.repo_url, args.token)
        print(f"Found {len(commits)} commits:")
        for commit in commits[:5]:  # Show first 5
            print(f"{commit['hash'][:8]} - {commit['message']}")
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    main()
```

## Installing Optional Dependencies

To use `async_handler` with web frameworks like Flask, you should install gradelib with the `web` extras:

```bash
pip install gradelib[web]
```

This will install `nest_asyncio` which is used internally by `async_handler` to handle nested event loops.

## Alternative: Using asyncio.run()

If you prefer not to use the `async_handler` decorator, you can still use the standard Python asyncio approach:

```python
import asyncio
from gradelib import setup_async, GitHubOAuthClient

setup_async()

async def main():
    token = await GitHubOAuthClient.exchange_code_for_token(
        client_id="YOUR_CLIENT_ID",
        client_secret="YOUR_CLIENT_SECRET",
        code="CODE_FROM_GITHUB",
        redirect_uri="YOUR_REDIRECT_URI"
    )
    print(f"Token: {token}")

# Run the async function
asyncio.run(main())
```

## When to Use Native Async Frameworks

For applications with heavy async usage, consider using natively async frameworks:

- **Quart** instead of Flask for web applications
- **AIOHTTP** or **httpx** for HTTP clients
- **FastAPI** for API development

In these frameworks, you can use `await` directly without needing the `async_handler` decorator.
