"""
gradelib - Python extension module for analyzing Git repositories and Taiga projects.

This module provides classes and functions for working with Git repositories and Taiga projects.
"""

# Import core module
from .gradelib import *

# Try to explicitly import TaigaClient if it exists in the taiga submodule
try:
    from .gradelib import TaigaClient
except ImportError:
    try:
        from .taiga import TaigaClient
    except ImportError:
        import sys
        print(f"Warning: TaigaClient could not be imported. sys.path={sys.path}", file=sys.stderr) 