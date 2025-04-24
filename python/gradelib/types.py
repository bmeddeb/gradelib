"""
Typed wrapper classes for GradeLib.

This module provides proper Python dataclass wrappers around the Rust-generated
classes to improve static type analysis support.
"""

from __future__ import annotations
from dataclasses import dataclass
from typing import Optional, Literal, Dict, List, Union, TypeVar, Any

# Define type literals for better type safety
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

    @classmethod
    def from_rust(cls, rust_status) -> CloneStatus:
        """Convert a Rust-generated status object to a Python dataclass."""
        return cls(
            status_type=rust_status.status_type,
            progress=rust_status.progress,
            error=rust_status.error
        )


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

    @classmethod
    def from_rust(cls, rust_task) -> CloneTask:
        """Convert a Rust-generated task object to a Python dataclass."""
        return cls(
            url=rust_task.url,
            status=CloneStatus.from_rust(rust_task.status),
            temp_dir=rust_task.temp_dir
        )


# TypedDict classes for return types
class CommitInfo(dict):
    """Information about a git commit."""
    pass


class BlameLineInfo(dict):
    """Information about a single line from git blame."""
    pass


class CollaboratorInfo(dict):
    """Information about a repository collaborator."""
    pass


class IssueInfo(dict):
    """Information about a GitHub issue."""
    pass


class PullRequestInfo(dict):
    """Information about a GitHub pull request."""
    pass


class CodeReviewInfo(dict):
    """Information about a GitHub code review."""
    pass


class CommentInfo(dict):
    """Information about a GitHub comment."""
    pass


class BranchInfo(dict):
    """Information about a git branch."""
    pass


# Type conversion functions
def convert_clone_tasks(rust_tasks: Dict[str, Any]) -> Dict[str, CloneTask]:
    """Convert Rust CloneTask objects to Python CloneTask dataclasses."""
    return {url: CloneTask.from_rust(task) for url, task in rust_tasks.items()}