#!/usr/bin/env python3
"""
Example demonstrating the bulk_blame method of GitHubProvider.

This script shows how to analyze file blame information to track
ownership and changes to code lines across multiple files.

Usage:
    python test_bulk_blame.py [REPO_URL] [FILE_PATH...]

Environment variables:
    GITHUB_USERNAME: GitHub username for authentication
    GITHUB_TOKEN: GitHub personal access token for authentication
"""
import os
import sys
import asyncio
from pathlib import Path
from collections import Counter, defaultdict

# Add project root to PYTHONPATH
PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from gradelib import setup_async
from github import GitHubProvider

# Default test repository and files
DEFAULT_REPO = "https://github.com/bmeddeb/gradelib"
DEFAULT_FILES = ["README.md", "Cargo.toml", "pyproject.toml"]

async def main():
    """Demonstrate bulk blame with GitHubProvider"""
    # Get repo URL and file paths from command line if provided
    args = sys.argv[1:]
    if args:
        repo_url = args[0]
        file_paths = args[1:] if len(args) > 1 else DEFAULT_FILES
    else:
        repo_url = DEFAULT_REPO
        file_paths = DEFAULT_FILES
    
    # Initialize the async runtime
    setup_async()
    
    # Get GitHub credentials from environment variables
    username = os.environ.get("GITHUB_USERNAME")
    token = os.environ.get("GITHUB_TOKEN")
    
    if not username or not token:
        print("⚠️ Environment variables GITHUB_USERNAME and GITHUB_TOKEN are required")
        print("Using placeholder values for demo purposes - API calls will fail")
        username = "demo_user"
        token = "demo_token"
    
    # Create a provider and clone the repository
    provider = GitHubProvider(username=username, token=token)
    print(f"✓ Created GitHubProvider with username: {username}")
    
    # Clone the repository first
    print(f"\nCloning repository: {repo_url}")
    try:
        await provider.clone(repo_url)
        print(f"✓ Successfully cloned {repo_url}")
    except Exception as e:
        print(f"✗ Failed to clone repository: {e}")
        return
    
    # Perform bulk blame
    print(f"\nAnalyzing blame for {len(file_paths)} files:")
    for file_path in file_paths:
        print(f"  - {file_path}")
    
    try:
        blame_results = await provider.bulk_blame(repo_url, file_paths)
        print(f"✓ Completed blame analysis for {len(blame_results)} files")
        
        # File-level overview
        print("\nFile-level blame overview:")
        
        # Collect author statistics across all files
        author_lines = defaultdict(int)
        author_files = defaultdict(set)
        file_stats = {}
        
        # Process each file
        for file_path, blame_data in blame_results.items():
            if isinstance(blame_data, list):
                # Collect author contributions for this file
                file_authors = Counter()
                
                for line in blame_data:
                    author = line['author_name']
                    file_authors[author] += 1
                    author_lines[author] += 1
                    author_files[author].add(file_path)
                
                # Save file stats
                total_lines = len(blame_data)
                top_author, top_count = file_authors.most_common(1)[0] if file_authors else ("N/A", 0)
                top_percentage = (top_count / total_lines) * 100 if total_lines > 0 else 0
                
                file_stats[file_path] = {
                    'total_lines': total_lines,
                    'contributors': len(file_authors),
                    'top_author': top_author,
                    'top_percentage': top_percentage
                }
                
                print(f"  {file_path}:")
                print(f"    {total_lines} lines, {len(file_authors)} contributors")
                print(f"    Primary author: {top_author} ({top_percentage:.1f}%)")
            else:
                # Error occurred for this file
                print(f"  {file_path}: Error - {blame_data}")
        
        # Overall statistics
        if author_lines:
            print("\nOverall contribution statistics:")
            
            # Sort by line count (highest first)
            sorted_authors = sorted(author_lines.items(), key=lambda x: x[1], reverse=True)
            total_lines = sum(author_lines.values())
            
            print(f"  Total lines across all files: {total_lines}")
            print(f"  Total contributors: {len(author_lines)}")
            
            # Show top contributors
            print("\nTop contributors:")
            for i, (author, lines) in enumerate(sorted_authors[:5], 1):
                percentage = (lines / total_lines) * 100
                files_touched = len(author_files[author])
                file_percentage = (files_touched / len(file_stats)) * 100
                
                print(f"  {i}. {author}:")
                print(f"     {lines} lines ({percentage:.1f}% of total)")
                print(f"     Touched {files_touched} files ({file_percentage:.1f}% of files)")
            
            # Detailed file analysis
            print("\nDetailed file analysis:")
            print("  File                  | Lines | Top Contributor    | % of File")
            print("  ----------------------|-------|-------------------|----------")
            
            for file_path, stats in file_stats.items():
                # Format for fixed-width display
                file_name = Path(file_path).name[:20].ljust(20)
                lines = str(stats['total_lines']).ljust(5)
                author = stats['top_author'][:17].ljust(17)
                percentage = f"{stats['top_percentage']:.1f}%".ljust(8)
                
                print(f"  {file_name} | {lines} | {author} | {percentage}")
        
    except Exception as e:
        print(f"✗ Failed to perform bulk blame: {e}")

if __name__ == "__main__":
    asyncio.run(main())
