# GradeLib Usage Examples

This document provides comprehensive examples of how to use the GradeLib library for analyzing GitHub repositories. GradeLib is a high-performance library built with Rust and Python bindings, designed to facilitate repository analysis for grading and assessment purposes.

## Table of Contents

- [Setup](#setup)
- [Repository Management](#repository-management)
  - [Creating a RepoManager](#creating-a-repomanager)
  - [Cloning Repositories](#cloning-repositories)
  - [Monitoring Clone Status](#monitoring-clone-status)
- [Repository Analysis](#repository-analysis)
  - [Commit Analysis](#commit-analysis)
  - [Blame Analysis](#blame-analysis)
  - [Branch Analysis](#branch-analysis)
  - [Collaborator Analysis](#collaborator-analysis)
- [Advanced Usage](#advanced-usage)
  - [Parallel Processing](#parallel-processing)
  - [Error Handling](#error-handling)

---

## Setup

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

## Repository Management

### Creating a RepoManager

The `RepoManager` class is the central component for repository operations:

```python
# Create a repo manager with GitHub credentials
manager = RepoManager(
    urls=repo_urls, 
    github_username=github_username,
    github_token=github_token
)
```

### Cloning Repositories

You can clone all repositories or a specific repository:

```python
# Clone all repositories
await manager.clone_all()

# Clone a specific repository
await manager.clone("https://github.com/username/specific-repo")
```

### Monitoring Clone Status

Monitor the progress of cloning operations with detailed status information:

```python
async def monitor_cloning(manager, repo_urls):
    """Monitor and display detailed clone progress for repositories."""
    completed = set()
    all_done = False

    while not all_done:
        tasks = await manager.fetch_clone_tasks()
        all_done = True  # Assume all are done until we find one that isn't

        for url in repo_urls:
            if url in tasks:
                task = tasks[url]
                status = task.status

                # Skip repositories we've already reported as complete
                if url in completed:
                    continue

                # Check status and provide appropriate information
                if status.status_type == "queued":
                    print(f"\r⏱️ {url}: Queued for cloning", end='', flush=True)
                    all_done = False
                    
                elif status.status_type == "cloning":
                    all_done = False
                    progress = status.progress or 0
                    bar_length = 30
                    filled_length = int(bar_length * progress / 100)
                    bar = '█' * filled_length + '░' * (bar_length - filled_length)
                    print(f"\r⏳ {url}: [{bar}] {progress}%", end='', flush=True)
                    
                elif status.status_type == "completed":
                    # Show details about the completed repository
                    print(f"\n✅ {url}: Clone completed successfully")
                    if task.temp_dir:
                        print(f"   📁 Local path: {task.temp_dir}")
                    completed.add(url)
                    
                elif status.status_type == "failed":
                    # Show error details
                    print(f"\n❌ {url}: Clone failed")
                    if status.error:
                        print(f"   ⚠️ Error: {status.error}")
                    completed.add(url)

        if not all_done:
            await asyncio.sleep(0.5)  # Poll every half-second
    
    print("\nAll repository operations completed.")

# Usage
await monitor_cloning(manager, repo_urls)
```

This monitoring function provides complete details about:
- Queue status
- Cloning progress with visual progress bar
- Local paths for completed clones
- Detailed error information for failed operations

## Repository Analysis

### Commit Analysis

Analyze the commit history of a repository:

```python
# Analyze commits for a specific repository
commit_history = await manager.analyze_commits("https://github.com/username/repo")

# Process the commit data
for commit in commit_history:
    # Each commit is a dictionary with detailed information
    print(f"Commit: {commit['sha'][:8]}")
    print(f"Author: {commit['author_name']} <{commit['author_email']}>")
    print(f"Date: {commit['author_timestamp']}") # Unix timestamp
    print(f"Message: {commit['message']}")
    print(f"Changes: +{commit['additions']} -{commit['deletions']}")
    print(f"Is Merge: {commit['is_merge']}")
    print("---")

# Convert to pandas DataFrame for analysis
import pandas as pd
df = pd.DataFrame(commit_history)

# Example analysis: Most active contributors
author_counts = df['author_name'].value_counts()
print("Most active contributors:")
print(author_counts.head())

# Example analysis: Commit activity over time
df['date'] = pd.to_datetime(df['author_timestamp'], unit='s')
activity = df.set_index('date').resample('D').size()
print("Commit activity by day:")
print(activity)
```

### Blame Analysis

Perform Git blame on specific files to see who wrote each line:

```python
# Define the repository and files to blame
target_repo = "https://github.com/username/repo"
file_paths = [
    "src/main.py",
    "src/utils.py",
    "README.md"
]

# Perform blame analysis
blame_results = await manager.bulk_blame(target_repo, file_paths)

# Process the blame results
for file_path, result in blame_results.items():
    print(f"\nFile: {file_path}")
    
    if isinstance(result, str):
        # If result is a string, it's an error message
        print(f"Error: {result}")
        continue
    
    # Result is a list of line info dictionaries
    print(f"Lines analyzed: {len(result)}")
    
    # Group by author
    authors = {}
    for line in result:
        author = line['author_name']
        if author not in authors:
            authors[author] = 0
        authors[author] += 1
    
    # Print author contribution
    print("Author contribution:")
    for author, count in sorted(authors.items(), key=lambda x: x[1], reverse=True):
        percentage = (count / len(result)) * 100
        print(f"{author}: {count} lines ({percentage:.1f}%)")
```

### Branch Analysis

Analyze branch information for multiple repositories:

```python
# Analyze branches for repositories
branches = await manager.analyze_branches(repo_urls)

# Process the branch information
for repo_url, repo_branches in branches.items():
    if isinstance(repo_branches, str):
        # This is an error message
        print(f"Error analyzing branches for {repo_url}: {repo_branches}")
        continue
        
    print(f"\nRepository: {repo_url}")
    print(f"Found {len(repo_branches)} branches")
    
    # Count local vs remote branches
    local_branches = [b for b in repo_branches if not b['is_remote']]
    remote_branches = [b for b in repo_branches if b['is_remote']]
    print(f"Local branches: {len(local_branches)}")
    print(f"Remote branches: {len(remote_branches)}")
    
    # Find the default branch (usually HEAD)
    head_branches = [b for b in repo_branches if b['is_head']]
    if head_branches:
        print(f"Default branch: {head_branches[0]['name']}")
    
    # Get the most recent branches by commit time
    branches_by_time = sorted(repo_branches, key=lambda b: b['author_time'], reverse=True)
    print("\nMost recently updated branches:")
    for branch in branches_by_time[:5]:  # Top 5
        print(f"  - {branch['name']} (Last commit: {branch['commit_message'].split('\n')[0]})")
```

### Collaborator Analysis

Fetch and analyze collaborators information for repositories:

```python
# Fetch collaborator information
collaborators = await manager.fetch_collaborators(repo_urls)

# Process collaborator data
for repo_url, repo_collaborators in collaborators.items():
    print(f"\nRepository: {repo_url}")
    print(f"Found {len(repo_collaborators)} collaborators")
    
    # Print collaborator information
    for collab in repo_collaborators:
        print(f"  - {collab['login']}")
        
        # Display additional information if available
        if collab.get('full_name'):
            print(f"    Name: {collab['full_name']}")
        
        if collab.get('email'):
            print(f"    Email: {collab['email']}")

# Convert to pandas DataFrame for analysis
import pandas as pd

all_collaborators = []
for repo_url, repo_collaborators in collaborators.items():
    repo_name = '/'.join(repo_url.split('/')[-2:])
    
    for collab in repo_collaborators:
        collab_data = {
            'Repository': repo_name,
            'Login': collab['login'],
            'GitHub ID': collab['github_id'],
            'Name': collab.get('full_name', 'N/A'),
            'Email': collab.get('email', 'N/A'),
        }
        all_collaborators.append(collab_data)

# Create DataFrame
df = pd.DataFrame(all_collaborators)
print("\nCollaborator DataFrame:")
print(df)
```

## Advanced Usage

### Parallel Processing

GradeLib uses parallel processing for performance-intensive operations:

- `analyze_commits`: Uses Rayon for parallel commit analysis
- `bulk_blame`: Processes multiple files in parallel with Tokio tasks
- `analyze_branches`: Uses Rayon for parallel branch extraction
- `fetch_collaborators`: Fetches collaborator data concurrently

These operations automatically benefit from parallelism without additional configuration.

### Error Handling

GradeLib provides structured error handling. Here's an example of robust error handling:

```python
async def run_with_error_handling():
    try:
        # Try to analyze commits
        commits = await manager.analyze_commits("https://github.com/username/repo")
        print(f"Successfully analyzed {len(commits)} commits")
    except ValueError as e:
        # ValueErrors are raised for application-specific errors
        print(f"Application error: {e}")
    except Exception as e:
        # Other exceptions are unexpected errors
        print(f"Unexpected error: {e}")
        
    # For methods that return errors as strings instead of raising exceptions
    branches = await manager.analyze_branches(repo_urls)
    for repo_url, result in branches.items():
        if isinstance(result, str):
            print(f"Error analyzing branches for {repo_url}: {result}")
        else:
            print(f"Successfully analyzed {len(result)} branches for {repo_url}")

# Run the function
await run_with_error_handling()
```

---

## Full Example

Here's a complete example putting everything together:

```python
import asyncio
import os
import pandas as pd
from gradelib.gradelib import setup_async, RepoManager

async def analyze_repositories(repo_urls, github_username, github_token):
    # Initialize async runtime
    setup_async()
    
    # Create repo manager
    manager = RepoManager(repo_urls, github_username, github_token)
    
    # Clone repositories
    print("Cloning repositories...")
    await manager.clone_all()
    
    # Monitor cloning progress with detailed information
    completed = set()
    all_done = False
    while not all_done:
        tasks = await manager.fetch_clone_tasks()
        all_done = True
        
        for url in repo_urls:
            if url in tasks and url not in completed:
                task = tasks[url]
                status = task.status
                
                if status.status_type == "queued":
                    print(f"\r⏱️ {url}: Queued for cloning", end='', flush=True)
                    all_done = False
                    
                elif status.status_type == "cloning":
                    all_done = False
                    progress = status.progress or 0
                    bar_length = 30
                    filled_length = int(bar_length * progress / 100)
                    bar = '█' * filled_length + '░' * (bar_length - filled_length)
                    print(f"\r⏳ {url}: [{bar}] {progress}%", end='', flush=True)
                    
                elif status.status_type == "completed":
                    print(f"\n✅ {url}: Clone completed successfully")
                    if task.temp_dir:
                        print(f"   📁 Local path: {task.temp_dir}")
                    completed.add(url)
                    
                elif status.status_type == "failed":
                    print(f"\n❌ {url}: Clone failed")
                    if status.error:
                        print(f"   ⚠️ Error: {status.error}")
                    completed.add(url)
        
        if not all_done:
            await asyncio.sleep(0.5)
    
    print("\nAll repository operations completed.")
    
    # Analyze commits
    print("\nAnalyzing commits...")
    all_commits = {}
    for url in repo_urls:
        try:
            commits = await manager.analyze_commits(url)
            all_commits[url] = commits
            print(f"Found {len(commits)} commits in {url}")
        except Exception as e:
            print(f"Error analyzing commits for {url}: {e}")
    
    # Analyze branches
    print("\nAnalyzing branches...")
    branches = await manager.analyze_branches(repo_urls)
    for url, branch_data in branches.items():
        if isinstance(branch_data, str):
            print(f"Error analyzing branches for {url}: {branch_data}")
        else:
            print(f"Found {len(branch_data)} branches in {url}")
    
    # Fetch collaborators
    print("\nFetching collaborators...")
    collaborators = await manager.fetch_collaborators(repo_urls)
    for url, collab_data in collaborators.items():
        if isinstance(collab_data, str):
            print(f"Error fetching collaborators for {url}: {collab_data}")
        else:
            print(f"Found {len(collab_data)} collaborators in {url}")
    
    # Return all collected data
    return {
        "commits": all_commits,
        "branches": branches,
        "collaborators": collaborators
    }

# Run the analysis
if __name__ == "__main__":
    # Get GitHub credentials
    github_username = os.environ.get("GITHUB_USERNAME")
    github_token = os.environ.get("GITHUB_TOKEN")
    
    if not github_username or not github_token:
        print("Please set GITHUB_USERNAME and GITHUB_TOKEN environment variables")
        exit(1)
    
    # List of repositories to analyze
    repos = [
        "https://github.com/bmeddeb/gradelib",
        "https://github.com/PyO3/pyo3"
    ]
    
    # Run async analysis
    results = asyncio.run(analyze_repositories(repos, github_username, github_token))
    
    # Print summary
    print("\n===== ANALYSIS SUMMARY =====")
    for repo in repos:
        repo_name = repo.split('/')[-1]
        print(f"\nRepository: {repo_name}")
        
        # Commit stats
        if repo in results["commits"]:
            commits = results["commits"][repo]
            authors = set(c["author_name"] for c in commits)
            print(f"Total commits: {len(commits)}")
            print(f"Unique authors: {len(authors)}")
            
            # Find most recent commit
            if commits:
                recent = max(commits, key=lambda c: c["author_timestamp"])
                print(f"Most recent commit: {recent['message'].split('\n')[0]}")
        
        # Branch stats
        if repo in results["branches"] and isinstance(results["branches"][repo], list):
            branches = results["branches"][repo]
            local = sum(1 for b in branches if not b["is_remote"])
            remote = sum(1 for b in branches if b["is_remote"])
            print(f"Branches: {len(branches)} (Local: {local}, Remote: {remote})")
        
        # Collaborator stats
        if repo in results["collaborators"] and isinstance(results["collaborators"][repo], list):
            collabs = results["collaborators"][repo]
            print(f"Collaborators: {len(collabs)}")
```

---

*This document is a living resource and will be updated as new functionality is added to GradeLib.*
