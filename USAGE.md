# GradeLib Usage Guide

This guide demonstrates how to use the `gradelib` library to interact with GitHub repositories, analyze code, and extract useful information for educational assessment purposes.

## Installation

```bash
pip install gradelib
```

## Basic Setup

Before using any GitHub-related functionality, you need to initialize the async runtime:

```python
from gradelib import setup_async

# Initialize the async runtime
setup_async()
```

## GitHub Provider

The `GitHubProvider` class is the main interface for interacting with GitHub repositories.

### Initialization

```python
from gradelib import GitHubProvider

# Initialize with GitHub credentials
github = GitHubProvider(
    username="your_github_username",  # Your GitHub username
    token="your_github_token",        # Your GitHub personal access token
    urls=[]                           # List of repository URLs to work with (can be empty initially)
)

# Initialize with repositories
github = GitHubProvider(
    username="your_github_username",
    token="your_github_token",
    urls=[
        "https://github.com/username/repo1",
        "https://github.com/username/repo2"
    ]
)
```

### Cloning Repositories

```python
import asyncio

# Clone a single repository
async def clone_repo():
    repo_url = "https://github.com/username/repo"
    await github.clone(repo_url)

# Run the async function
asyncio.run(clone_repo())
```

### Cloning Multiple Repositories

You can clone multiple repositories by providing the URLs during initialization or by using clone_all:

```python
import asyncio

# Method 1: Initialize with repositories and then clone all
async def clone_all_repos_method1():
    # Initialize with multiple repository URLs
    github = GitHubProvider(
        username="your_github_username",
        token="your_github_token",
        urls=[
            "https://github.com/username/repo1",
            "https://github.com/username/repo2",
            "https://github.com/username/repo3"
        ]
    )

    # Clone all repositories specified during initialization
    await github.clone_all()

# Method 2: Add repositories after initialization and then clone all
async def clone_all_repos_method2():
    # Initialize with empty URLs
    github = GitHubProvider(
        username="your_github_username",
        token="your_github_token",
        urls=[]
    )

    # Add repositories
    repos = [
        "https://github.com/username/repo1",
        "https://github.com/username/repo2",
        "https://github.com/username/repo3"
    ]

    # Clone each repository
    for repo_url in repos:
        await github.clone(repo_url)

# Run the async function
asyncio.run(clone_all_repos_method1())
# or
asyncio.run(clone_all_repos_method2())
```

### Monitoring Clone Progress

Cloning is performed asynchronously in the background. You can monitor the progress of cloning operations using the `fetch_clone_tasks` method:

```python
import asyncio
import time

async def monitor_clone_progress():
    # Initialize the GitHub provider
    github = GitHubProvider(
        username="your_github_username",
        token="your_github_token",
        urls=[]
    )

    # Start cloning a repository
    repo_url = "https://github.com/username/repo"
    await github.clone(repo_url)

    # Monitor the cloning progress
    completed = False
    while not completed:
        # Get the current status of all clone tasks
        tasks = await github.fetch_clone_tasks()

        # Check the status of our specific repository
        if repo_url in tasks:
            task_info = tasks[repo_url]

            # Check the status type
            status_type = task_info.get("status_type")

            if status_type == "queued":
                print(f"Repository {repo_url} is queued for cloning...")

            elif status_type == "cloning":
                progress = task_info.get("progress", 0)
                print(f"Cloning {repo_url}... Progress: {progress}%")

            elif status_type == "completed":
                print(f"Repository {repo_url} has been cloned successfully!")
                completed = True

            elif status_type == "failed":
                error = task_info.get("error", "Unknown error")
                print(f"Failed to clone {repo_url}: {error}")
                completed = True

        # Wait a bit before checking again
        if not completed:
            await asyncio.sleep(1)

# Run the monitoring function
asyncio.run(monitor_clone_progress())
```

The `fetch_clone_tasks` method returns a dictionary where:
- Keys are repository URLs
- Values are dictionaries containing:
  - `status_type`: One of "queued", "cloning", "completed", or "failed"
  - `progress`: For "cloning" status, indicates the percentage (0-100) of completion
  - `error`: For "failed" status, contains the error message

### Fetching Clone Tasks

Retrieve information about clone tasks:

```python
async def get_clone_tasks():
    # Get all clone tasks
    tasks = await github.fetch_clone_tasks()
    # tasks is a dictionary with repo URLs as keys

    for repo_url, task_info in tasks.items():
        print(f"Repository: {repo_url}")
        print(f"Task Status: {task_info['status_type']}")

        # Additional information based on status
        if task_info['status_type'] == 'cloning':
            print(f"Progress: {task_info['progress']}%")
        elif task_info['status_type'] == 'failed':
            print(f"Error: {task_info['error']}")

asyncio.run(get_clone_tasks())
```

