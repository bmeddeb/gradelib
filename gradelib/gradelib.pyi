# Stubs for the gradelib Rust library
# Generated based on src/lib.rs

from typing import Any, Dict, List, Optional, Awaitable, Union, Mapping, TypedDict

# --- Top-level Functions ---

def setup_async() -> None:
    """Initializes the asynchronous runtime environment needed for manager operations."""
    ...

# --- Type Aliases for Complex Return Types ---

# Represents the structure returned for each line in a successful blame result.
# Corresponds to the Rust `BlameLineInfo` struct, but returned as a dict.
BlameLineDict = Mapping[str, Union[str, int]]

# Represents the result for a single file in bulk_blame: either a list of blame lines or an error string.
BlameResultForFile = Union[List[BlameLineDict], str]

# Represents the overall result of a bulk_blame call: a map from file path to its blame result.
BulkBlameResult = Mapping[str, BlameResultForFile]

# Represents a dictionary containing information about a single commit.
class CommitDict(TypedDict):
    """Dictionary representation of a Git commit with detailed metadata."""
    sha: str                # Commit hash
    repo_name: str          # Repository name (usually owner/repo format)
    message: str            # Commit message
    author_name: str        # Author's name
    author_email: str       # Author's email
    author_timestamp: int   # Author timestamp (seconds since epoch)
    author_offset: int      # Author timezone offset in minutes
    committer_name: str     # Committer's name
    committer_email: str    # Committer's email
    committer_timestamp: int # Committer timestamp (seconds since epoch)
    committer_offset: int   # Committer timezone offset in minutes
    additions: int          # Number of lines added in this commit
    deletions: int          # Number of lines deleted in this commit
    is_merge: bool          # Whether this is a merge commit (has more than one parent)

# Represents a dictionary containing information about a single code review.
class ReviewDict(TypedDict):
    """Dictionary representation of a GitHub code review."""
    id: int                 # Review ID
    pr_number: int          # Pull request number
    user_login: str         # Reviewer's GitHub login
    user_id: int            # Reviewer's GitHub user ID
    body: Optional[str]     # Review comment body (can be None)
    state: str              # Review state (APPROVED, CHANGES_REQUESTED, COMMENTED, etc.)
    submitted_at: str       # Timestamp when review was submitted
    commit_id: str          # Commit SHA that was reviewed
    html_url: str           # URL to view the review on GitHub

# Represents a dictionary containing information about a comment.
class CommentDict(TypedDict):
    """Dictionary representation of a GitHub comment."""
    id: int                       # Comment ID
    comment_type: str             # Type of comment: 'issue', 'commit', 'pull_request', or 'review_comment'
    user_login: str               # Commenter's GitHub login
    user_id: int                  # Commenter's GitHub user ID
    body: str                     # Comment text content
    created_at: str               # Timestamp when comment was created
    updated_at: str               # Timestamp when comment was last updated
    html_url: str                 # URL to view the comment on GitHub
    issue_number: Optional[int]   # Issue number (only for issue comments)
    pull_request_number: Optional[int]  # PR number (only for PR and review comments)
    commit_id: Optional[str]      # Commit ID (only for review comments)
    commit_sha: Optional[str]     # Commit SHA (only for commit comments)
    path: Optional[str]           # File path (only for review and commit comments)
    position: Optional[int]       # Line position (only for review and commit comments)
    line: Optional[int]           # Line number (only for review and commit comments)


# --- Exposed Classes ---

class CloneStatus:
    """Represents the status of a cloning operation. Corresponds to ExposedCloneStatus.

    Attributes:
        status_type: The type of status ('queued', 'cloning', 'completed', 'failed').
        progress: The cloning progress percentage (0-100), if status_type is 'cloning'.
        error: An error message, if status_type is 'failed'.
    """
    status_type: str
    progress: Optional[int]
    error: Optional[str]

    # Note: PyO3 typically doesn't generate an __init__ for simple structs exposed like this.
    # Instantiation happens internally or via other methods (like fetch_clone_tasks).
    def __init__(self, *args, **kwargs) -> None: ... # Stub for type checker


