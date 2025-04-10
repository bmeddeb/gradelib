# Stub file for the Rust module 'gradelib'

from typing import Awaitable, Dict, List, Optional, TypedDict, Union

# --- Clone Status and Task ---
class CloneStatus:
    """Represents the status of a cloning operation."""
    @property
    def status_type(self) -> str:
        """The type of status (e.g., 'queued', 'cloning', 'completed', 'failed')."""
        ...
    @property
    def progress(self) -> int | None:
        """Cloning progress percentage (0-100) if status_type is 'cloning', otherwise None."""
        ...
    @property
    def error(self) -> str | None:
        """Error message if status_type is 'failed', otherwise None."""
        ...

class CloneTask:
    """Represents a cloning task and its status for a specific repository."""
    @property
    def url(self) -> str:
        """The URL of the repository being cloned."""
        ...
    @property
    def status(self) -> CloneStatus:
        """The current status object for this cloning task."""
        ...
    @property
    def temp_dir(self) -> str | None:
        """The path to the temporary directory where the repo was cloned if completed, otherwise None."""
        ...

# --- TypedDict for Blame Line Info ---
class BlameLineDict(TypedDict):
    """Structure of the dictionary returned for each line in bulk_blame."""
    commit_id: str
    author_name: str
    author_email: str
    orig_line_no: int
    final_line_no: int
    line_content: str

# --- RepoManager Class ---
class RepoManager:
    """Manages asynchronous cloning and blaming operations for multiple Git repositories."""

    def __init__(self, urls: list[str], github_username: str, github_token: str) -> None:
        """
        Initializes the RepoManager.

        Args:
            urls: A list of repository URLs (e.g., 'https://github.com/user/repo.git').
            github_username: Your GitHub username for authentication during clone.
            github_token: Your GitHub Personal Access Token (PAT) for authentication.
        """
        ...

    def clone_all(self) -> Awaitable[None]:
        """
        Clones all repositories provided during initialization asynchronously.
        """
        ...

    def fetch_clone_tasks(self) -> Awaitable[dict[str, CloneTask]]:
        """
        Fetches the current status of all managed cloning tasks asynchronously.

        Returns:
            An awaitable that resolves to a dictionary mapping repository URLs
            to their corresponding CloneTask status objects.
        """
        ...

    def clone(self, url: str) -> Awaitable[None]:
        """
        Clones a single repository specified by URL asynchronously.

        Args:
            url: The full URL of the repository to clone (must be one managed by this instance).
        """
        ...

    def bulk_blame(
            self, target_repo_url: str, file_paths: list[str]
    ) -> Awaitable[Dict[str, Union[List[BlameLineDict], str]]]:
        """
        Performs 'git blame' on multiple files within a cloned repository asynchronously.

        Args:
            target_repo_url: The URL of the repository (must have been previously cloned
                             successfully by this manager instance).
            file_paths: A list of file paths relative to the repository root to blame.

        Returns:
            An awaitable that resolves to a dictionary where keys are the relative file paths.
            The values are either:
            - A list of dictionaries, where each dictionary represents a blamed line.
            - A string containing an error message if blaming that specific file failed.

        Raises:
            ValueError: If the target_repo_url is not found, not completed, or
                        another pre-check fails for the bulk operation.
        """
        ...

# --- Setup Function ---
def setup_async() -> None:
    """
    Initializes the asynchronous runtime environment (Tokio) needed for
    the RepoManager's operations. Call once early in your application if using
    certain async integrations (may be optional with newer pyo3-asyncio).
    """
    ...