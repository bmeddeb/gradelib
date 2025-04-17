#!/usr/bin/env python
"""
Run all tests for the gradelib package
"""
import os
import sys
import unittest
from pathlib import Path

# Add the project root to sys.path
PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))


def run_tests():
    """Run all tests in the tests directory"""
    print("Running all gradelib tests...\n")

    # Discover and run all tests in the tests directory
    test_loader = unittest.TestLoader()
    test_suite = test_loader.discover(start_dir=str(
        Path(__file__).parent), pattern='test_*.py')

    test_runner = unittest.TextTestRunner(verbosity=1)
    result = test_runner.run(test_suite)

    # Print a summary
    print("\nTest Summary:")
    print(f"  Ran {result.testsRun} tests")
    print(f"  Failures: {len(result.failures)}")
    print(f"  Errors: {len(result.errors)}")
    print(f"  Skipped: {len(result.skipped)}")

    # Return non-zero exit code if there were failures or errors
    return len(result.failures) + len(result.errors)


if __name__ == "__main__":
    sys.exit(run_tests())