### Analyzing Commits

Analyze commits in a repository:

```python
async def analyze_repo_commits():
    repo_url = "https://github.com/username/repo"

    # Get all commits in the repository
    commits = await github.analyze_commits(repo_url)

    for commit in commits:
        print(f"Commit: {commit['sha']}")
        print(f"Author: {commit['author_name']} <{commit['author_email']}>")
        print(f"Message: {commit['message']}")
        print(f"Additions: {commit['additions']}, Deletions: {commit['deletions']}")
        print(f"Merge commit: {commit['is_merge']}")
        print("---")

asyncio.run(analyze_repo_commits())
```

### Analyzing File Blame

Get line-by-line blame information for files:

```python
async def analyze_file_blame():
    repo_url = "https://github.com/username/repo"
    files = ["README.md", "src/main.py", "docs/usage.md"]

    # Get blame information for multiple files
    blame_results = await github.bulk_blame(repo_url, files)

    for file_path, blame_data in blame_results.items():
        print(f"File: {file_path}")

        if isinstance(blame_data, list):
            for line in blame_data:
                print(f"Line {line['final_line_no']}: {line['line_content']}")
                print(f"Author: {line['author_name']} <{line['author_email']}>")
                print(f"Commit: {line['commit_id']}")
        else:
            print(f"Error: {blame_data}")

        print("---")

asyncio.run(analyze_file_blame())
```

### Analyzing Branches

Get information about branches in repositories:

```python
async def analyze_repo_branches():
    repos = ["https://github.com/username/repo1", "https://github.com/username/repo2"]

    # Analyze branches across multiple repositories
    branch_results = await github.analyze_branches(repos)

    for repo_url, branches in branch_results.items():
        print(f"Repository: {repo_url}")

        if isinstance(branches, list):
            for branch in branches:
                print(f"Branch: {branch['name']}")
                print(f"Is remote: {branch['is_remote']}")
                print(f"Commit ID: {branch['commit_id']}")
                print(f"Commit message: {branch['commit_message']}")
                print(f"Author: {branch['author_name']} <{branch['author_email']}>")
                print(f"Is HEAD: {branch['is_head']}")
                print("---")

asyncio.run(analyze_repo_branches())
```

### Fetching Collaborators

Get information about repository collaborators:

```python
async def get_repo_collaborators():
    repos = ["https://github.com/username/repo1", "https://github.com/username/repo2"]

    # Get collaborators for multiple repositories
    collaborator_results = await github.fetch_collaborators(repos)

    for repo_url, collaborators in collaborator_results.items():
        print(f"Repository: {repo_url}")

        if isinstance(collaborators, list):
            for collab in collaborators:
                print(f"Username: {collab['login']}")
                print(f"GitHub ID: {collab['github_id']}")
                # Other available fields may include permissions, etc.

        print("---")

asyncio.run(get_repo_collaborators())
```

### Fetching Issues

Get information about repository issues:

```python
async def get_repo_issues():
    repos = ["https://github.com/username/repo1", "https://github.com/username/repo2"]

    # Get open issues
    open_issues = await github.fetch_issues(repos, state="open")

    # Get closed issues
    closed_issues = await github.fetch_issues(repos, state="closed")

    # Get all issues
    all_issues = await github.fetch_issues(repos, state="all")

    # Example of processing issues
    for repo_url, issues in open_issues.items():
        print(f"Repository: {repo_url}")

        if isinstance(issues, list):
            for issue in issues:
                print(f"Issue #{issue['number']}: {issue['title']}")
                print(f"State: {issue['state']}")
                print(f"Created: {issue['created_at']}")
                print(f"Updated: {issue['updated_at']}")
                print(f"User: {issue['user_login']} (ID: {issue['user_id']})")
                print(f"Comments: {issue['comments_count']}")
                print(f"Is PR: {issue['is_pull_request']}")

                if issue['labels']:
                    print("Labels:", ", ".join(issue['labels']))

                if issue['assignees']:
                    print("Assignees:", ", ".join(issue['assignees']))

                print(f"URL: {issue['html_url']}")

        print("---")

asyncio.run(get_repo_issues())
```

### Fetching Pull Requests

Get information about repository pull requests:

