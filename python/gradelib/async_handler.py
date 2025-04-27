"""
Async utilities for gradelib.

This module provides utilities for handling asynchronous operations in
synchronous contexts like Flask routes.
"""

import asyncio
import functools
from typing import Callable, Any
import sys

# Import nest_asyncio for handling nested event loops
try:
    import nest_asyncio
    HAS_NEST_ASYNCIO = True
except ImportError:
    HAS_NEST_ASYNCIO = False

# When this module is imported, apply nest_asyncio if available
if HAS_NEST_ASYNCIO:
    try:
        nest_asyncio.apply()
    except Exception:
        # Log warning but don't fail if nest_asyncio.apply() fails
        import warnings
        warnings.warn("Failed to apply nest_asyncio. Nested event loops may not work properly.")


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
    @functools.wraps(func)
    def wrapper(*args: Any, **kwargs: Any) -> Any:
        try:
            # Try to get the current event loop
            try:
                loop = asyncio.get_event_loop()
            except RuntimeError:
                # If there is no event loop in this thread, create a new one
                loop = asyncio.new_event_loop()
                asyncio.set_event_loop(loop)
            
            # Run the async function in this thread's event loop
            return loop.run_until_complete(func(*args, **kwargs))
        except Exception as e:
            # Get the name of the function for better error messages
            func_name = getattr(func, "__name__", repr(func))
            error_msg = f"Error in async_handler for {func_name}: {str(e)}"
            
            # Print to stderr for logging
            print(error_msg, file=sys.stderr)
            
            # Re-raise the exception with the original traceback
            raise
    
    return wrapper
