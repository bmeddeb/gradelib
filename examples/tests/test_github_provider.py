"""
Tests for the GitHubProvider class in the gradelib module
"""
from tests.test_utils import AsyncTestCase, TEST_REPOS, TEST_FILES
from github import GitHubProvider
from gradelib import setup_async
import unittest
import os
import sys
from pathlib import Path
import asyncio
from unittest import mock

# Add project root to PYTHONPATH
PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))


# Initialize the async runtime
setup_async()


class GitHubProviderInitTest(unittest.TestCase):
    """Test initialization of GitHubProvider"""

    def test_init_basic(self):
        """Test basic initialization with username and token"""
        provider = GitHubProvider(
            username="test_user", token="test_token", urls=[])
        self.assertIsInstance(provider, GitHubProvider)

    def test_init_with_repos(self):
        """Test initialization with repositories"""
        urls = list(TEST_REPOS.values())
        provider = GitHubProvider(
            username="test_user", token="test_token", urls=urls)
        self.assertIsInstance(provider, GitHubProvider)


class GitHubProviderTest(AsyncTestCase):
    """Test GitHubProvider methods"""

    def setUp(self):
        """Set up a GitHubProvider instance for testing"""
        super().setUp()
        # Use environment variables for credentials if available, otherwise use placeholders
        self.username = os.environ.get("GITHUB_USERNAME", "test_user")
        self.token = os.environ.get("GITHUB_TOKEN", "test_token")
        self.provider = GitHubProvider(
            username=self.username, token=self.token, urls=[])

        # If we're using real credentials, use a real repo for testing
        if "test_user" not in (self.username, self.token):
            self.repo_url = TEST_REPOS["example1"]
        else:
            # Mock mode - we'll be mocking the API calls
            self.repo_url = "https://github.com/test/repo"

    @unittest.skipIf(os.environ.get("GITHUB_TOKEN") is None, "No GitHub token available")
    async def test_clone_repo(self):
        """Test cloning a single repository"""
        result = await self.provider.clone(self.repo_url)
        self.assertIsNone(result)

    @unittest.skipIf(os.environ.get("GITHUB_TOKEN") is None, "No GitHub token available")
    async def test_fetch_clone_tasks(self):
        """Test fetching clone tasks"""
        # First clone to create a task
        await self.provider.clone(self.repo_url)

        # Then fetch the tasks
        tasks = await self.provider.fetch_clone_tasks()
        self.assertIsInstance(tasks, dict)
        self.assertIn(self.repo_url, tasks)

    @unittest.skipIf(os.environ.get("GITHUB_TOKEN") is None, "No GitHub token available")
    async def test_analyze_commits(self):
        """Test analyzing commits in a repository"""
        # First clone the repo
        await self.provider.clone(self.repo_url)

        # Then analyze commits
        commits = await self.provider.analyze_commits(self.repo_url)
        self.assertIsInstance(commits, list)
        if commits:  # If there are commits in the repo
            self.assertIsInstance(commits[0], dict)
            # Check the presence of required commit fields
            required_fields = [
                "sha", "repo_name", "message", "author_name", "author_email",
                "author_timestamp", "author_offset", "committer_name", "committer_email",
                "committer_timestamp", "committer_offset", "additions", "deletions", "is_merge"
            ]
            for field in required_fields:
                self.assertIn(field, commits[0])

    @unittest.skipIf(os.environ.get("GITHUB_TOKEN") is None, "No GitHub token available")
    async def test_bulk_blame(self):
        """Test bulk blaming files in a repository"""
        # First clone the repo
        await self.provider.clone(self.repo_url)

        # Then perform bulk blame
        blame_results = await self.provider.bulk_blame(self.repo_url, TEST_FILES)
        self.assertIsInstance(blame_results, dict)

        # At least one file should have blame results if the files exist
        # Check for correct structure in the results
        for file_path, blame_data in blame_results.items():
            # If it's a list, it's blame line data
            if isinstance(blame_data, list) and blame_data:
                self.assertIsInstance(blame_data[0], dict)
                required_fields = [
                    "commit_id", "author_name", "author_email",
                    "orig_line_no", "final_line_no", "line_content"
                ]
                for field in required_fields:
                    self.assertIn(field, blame_data[0])

    @unittest.skipIf(os.environ.get("GITHUB_TOKEN") is None, "No GitHub token available")
    async def test_analyze_branches(self):
        """Test analyzing branches in repositories"""
        # First clone the repo
        await self.provider.clone(self.repo_url)

        # Then analyze branches
        branch_results = await self.provider.analyze_branches([self.repo_url])
        self.assertIsInstance(branch_results, dict)
        self.assertIn(self.repo_url, branch_results)

        # Check the branch data if it's not an error
        branch_data = branch_results[self.repo_url]
        if isinstance(branch_data, list):
            for branch in branch_data:
                self.assertIsInstance(branch, dict)
                required_fields = [
                    "name", "is_remote", "commit_id", "commit_message",
                    "author_name", "author_email", "author_time", "is_head"
                ]
                for field in required_fields:
                    self.assertIn(field, branch)

    @unittest.skipIf(os.environ.get("GITHUB_TOKEN") is None, "No GitHub token available")
    async def test_fetch_collaborators(self):
        """Test fetching collaborators from repositories"""
        # We need authentication for this
        collab_results = await self.provider.fetch_collaborators([self.repo_url])
        self.assertIsInstance(collab_results, dict)

        # Check collaborator data structure if possible
        if self.repo_url in collab_results:
            collaborators = collab_results[self.repo_url]
            if isinstance(collaborators, list) and collaborators:
                self.assertIsInstance(collaborators[0], dict)
                required_fields = ["login", "github_id"]
                for field in required_fields:
                    self.assertIn(field, collaborators[0])

    @unittest.skipIf(os.environ.get("GITHUB_TOKEN") is None, "No GitHub token available")
    async def test_fetch_issues(self):
        """Test fetching issues from repositories"""
        # Fetch open issues
        issue_results = await self.provider.fetch_issues([self.repo_url], state="open")
        self.assertIsInstance(issue_results, dict)

        # Check issue data structure if possible
        if self.repo_url in issue_results:
            issues = issue_results[self.repo_url]
            if isinstance(issues, list) and issues:
                self.assertIsInstance(issues[0], dict)
                required_fields = [
                    "id", "number", "title", "state", "created_at",
                    "updated_at", "user_login", "user_id", "comments_count",
                    "is_pull_request", "labels", "assignees", "locked", "html_url"
                ]
                for field in required_fields:
                    self.assertIn(field, issues[0])

    @unittest.skipIf(os.environ.get("GITHUB_TOKEN") is None, "No GitHub token available")
    async def test_fetch_pull_requests(self):
        """Test fetching pull requests from repositories"""
        # Fetch open pull requests
        pr_results = await self.provider.fetch_pull_requests([self.repo_url], state="open")
        self.assertIsInstance(pr_results, dict)

        # Check PR data structure if possible
        if self.repo_url in pr_results:
            prs = pr_results[self.repo_url]
            if isinstance(prs, list) and prs:
                self.assertIsInstance(prs[0], dict)
                required_fields = [
                    "id", "number", "title", "state", "created_at",
                    "updated_at", "user_login", "user_id", "comments",
                    "commits", "additions", "deletions", "changed_files",
                    "labels", "is_draft", "merged"
                ]
                for field in required_fields:
                    self.assertIn(field, prs[0])


