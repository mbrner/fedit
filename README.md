# FEdit - Safe File Editing

[![CI](https://github.com/mbrner/fedit/actions/workflows/ci.yml/badge.svg)](https://github.com/mbrner/fedit/actions/workflows/ci.yml)
[![Python 3.10-3.14](https://img.shields.io/badge/python-3.10%20%7C%203.11%20%7C%203.12%20%7C%203.13%20%7C%203.14-blue)](https://www.python.org)
[![Rust](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org)
[![Platform: Linux | macOS](https://img.shields.io/badge/platform-Linux%20%7C%20macOS-lightgrey)](https://github.com/mbrner/fedit)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)

FEdit is a Rust/Python tool for safe, atomic file edits. It provides exact-match search-and-replace with automatic line ending and encoding preservation.

## Features

- **Atomic writes** - Changes written via temp file + rename (no partial writes)
- **Fuzzy matching** - Handles smart quotes, Unicode dashes, trailing whitespace
- **Structured editing** - Edit JSON/YAML/TOML files by key path
- **"Did you mean" suggestions** - Typo-friendly key-not-found errors with edit-distance-based suggestions
- **Encoding support** - UTF-8, UTF-16, ISO-8859-1, Windows-1252
- **Line ending preservation** - Auto-detects and preserves LF/CRLF

## Installation

```bash
pip install maturin
maturin develop
```

## Python API

```python
import fedit

# Text replacement
fedit.edit("config.py", "DEBUG = True", "DEBUG = False")
fedit.edit("code.py", "old_name", "new_name", multiple=True)

# Structured editing (JSON/YAML/TOML)
fedit.edit_structured("config.json", "server.port", 8080)
fedit.edit_structured("config.yaml", "database.host", "localhost")
fedit.edit_structured("Cargo.toml", "package.version", '"2.0.0"')

# Typo in key path? Get a helpful suggestion:
# ValueError: Key not found: Key 'sever' not found. Did you mean 'server'?

# Preview changes
result = fedit.edit("file.txt", "old", "new", dry_run=True)
print(result.diff)
```

## CLI Usage

```bash
# Text replacement
fedit file.txt "old text" "new text"
fedit file.txt "old" "new" -m              # Replace all occurrences
fedit file.txt "old" "new" -n              # Dry run (preview)
fedit file.txt "old" "new" -d              # Show diff

# Structured editing
fedit config.json -s server.port 8080
fedit config.yaml -s database.host localhost
fedit settings.toml -s version '"1.0.0"' -f toml
```

### CLI Options

| Option | Description |
|--------|-------------|
| `-s, --structured` | Edit by key path (JSON/YAML/TOML) |
| `-f, --format` | Force file format (json, yaml, toml) |
| `-m, --multiple` | Replace all occurrences |
| `-n, --dry-run` | Preview without modifying |
| `-d, --diff` | Show unified diff |
| `-e, --encoding` | File encoding (default: utf-8) |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | No match found / Multiple matches |
| 2 | File not found / Invalid arguments |

## Building from Source

```bash
# Prerequisites: Rust 1.82+, Python 3.10-3.14, maturin

cargo build --release                           # Rust only
maturin develop                                 # Python package (dev mode)
maturin build --release                         # Build wheel
cargo test                                      # Run Rust tests (62 tests)
uv run --with pytest pytest tests/ -v          # Run Python tests (29 tests)
```

## Project Structure

```
fedit/
├── src/
│   ├── api.rs            # Core Rust engine
│   ├── lib.rs            # PyO3 bindings
│   ├── structured.rs     # JSON/YAML/TOML support
│   └── fedit/            # Python package
│       ├── __init__.py   # Python API
│       └── _core.pyi     # Type stubs
├── tests/
│   └── test_fedit.py     # Python tests
├── Cargo.toml
└── pyproject.toml
```

## License

See LICENSE file.
