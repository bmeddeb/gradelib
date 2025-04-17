# GradeLib Examples

This directory contains example Python scripts that demonstrate how to use the functions exposed from Rust to Python in the `gradelib` module.

## Test Suite

The `tests` directory contains unit tests for all functions in the `gradelib` module. You can run the tests using:

```bash
python run_all_tests.py
```

## Example Scripts

Each example script demonstrates a specific function or feature of the `gradelib` module:

### Basic Functions

- **test_setup_async.py**: Demonstrates how to initialize the async runtime needed for other operations.
- **test_github_init.py**: Shows how to initialize a GitHubProvider instance.

### Repository Management

- **test_clone_repo.py**: Shows how to clone repositories and manage clone tasks.

### Git Analysis

- **test_analyze_commits.py**: Demonstrates commit history analysis.
- **test_bulk_blame.py**: Shows how to analyze file blame information.
- **test_analyze_branches.py**: Demonstrates branch analysis.

### GitHub API Interaction

- **test_fetch_collaborators.py**: Shows how to retrieve collaborator information.
- **test_fetch_issues.py**: Demonstrates fetching and analyzing issues.
- **test_fetch_pull_requests.py**: Shows how to work with pull requests.

### Comprehensive Example

- **test_all_functions.py**: A comprehensive example that demonstrates all functions in the `gradelib` module.

## Running the Examples

To run an example script, use:

```bash
python <example_script>.py
```

Most examples require GitHub authentication. You can provide this by setting the following environment variables:

```bash
export GITHUB_USERNAME=your_username
export GITHUB_TOKEN=your_personal_access_token
```

## GitHub Credentials

For the example scripts to work properly with GitHub's API, you need to:

1. Create a personal access token with the following scopes:
   - `repo`: Full control of private repositories
   - `read:org`: Read organization information
   - `user`: Read user information

2. Set the environment variables:
   ```bash
   export GITHUB_USERNAME=your_username
   export GITHUB_TOKEN=your_personal_access_token
   ```

## Custom Repositories

Most example scripts allow you to specify one or more repository URLs as command-line arguments. For example:

```bash
python test_analyze_commits.py https://github.com/username/repo
```

## Error Handling

These example scripts include error handling to demonstrate how to properly handle errors that may occur when using the `gradelib` module. Pay attention to the try/except blocks to understand how to handle errors in your own code.

## Async Operations

All network and IO operations in the `gradelib` module are asynchronous. The examples use `asyncio.run()` to run the async functions, but you can also use any other async framework or integration with your existing async code.

Remember to call `setup_async()` once at the beginning of your program to initialize the async runtime.

## Working with Results

The results returned by `gradelib` functions are Python data structures (lists, dictionaries, etc.) that you can process just like any other Python data. The examples show how to:

- Process lists of commits, branches, issues, pull requests, etc.
- Analyze and extract insights from Git data
- Generate reports and statistics
- Handle errors and edge cases

## Command-Line Arguments

Most examples accept command-line arguments for specifying repositories or other parameters. See the docstring at the top of each script for details on the available arguments.

## Contributing

If you create additional examples or improve the existing ones, please feel free to submit a pull request.
