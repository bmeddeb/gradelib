# Create tests/test_gradelib.py
import asyncio
import os
import tempfile
import pytest

# Ensure we import our extension module
import gradelib


@pytest.fixture(autouse=True)
def setup_environment(tmp_path, monkeypatch):
    # Monkeypatch a small git repo in a temp dir
    repo_dir = tmp_path / "mini_repo"
    os.mkdir(repo_dir)

    # Initialize an empty git repo
    os.system(f"git init {repo_dir}")

    # Set Git configuration for this repository
    os.system(f"git -C {repo_dir} config user.email 'test@example.com'")
    os.system(f"git -C {repo_dir} config user.name 'Test User'")

    # Create a dummy file and commit
    file_path = repo_dir / "README.md"
    file_path.write_text("# Hello")

    # Add and commit with proper error checking
    result = os.system(
        f"cd {repo_dir} && git add README.md && git commit -m 'initial'")
    if result != 0:
        pytest.fail(f"Failed to commit test file, git exit code: {result}")

    # Patch RepoManager to use this local repo
    monkeypatch.setenv("GITHUB_TOKEN", "fake-token")
    return str(repo_dir)


@pytest.mark.asyncio
async def test_setup_async():
    # Should not raise
    gradelib.setup_async()


@pytest.mark.asyncio
async def test_clone_and_bulk_blame(tmp_path, setup_environment):
    repo_path = setup_environment
    manager = gradelib.RepoManager([repo_path], "user", "token")
    # Clone the repo
    await manager.clone_all()
    tasks = await manager.fetch_clone_tasks()
    assert repo_path in tasks
    task = tasks[repo_path]
    assert task.status.status_type == "completed"

    # Perform blame on README.md
    result = await manager.bulk_blame(repo_path, ["README.md"])
    assert "README.md" in result
    blame_lines = result["README.md"]
    assert isinstance(blame_lines, list)
    assert len(blame_lines) > 0

# TODO: Add tests for analyze_commits, fetch_collaborators, etc.
