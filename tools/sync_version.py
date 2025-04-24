#!/usr/bin/env python
"""
Version synchronization tool for GradeLib.

This script ensures that the version number is synchronized between 
Cargo.toml and pyproject.toml.
"""

import os
import sys
import re
import toml
import argparse
from pathlib import Path

def main():
    parser = argparse.ArgumentParser(description='Synchronize version between Cargo.toml and pyproject.toml')
    parser.add_argument('--version', type=str, help='Set specific version (e.g., 0.1.2)')
    parser.add_argument('--bump', choices=['major', 'minor', 'patch'], 
                        help='Bump version (major, minor, or patch)')
    args = parser.parse_args()
    
    # Find project root (directory containing Cargo.toml and pyproject.toml)
    project_root = Path(__file__).parent.parent.absolute()
    cargo_path = project_root / 'Cargo.toml'
    pyproject_path = project_root / 'pyproject.toml'
    
    # Ensure both files exist
    if not cargo_path.exists() or not pyproject_path.exists():
        print(f"Error: Could not find Cargo.toml or pyproject.toml in {project_root}")
        return 1
    
    # Read current versions
    cargo_data = toml.load(cargo_path)
    cargo_version = cargo_data['package']['version']
    
    try:
        pyproject_data = toml.load(pyproject_path)
        python_version = pyproject_data['project']['version']
    except (KeyError, toml.TomlDecodeError):
        print("Warning: Could not extract version from pyproject.toml. Using Cargo.toml version.")
        python_version = cargo_version
    
    print(f"Current versions: Cargo.toml={cargo_version}, pyproject.toml={python_version}")
    
    # Determine the new version
    if args.version:
        new_version = args.version
    elif args.bump:
        # Parse current version
        major, minor, patch = map(int, cargo_version.split('.'))
        
        if args.bump == 'major':
            major += 1
            minor = 0
            patch = 0
        elif args.bump == 'minor':
            minor += 1
            patch = 0
        elif args.bump == 'patch':
            patch += 1
        
        new_version = f"{major}.{minor}.{patch}"
    else:
        # If no version specified, use Cargo.toml version as reference
        new_version = cargo_version
    
    print(f"Setting version to: {new_version}")
    
    # Update Cargo.toml
    cargo_data['package']['version'] = new_version
    with open(cargo_path, 'w') as f:
        toml.dump(cargo_data, f)
    
    # Update pyproject.toml
    try:
        pyproject_data['project']['version'] = new_version
        with open(pyproject_path, 'w') as f:
            toml.dump(pyproject_data, f)
    except KeyError:
        print("Warning: Could not update pyproject.toml version field (missing structure).")
    
    print(f"Version synchronized to {new_version} in both files.")
    return 0

if __name__ == "__main__":
    sys.exit(main())
