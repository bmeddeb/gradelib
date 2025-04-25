import os
import pytest
import asyncio

import gradelib

# Custom + public test repos
TEST_REPOS = [
    "https://github.com/bmeddeb/alpha-repo",
    "https://github.com/bmeddeb/alpha-repo-2",
    "https://github.com/bmeddeb/beta-repo",
    "https://github.com/bmeddeb/gamma-repo",
    "https://github.com/octocat/Hello-World",
    "https://github.com/githubtraining/hellogitworld",
    "https://github.com/barryclark/jekyll-now",
]


@pytest.fixture(autouse=True)
def set_github_token(monkeypatch):
    token = os.getenv("GITHUB_TOKEN", "fake-token")
    monkeypatch.setenv("GITHUB_TOKEN", token)
    return token


@pytest.mark.asyncio
async def test_setup_async():
    gradelib.setup_async()


@pytest.mark.asyncio
async def test_local_clone_and_blame(tmp_path, monkeypatch):
    """Tests cloning a locally created repo and performing blame."""
    repo_dir = tmp_path / "mini_repo"
    repo_dir.mkdir()

    # Initialize repo
    os.system(f"git init {repo_dir}")
    os.system(f"git -C {repo_dir} config user.name 'Test User'")
    os.system(f"git -C {repo_dir} config user.email 'test@example.com'")
    readme = repo_dir / "workflow_usage.md"
    readme.write_text("Hello\n")
    os.system(f"git -C {repo_dir} add workflow_usage.md")
    os.system(f"git -C {repo_dir} commit -m 'initial commit'")

    local_path = str(repo_dir)
    manager = gradelib.RepoManager([local_path], "user", "token")
    await manager.clone_all()
    tasks = await manager.fetch_clone_tasks()

    assert local_path in tasks
    assert tasks[local_path].status.status_type == "completed"

    result = await manager.bulk_blame(local_path, ["workflow_usage.md"])
    assert "workflow_usage.md" in result
    assert isinstance(result["workflow_usage.md"], list)
    assert len(result["workflow_usage.md"]) > 0


@pytest.mark.asyncio
async def test_analyze_commits(tmp_path, monkeypatch):
    """Test commit analysis on a local repo."""
    repo_dir = tmp_path / "commit_repo"
    repo_dir.mkdir()
    os.system(f"git init {repo_dir}")
    os.system(f"git -C {repo_dir} config user.name 'Test User'")
    os.system(f"git -C {repo_dir} config user.email 'test@example.com'")
    file = repo_dir / "main.py"
    file.write_text("print('Hello')\n")
    os.system(
        f"git -C {repo_dir} add main.py && git -C {repo_dir} commit -m 'Initial'")
    file.write_text("print('Updated')\n")
    os.system(
        f"git -C {repo_dir} add main.py && git -C {repo_dir} commit -m 'Second commit'")

    local_path = str(repo_dir)
    manager = gradelib.RepoManager([local_path], "user", "token")
    await manager.clone_all()
    commits = await manager.analyze_commits(local_path)
    assert isinstance(commits, list)
    assert len(commits) >= 2
    assert any("Second commit" in c["message"] for c in commits)


@pytest.mark.asyncio
async def test_bulk_clone_real_repos(set_github_token):
    """Tests cloning multiple real GitHub repos."""
    manager = gradelib.RepoManager(
        TEST_REPOS, "your-username", os.environ["GITHUB_TOKEN"])
    await manager.clone_all()
    tasks = await manager.fetch_clone_tasks()

    for repo in TEST_REPOS:
        task = tasks[repo]
        print(f"{repo} -> {task.status.status_type}")
        if repo == "https://github.com/bmeddeb/alpha-repo-2":
            # This is the intentionally invalid repo; should fail
            assert task.status.status_type == "failed"
            assert isinstance(task.status.error, str)
            print(f"Expected failure for {repo}: {task.status.error}")
        else:
            if task.status.status_type == "failed":
                print(f"Error: {task.status.error}")
            assert task.status.status_type == "completed", f"{repo} did not complete: {task.status.error}"


@pytest.mark.asyncio
@pytest.mark.skipif("GITHUB_TOKEN" not in os.environ, reason="Requires real GitHub token")
async def test_fetch_collaborators(set_github_token):
    """Tests fetching collaborators for known public repos, including an invalid repo."""
    manager = gradelib.RepoManager(
        TEST_REPOS, "your-username", os.environ["GITHUB_TOKEN"])
    collaborators = await manager.fetch_collaborators(TEST_REPOS)

    for repo, data in collaborators.items():
        if repo == "https://github.com/bmeddeb/alpha-repo-2":
            # This is the intentionally invalid repo; should return an error string
            assert isinstance(
                data, str), f"Expected error string for {repo}, got {type(data)}"
            print(f"{repo} correctly returned error: {data}")
        else:
            # For other repos, accept either a list (success) or a string (error)
            if isinstance(data, list):
                assert all("login" in c for c in data)
                print(f"{repo} returned {len(data)} collaborators")
            else:
                print(f"{repo} returned error: {data}")
                assert isinstance(data, str)


@pytest.mark.asyncio
async def test_fetch_issues(set_github_token):
    """
    Test fetching GitHub issues from multiple public repositories.
    """
    manager = gradelib.RepoManager(
        TEST_REPOS, "your-username", os.environ["GITHUB_TOKEN"])
    results = await manager.fetch_issues(TEST_REPOS)

    for repo_url, data in results.items():
        if isinstance(data, list):
            for issue in data:
                assert "title" in issue
                assert "state" in issue
                assert "user_login" in issue
        else:
            # If it's not a list, it's an error string
            assert isinstance(data, str)


@pytest.mark.asyncio
async def test_fetch_pull_requests(set_github_token):
    """
    Test fetching GitHub pull requests from multiple public repositories.
    """
    manager = gradelib.RepoManager(
        TEST_REPOS, "your-username", os.environ["GITHUB_TOKEN"])
    results = await manager.fetch_pull_requests(TEST_REPOS)

    for repo_url, data in results.items():
        if isinstance(data, list):
            for pr in data:
                assert "title" in pr
                assert "user_login" in pr
                assert pr["state"] in ("open", "closed")
        else:
            # Expect error string on failure
            assert isinstance(data, str)
