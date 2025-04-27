# GitHub OAuth Code Exchange in gradelib

## When Do You Need This?

- **If you have a personal access token (PAT):**
  - You do **not** need to use the OAuth code exchange helper. Just pass your token to `RepoManager` or other gradelib classes.
- **If you are using a GitHub App or OAuth App:**
  - You will receive an **authorization code** after the user authorizes your app.
  - You must exchange this code for an **access token** before you can use the GitHub API.
  - This is where `GitHubOAuthClient.exchange_code_for_token` comes in!

## OAuth Flow Overview

1. **Redirect the user to GitHub's authorization URL:**
   - Example: `https://github.com/login/oauth/authorize?client_id=YOUR_CLIENT_ID&redirect_uri=YOUR_REDIRECT_URI&scope=repo`
2. **User authorizes your app.**
3. **GitHub redirects back to your app** with a `code` parameter in the URL.
4. **Exchange the code for an access token** using the helper in gradelib.

## Endpoints
- **Authorization URL:** `https://github.com/login/oauth/authorize`
- **Token Exchange URL:** `https://github.com/login/oauth/access_token`
- **GitHub API Base URL:** `https://api.github.com`

## Example Usage in Python

```python
import asyncio
from gradelib import GitHubOAuthClient, setup_async

# Initialize async runtime environment
setup_async()

async def get_token():
    token = await GitHubOAuthClient.exchange_code_for_token(
        client_id="YOUR_CLIENT_ID",
        client_secret="YOUR_CLIENT_SECRET",
        code="CODE_FROM_GITHUB",
        redirect_uri="YOUR_REDIRECT_URI"
    )
    print("Access token:", token)

# Run this after you have received the code from the OAuth redirect
asyncio.run(get_token())
```

## Using the async_handler for Web Applications

Gradelib now includes an `async_handler` decorator that makes it easy to use async functions in synchronous contexts like Flask routes:

```python
from gradelib import GitHubOAuthClient, setup_async, async_handler

# Initialize async runtime environment
setup_async()

@async_handler
async def exchange_token(code):
    """Exchange authorization code for access token."""
    return await GitHubOAuthClient.exchange_code_for_token(
        client_id="YOUR_CLIENT_ID",
        client_secret="YOUR_CLIENT_SECRET",
        code=code,
        redirect_uri="YOUR_REDIRECT_URI"
    )

# Now you can call this from synchronous code:
def handle_callback(code):
    token = exchange_token(code)  # No need for asyncio.run()!
    # Use the token...
```

## Full OAuth Flow Example

1. **Direct user to GitHub for authorization:**
   ```
   https://github.com/login/oauth/authorize?client_id=YOUR_CLIENT_ID&redirect_uri=YOUR_REDIRECT_URI&scope=repo
   ```
2. **User logs in and authorizes.**
3. **GitHub redirects to:**
   ```
   YOUR_REDIRECT_URI?code=THE_CODE
   ```
4. **Exchange the code for a token:**
   - Use the Python example above.
5. **Use the access token with gradelib:**
   ```python
   from gradelib import RepoManager
   manager = RepoManager([
       "https://github.com/owner/repo"
   ], github_token=token)
   # ... use manager as normal ...
   ```

## Flask Example with async_handler

To use gradelib's OAuth client with Flask, you'll need to install the `web` extras:

```bash
pip install gradelib[web]
```

Then you can use the built-in `async_handler` to simplify your code:

```python
"""
Flask OAuth Example with gradelib

This is a minimal, single-file Flask application that demonstrates how to use
the GitHubOAuthClient from the gradelib library to handle GitHub OAuth authentication.
"""

import os
from flask import Flask, redirect, request, session, url_for
from dotenv import load_dotenv
from gradelib import GitHubOAuthClient, setup_async, async_handler

# Load environment variables from .env file
load_dotenv()

# Initialize the async runtime environment
setup_async()

# Configure app
app = Flask(__name__)
app.secret_key = os.getenv('SECRET_KEY', 'dev-secret-key')
app.config['THREADED'] = False  # Avoid threading issues with asyncio

# GitHub OAuth configuration
GITHUB_CLIENT_ID = os.getenv('GITHUB_CLIENT_ID')
GITHUB_CLIENT_SECRET = os.getenv('GITHUB_CLIENT_SECRET')
GITHUB_REDIRECT_URI = os.getenv('GITHUB_REDIRECT_URI', 'http://127.0.0.1:5000/callback')

@app.route('/')
def index():
    """Simple home page with login link or token display."""
    token = session.get('github_token')
    if token:
        return f"""
        <h1>GitHub OAuth Demo</h1>
        <p>You are authenticated!</p>
        <p>Token: {token[:10]}...</p>
        <p><a href="/logout">Logout</a></p>
        """
    else:
        return f"""
        <h1>GitHub OAuth Demo</h1>
        <p>You are not authenticated.</p>
        <p><a href="/login">Login with GitHub</a></p>
        """

@app.route('/login')
def login():
    """Redirect to GitHub's authorization page."""
    auth_url = (
        f"https://github.com/login/oauth/authorize"
        f"?client_id={GITHUB_CLIENT_ID}"
        f"&redirect_uri={GITHUB_REDIRECT_URI}"
        f"&scope=repo"  # Adjust scopes as needed
    )
    return redirect(auth_url)

@app.route('/callback')
def callback():
    """Handle the OAuth callback from GitHub."""
    # Get the authorization code from the query parameters
    code = request.args.get('code')
    error = request.args.get('error')

    if error:
        return f"Authorization failed: {error}"

    if not code:
        return "Missing authorization code", 400

    try:
        # Exchange the code for a token using our async_handler
        token = exchange_token(
            client_id=GITHUB_CLIENT_ID,
            client_secret=GITHUB_CLIENT_SECRET,
            code=code,
            redirect_uri=GITHUB_REDIRECT_URI
        )
        
        # Store the token in the session
        session['github_token'] = token
        
        return redirect(url_for('index'))
    
    except Exception as e:
        return f"Error authenticating with GitHub: {str(e)}"

@app.route('/logout')
def logout():
    """Clear the session data."""
    session.pop('github_token', None)
    return redirect(url_for('index'))

# Use the async_handler decorator from gradelib
@async_handler
async def exchange_token(client_id, client_secret, code, redirect_uri):
    """Exchange authorization code for access token."""
    return await GitHubOAuthClient.exchange_code_for_token(
        client_id=client_id,
        client_secret=client_secret,
        code=code,
        redirect_uri=redirect_uri
    )

if __name__ == '__main__':
    # Make sure to run with threading disabled
    app.run(debug=True, threaded=False, port=5000)
```

## Quart Integration

For better performance and more natural async code, consider using [Quart](https://pgjones.gitlab.io/quart/) instead of Flask. Quart supports `async/await` syntax directly:

```python
import os
from quart import Quart, redirect, request, session, url_for
from dotenv import load_dotenv
from gradelib import GitHubOAuthClient, setup_async

# Load environment variables and set up async
load_dotenv()
setup_async()

app = Quart(__name__)
app.secret_key = os.getenv('SECRET_KEY', 'dev-secret-key')

# ... routes similar to Flask example ...

@app.route('/callback')
async def callback():
    code = request.args.get('code')
    # No async_handler needed - can use await directly!
    token = await GitHubOAuthClient.exchange_code_for_token(
        client_id=os.getenv('GITHUB_CLIENT_ID'),
        client_secret=os.getenv('GITHUB_CLIENT_SECRET'),
        code=code,
        redirect_uri=os.getenv('GITHUB_REDIRECT_URI')
    )
    session['github_token'] = token
    return redirect(url_for('index'))

# ... rest of app ...
```

## Notes
- The access token you receive can be used for all GitHub API calls until it expires or is revoked.
- You only need to use the OAuth code exchange if you do **not** already have a personal access token.
- This helper is especially useful for web apps, desktop apps, or any integration where the user authorizes via GitHub's OAuth flow.
- The `async_handler` decorator is included with gradelib to make it easier to use async functions in synchronous contexts.
- For web applications with heavy async usage, consider using Quart instead of Flask for native async support.