class CloneTask:
    """Represents a repository cloning task. Corresponds to ExposedCloneTask.

    Attributes:
        url: The URL of the repository.
        status: The current status of the clone operation (CloneStatus object).
        temp_dir: The path to the temporary directory where the repo was cloned,
                  if the clone is completed.
    """
    url: str
    status: CloneStatus
    temp_dir: Optional[str]

    def __init__(self, *args, **kwargs) -> None: ... # Stub for type checker


class RepoManager:
    """Manages cloning and blaming operations for multiple Git repositories.

    Corresponds to the Rust RepoManager struct.
    """
    def __init__(self, urls: List[str], github_username: str, github_token: str) -> None:
        """Initializes the RepoManager with a list of repository URLs and GitHub credentials."""
        ...

    def clone_all(self) -> Awaitable[None]:
        """Clones all repositories configured in this manager instance asynchronously.

        Returns:
            An awaitable that completes when all cloning attempts are initiated.
        """
        ...

    def fetch_clone_tasks(self) -> Awaitable[Dict[str, CloneTask]]:
        """Fetches the current status of all cloning tasks asynchronously.

        Returns:
            An awaitable that resolves to a dictionary mapping repository URLs
            to CloneTask objects.
        """
        ...

    def clone(self, url: str) -> Awaitable[None]:
        """Clones a single repository specified by URL asynchronously.

        Args:
            url: The URL of the repository to clone.

        Returns:
            An awaitable that completes when the cloning attempt is initiated.
        """
        ...

    def bulk_blame(self, target_repo_url: str, file_paths: List[str]) -> Awaitable[BulkBlameResult]:
        """Performs 'git blame' on multiple files within a cloned repository asynchronously.

        Requires the target repository to have been successfully cloned first.

        Args:
            target_repo_url: The URL of the repository (must be managed and cloned).
            file_paths: A list of paths relative to the repository root to blame.

        Returns:
            An awaitable that resolves to a dictionary mapping each requested file path
            to either:
            - A list of dictionaries (BlameLineDict), each representing a blamed line.
            - An error string, if blaming that specific file failed.

        Raises:
            ValueError: If the target repository is not found or not successfully cloned.
                      (Raised when the awaitable is resolved).
        """
        ...

    def analyze_commits(self, target_repo_url: str) -> Awaitable[List[CommitDict]]:
        """Analyzes the commit history of a cloned repository asynchronously.

        Extracts detailed information about each commit using high-performance parallel processing.

        Args:
            target_repo_url: The URL of the repository (must be managed and cloned).

        Returns:
            An awaitable that resolves to a list of dictionaries, each representing a commit.
            Each commit includes metadata such as the SHA, author, message, timestamps,
            and the number of additions/deletions.

        Raises:
            ValueError: If the target repository is not found or not successfully cloned,
                        or if the URL format is not recognized.
                        (Raised when the awaitable is resolved).
        """
        ...

    def fetch_collaborators(self, repo_urls: List[str]) -> Awaitable[Dict[str, List[Dict[str, Any]]]]:
        """Fetches collaborator information for multiple repositories asynchronously.

        Args:
            repo_urls: A list of repository URLs to fetch collaborator information for.

        Returns:
            An awaitable that resolves to a dictionary mapping repository URLs to
            lists of collaborator information dictionaries. Each collaborator dictionary
            contains 'login', 'github_id', 'full_name', 'email', and 'avatar_url' fields.

        Raises:
            ValueError: If there is an error fetching collaborator information.
                        (Raised when the awaitable is resolved).
        """
        ...

    def fetch_issues(self, repo_urls: List[str], state: Optional[str] = None) -> Awaitable[Dict[str, Union[List[Dict[str, Any]], str]]]:
        """Fetches issue information for multiple repositories asynchronously.

        Args:
            repo_urls: A list of repository URLs to fetch issue information for.
            state: Optional filter for issue state. Can be "open", "closed", or "all".
                  If None, defaults to "all".

        Returns:
            An awaitable that resolves to a dictionary mapping repository URLs to
            either:
            - A list of dictionaries, each representing an issue.
            - An error string, if issue fetching for that repository failed.

        Raises:
            ValueError: If there is an error fetching issue information.
                        (Raised when the awaitable is resolved).
        """
        ...

    def fetch_pull_requests(self, repo_urls: List[str], state: Optional[str] = None) -> Awaitable[Dict[str, Union[List[Dict[str, Any]], str]]]:
        """Fetches pull request information for multiple repositories asynchronously.

        Args:
            repo_urls: A list of repository URLs to fetch pull request information for.
            state: Optional filter for pull request state. Can be "open", "closed", or "all".
                  If None, defaults to "all".

        Returns:
            An awaitable that resolves to a dictionary mapping repository URLs to
            either:
            - A list of dictionaries, each representing a pull request.
            - An error string, if pull request fetching for that repository failed.

        Raises:
            ValueError: If there is an error fetching pull request information.
                        (Raised when the awaitable is resolved).
        """
        ...

    def fetch_code_reviews(self, repo_urls: List[str]) -> Awaitable[Dict[str, Union[Dict[str, List[ReviewDict]], str]]]:
        """Fetches code review information for multiple repositories asynchronously.

        Args:
            repo_urls: A list of repository URLs to fetch code review information for.

        Returns:
            An awaitable that resolves to a dictionary mapping repository URLs to
            either:
            - A dictionary mapping PR numbers (as strings) to lists of review dictionaries.
            - An error string, if review fetching for that repository failed.

        Raises:
            ValueError: If there is an error fetching code review information.
                        (Raised when the awaitable is resolved).
        """
        ...

    def fetch_comments(self, repo_urls: List[str], comment_types: Optional[List[str]] = None) -> Awaitable[Dict[str, Union[List[CommentDict], str]]]:
        """Fetches comments of various types for multiple repositories asynchronously.

        Args:
            repo_urls: A list of repository URLs to fetch comments for.
            comment_types: Optional filter for comment types. Can include "issue", "commit",
                           "pull_request" (or "pullrequest"), and "review_comment" (or "reviewcomment").
                           If None, all comment types are fetched.

        Returns:
            An awaitable that resolves to a dictionary mapping repository URLs to
            either:
            - A list of comment dictionaries.
            - An error string, if comment fetching for that repository failed.

        Raises:
            ValueError: If there is an error fetching comments or an invalid comment type is specified.
                        (Raised when the awaitable is resolved).
        """
        ...

    def analyze_branches(self, repo_urls: List[str]) -> Awaitable[Dict[str, Union[List[Dict[str, Any]], str]]]:
        """Analyzes branches in cloned repositories.

        Args:
            repo_urls: A list of repository URLs to analyze branches for.

        Returns:
            An awaitable that resolves to a dictionary mapping repository URLs to
            either:
            - A list of dictionaries, each representing a branch.
            - An error string, if branch analysis for that repository failed.

        Raises:
            ValueError: If there is an error processing the branch information.
                        (Raised when the awaitable is resolved).
        """
        ...

