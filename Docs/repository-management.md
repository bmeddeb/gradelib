
[Repository-Management](repository-management.md)

# Repository Management

## Creating a RepoManager

The `RepoManager` class is the central component for repository operations:

```python
# Create a repo manager with GitHub credentials
manager = RepoManager(
    urls=repo_urls,
    github_username=github_username,
    github_token=github_token
)
```
## Cloning Repositories
You can clone all repositories or a specific repository:

```python
# Clone all repositories
await manager.clone_all()

# Clone a specific repository
await manager.clone("https://github.com/username/specific-repo")
```

## Monitoring Clone Status:
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
```
# Usage

```python
await monitor_cloning(manager, repo_urls) 
```
```

```