```python
async def get_repo_pull_requests():
    repos = ["https://github.com/username/repo1", "https://github.com/username/repo2"]

    # Get open PRs
    open_prs = await github.fetch_pull_requests(repos, state="open")

    # Get closed PRs
    closed_prs = await github.fetch_pull_requests(repos, state="closed")

    # Get all PRs
    all_prs = await github.fetch_pull_requests(repos, state="all")

    # Example of processing PRs
    for repo_url, prs in open_prs.items():
        print(f"Repository: {repo_url}")

        if isinstance(prs, list):
            for pr in prs:
                print(f"PR #{pr['number']}: {pr['title']}")
                print(f"State: {pr['state']}")
                print(f"Created: {pr['created_at']}")
                print(f"Updated: {pr['updated_at']}")
                print(f"User: {pr['user_login']} (ID: {pr['user_id']})")

                print(f"Comments: {pr['comments']}")
                print(f"Commits: {pr['commits']}")
                print(f"Additions: {pr['additions']}")
                print(f"Deletions: {pr['deletions']}")
                print(f"Changed files: {pr['changed_files']}")

                if pr['labels']:
                    print("Labels:", ", ".join(pr['labels']))

                print(f"Draft: {pr['is_draft']}")
                print(f"Merged: {pr['merged']}")

        print("---")

asyncio.run(get_repo_pull_requests())
```

## Complete Example

Here's a comprehensive example that demonstrates how to use multiple features together, including proper error handling:

