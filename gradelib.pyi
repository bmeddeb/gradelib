# Stubs for the gradelib Rust library
# Generated based on src/lib.rs

from typing import Any, Dict, List, Optional, Awaitable, Union, Mapping, TypedDict, int, overload

# Main module


def setup_async() -> None:
    """Initializes the asynchronous runtime environment needed for provider operations."""
    ...


# Type Aliases and TypedDicts
BlameLineDict = Mapping[str, Union[str, int]]
BlameResultForFile = Union[List[BlameLineDict], str]
BulkBlameResult = Mapping[str, BlameResultForFile]


class CommitDict(TypedDict):
    """Dictionary representation of a Git commit with detailed metadata."""
    sha: str
    repo_name: str
    message: str
    author_name: str
    author_email: str
    author_timestamp: int
    author_offset: int
    committer_name: str
    committer_email: str
    committer_timestamp: int
    committer_offset: int
    additions: int
    deletions: int
    is_merge: bool

# GitHub module


class GitHubProvider:
    """Provider for GitHub operations."""

    def __init__(self, username: str, token: str, urls: Optional[List[str]] = None) -> None:
        """Initializes the GitHubProvider with GitHub credentials.

        Args:
            username: GitHub username
            token: GitHub personal access token
            urls: Optional list of repository URLs to initialize with
        """
        ...

    def clone_all(self) -> Awaitable[None]:
        """Clones all repositories configured in this provider asynchronously."""
        ...

    def clone(self, url: str) -> Awaitable[None]:
        """Clones a single repository specified by URL asynchronously."""
        ...

    def fetch_clone_tasks(self) -> Awaitable[Dict[str, Dict[str, Any]]]:
        """Fetches the current status of all cloning tasks asynchronously."""
        ...

    def analyze_commits(self, target_repo_url: str) -> Awaitable[List[CommitDict]]:
        """Analyzes the commit history of a cloned repository asynchronously."""
        ...

    def analyze_branches(self, repo_urls: List[str]) -> Awaitable[Dict[str, Union[List[Dict[str, Any]], str]]]:
        """Analyzes branches in cloned repositories."""
        ...

    def bulk_blame(self, target_repo_url: str, file_paths: List[str]) -> Awaitable[BulkBlameResult]:
        """Performs 'git blame' on multiple files within a cloned repository asynchronously."""
        ...

    def fetch_collaborators(self, repo_urls: List[str]) -> Awaitable[Dict[str, List[Dict[str, Any]]]]:
        """Fetches collaborator information for multiple repositories asynchronously."""
        ...

    def fetch_issues(self, repo_urls: List[str], state: Optional[str] = None) -> Awaitable[Dict[str, Union[List[Dict[str, Any]], str]]]:
        """Fetches issue information for multiple repositories asynchronously."""
        ...

    def fetch_pull_requests(self, repo_urls: List[str], state: Optional[str] = None) -> Awaitable[Dict[str, Union[List[Dict[str, Any]], str]]]:
        """Fetches pull request information for multiple repositories asynchronously."""
        ...