# --- Taiga Module ---

class TaigaClient:
    """
    Client for interacting with the Taiga API.

    Provides methods to fetch data from Taiga projects including projects, sprints,
    user stories, tasks, and task history.
    """

    def __init__(self, base_url: str, auth_token: str, username: str) -> None:
        """
        Initialize a TaigaClient with the given credentials.

        Args:
            base_url: The base URL for the Taiga API (e.g., "https://api.taiga.io/api/v1/")
            auth_token: Authentication token for the Taiga API
            username: Username for the Taiga account
        """
        ...

    def fetch_project_data(self, slug: str) -> Awaitable[Dict[str, Any]]:
        """
        Fetch comprehensive data for a single Taiga project by its slug.

        Args:
            slug: The project slug

        Returns:
            An awaitable that resolves to a dictionary containing all project data:
            - project: Basic project information
            - members: List of project members
            - sprints: List of project sprints/milestones
            - user_stories: Dictionary mapping sprint IDs to lists of user stories
            - tasks: Dictionary mapping sprint IDs to lists of tasks
            - task_histories: Dictionary mapping task IDs to lists of history events

        Raises:
            ValueError: If there is an error fetching project data
        """
        ...

    def fetch_multiple_projects(self, slugs: List[str]) -> Awaitable[Dict[str, Union[bool, str]]]:
        """
        Fetch data for multiple Taiga projects concurrently.

        Args:
            slugs: List of project slugs to fetch data for

        Returns:
            An awaitable that resolves to a dictionary mapping project slugs to:
            - True: If the project was successfully fetched
            - Error message: If there was an error fetching the project

        Raises:
            ValueError: If there is an error with the Taiga API
        """
        ...