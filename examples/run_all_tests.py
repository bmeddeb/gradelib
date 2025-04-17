#!/usr/bin/env python3
"""
Run all unit tests for the gradelib module.

This script discovers and runs all test cases in the tests directory.

Usage:
    python run_all_tests.py
"""
import os
import sys
import unittest
from pathlib import Path

# Add project root to PYTHONPATH
PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

def main():
    """Discover and run all tests"""
    print("Running all gradelib tests...\n")
    
    # Define the test directory
    test_dir = os.path.join(PROJECT_ROOT, "examples", "tests")
    
    # Create test loader
    loader = unittest.TestLoader()
    
    # Discover tests in the tests directory
    test_suite = loader.discover(test_dir, pattern="test_*.py")
    
    # Create test runner
    runner = unittest.TextTestRunner(verbosity=2)
    
    # Run tests
    result = runner.run(test_suite)
    
    # Print summary
    print("\nTest Summary:")
    print(f"  Ran {result.testsRun} tests")
    print(f"  Failures: {len(result.failures)}")
    print(f"  Errors: {len(result.errors)}")
    print(f"  Skipped: {len(result.skipped)}")
    
    # Return non-zero exit code if tests failed
    return 0 if result.wasSuccessful() else 1

if __name__ == "__main__":
    sys.exit(main())
