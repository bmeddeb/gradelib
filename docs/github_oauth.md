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

## Notes
- The access token you receive can be used for all GitHub API calls until it expires or is revoked.
- You only need to use the OAuth code exchange if you do **not** already have a personal access token.
- This helper is especially useful for web apps, desktop apps, or any integration where the user authorizes via GitHub's OAuth flow.