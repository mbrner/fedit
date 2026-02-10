# AGENTS.md - FEdit Development Guide

This document provides context for AI agents and developers working on the FEdit codebase.

## Project Overview

**FEdit** (Exact File Edit Toolkit) is a POSIX-focused Rust/Python hybrid tool for structured search-and-replace operations. It provides safe, atomic file edits with exact-match search-and-replace while preserving file integrity, line endings, and encodings.

## Architecture

FEdit uses a hybrid Rust/Python architecture:
- **Rust core** (`src/lib.rs`, `src/api.rs`): Core replacement engine with PyO3 Python bindings
- **Python CLI** (`bin/fedit.py`): Main command-line interface
- **Structured mode scripts**: Separate handlers for JSON and YAML/TOML formats

## Project Structure

```
fedit/
├── bin/                           # CLI scripts
│   ├── fedit.py                   # Main CLI entry point
│   ├── fedit_structured_json.py   # Structured JSON path replacement
│   ├── fedit_structured_yaml_toml.py  # Structured YAML/TOML replacement
│   └── gen_man.sh                 # Manpage generation script
├── src/
│   ├── lib.rs                     # Rust library with PyO3 module
│   ├── api.rs                     # Core Rust replacement engine
│   ├── bin/
│   │   └── fedit_man.rs           # Rust binary for manpage generation
│   └── fedit/                     # Python package
│       ├── __init__.py            # Python module entry point
│       ├── _core.pyi              # Type stubs for Rust bindings
│       └── py.typed               # PEP 561 type marker
├── tasks/                         # Product requirements documents
├── .github/workflows/             # CI/CD configuration
├── Cargo.toml                     # Rust project configuration
├── pyproject.toml                 # Python packaging configuration
└── README.md                      # User documentation
```

## Feature Tracking

### Implemented Features

- **US-001**: Single Exact-Match Replacement
- **US-002**: Multiple Match Replacement (`-m/--multiple`)
- **US-003**: Atomic File Write (temp file + rename)
- **US-004**: Encoding Support (`-e/--encoding`) - UTF-8, UTF-16, ISO-8859-1, Windows-1252
- **US-005**: Line Ending Preservation (LF/CRLF auto-detection)
- **US-006**: Dry Run Mode (`-n/--dry-run`)
- **US-007**: Whitespace-Insensitive Search (`-w/--ignore-whitespace`)
- **US-011**: Structured Key Mode - JSON (`-s/--structured`)
- **US-012**: Structured Key Mode - YAML/TOML (experimental)
- **US-014**: Python Wheel Packaging
- **US-015**: CLI Manpage Generation
- **US-016**: Cross-Platform Binary Distribution

### US-011: Structured Key Mode - JSON

This feature introduces structured key path mode for JSON files. When `-s/--structured` is supplied:
- The search string is treated as a JSON key path (supporting nested keys and array indices)
- The replace string becomes the new value for that path
- JSON formatting is preserved by applying changes via a parsed JSON tree
- Strict path resolution returns errors for invalid or ambiguous paths

**Example usage:**
```bash
python bin/fedit_structured_json.py -s config.json "settings.port" "8080"
python bin/fedit_structured_json.py -s data.json "items[0].name" '"new-name"'
```

### US-012: Structured Key Mode - YAML/TOML

Experimental support for YAML and TOML structured replacement:
- Uses regex-based line manipulation to preserve formatting/comments
- Detects format by file extension or `--format` flag

**Example usage:**
```bash
python bin/fedit_structured_yaml_toml.py config.yaml "database.host" "localhost"
python bin/fedit_structured_yaml_toml.py -f toml settings.toml "server.port" "3000"
```

## Build & Development

### Prerequisites

- **Rust**: Edition 2024 (nightly or recent stable)
- **Python**: 3.9 - 3.12
- **Maturin**: For building Python wheels with Rust extensions
- **C compiler/linker**: Required for Rust builds (`build-essential` on Ubuntu/Debian, `gcc` on Fedora, `base-devel` on Arch, Xcode CLI tools on macOS)

### Building

```bash
# Build Rust library only
cargo build --release

# Build Python wheel with maturin (recommended)
pip install maturin
maturin develop          # Install in development mode
maturin build            # Build wheel for distribution

# Using uv
uv pip install -e .
```

### Running Without Build

The Python CLI can be run directly without building the Rust extension:
```bash
python bin/fedit.py <path> <search> <replace>
```

## Testing

Currently, no formal test suite exists. Manual testing can be performed:

```bash
# Basic replacement
python bin/fedit.py test.txt "old text" "new text"

# Multiple replacements
python bin/fedit.py test.txt "old" "new" --multiple

# Dry run
python bin/fedit.py test.txt "search" "replace" --dry-run

# Whitespace-insensitive
python bin/fedit.py test.txt "hello  world" "goodbye" -w

# Structured JSON mode
python bin/fedit_structured_json.py -s config.json "settings.port" "8080"

# Structured YAML/TOML
python bin/fedit_structured_yaml_toml.py config.yaml "database.host" "localhost"
```

## CI/CD

GitHub Actions workflow (`.github/workflows/us-016-binary-distribution.yml`) builds cross-platform binaries on tag pushes (`v*`):
- Linux x86_64 and aarch64
- macOS x86_64 and arm64
- Generates SHA256 checksums for all binaries

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | No matches found / Multiple matches without `--multiple` |
| 2 | File not found / Invalid arguments |
| 3 | Write error |
| 4 | Encoding error |

## Future Work

- Expand test coverage with automated tests
- Complete PyO3 bindings to expose full Rust API to Python
- Add support for more structured formats
- Improve YAML/TOML structured mode robustness
