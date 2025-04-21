[Setup](setup.md)

Before using GradeLib, ensure you have the necessary environment set up:

```python
import asyncio
import os
from gradelib.gradelib import setup_async, RepoManager

# Initialize the async runtime environment
setup_async()

# Set GitHub credentials (preferably from environment variables for security)
github_username = os.environ.get("GITHUB_USERNAME", "your_username")
github_token = os.environ.get("GITHUB_TOKEN", "your_personal_access_token")

# List of repositories to analyze
repo_urls = [
    "https://github.com/username/repo1",
    "https://github.com/username/repo2",
]
```