```python
import asyncio
import os
from gradelib import setup_async, GitHubProvider

async def analyze_student_repositories():
    # Initialize the async runtime
    setup_async()

    # Get GitHub credentials from environment variables for security
    username = os.environ.get("GITHUB_USERNAME")
    token = os.environ.get("GITHUB_TOKEN")

    if not username or not token:
        print("Error: GitHub credentials not found in environment variables.")
        print("Please set GITHUB_USERNAME and GITHUB_TOKEN environment variables.")
        return

    try:
        # Set up the GitHub provider
        github = GitHubProvider(
            username=username,
            token=token,
            urls=[]
        )

        # Student repositories to analyze
        student_repos = [
            "https://github.com/student1/assignment-repo",
            "https://github.com/student2/assignment-repo",
            "https://github.com/student3/assignment-repo"
        ]

        # Start cloning all repositories
        for repo_url in student_repos:
            try:
                print(f"Starting clone for {repo_url}...")
                await github.clone(repo_url)
            except Exception as e:
                print(f"Error starting clone for {repo_url}: {str(e)}")

        # Monitor cloning progress
        all_completed = False
        while not all_completed:
            all_completed = True
            tasks = await github.fetch_clone_tasks()

            for repo_url in student_repos:
                if repo_url not in tasks:
                    print(f"Warning: {repo_url} not found in tasks")
                    continue

                task = tasks[repo_url]
                status = task.get("status_type", "unknown")

                if status == "queued":
                    print(f"{repo_url}: Queued for cloning")
                    all_completed = False
                elif status == "cloning":
                    progress = task.get("progress", 0)
                    print(f"{repo_url}: Cloning in progress ({progress}%)")
                    all_completed = False
                elif status == "failed":
                    error = task.get("error", "Unknown error")
                    print(f"{repo_url}: Clone failed - {error}")
                elif status != "completed":
                    print(f"{repo_url}: Unknown status - {status}")
                    all_completed = False
                else:
                    print(f"{repo_url}: Clone completed")

            if not all_completed:
                await asyncio.sleep(1)

        print("\nAll repositories cloned. Starting analysis...\n")

        # Important files to analyze across repositories
        important_files = [
            "README.md",
            "src/main.py",
            "src/utils.py",
            "tests/test_main.py"
        ]

        # Analyze repositories
        for repo_url in student_repos:
            print(f"\nAnalyzing repository: {repo_url}")

            # 1. Analyze commits
            try:
                commits = await github.analyze_commits(repo_url)
                print(f"Found {len(commits)} commits")

                # Get some statistics
                authors = {}
                for commit in commits:
                    author = commit["author_name"]
                    if author in authors:
                        authors[author] += 1
                    else:
                        authors[author] = 1

                print("Commit distribution by author:")
                for author, count in authors.items():
                    print(f"  {author}: {count} commits")
            except Exception as e:
                print(f"Error analyzing commits: {str(e)}")

            # 2. Analyze branches
            try:
                branch_results = await github.analyze_branches([repo_url])
                if repo_url in branch_results:
                    branches = branch_results[repo_url]
                    if isinstance(branches, list):
                        print(f"Found {len(branches)} branches:")
                        for branch in branches:
                            head_marker = "*" if branch["is_head"] else " "
                            remote_status = "remote" if branch["is_remote"] else "local"
                            print(f"  {head_marker} {branch['name']} ({remote_status})")
                    else:
                        print(f"Error with branches: {branches}")
            except Exception as e:
                print(f"Error analyzing branches: {str(e)}")

            # 3. Analyze blame for important files
            try:
                blame_results = await github.bulk_blame(repo_url, important_files)
                print("\nFile contribution analysis:")

                for file_path, blame_data in blame_results.items():
                    if isinstance(blame_data, list):
                        if not blame_data:
                            print(f"  {file_path}: File exists but is empty")
                            continue

                        # Calculate contribution by author
                        authors = {}
                        total_lines = len(blame_data)

                        for line in blame_data:
                            author = line["author_name"]
                            if author in authors:
                                authors[author] += 1
                            else:
                                authors[author] = 1

                        print(f"  {file_path} ({total_lines} lines):")
                        for author, lines in authors.items():
                            percentage = round((lines / total_lines) * 100, 1)
                            print(f"    {author}: {lines} lines ({percentage}%)")
                    else:
                        print(f"  {file_path}: {blame_data}")
            except Exception as e:
                print(f"Error analyzing file blame: {str(e)}")

            # 4. Fetch pull requests
            try:
                pr_results = await github.fetch_pull_requests([repo_url], state="all")
                if repo_url in pr_results:
                    prs = pr_results[repo_url]
                    if isinstance(prs, list):
                        print(f"\nFound {len(prs)} pull requests:")
                        for pr in prs:
                            state = "MERGED" if pr["merged"] else pr["state"].upper()
                            print(f"  #{pr['number']} {pr['title']} ({state})")
                            print(f"    by {pr['user_login']} - +{pr['additions']}/-{pr['deletions']} in {pr['changed_files']} files")
                    else:
                        print(f"Error with pull requests: {prs}")
            except Exception as e:
                print(f"Error fetching pull requests: {str(e)}")

            # 5. Fetch issues
            try:
                issue_results = await github.fetch_issues([repo_url], state="all")
                if repo_url in issue_results:
                    issues = issue_results[repo_url]
                    if isinstance(issues, list):
                        # Filter out pull requests from issues
                        real_issues = [i for i in issues if not i["is_pull_request"]]
                        print(f"\nFound {len(real_issues)} issues:")
                        for issue in real_issues:
                            print(f"  #{issue['number']} {issue['title']} ({issue['state'].upper()})")
                            if issue["labels"]:
                                print(f"    Labels: {', '.join(issue['labels'])}")
                    else:
                        print(f"Error with issues: {issues}")
            except Exception as e:
                print(f"Error fetching issues: {str(e)}")

            # 6. Fetch collaborators
            try:
                collab_results = await github.fetch_collaborators([repo_url])
                if repo_url in collab_results:
                    collaborators = collab_results[repo_url]
                    print(f"\nRepository collaborators ({len(collaborators)}):")
                    for collab in collaborators:
                        print(f"  {collab['login']}")
            except Exception as e:
                print(f"Error fetching collaborators: {str(e)}")

        print("\nAnalysis complete!")

    except Exception as e:
        print(f"Unexpected error during analysis: {str(e)}")

# Run the analysis
if __name__ == "__main__":
    asyncio.run(analyze_student_repositories())

## Error Handling

It's important to handle errors properly when working with external services:

```python
async def safe_repo_analysis():
    try:
        # Initialize GitHub provider
        github = GitHubProvider(
            username="your_username",
            token="your_token",
            urls=[]
        )

        # Try to clone a repository
        repo_url = "https://github.com/username/repo"
        await github.clone(repo_url)

        # Continue with analysis
        commits = await github.analyze_commits(repo_url)
        print(f"Successfully analyzed {len(commits)} commits")

    except Exception as e:
        print(f"Error analyzing repository: {str(e)}")
        # Handle the error appropriately

asyncio.run(safe_repo_analysis())
```

## Best Practices

1. **Token Security**: Never hardcode your GitHub token in your scripts. Use environment variables or secure credential storage.

2. **Rate Limiting**: Be aware of GitHub API rate limits when making multiple requests.

3. **Asyncio Usage**: Since the library uses asyncio, make sure to use `await` when calling asynchronous methods and run them within an async function using `asyncio.run()`.

4. **Error Handling**: Always wrap your API calls in try-except blocks to handle potential errors gracefully.

5. **Large Repos**: When working with large repositories, be mindful of memory usage when analyzing commits or blame data.

This comprehensive example demonstrates:

1. **Proper setup** with async runtime initialization
2. **Secure credential management** using environment variables
3. **Multiple repository handling** for analyzing several student repositories
4. **Progress monitoring** of clone operations
5. **Comprehensive data analysis** including commits, branches, blame, PRs, issues, and collaborators
6. **Error handling** at multiple levels to ensure the script continues even if parts fail
7. **Meaningful output formatting** to present the analysis results clearly