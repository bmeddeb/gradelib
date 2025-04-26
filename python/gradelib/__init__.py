from typing import Dict, List, Optional, Union

from .gradelib import setup_async as _setup_async
from .gradelib import RepoManager as _RustRepoManager
from .gradelib import TaigaClient
from .types import (
    CloneStatus, CloneTask,
    CommitInfo, BlameLineInfo, CollaboratorInfo,
    IssueInfo, PullRequestInfo, CodeReviewInfo,
    CommentInfo, BranchInfo,
    CloneStatusType, CommentType,
    convert_clone_tasks,
)

__all__ = [
    "setup_async",
    "RepoManager",
    "CloneStatus",
    "CloneTask",
    "TaigaClient",
    "CloneStatusType",
    "CommentType",
]

try:
    import importlib.metadata
    __version__ = importlib.metadata.version("gradelib")
except importlib.metadata.PackageNotFoundError:
    __version__ = "0.0.0"


class RepoManager:
    """
    Manages Git repositories for analysis, providing high-performance clone and analysis operations.

    This class provides a Python-friendly interface to the underlying Rust implementation.
    """

    def __init__(self, urls: List[str], github_username: str, github_token: str) -> None:
        """
        Initialize a new RepoManager with GitHub credentials.

        Args:
            urls: List of repository URLs to manage
            github_username: GitHub username for authentication
            github_token: GitHub personal access token for authentication
        """
        self._rust_manager = _RustRepoManager(
            urls, github_username, github_token)

    async def clone_all(self) -> None:
        """
        Clones all repositories configured in this manager instance asynchronously.

        Returns:
            None
        """
        return await self._rust_manager.clone_all()

    async def fetch_clone_tasks(self) -> Dict[str, CloneTask]:
        """
        Fetches the current status of all cloning tasks asynchronously.

        Returns:
            A dictionary mapping repository URLs to CloneTask objects
        """
        rust_tasks = await self._rust_manager.fetch_clone_tasks()
        if rust_tasks is None:
            raise ValueError("Failed to fetch clone tasks")
        return convert_clone_tasks(rust_tasks)

    async def clone(self, url: str) -> None:
        """
        Clones a single repository specified by URL asynchronously.

        Args:
            url: The repository URL to clone

        Returns:
            None
        """
        return await self._rust_manager.clone(url)

    async def bulk_blame(self, repo_path: str, file_paths: List[str]) -> Dict[str, Union[List[BlameLineInfo], str]]:
        """
        Performs 'git blame' on multiple files within a cloned repository asynchronously.

        Args:
            repo_path: The local path to the cloned repository to analyze
            file_paths: List of file paths within the repository to blame

        Returns:
            Dictionary mapping file paths to either blame information or error strings
        """
        result = await self._rust_manager.bulk_blame(repo_path, file_paths)
        if not isinstance(result, dict):
            raise TypeError(
                f"Expected Dict[str, Union[List[BlameLineInfo], str]], got {type(result)}")
        return result

    async def analyze_commits(self, repo_path: str) -> List[CommitInfo]:
        """
        Analyzes the commit history of a cloned repository asynchronously.

        Args:
            repo_path: The local path to the cloned repository to analyze

        Returns:
            List of commit information objects

        Raises:
            ValueError: If the repository path is invalid or not a valid git repository
        """
        result = await self._rust_manager.analyze_commits(repo_path)
        if not isinstance(result, list):
            raise TypeError(f"Expected List[CommitInfo], got {type(result)}")
        return result

    async def fetch_collaborators(self, repo_urls: List[str]) -> Dict[str, List[CollaboratorInfo]]:
        """
        Fetches collaborator information for multiple repositories.

        Args:
            repo_urls: List of repository URLs to analyze

        Returns:
            Dictionary mapping repository URLs to lists of collaborator information
        """
        result = await self._rust_manager.fetch_collaborators(repo_urls)
        if not isinstance(result, dict):
            raise TypeError(
                f"Expected Dict[str, List[CollaboratorInfo]], got {type(result)}")
        return result

    async def fetch_issues(self, repo_urls: List[str], state: Optional[str] = None) -> Dict[str, Union[List[IssueInfo], str]]:
        """
        Fetches issue information for multiple repositories.

        Args:
            repo_urls: List of repository URLs to analyze
            state: Optional filter for issue state ("open", "closed", or "all")

        Returns:
            Dictionary mapping repository URLs to either lists of issue information or error strings
        """
        result = await self._rust_manager.fetch_issues(repo_urls, state)
        if not isinstance(result, dict):
            raise TypeError(
                f"Expected Dict[str, Union[List[IssueInfo], str]], got {type(result)}")
        return result

    async def fetch_pull_requests(self, repo_urls: List[str], state: Optional[str] = None) -> Dict[str, Union[List[PullRequestInfo], str]]:
        """
        Fetches pull request information for multiple repositories.

        Args:
            repo_urls: List of repository URLs to analyze
            state: Optional filter for pull request state ("open", "closed", or "all")

        Returns:
            Dictionary mapping repository URLs to either lists of pull request information or error strings
        """
        result = await self._rust_manager.fetch_pull_requests(repo_urls, state)
        if not isinstance(result, dict):
            raise TypeError(
                f"Expected Dict[str, Union[List[PullRequestInfo], str]], got {type(result)}")
        return result

    async def fetch_code_reviews(self, repo_urls: List[str]) -> Dict[str, Union[Dict[str, List[CodeReviewInfo]], str]]:
        """
        Fetches code review information for multiple repositories.

        Args:
            repo_urls: List of repository URLs to analyze

        Returns:
            Dictionary mapping repository URLs to either dictionaries mapping PR numbers to lists of code review information, or error strings
        """
        result = await self._rust_manager.fetch_code_reviews(repo_urls)
        if not isinstance(result, dict):
            raise TypeError(
                f"Expected Dict[str, Union[Dict[str, List[CodeReviewInfo]], str]], got {type(result)}")
        return result

    async def fetch_comments(self, repo_urls: List[str], comment_types: Optional[List[str]] = None) -> Dict[str, Union[List[CommentInfo], str]]:
        """
        Fetches comments of various types for multiple repositories.

        Args:
            repo_urls: List of repository URLs to analyze
            comment_types: Optional list of comment types to fetch ("issue", "commit", "pull_request", "review_comment")

        Returns:
            Dictionary mapping repository URLs to either lists of comment information or error strings
        """
        result = await self._rust_manager.fetch_comments(repo_urls, comment_types)
        if not isinstance(result, dict):
            raise TypeError(
                f"Expected Dict[str, Union[List[CommentInfo], str]], got {type(result)}")
        return result

    async def analyze_branches(self, repo_urls: List[str]) -> Dict[str, Union[List[BranchInfo], str]]:
        """
        Analyzes branches in cloned repositories.

        Args:
            repo_urls: List of repository URLs to analyze

        Returns:
            Dictionary mapping repository URLs to either lists of branch information or error strings
        """
        result = await self._rust_manager.analyze_branches(repo_urls)
        if not isinstance(result, dict):
            raise TypeError(
                f"Expected Dict[str, Union[List[BranchInfo], str]], got {type(result)}")
        return result


# Copy docstring from the Rust RepoManager class automatically
RepoManager.__doc__ = _RustRepoManager.__doc__


def setup_async() -> None:
    """
    Initializes the asynchronous runtime environment needed for manager operations.
    Must be called before using any async functionality in the library.

    Returns:
        None
    """
    return _setup_async()
