
<p align="center">
  <img src="assets/gradelib_e.png" alt="GradeLib Logo" width="200"/>
</p>

<h1 align="center">GradeLib</h1>

<p align="center">
  <strong>High-performance GitHub & Taiga analysis for grading software projects</strong><br>
  <em>Rust-powered backend, Python-friendly frontend</em>
</p>

<p align="center">
  <a href="https://pypi.org/project/gradelib/"><img src="https://img.shields.io/pypi/v/gradelib.svg" alt="PyPI Version"></a>
  <a href="https://github.com/bmeddeb/gradelib/actions"><img src="https://github.com/bmeddeb/gradelib/actions/workflows/ci.yml/badge.svg" alt="CI Status"></a>
  <img src="https://img.shields.io/badge/Built%20with-Rust%20%26%20Python-orange.svg" alt="Built with Rust and Python">
  <a href="https://github.com/bmeddeb/gradelib/blob/main/LICENSE"><img src="https://img.shields.io/github/license/bmeddeb/gradelib" alt="License"></a>
</p>

<p align="center">
  📚 <a href="https://bmeddeb.github.io/gradelib/">View Full Usage Guide</a>
</p>

---

## ⚙️ Installation

Install `gradelib` using either `pip` or [`uv`](https://github.com/astral-sh/uv):

```bash
# pip
pip install gradelib

# uv (recommended)
uv pip install gradelib

```

---

## 🚀 Quickstart

### Setup

```python
from gradelib.gradelib import setup_async, RepoManager
import os

setup_async()

manager = RepoManager(
    urls=["https://github.com/username/project"],
    github_username=os.getenv("GITHUB_USERNAME"),
    github_token=os.getenv("GITHUB_TOKEN")
)
```

### Clone & Analyze

```python
await manager.clone_all()
commits = await manager.analyze_commits("https://github.com/username/project")
for c in commits:
    print(c["author_name"], c["message"])
```

---

## 🧠 Features

- 🚀 Asynchronous repository cloning & analysis
- 📈 Commit history, blame, and contributor stats
- 🌿 Branch, issue, and pull request analytics
- 🔍 Taiga project integration with async API support
- 📊 Pandas-ready outputs for grading dashboards

---

## 🧪 Testing

GradeLib includes comprehensive test coverage for both Rust and Python components:

### Running Rust Tests

```bash
# Run all tests
cargo test

# Run specific test files
cargo test --test test_repo_manager
cargo test --test test_blame
cargo test --test test_commits
```

### Running Python Tests

```bash
# Run all tests
pytest

# Run with verbose output
pytest -v
```

For more detailed testing information, see [TESTING.md](TESTING.md).

---

## 🛠 GitHub Actions CI

Add this workflow to `.github/workflows/ci.yml` to test and build the Rust/Python hybrid:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - uses: dtolnay/rust-toolchain@stable

      - name: Install maturin
        run: pip install maturin

      - name: Build with maturin
        run: maturin develop

      - name: Run tests
        run: pytest
        
      - name: Run Rust tests
        run: cargo test
```

---

## 📘 Documentation Deployment (Optional)

Want to host documentation with GitHub Pages? Use `mkdocs`:

### 1. Install

```bash
pip install mkdocs mkdocs-material
```

### 2. Create docs

```bash
mkdocs new .
```

Place your markdown docs inside the `docs/` folder.

### 3. Deploy

```bash
mkdocs gh-deploy
```

Set your GitHub Pages source to the `gh-pages` branch.

---

## 📄 License

This project is licensed under the [MIT License](https://github.com/bmeddeb/gradelib/blob/main/LICENSE).

---

_Developed and maintained by [@bmeddeb](https://github.com/bmeddeb) — contributions are welcome!_
