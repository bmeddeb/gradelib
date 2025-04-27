[Advanced-Usage](advanced-usage.md)

# Advanced Usage

## Parallel Processing

GradeLib uses parallel processing for performance-intensive operations:

- `analyze_commits`: Uses Rayon for parallel commit analysis
- `bulk_blame`: Processes multiple files in parallel with Tokio tasks
- `analyze_branches`: Uses Rayon for parallel branch extraction
- `fetch_collaborators`: Fetches collaborator data concurrently. Returns a dict mapping each repo URL to either a list of collaborators or an error string. No exceptions are raised for individual failures.
- `fetch_pull_requests`: Fetches pull request data concurrently. Returns a dict mapping each repo URL to either a list of pull requests or an error string. No exceptions are raised for individual failures.

These operations automatically benefit from parallelism without additional configuration.

## Error Handling

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
```
# Run the function

```python
await run_with_error_handling()
```

## Pagination Control with max_pages

All major GitHub endpoints (issues, pull requests, comments, collaborators) support a `max_pages` argument:

- By default, all pages are fetched (no limit).
- Set `max_pages` to an integer to limit the number of pages fetched (each page contains up to 100 items).
- This is useful for performance, API rate limiting, or previewing large repositories.

Example:

```python
# Fetch only the first 2 pages of issues (up to 200 issues per repo)
issues = await manager.fetch_issues(repo_urls, max_pages=2)

# Fetch only the first page of pull requests
pull_requests = await manager.fetch_pull_requests(repo_urls, max_pages=1)

# Fetch only the first 3 pages of comments
comments = await manager.fetch_comments(repo_urls, max_pages=3)

# Fetch only the first 2 pages of collaborators
collaborators = await manager.fetch_collaborators(repo_urls, max_pages=2)
```