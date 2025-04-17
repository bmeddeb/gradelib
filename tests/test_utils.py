"""
Utility functions and classes for testing gradelib
"""
from gradelib import setup_async
import os
import sys
import unittest
import asyncio
import tempfile
from pathlib import Path

# Add project root to PYTHONPATH
PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))


# Test repositories for use in tests
TEST_REPOS = {
    "example1": "https://github.com/bmeddeb/gradelib",
    "example2": "https://github.com/bmeddeb/SER402-Team3",
    "example3": "https://github.com/amehlhase316/survivors-spring24C"
}

# Test files for blame operations
TEST_FILES = [
    "README.md",
    "Cargo.toml",
    "pyproject.toml"
]


class AsyncTestCase(unittest.TestCase):
    """Base class for async test cases"""

    def setUp(self):
        """Set up the test case"""
        # Create a new event loop for each test
        self.loop = asyncio.new_event_loop()
        asyncio.set_event_loop(self.loop)

    def tearDown(self):
        """Tear down the test case, cleaning up resources"""
        self.loop.run_until_complete(asyncio.sleep(0))
        self.loop.close()
        asyncio.set_event_loop(None)

    def run_async(self, coro):
        """Run a coroutine in the event loop"""
        return self.loop.run_until_complete(coro)
