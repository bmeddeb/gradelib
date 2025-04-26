
 [Repository-Analysis](repository-analysis.md)

# Repository Analysis

## Commit Analysis

Analyze the commit history of a repository:

```python
# Analyze commits for a specific repository
repo_path = "/path/to/local/clone"  # Use the local path from the clone task
commits = await manager.analyze_commits(repo_path)

# Process the commit data
for commit in commits:
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
df = pd.DataFrame(commits)

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

## Blame Analysis
Perform Git blame on specific files to see who wrote each line:
```python
# Define the repository and files to blame
repo_path = "/path/to/local/clone"  # Use the local path from the clone task
file_paths = [
    "src/main.py",
    "src/utils.py",
    "workflow_usage.md"
]

# Perform blame analysis
blame_results = await manager.bulk_blame(repo_path, file_paths)

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

# Branch Analysis
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
