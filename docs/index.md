[index](index.md)
# GradeLib Usage Examples

This document provides comprehensive examples of how to use the GradeLib library for analyzing GitHub repositories. GradeLib is a high-performance library built with Rust and Python bindings, designed to facilitate repository analysis for grading and assessment purposes.

## Pagination Support for Large Repositories

GradeLib supports efficient pagination for all major GitHub endpoints, including issues, pull requests, comments, collaborators, and code reviews. The `max_pages` argument allows you to control how many pages of results are fetched from the GitHub API (each page contains up to 100 items). By default, all pages are fetched, but you can limit this for performance or preview purposes:

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

## Table of Contents

- [Setup](setup.md)
- [Repository Management](repository-management.md)
- [Repository Analysis](repository-analysis.md)
  - [Commit Analysis](repository-analysis.md#commit-analysis)
  - [Blame Analysis](repository-analysis.md#blame-analysis)
  - [Branch Analysis](repository-analysis.md#branch-analysis)
- [Collaborator Analysis](collaborator-analysis.md)
- [Pull Request Analysis](pull-request-analysis.md)
- [Issues Analysis](issues-analysis.md)
- [GitHub OAuth](github_oauth.md)
- [Async Utilities](async_utilities.md)
- [Advanced Usage](advanced-usage.md)
- [Full Example](full-example.md)
- [Taiga Provider](taiga-provider.md)
- [Polars Usage](polars-usage.md)

---

*This document is a living resource and will be updated as new functionality is added to GradeLib.*