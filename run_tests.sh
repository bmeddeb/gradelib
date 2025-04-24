#!/bin/bash
# Python test script for GradeLib
# Runs Python tests and generates coverage reports

set -e  # Exit on any error

echo "=== GradeLib Python Tests ==="
echo ""

# Check for required tools
command -v pytest >/dev/null 2>&1 || { echo "pytest is required but not installed. Run: pip install pytest pytest-cov"; exit 1; }

# Set working directory
cd "$(dirname "$0")"

echo "=== Running Python tests with coverage ==="
python -m pytest --cov=gradelib --cov-report=term --cov-report=html

echo ""
echo "=== Coverage Report ==="
echo "HTML coverage report generated in htmlcov/index.html"

# Optionally, open the coverage report
if [[ "$OSTYPE" == "darwin"* ]]; then
    open htmlcov/index.html
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if command -v xdg-open >/dev/null 2>&1; then
        xdg-open htmlcov/index.html
    fi
fi

echo ""
echo "=== Test Suite Complete ==="
