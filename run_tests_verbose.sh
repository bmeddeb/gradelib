#!/bin/bash
# Verbose Python test script for GradeLib
# Provides detailed test output with coverage reports

set -e  # Exit on any error

echo "=== GradeLib Python Tests (Verbose Mode) ==="
echo ""

# Set working directory
cd "$(dirname "$0")"

echo "=== Running Python tests with verbose output and coverage ==="
python -m pytest -v --cov=gradelib --cov-report=term-missing --cov-report=html

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
