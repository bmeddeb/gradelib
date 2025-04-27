"""
Flask OAuth Example with gradelib

This is a minimal, single-file Flask application that demonstrates how to use
the GitHubOAuthClient from the gradelib library to handle GitHub OAuth authentication.

To run this example:
1. Install required packages:
   pip install gradelib[web]

2. Create a .env file with your GitHub OAuth credentials:
   GITHUB_CLIENT_ID=your_client_id
   GITHUB_CLIENT_SECRET=your_client_secret
   GITHUB_REDIRECT_URI=http://127.0.0.1:5000/callback
   SECRET_KEY=your_secret_key

3. Run the app:
   python flask_oauth_example.py
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
        # Exchange the code for a token using the async_handler
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
