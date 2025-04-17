"""
Tests for basic functions in the gradelib module
"""
import unittest
import sys
from pathlib import Path

# Add project root to PYTHONPATH
PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from gradelib import setup_async

class BasicFunctionsTest(unittest.TestCase):
    """Test basic functions in the gradelib module"""
    
    def test_setup_async(self):
        """Test the setup_async function which initializes the async runtime"""
        # This function doesn't return anything, just ensure it doesn't raise an exception
        try:
            setup_async()
            self.assertTrue(True)  # If we got here, the function didn't raise an exception
        except Exception as e:
            self.fail(f"setup_async() raised {type(e).__name__} unexpectedly: {e}")

if __name__ == "__main__":
    unittest.main()
