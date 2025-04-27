"""
Async utilities for gradelib.

This module provides utilities for handling asynchronous operations in
synchronous contexts like Flask routes.
"""

import asyncio
import functools
from typing import Callable, Any
import sys
import inspect

# Import nest_asyncio for handling nested event loops
import nest_asyncio
nest_asyncio.apply()

def async_handler(func: Callable) -> Callable:
    """
    Decorator to handle async functions in synchronous contexts.
    
    This is particularly useful for using async functions from gradelib in
    Flask routes or other synchronous contexts. It handles event loop management
    and ensures that async functions can be called safely from any thread.
    
    Args:
        func: The async function to be wrapped
        
    Returns:
        A synchronous function that can be used in synchronous contexts
    
    Example:
        ```python
        from flask import Flask
        from gradelib import GitHubOAuthClient, async_handler
        
        app = Flask(__name__)
        
        @app.route('/callback')
        def callback():
            code = request.args.get('code')
            token = exchange_token(code)
            # ...rest of the handler
        
        @async_handler
        async def exchange_token(code):
            return await GitHubOAuthClient.exchange_code_for_token(
                client_id="...",
                client_secret="...",
                code=code,
                redirect_uri="..."
            )
        ```
    """
    # Use a flag in function attributes to detect decorated functions
    func.__is_async_handler__ = True
    
    @functools.wraps(func)
    def wrapper(*args: Any, **kwargs: Any) -> Any:
        # The key insight: don't try to detect if we're in a coroutine
        # Instead, look at the current call stack
        try:
            # Create the coroutine object
            coro = func(*args, **kwargs)
            
            # Get the current frame and its caller
            current_frame = inspect.currentframe()
            caller_frame = current_frame.f_back if current_frame else None
            
            if caller_frame:
                # Check if our caller is in an async context (is awaiting something)
                caller_code = caller_frame.f_code
                caller_func_name = caller_code.co_name
                
                # If we're being called within an async function or coroutine
                # we should return the coroutine for awaiting
                if (asyncio.iscoroutinefunction(caller_code) or 
                    caller_func_name.startswith('_async_') or
                    caller_func_name == 'coro' or
                    caller_func_name == 'wrapper'):
                    
                    # Just return the coroutine for awaiting
                    return coro
            
            # If we reach here, we're being called from synchronous code
            # Get an event loop and run the coroutine
            try:
                loop = asyncio.get_event_loop()
            except RuntimeError:
                # If no loop exists, create one
                loop = asyncio.new_event_loop()
                asyncio.set_event_loop(loop)
            
            # Run the coroutine
            return loop.run_until_complete(coro)
            
        except Exception as e:
            # Get the name of the function for better error messages
            func_name = getattr(func, "__name__", repr(func))
            error_msg = f"Error in async_handler for {func_name}: {str(e)}"
            
            # Print to stderr for logging
            print(error_msg, file=sys.stderr)
            
            # Re-raise the exception with the original traceback
            raise
    
    # Mark the wrapper as an async_handler wrapper
    wrapper.__is_async_handler_wrapper__ = True
    return wrapper
