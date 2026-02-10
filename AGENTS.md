# AGENTS.md - FEdit Development Guide

## Project Overview

**FEdit** is a Rust/Python tool for safe, atomic file editing with search-and-replace. It provides fuzzy text matching and structured editing for JSON/YAML/TOML files.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Python API                              │
│  fedit.edit()              fedit.edit_structured()          │
│  src/fedit/__init__.py                                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    PyO3 Bindings                            │
│  src/lib.rs                                                 │
│  - edit_fuzzy()           - edit_structured_file()          │
│  - EditResultWithDiff     - StructuredEditResult            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Rust Core                               │
│  src/api.rs              src/structured.rs                  │
│  - Text replacement      - JSON/JSONC/JSON5 editing         │
│  - Fuzzy matching        - TOML editing (toml_edit)         │
│  - Diff generation       - YAML editing (serde_yaml)        │
│  - BOM handling          - Key path parsing                 │
│  - Line ending detection                                    │
└─────────────────────────────────────────────────────────────┘
```

## Project Structure

```
fedit/
├── src/
│   ├── api.rs              # Core replacement engine (1000+ lines)
│   │                       # - replace_in_content()
│   │                       # - fuzzy_find_text(), normalize_for_fuzzy_match()
│   │                       # - generate_diff() (LCS-based)
│   │                       # - BOM handling, line ending detection
│   │
│   ├── lib.rs              # PyO3 Python bindings (550+ lines)
│   │                       # - edit(), edit_fuzzy()
│   │                       # - edit_structured_file(), edit_structured_string()
│   │                       # - All Py* wrapper classes
│   │
│   ├── structured.rs       # Structured file editing (800+ lines)
│   │                       # - JSON/JSONC/JSON5 via serde_json/json5
│   │                       # - TOML via toml_edit (preserves formatting)
│   │                       # - YAML via serde_yaml
│   │                       # - Key path parsing: "foo.bar[0].baz"
│   │
│   └── fedit/              # Python package
│       ├── __init__.py     # Simplified Python API
│       │                   # - edit(path, old, new)
│       │                   # - edit_structured(path, key, value)
│       ├── _core.pyi       # Type stubs for Rust bindings
│       └── py.typed        # PEP 561 marker
│
├── Cargo.toml              # Rust dependencies
├── pyproject.toml          # Python packaging (maturin)
└── README.md               # User documentation
```

## Key Features

### Text Editing (`api.rs`)
- **Fuzzy matching**: Normalizes smart quotes (`""''` → `"'`), Unicode dashes, special spaces
- **Atomic writes**: temp file + rename pattern
- **BOM handling**: Strips/preserves UTF-8 BOM
- **Line endings**: Auto-detects LF/CRLF, preserves on write
- **Diff generation**: LCS-based unified diff with line numbers

### Structured Editing (`structured.rs`)
- **JSON**: Standard JSON via `serde_json`
- **JSONC**: JSON with comments (strips comments before parsing)
- **JSON5**: Relaxed JSON via `json5` crate
- **TOML**: Via `toml_edit` (preserves formatting and comments)
- **YAML**: Via `serde_yaml`
- **Key paths**: `"server.port"`, `"users[0].name"`, `"config.items[2].value"`

## Python API

```python
import fedit

# Text replacement (fuzzy matching enabled)
result = fedit.edit("file.py", "old_text", "new_text")
result = fedit.edit("file.py", "old", "new", multiple=True)
result = fedit.edit("file.py", "old", "new", dry_run=True)

# Structured editing
result = fedit.edit_structured("config.json", "server.port", 8080)
result = fedit.edit_structured("config.yaml", "db.host", "localhost")
```

## CLI Usage

```bash
# Text mode
fedit file.txt "old" "new"
fedit file.txt "old" "new" -m        # Multiple
fedit file.txt "old" "new" -n -d     # Dry run with diff

# Structured mode
fedit config.json -s server.port 8080
fedit config.yaml -s database.host localhost
```

## Building & Testing

```bash
cargo build --release    # Build Rust
cargo test               # Run 40 tests
maturin develop          # Install Python package
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | No match / Multiple matches |
| 2 | File not found / Invalid args |

## Dependencies

**Rust** (`Cargo.toml`):
- `pyo3` - Python bindings
- `serde`, `serde_json` - JSON serialization
- `json5` - JSON5 parsing
- `toml_edit` - TOML with formatting preservation
- `serde_yaml` - YAML support
- `regex` - Whitespace normalization

**Python** (`pyproject.toml`):
- `maturin` - Build system for Rust extensions
