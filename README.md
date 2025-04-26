<p align="center">
  <img src="https://bmeddeb.github.io/gradelib/assets/images/gradelib_e.png" alt="GradeLib Logo" width="200"/>
</p>

<h1 align="center">GradeLib</h1>

<p align="center">
  <strong>High-performance GitHub & Taiga analysis for grading software projects</strong><br>
  <em>Rust-powered backend, Python-friendly frontend</em>
</p>

<p align="center">
  <a href="https://pypi.org/project/gradelib/"><img src="https://img.shields.io/pypi/v/gradelib.svg" alt="PyPI Version"></a>
  <a href="https://github.com/bmeddeb/gradelib/actions"><img src="https://github.com/bmeddeb/gradelib/actions/workflows/ci.yml/badge.svg" alt="CI Status"></a>
  <a href="https://bmeddeb.github.io/gradelib/"><img src="https://img.shields.io/badge/docs-gradelib-blue.svg" alt="Documentation"></a>
  <img src="https://img.shields.io/badge/Built%20with-Rust%20%26%20Python-orange.svg" alt="Built with Rust and Python">
  <a href="https://github.com/bmeddeb/gradelib/blob/main/LICENSE"><img src="https://img.shields.io/github/license/bmeddeb/gradelib" alt="License"></a>
</p>



## 📋 Overview

GradeLib is a high-performance library for educators and teaching assistants to analyze GitHub repositories and Taiga projects. The library combines the speed of Rust with the ease of use of Python to provide powerful tools for grading software engineering projects.

**Key Advantage**: GradeLib excels at parallel processing of multiple repositories and projects simultaneously, using parallelism for computationally intensive tasks (like commit analysis) and asynchronous execution for I/O bound operations (like API calls), making it dramatically faster than traditional tools.

## ⚙️ Installation

```bash
# Install with pip
pip install gradelib

# Or with uv (recommended)
uv pip install gradelib
```

## 🚀 Quick Start

### Analyzing Multiple GitHub Repositories

```python
from gradelib import setup_async, RepoManager
import os
import asyncio

# Required initialization for async operations
setup_async()

# Define repositories to analyze
repo_urls = [
    "https://github.com/student1/project",
    "https://github.com/student2/project",
    "https://github.com/student3/project"
]

# Create a repository manager with multiple repos
manager = RepoManager(
    urls=repo_urls,
    github_username=os.getenv("GITHUB_USERNAME"),
    github_token=os.getenv("GITHUB_TOKEN")
)

# Clone all repositories in parallel
await manager.clone_all()

# Get clone status and repo paths
clone_tasks = await manager.fetch_clone_tasks()

# Process pull requests from all repos in a single API call
pull_requests = await manager.fetch_pull_requests(repo_urls, state="all")

# Analyze commit history for all repos using their URLs
commits_per_repo = {}
for repo_url in repo_urls:
    commits_per_repo[repo_url] = await manager.analyze_commits(repo_url)

# Generate statistics
for repo_url, commits in commits_per_repo.items():
    print(f"Repository: {repo_url}")
    print(f"Total commits: {len(commits)}")
    authors = {commit['author_name'] for commit in commits}
    print(f"Contributors: {', '.join(authors)}")
    print("---")
```

### Analyzing Multiple Taiga Projects

```python
from gradelib import TaigaClient

# Initialize Taiga client
client = TaigaClient(
    base_url="https://api.taiga.io/api/v1/",
    auth_token=os.getenv("TAIGA_TOKEN"),
    username=os.getenv("TAIGA_USERNAME")
)

# Fetch multiple projects concurrently
team_slugs = ["team1-project", "team2-project", "team3-project"]
results = await client.fetch_multiple_projects(team_slugs)

# Process each project's data
for slug in team_slugs:
    if results[slug] is True:  # Successfully fetched
        project_data = await client.fetch_project_data(slug)

        # Analyze sprint metrics
        sprints = project_data["sprints"]
        print(f"Project: {slug}")
        print(f"Number of sprints: {len(sprints)}")

        # Analyze user stories
        user_stories = project_data["user_stories"]
        print(f"User stories: {len(user_stories)}")

        # Task distribution analysis
        tasks = project_data["tasks"]
        print(f"Total tasks: {len(tasks)}")
        print("---")
```

## 🧠 Core Features

### Parallel and Asynchronous Processing

- **Bulk Repository Operations**
  - Clone multiple repositories simultaneously
  - Process commits, blame, and branches in parallel
  - Concurrent API calls to GitHub for issues, PRs, and more

- **Bulk Project Management Analysis**
  - Fetch and analyze multiple Taiga projects simultaneously
  - Concurrent processing of sprints, stories, and tasks
  - Efficient handling of large datasets

### GitHub Repository Analysis

- **Asynchronous Repository Management**
  - Bulk cloning and analysis of multiple repositories
  - Status tracking for clone operations

- **Code Analysis**
  - Commit history and blame information
  - Branch analysis and comparison
  - Detailed collaborator statistics

- **GitHub Workflow Analysis**
  - Issues and pull requests (with filtering by state)
  - Code reviews and comments
  - Contributor activity metrics

### Taiga Project Analysis

- **Project Data Retrieval**
  - Complete project data including members, sprints, stories
  - Efficient asynchronous API handling
  - Support for both public and private projects

- **Agile Metrics**
  - Sprint progress and velocity
  - Task completion and assignment tracking
  - Team contribution analytics

` Note that Taiga API is more often than not is down, and providing auth token times out. Public Projects do nor require a token you can skip username/password and directly request your project data `
## 📚 Documentation

Full documentation is available at: [https://bmeddeb.github.io/gradelib/](https://bmeddeb.github.io/gradelib/)

The documentation includes:

- Full API Reference
- Setup and installation guides
- Advanced usage examples
- GitHub and Taiga integration details
- Contribution guidelines

## 🧪 Testing

```bash
# Run Python tests
pytest

# Run with verbose output
pytest -v
```

## 📄 License

This project is licensed under the [MIT License](https://github.com/bmeddeb/gradelib/blob/main/LICENSE).

---

_Developed by [@bmeddeb](https://github.com/bmeddeb) — contributions are welcome!_
