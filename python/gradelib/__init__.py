from typing import Dict, List, Optional, Union

from .async_handler import async_handler
from .gradelib import GitHubOAuthClient
from .gradelib import RepoManager as _RustRepoManager
from .gradelib import TaigaClient
from .gradelib import setup_async as _setup_async
from .types import (BlameLineInfo, BranchInfo, CloneStatus, CloneStatusType,
                    CloneTask, CodeReviewInfo, CollaboratorInfo, CommentInfo,
                    CommentType, CommitInfo, IssueInfo, PullRequestInfo,
                    convert_clone_tasks)

__all__ = [
    "setup_async",
    "RepoManager",
    "CloneStatus",
    "CloneTask",
    "TaigaClient",
    "CloneStatusType",
    "CommentType",
    "async_handler",
    "GitHubOAuthClient",
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

    def __init__(self) -> None:
        """
        Initialize with a Rust RepoManager instance. This constructor is not meant to be
        called directly. Use the class methods `create` or `create_async` instead.
        """
        self._rust_manager = None

    @classmethod
    async def create(
        cls,
        urls: List[str],
        github_token: str,
        github_username: Optional[str] = None,
        no_cache: bool = False,
    ) -> 'RepoManager':
        """
        Create a new RepoManager with GitHub credentials.

        Args:
            urls: List of repository URLs to manage
            github_token: GitHub personal access token for authentication
            github_username: GitHub username for authentication (optional)
            no_cache: Flag to indicate whether to use cache

        Returns:
            A new RepoManager instance
        """
        manager = cls()
        manager._rust_manager = await _RustRepoManager.create_async(
            urls,
            github_token,
            github_username,
            no_cache,
        )
        return manager

    async def clone_all(self) -> None:
        """
        Clones all repositories configured in this manager instance asynchronously.

        Returns:
            None
        """
        if not self._rust_manager:
            raise RuntimeError(
                "RepoManager not properly initialized. Use the 'create' class method.")
        return await self._rust_manager.clone_all()

    async def fetch_clone_tasks(self) -> Dict[str, CloneTask]:
        """
        Fetches the current status of all cloning tasks asynchronously.

        Returns:
            A dictionary mapping repository URLs to CloneTask objects
        """
        if not self._rust_manager:
            raise RuntimeError(
                "RepoManager not properly initialized. Use the 'create' class method.")
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
        if not self._rust_manager:
            raise RuntimeError(
                "RepoManager not properly initialized. Use the 'create' class method.")
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
        if not self._rust_manager:
            raise RuntimeError(
                "RepoManager not properly initialized. Use the 'create' class method.")
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
        if not self._rust_manager:
            raise RuntimeError(
                "RepoManager not properly initialized. Use the 'create' class method.")
        result = await self._rust_manager.analyze_commits(repo_path)
        if not isinstance(result, list):
            raise TypeError(f"Expected List[CommitInfo], got {type(result)}")
        return result

    async def fetch_collaborators(self, repo_urls: List[str], max_pages: Optional[int] = None) -> Dict[str, List[CollaboratorInfo]]:
        """
        Fetches collaborator information for multiple repositories.

        Args:
            repo_urls: List of repository URLs to analyze
            max_pages: Optional maximum number of pages to fetch (None = fetch all)

        Returns:
            Dictionary mapping repository URLs to lists of collaborator information
        """
        if not self._rust_manager:
            raise RuntimeError(
                "RepoManager not properly initialized. Use the 'create' class method.")
        result = await self._rust_manager.fetch_collaborators(repo_urls, max_pages)
        if not isinstance(result, dict):
            raise TypeError(
                f"Expected Dict[str, List[CollaboratorInfo]], got {type(result)}")
        return result

    async def fetch_issues(self, repo_urls: List[str], state: Optional[str] = None, max_pages: Optional[int] = None) -> Dict[str, Union[List[IssueInfo], str]]:
        """
        Fetches issue information for multiple repositories.

        Args:
            repo_urls: List of repository URLs to analyze
            state: Optional filter for issue state ("open", "closed", or "all")
            max_pages: Optional maximum number of pages to fetch (None = fetch all)

        Returns:
            Dictionary mapping repository URLs to either lists of issue information or error strings
        """
        if not self._rust_manager:
            raise RuntimeError(
                "RepoManager not properly initialized. Use the 'create' class method.")
        result = await self._rust_manager.fetch_issues(repo_urls, state, max_pages)
        if not isinstance(result, dict):
            raise TypeError(
                f"Expected Dict[str, Union[List[IssueInfo], str]], got {type(result)}")
        return result

    async def fetch_pull_requests(self, repo_urls: List[str], state: Optional[str] = None, max_pages: Optional[int] = None) -> Dict[str, Union[List[PullRequestInfo], str]]:
        """
        Fetches pull request information for multiple repositories.

        Args:
            repo_urls: List of repository URLs to analyze
            state: Optional filter for pull request state ("open", "closed", or "all")
            max_pages: Optional maximum number of pages to fetch (None = fetch all)

        Returns:
            Dictionary mapping repository URLs to either lists of pull request information or error strings
        """
        if not self._rust_manager:
            raise RuntimeError(
                "RepoManager not properly initialized. Use the 'create' class method.")
        result = await self._rust_manager.fetch_pull_requests(repo_urls, state, max_pages)
        if not isinstance(result, dict):
            raise TypeError(
                f"Expected Dict[str, Union[List[PullRequestInfo], str]], got {type(result)}")
        return result

    async def fetch_code_reviews(self, repo_urls: List[str], max_pages: Optional[int] = None) -> Dict[str, Union[Dict[str, List[CodeReviewInfo]], str]]:
        """
        Fetches code review information for multiple repositories.

        Args:
            repo_urls: List of repository URLs to analyze
            max_pages: Optional maximum number of pages to fetch (None = fetch all)

        Returns:
            Dictionary mapping repository URLs to either dictionaries mapping PR numbers to lists of code review information, or error strings
        """
        if not self._rust_manager:
            raise RuntimeError(
                "RepoManager not properly initialized. Use the 'create' class method.")
        result = await self._rust_manager.fetch_code_reviews(repo_urls, max_pages)
        if not isinstance(result, dict):
            raise TypeError(
                f"Expected Dict[str, Union[Dict[str, List[CodeReviewInfo]], str]], got {type(result)}")
        return result

    async def fetch_comments(self, repo_urls: List[str], comment_types: Optional[List[str]] = None, max_pages: Optional[int] = None) -> Dict[str, Union[List[CommentInfo], str]]:
        """
        Fetches comments of various types for multiple repositories.

        Args:
            repo_urls: List of repository URLs to analyze
            comment_types: Optional list of comment types to fetch ("issue", "commit", "pull_request", "review_comment")
            max_pages: Optional maximum number of pages to fetch (None = fetch all)

        Returns:
            Dictionary mapping repository URLs to either lists of comment information or error strings
        """
        if not self._rust_manager:
            raise RuntimeError(
                "RepoManager not properly initialized. Use the 'create' class method.")
        result = await self._rust_manager.fetch_comments(repo_urls, comment_types, max_pages)
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
        if not self._rust_manager:
            raise RuntimeError(
                "RepoManager not properly initialized. Use the 'create' class method.")
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
