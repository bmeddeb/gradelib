from .gradelib import setup_async, github_module

# Expose the GitHubProvider directly
GitHubProvider = github_module.GitHubProvider

__all__ = ["setup_async", "GitHubProvider"]
