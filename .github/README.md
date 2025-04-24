# GitHub Actions Workflows for GradeLib

This directory contains GitHub Actions workflows that automate testing, building, and deployment of GradeLib.

## Workflows

### 1. Check Version and Release (`check-version.yml`)

This workflow automatically checks for version changes in `Cargo.toml` and `pyproject.toml` files. When a version change is detected, it:

1. Creates a new Git tag matching the version (e.g., `v0.1.2`)
2. Creates a GitHub Release
3. Triggers the PyPI publishing workflow

**Trigger:** Runs on pushes to `main` branch that modify `Cargo.toml` or `pyproject.toml` files.

### 2. Build and Publish to PyPI (`publish.yml`)

This workflow builds and publishes the package to PyPI.

1. Builds wheel files for multiple platforms (Linux, macOS, Windows)
2. Builds source distribution (sdist)
3. Uploads all packages to PyPI

**Trigger:** Runs when a new tag matching `v*` is created (automatically by the check-version workflow or manually).

## Setting Up Secrets

To use the PyPI publishing workflow, you need to set up the following secrets in your GitHub repository:

1. `PYPI_USERNAME`: Your PyPI username (or `__token__` if using API tokens)
2. `PYPI_API_TOKEN`: Your PyPI password or API token

## Manual Release Process

You can also trigger a release manually:

1. Update the version in both `Cargo.toml` and `pyproject.toml` (or use the `tools/sync_version.py` script)
2. Commit and push the changes to `main`
3. The `check-version.yml` workflow will automatically create a release and tag

Alternatively, you can manually dispatch the workflow from the GitHub Actions tab.

## Version Management

Use the provided `tools/sync_version.py` script to manage versions across both configuration files:

```bash
# Set a specific version
python tools/sync_version.py --version 0.2.0

# Bump patch version (0.1.0 -> 0.1.1)
python tools/sync_version.py --bump patch

# Bump minor version (0.1.0 -> 0.2.0)
python tools/sync_version.py --bump minor

# Bump major version (0.1.0 -> 1.0.0)
python tools/sync_version.py --bump major
```
