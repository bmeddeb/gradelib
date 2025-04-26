# Using GradeLib with Polars for High-Performance Data Analysis

This guide demonstrates how to use [polars](https://pola.rs/)—a fast, Rust-based DataFrame library—with GradeLib for analyzing GitHub repositories. This is ideal for users who want high performance and seamless integration with the Rust ecosystem.

---

## Requirements

Install GradeLib and polars:

```bash
pip install gradelib polars
```

---

## Example: Analyze Multiple Repositories with Polars

This example shows how to:
- Clone multiple repositories
- Analyze commits
- Perform blame analysis
- Fetch branches, collaborators, pull requests, and code reviews
- Return results as polars DataFrames for further analysis

```python
import asyncio
import os
import polars as pl
from gradelib.gradelib import setup_async, RepoManager

async def analyze_with_polars(repo_urls, github_username, github_token):
    setup_async()
    manager = RepoManager(repo_urls, github_username, github_token)

    # Clone repositories
    print("Cloning repositories...")
    await manager.clone_all()
    tasks = await manager.fetch_clone_tasks()
    repo_paths = {url: tasks[url].temp_dir for url in repo_urls if tasks[url].status.status_type == "completed"}

    # Analyze commits for each repo and collect as polars DataFrames
    print("\nAnalyzing commits...")
    commit_dfs = {}
    for url, path in repo_paths.items():
        commits = await manager.analyze_commits(path)
        commit_dfs[url] = pl.DataFrame(commits)
        print(f"{url}: {len(commits)} commits")

    # Blame analysis for a file in each repo
    print("\nPerforming blame analysis...")
    blame_dfs = {}
    for url, path in repo_paths.items():
        # Example: blame the README file if it exists
        file_paths = ["README.md"]
        blame_results = await manager.bulk_blame(path, file_paths)
        for file, result in blame_results.items():
            if isinstance(result, list):
                blame_dfs[(url, file)] = pl.DataFrame(result)
                print(f"{url} {file}: {len(result)} lines analyzed")
            else:
                print(f"{url} {file}: {result}")

    # Fetch branches
    print("\nFetching branches...")
    branches = await manager.analyze_branches(list(repo_paths.values()))
    branch_dfs = {}
    for path, branch_data in branches.items():
        if isinstance(branch_data, list):
            branch_dfs[path] = pl.DataFrame(branch_data)
            print(f"{path}: {len(branch_data)} branches")
        else:
            print(f"{path}: {branch_data}")

    # Fetch collaborators
    print("\nFetching collaborators...")
    collaborators = await manager.fetch_collaborators(list(repo_paths.values()))
    collaborator_dfs = {}
    for path, collab_data in collaborators.items():
        if isinstance(collab_data, list):
            collaborator_dfs[path] = pl.DataFrame(collab_data)
            print(f"{path}: {len(collab_data)} collaborators")
        else:
            print(f"{path}: {collab_data}")

    # Fetch pull requests
    print("\nFetching pull requests...")
    pull_requests = await manager.fetch_pull_requests(list(repo_paths.values()))
    pr_dfs = {}
    for path, pr_data in pull_requests.items():
        if isinstance(pr_data, list):
            pr_dfs[path] = pl.DataFrame(pr_data)
            print(f"{path}: {len(pr_data)} pull requests")
        else:
            print(f"{path}: {pr_data}")

    # Fetch code reviews (if supported)
    print("\nFetching code reviews...")
    # If GradeLib supports code review fetching, add here. Otherwise, skip or document as TODO.

    # Return all polars DataFrames for further analysis
    return {
        "commits": commit_dfs,
        "blame": blame_dfs,
        "branches": branch_dfs,
        "collaborators": collaborator_dfs,
        "pull_requests": pr_dfs
    }

# Example usage
if __name__ == "__main__":
    github_username = os.environ.get("GITHUB_USERNAME")
    github_token = os.environ.get("GITHUB_TOKEN")
    repos = [
        "https://github.com/bmeddeb/gradelib",
        "https://github.com/PyO3/pyo3"
    ]
    results = asyncio.run(analyze_with_polars(repos, github_username, github_token))
    # Example: print the first few rows of the commit DataFrame for each repo
    for url, df in results["commits"].items():
        print(f"\nCommits for {url}:")
        print(df.head())
```

---

## Tips
- You can use all of polars' powerful DataFrame operations for further analysis, grouping, and visualization.
- Easily convert between polars and pandas if needed:
  ```python
  # pandas to polars
  pl_df = pl.from_pandas(pd_df)
  # polars to pandas
  pd_df = pl_df.to_pandas()
  ```

---

*See the rest of the documentation for more details on each GradeLib function and advanced usage patterns.*