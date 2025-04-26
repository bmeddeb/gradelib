#!/bin/bash

# 📦 tools/release.sh
# Usage:
# - If no argument: reads version from pyproject.toml
# - If argument provided: uses that version directly

set -e  # Exit on any error

VERSION="$1"

if [ -z "$VERSION" ]; then
  echo "🔎 No version supplied — auto-detecting from pyproject.toml..."
  VERSION=$(python3 -c "import toml; print(toml.load('pyproject.toml')['project']['version'])")
fi

if [ -z "$VERSION" ]; then
  echo "❌ Error: Could not determine version."
  exit 1
fi

echo "🔵 Preparing release for version v$VERSION..."

# Check for uncommitted changes
if [[ -n $(git status --porcelain) ]]; then
  echo "❌ Error: You have uncommitted changes. Please commit them first."
  exit 1
fi

# Confirm action
read -p "⚡ Are you sure you want to create and push tag v$VERSION? (y/n) " -n 1 -r
echo    # new line
if [[ ! $REPLY =~ ^[Yy]$ ]]
then
    echo "❌ Release aborted."
    exit 1
fi

# Create Git tag
git tag "v$VERSION"

# Push main branch and tag
git push origin main
git push origin "v$VERSION"

# Create GitHub release using GitHub CLI (gh) if available
if command -v gh &> /dev/null; then
  echo "📝 Creating GitHub draft release using GitHub CLI..."
  gh release create "v$VERSION" --title "Release v$VERSION" --draft
  echo "✅ GitHub draft release v$VERSION created!"
  echo "👉 Please add release notes at: https://github.com/$(git remote get-url origin | sed 's/.*github.com[\/:]\(.*\)\.git/\1/')/releases/"
  echo "👉 Remember to publish the release after adding your notes!"
else
  echo "⚠️ GitHub CLI not found. Create your release manually."
  echo "👉 Please create a release manually at: https://github.com/$(git remote get-url origin | sed 's/.*github.com[\/:]\(.*\)\.git/\1/')/releases/new?tag=v$VERSION"
  echo "👉 Suggested release notes from v0.1.4:"
  echo "- All return types are validated (defensive programming)"
  echo "- Full docstrings for each method"
  echo "- Automatic Rust docstring inheritance"
  echo "- Consistent formatting and type hints"
  echo "- Fully public library-ready standard"
fi

echo "✅ Release v$VERSION created and pushed!"
