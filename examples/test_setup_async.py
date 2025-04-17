#!/usr/bin/env python3
"""
Example demonstrating the setup_async function.

This function initializes the asynchronous runtime environment
needed for provider operations in gradelib.

Usage:
    python test_setup_async.py
"""
import sys
from pathlib import Path

# Add project root to PYTHONPATH
PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from gradelib import setup_async

def main():
    """Demonstrate the setup_async function"""
    print("Initializing the asynchronous runtime environment...")
    setup_async()
    print("✓ Async runtime initialized successfully!")
    print("\nThis function must be called before using any async functionality in gradelib.")
    print("It configures the Tokio runtime in the Rust backend to handle asynchronous tasks.")

if __name__ == "__main__":
    main()
