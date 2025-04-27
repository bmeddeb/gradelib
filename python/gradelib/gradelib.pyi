"""Type stubs for gradelib - High-performance GitHub & Taiga analysis."""

from __future__ import annotations
from typing import Dict, List, Optional, Union, Any, Callable, Awaitable, Literal, overload, TypedDict, TypeVar, ParamSpec
from dataclasses import dataclass
import os
import pathlib

# Define module exports for clarity
__all__ = [
    "setup_async",
    "RepoManager",
    "CloneStatus",
    "CloneTask",
    "TaigaClient",
    "GitHubOAuthClient",
    "async_handler",
]

# Status type literals
CloneStatusType = Literal["queued", "cloning", "completed", "failed"]
CommentType = Literal["issue", "commit", "pull_request", "review_comment"]

# Dataclass for clone status


@dataclass
class CloneStatus:
    """
    Represents the status of a repository clone operation.

    This class is returned from RepoManager methods and should not be instantiated directly.
    """
    status_type: CloneStatusType
    progress: Optional[int] = None
    error: Optional[str] = None

# Dataclass for clone task


@dataclass
class CloneTask:
    """
    Represents a repository clone task with its status and location.

    This class is returned from RepoManager methods and should not be instantiated directly.
    """
    url: str
    status: CloneStatus
    temp_dir: Optional[str] = None

# Type definitions for various return types


class CommitInfo(TypedDict):
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


class BlameLineInfo(TypedDict):
    commit_id: str
    author_name: str
    author_email: str
    orig_line_no: int
    final_line_no: int
    line_content: str


class CollaboratorInfo(TypedDict):
    login: str
    github_id: int
    full_name: Optional[str]
    email: Optional[str]
    avatar_url: Optional[str]


class IssueInfo(TypedDict):
    id: int
    number: int
    title: str
    state: str
    created_at: str
    updated_at: str
    closed_at: Optional[str]
    user_login: str
    user_id: int
    body: Optional[str]
    comments_count: int
    is_pull_request: bool
    labels: List[str]
    assignees: List[str]
    milestone: Optional[str]
    locked: bool
    html_url: str


class PullRequestInfo(TypedDict):
    id: int
    number: int
    title: str
    state: str
    created_at: str
    updated_at: str
    closed_at: Optional[str]
    merged_at: Optional[str]
    user_login: str
    user_id: int
    body: Optional[str]
    comments: int
    commits: int
    additions: int
    deletions: int
    changed_files: int
    mergeable: Optional[bool]
    labels: List[str]
    draft: bool
    merged: bool
    merged_by: Optional[str]


class CodeReviewInfo(TypedDict):
    id: int
    pr_number: int
    user_login: str
    user_id: int
    body: Optional[str]
    state: str
    submitted_at: str
    commit_id: str
    html_url: str


class CommentInfo(TypedDict):
    id: int
    comment_type: CommentType
    user_login: str
    user_id: int
    body: str
    created_at: str
    updated_at: str
    html_url: str
    issue_number: Optional[int]
    pull_request_number: Optional[int]
    commit_id: Optional[str]
    path: Optional[str]
    position: Optional[int]
    line: Optional[int]
    commit_sha: Optional[str]


class BranchInfo(TypedDict):
    name: str
    is_remote: bool
    commit_id: str
    commit_message: str
    author_name: str
    author_email: str
    author_time: int
    is_head: bool
    remote_name: Optional[str]

# Repository Manager class


