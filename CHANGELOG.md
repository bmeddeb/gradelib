# Changelog

## [v0.2.0] - 2025-04-27

### 🚀 New Feature: Pagination Support for Large Repositories

GradeLib now supports **efficient pagination** for all major GitHub endpoints, making it easier to analyze large repositories without hitting API or memory limits.

#### Endpoints Supporting Pagination
- **Issues** (`fetch_issues`)
- **Pull Requests** (`fetch_pull_requests`)
- **Comments** (`fetch_comments`)
- **Collaborators** (`fetch_collaborators`)
- **Code Reviews** (`fetch_code_reviews`)

#### How It Works
- All supported endpoints now accept an optional `max_pages` argument.
- By default, **all pages** are fetched (no limit), ensuring you get the complete dataset.
- To limit the number of results (for performance, preview, or rate limiting), set `max_pages` to an integer. Each page contains up to 100 items.

##### Example Usage
```python
# Fetch only the first 2 pages of issues (up to 200 issues per repo)
issues = await manager.fetch_issues(repo_urls, max_pages=2)

# Fetch only the first page of pull requests
pull_requests = await manager.fetch_pull_requests(repo_urls, max_pages=1)

# Fetch only the first 3 pages of comments
comments = await manager.fetch_comments(repo_urls, max_pages=3)

# Fetch only the first 2 pages of collaborators
collaborators = await manager.fetch_collaborators(repo_urls, max_pages=2)

# Fetch only the first 2 pages of code reviews
code_reviews = await manager.fetch_code_reviews(repo_urls, max_pages=2)
```

#### Default Behavior
- If `max_pages` is not specified, GradeLib will fetch **all available pages** for the endpoint.
- This ensures backward compatibility and complete data retrieval by default.

#### Why Pagination?
- **Performance:** Avoids loading thousands of items into memory at once.
- **API Rate Limiting:** Reduces the number of API calls for preview or sampling.
- **Flexibility:** Lets you control the trade-off between completeness and speed.

#### Documentation
- All relevant documentation and usage examples have been updated to reflect the new pagination feature.
- See the [Advanced Usage](docs/advanced-usage.md) and endpoint-specific docs for details.

---

Older releases are not shown here. See the project history for previous changes.