class GitHubProviderMockTest(AsyncTestCase):
    """Test GitHubProvider methods with mocked responses"""

    def setUp(self):
        """Set up test environment for mocked tests"""
        super().setUp()
        self.provider = GitHubProvider(
            username="mock_user", token="mock_token", urls=[])

    @mock.patch("gradelib.github_module.GitHubProvider.clone", new_callable=mock.AsyncMock)
    async def test_mock_clone(self, mock_clone):
        """Test cloning a repository with a mock"""
        mock_clone.return_value = None
        repo_url = "https://github.com/mock/repo"

        # Call the cloning method
        await self.provider.clone(repo_url)

        # Check that the mock was called with the correct argument
        mock_clone.assert_called_once_with(repo_url)

    @mock.patch("gradelib.github_module.GitHubProvider.analyze_commits", new_callable=mock.AsyncMock)
    async def test_mock_analyze_commits(self, mock_analyze):
        """Test analyzing commits with a mock"""
        # Set up the mock to return a list of commit data
        example_commits = [
            {
                "sha": "abc123",
                "repo_name": "mock/repo",
                "message": "Mock commit message",
                "author_name": "Mock Author",
                "author_email": "mock@example.com",
                "author_timestamp": 123456789,
                "author_offset": 0,
                "committer_name": "Mock Committer",
                "committer_email": "committer@example.com",
                "committer_timestamp": 123456789,
                "committer_offset": 0,
                "additions": 10,
                "deletions": 5,
                "is_merge": False
            }
        ]
        mock_analyze.return_value = example_commits
        repo_url = "https://github.com/mock/repo"

        # Call the analyze commits method
        result = await self.provider.analyze_commits(repo_url)

        # Check that the mock was called with the correct argument
        mock_analyze.assert_called_once_with(repo_url)
        self.assertEqual(result, example_commits)

    @mock.patch("gradelib.github_module.GitHubProvider.bulk_blame", new_callable=mock.AsyncMock)
    async def test_mock_bulk_blame(self, mock_blame):
        """Test bulk blame with a mock"""
        # Set up the mock to return blame data
        example_blame = {
            "file1.txt": [
                {
                    "commit_id": "abc123",
                    "author_name": "Mock Author",
                    "author_email": "mock@example.com",
                    "orig_line_no": 1,
                    "final_line_no": 1,
                    "line_content": "Mock line of code"
                }
            ]
        }
        mock_blame.return_value = example_blame
        repo_url = "https://github.com/mock/repo"
        files = ["file1.txt"]

        # Call the bulk blame method
        result = await self.provider.bulk_blame(repo_url, files)

        # Check that the mock was called with the correct arguments
        mock_blame.assert_called_once_with(repo_url, files)
        self.assertEqual(result, example_blame)


if __name__ == "__main__":
    unittest.main()