class RepoManager:
    """
    Manages Git repositories for analysis, providing high-performance clone and analysis operations.
    """

    def __init__(self, urls: List[str], github_username: str, github_token: str) -> None:
        """
        Initialize a new RepoManager with GitHub credentials.

        Args:
            urls: List of repository URLs to manage
            github_username: GitHub username for authentication
            github_token: GitHub personal access token for authentication
        """
        ...

    async def clone_all(self) -> None:
        """
        Clones all repositories configured in this manager instance asynchronously.

        Returns:
            None
        """
        ...

    async def fetch_clone_tasks(self) -> Dict[str, CloneTask]:
        """
        Fetches the current status of all cloning tasks asynchronously.

        Returns:
            A dictionary mapping repository URLs to CloneTask objects
        """
        ...

    async def clone(self, url: str) -> None:
        """
        Clones a single repository specified by URL asynchronously.

        Args:
            url: The repository URL to clone

        Returns:
            None
        """
        ...

    async def bulk_blame(self, repo_path: str, file_paths: List[str]) -> Dict[str, Union[List[BlameLineInfo], str]]:
        """
        Performs 'git blame' on multiple files within a cloned repository asynchronously.

        Args:
            repo_path: The local path to the cloned repository to analyze
            file_paths: List of file paths within the repository to blame

        Returns:
            Dictionary mapping file paths to either blame information or error strings

        Raises:
            ValueError: If the repository path is invalid or not a valid git repository
        """
        ...

    async def analyze_commits(self, repo_path: str) -> List[CommitInfo]:
        """
        Analyzes the commit history of a cloned repository asynchronously.

        Args:
            repo_path: The local path to the cloned repository to analyze

        Returns:
            List of commit information dictionaries

        Raises:
            ValueError: If the repository path is invalid or not a valid git repository
        """
        ...

    async def fetch_collaborators(self, repo_urls: List[str]) -> Dict[str, Union[List[CollaboratorInfo], str]]:
        """
        Fetches collaborator information for multiple repositories.

        Args:
            repo_urls: List of repository URLs to analyze

        Returns:
            Dictionary mapping repository URLs to either lists of collaborator information (on success)
            or error strings (on failure for that repo). No exceptions are raised for individual failures.

        Raises:
            ValueError: If there is a catastrophic error affecting all repositories
        """
        ...

    async def fetch_issues(self, repo_urls: List[str], state: Optional[str] = None) -> Dict[str, Union[List[IssueInfo], str]]:
        """
        Fetches issue information for multiple repositories.

        Args:
            repo_urls: List of repository URLs to analyze
            state: Optional filter for issue state ("open", "closed", or "all")

        Returns:
            Dictionary mapping repository URLs to either lists of issue information or error strings

        Raises:
            ValueError: If there is an error fetching issue information
        """
        ...

    async def fetch_pull_requests(self, repo_urls: List[str], state: Optional[str] = None) -> Dict[str, Union[List[PullRequestInfo], str]]:
        """
        Fetches pull request information for multiple repositories.

        Args:
            repo_urls: List of repository URLs to analyze
            state: Optional filter for pull request state ("open", "closed", "all")

        Returns:
            Dictionary mapping repository URLs to either lists of pull request information or error strings

        Raises:
            ValueError: If there is an error fetching pull request information
        """
        ...

    async def fetch_code_reviews(self, repo_urls: List[str]) -> Dict[str, Union[Dict[str, List[CodeReviewInfo]], str]]:
        """
        Fetches code review information for multiple repositories.

        Args:
            repo_urls: List of repository URLs to analyze

        Returns:
            Dictionary mapping repository URLs to either dictionaries mapping PR numbers to lists of code review information, or error strings

        Raises:
            ValueError: If there is an error fetching code review information
        """
        ...

    async def fetch_comments(self, repo_urls: List[str], comment_types: Optional[List[str]] = None) -> Dict[str, Union[List[CommentInfo], str]]:
        """
        Fetches comments of various types for multiple repositories.

        Args:
            repo_urls: List of repository URLs to analyze
            comment_types: Optional list of comment types to fetch ("issue", "commit", "pull_request", "review_comment")

        Returns:
            Dictionary mapping repository URLs to either lists of comment information or error strings

        Raises:
            ValueError: If there is an error fetching comment information or if an invalid comment type is specified
        """
        ...

    async def analyze_branches(self, repo_urls: List[str]) -> Dict[str, Union[List[BranchInfo], str]]:
        """
        Analyzes branches in cloned repositories.

        Args:
            repo_urls: List of repository URLs to analyze

        Returns:
            Dictionary mapping repository URLs to either lists of branch information or error strings

        Raises:
            ValueError: If there is an error analyzing branches
        """
        ...

# Taiga client for project management integration


class TaigaClient:
    """Client for interacting with the Taiga project management API."""

    def __init__(self, base_url: str, auth_token: Optional[str] = None, username: Optional[str] = None) -> None:
        """
        Initialize a new TaigaClient with authentication details.

        Args:
            base_url: The base URL for the Taiga API
            auth_token: Optional authentication token for Taiga API (required for private projects)
            username: Optional Taiga username (required for private projects)
        """
        ...

    async def fetch_project_data(self, slug: str) -> Dict[str, Any]:
        """
        Fetches complete project data from Taiga.

        Args:
            slug: The project slug identifier

        Returns:
            Dictionary containing all project data including members, sprints, user stories, tasks, and task histories

        Raises:
            ValueError: If there is an error fetching the project data
        """
        ...

    async def fetch_multiple_projects(self, slugs: List[str]) -> Dict[str, Union[bool, str]]:
        """
        Fetches multiple projects concurrently.

        Args:
            slugs: List of project slug identifiers

        Returns:
            Dictionary mapping project slugs to either True (success) or error strings (on failure for that slug).
            No exceptions are raised for individual failures; a single failure does not abort the batch.

        Raises:
            ValueError: If there is a catastrophic error affecting all projects
        """
        ...


def setup_async() -> None:
    """
    Initializes the asynchronous runtime environment needed for manager operations.
    Must be called before using any async functionality in the library.

    Returns:
        None
    """
    ...


class GitHubOAuthClient:
    """
    Helper for GitHub OAuth code exchange.

    Use this class to exchange an OAuth authorization code for an access token.
    """

    @staticmethod
    async def exchange_code_for_token(
        client_id: str,
        client_secret: str,
        code: str,
        redirect_uri: str
    ) -> str:
        """
        Exchanges a GitHub OAuth authorization code for an access token.

        Args:
            client_id: The GitHub App's client ID.
            client_secret: The GitHub App's client secret.
            code: The authorization code received from GitHub.
            redirect_uri: The redirect URI used in the OAuth flow.

        Returns:
            The access token as a string.

        Raises:
            RuntimeError: If the exchange fails.
        """
        ...


# Type variables for async_handler
P = ParamSpec('P')
T = TypeVar('T')

def async_handler(func: Callable[P, Awaitable[T]]) -> Callable[P, T]:
    """
    Decorator to handle async functions in synchronous contexts.
    
    This is particularly useful for using async functions from gradelib in
    Flask routes or other synchronous contexts. It handles event loop management
    and ensures that async functions can be called safely from any thread.
    
    Args:
        func: The async function to be wrapped
        
    Returns:
        A synchronous function that can be used in synchronous contexts
    
    Example:
        ```python
        from flask import Flask
        from gradelib import GitHubOAuthClient, async_handler
        
        @async_handler
        async def exchange_token(code):
            return await GitHubOAuthClient.exchange_code_for_token(
                client_id="...",
                client_secret="...",
                code=code,
                redirect_uri="..."
            )
            
        # Now you can call this from synchronous code:
        token = exchange_token(code)  # No need for asyncio.run()!
        ```
    """
    ...
