"""
Tests for the async_handler utility.

This module tests that the async_handler decorator correctly allows
async functions to be called from synchronous contexts.
"""

import asyncio
import pytest
from gradelib import setup_async, async_handler

# Initialize the async runtime
setup_async()

# Test async_handler with a simple async function
@async_handler
async def async_add(a, b):
    await asyncio.sleep(0.1)  # Simulate async operation
    return a + b

# Test async_handler with an async function that raises an exception
@async_handler
async def async_error():
    await asyncio.sleep(0.1)  # Simulate async operation
    raise ValueError("Test error")

def test_async_handler_success():
    """Test that async_handler correctly runs an async function and returns its result."""
    result = async_add(2, 3)
    assert result == 5

def test_async_handler_error():
    """Test that async_handler correctly propagates exceptions from the async function."""
    with pytest.raises(ValueError, match="Test error"):
        async_error()

def test_nested_async_handlers():
    """Test that nested async_handlers work correctly."""
    @async_handler
    async def outer():
        # Call another async_handler function from within an async function
        return async_add(5, 7)
    
    result = outer()
    assert result == 12

def test_async_handler_in_thread():
    """Test that async_handler works correctly when called from a different thread."""
    import threading
    import queue
    
    result_queue = queue.Queue()
    
    def thread_func():
        try:
            # Call the async function from a different thread
            result = async_add(10, 20)
            result_queue.put(("success", result))
        except Exception as e:
            result_queue.put(("error", str(e)))
    
    thread = threading.Thread(target=thread_func)
    thread.start()
    thread.join()
    
    status, result = result_queue.get()
    assert status == "success"
    assert result == 30

if __name__ == "__main__":
    # Run the tests manually if called directly
    test_async_handler_success()
    try:
        test_async_handler_error()
        assert False, "Expected test_async_handler_error to raise an exception"
    except ValueError:
        pass
    test_nested_async_handlers()
    test_async_handler_in_thread()
    print("All tests passed!")
