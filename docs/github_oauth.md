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
from gradelib import GitHubOAuthClient

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
## Flask example implementation
```python
# app.py
import os
import asyncio
from flask import Flask, redirect, request, render_template_string
from gradelib import GitHubOAuthClient

app = Flask(__name__)

# Configuration - set these in your environment
os.environ['GITHUB_CLIENT_ID'] = 'your_client_id'  # Replace with actual
os.environ['GITHUB_CLIENT_SECRET'] = 'your_client_secret'  # Replace with actual
os.environ['GITHUB_REDIRECT_URI'] = 'http://127.0.0.1:5000/github/callback'

@app.route('/')
def home():
    return render_template_string('''
        <h1>GitHub OAuth Demo</h1>
        <a href="/login"><button>Login with GitHub</button></a>
    ''')

@app.route('/login')
def login():
    # Redirect to GitHub's authorization page
    client_id = os.getenv('GITHUB_CLIENT_ID')
    redirect_uri = os.getenv('GITHUB_REDIRECT_URI')
    scope = 'repo'  # Adjust scopes as needed

    auth_url = (
        f"https://github.com/login/oauth/authorize"
        f"?client_id={client_id}"
        f"&redirect_uri={redirect_uri}"
        f"&scope={scope}"
    )
    return redirect(auth_url)

@app.route('/github/callback')
def callback():
    code = request.args.get('code')
    error = request.args.get('error')

    if error:
        return f"Authorization failed: {error}"

    if not code:
        return "Missing authorization code", 400

    try:
        # Exchange code for token
        token = asyncio.run(
            GitHubOAuthClient.exchange_code_for_token(
                client_id=os.getenv('GITHUB_CLIENT_ID'),
                client_secret=os.getenv('GITHUB_CLIENT_SECRET'),
                code=code,
                redirect_uri=os.getenv('GITHUB_REDIRECT_URI')
            )
        )

        return render_template_string('''
            <h1>Success!</h1>
            <p>Access Token: <code>{{ token }}</code></p>
            <p style="color: red;">Warning: Never expose this token publicly!</p>
        ''', token=token)

    except Exception as e:
        return render_template_string('''
            <h1>Error</h1>
            <p>{{ error }}</p>
        ''', error=str(e))

if __name__ == '__main__':
    app.run(debug=True)
```
## Notes
- The access token you receive can be used for all GitHub API calls until it expires or is revoked.
- You only need to use the OAuth code exchange if you do **not** already have a personal access token.
- This helper is especially useful for web apps, desktop apps, or any integration where the user authorizes via GitHub's OAuth flow.