
[Full-Example](full-example.md)

# Full Example

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

    # Fetch pull requests
    print("\nFetching pull requests...")
    pull_requests = await manager.fetch_pull_requests(repo_urls)
    for url, pr_data in pull_requests.items():
        if isinstance(pr_data, str):
            print(f"Error fetching pull requests for {url}: {pr_data}")
        else:
            print(f"Found {len(pr_data)} pull requests in {url}")

    # Fetch issues
    print("\nFetching issues...")
    issues = await manager.fetch_issues(repo_urls)
    for url, issue_data in issues.items():
        if isinstance(issue_data, str):
            print(f"Error fetching issues for {url}: {issue_data}")
        else:
            # Count actual issues (not PRs)
            actual_issues = [issue for issue in issue_data if not issue['is_pull_request']]
            print(f"Found {len(actual_issues)} issues in {url}")

    # Return all collected data
    return {
        "commits": all_commits,
        "branches": branches,
        "collaborators": collaborators,
        "pull_requests": pull_requests,
        "issues": issues
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

        # Pull request stats
        if repo in results["pull_requests"] and isinstance(results["pull_requests"][repo], list):
            prs = results["pull_requests"][repo]
            open_prs = sum(1 for pr in prs if pr["state"] == "open")
            merged_prs = sum(1 for pr in prs if pr["merged"])
            print(f"Pull requests: {len(prs)} (Open: {open_prs}, Merged: {merged_prs})")

        # Issue stats
        if repo in results["issues"] and isinstance(results["issues"][repo], list):
            all_issues = results["issues"][repo]
            # Count actual issues (not PRs)
            actual_issues = [issue for issue in all_issues if not issue["is_pull_request"]]
            open_issues = sum(1 for issue in actual_issues if issue["state"] == "open")
            closed_issues = sum(1 for issue in actual_issues if issue["state"] == "closed")
            print(f"Issues: {len(actual_issues)} (Open: {open_issues}, Closed: {closed_issues})")

```