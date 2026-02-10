# FEdit - Exact File Edit Toolkit

FEdit is a POSIX-focused Rust/Python hybrid tool for structured search-and-replace operations. It provides safe, atomic file edits with exact-match search-and-replace while preserving file integrity, line endings, and encodings.

## Features

- **Exact-match replacement** - Single or multiple occurrences
- **Atomic file writes** - Safe updates via temp file + rename
- **Encoding support** - UTF-8, UTF-16, ISO-8859-1, Windows-1252
- **Line ending preservation** - Auto-detects and preserves LF/CRLF
- **Whitespace-insensitive search** - Match regardless of whitespace variations
- **Dry run mode** - Preview changes without modifying files
- **Structured key mode** - JSON/YAML/TOML key-path replacement

## Installation

### From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/your-org/fedit.git
cd fedit

# Install with pip (requires maturin)
pip install maturin
maturin develop

# Or using uv
uv pip install -e .
```

### Pre-built Binaries

Pre-built binaries for Linux (x86_64, aarch64) and macOS (x86_64, arm64) are available as GitHub release assets with SHA256 checksums. See [Releases](https://github.com/your-org/fedit/releases).

### Python Wheel

```bash
pip install fedit  # (when published to PyPI)
```

## Usage

### Basic Search and Replace

```bash
# Replace single occurrence
fedit <file> <search> <replace>

# Replace all occurrences
fedit <file> <search> <replace> --multiple

# Preview changes without modifying file
fedit <file> <search> <replace> --dry-run
```

### CLI Options

| Option | Description |
|--------|-------------|
| `-m, --multiple` | Replace all occurrences (default: single match only) |
| `-n, --dry-run` | Preview changes without modifying the file |
| `-w, --ignore-whitespace` | Whitespace-insensitive search |
| `-e, --encoding` | File encoding: `utf-8`, `utf-16`, `iso-8859-1`, `windows-1252` |
| `-s, --structured` | Structured mode for key-path matching |

### Examples

```bash
# Basic replacement
python bin/fedit.py config.txt "old_value" "new_value"

# Replace all occurrences
python bin/fedit.py data.txt "foo" "bar" --multiple

# Dry run to preview changes
python bin/fedit.py script.sh "localhost" "127.0.0.1" --dry-run

# Whitespace-insensitive search
python bin/fedit.py code.py "def  hello" "def greet" -w

# Use specific encoding
python bin/fedit.py legacy.txt "text" "content" -e windows-1252
```

### Structured Mode

FEdit supports structured key-path replacement for JSON, YAML, and TOML files.

#### JSON

```bash
# Replace a nested JSON value
python bin/fedit_structured_json.py -s config.json "server.port" "8080"

# Replace array element
python bin/fedit_structured_json.py -s data.json "users[0].name" '"Alice"'
```

#### YAML/TOML (Experimental)

```bash
# YAML replacement
python bin/fedit_structured_yaml_toml.py config.yaml "database.host" "localhost"

# TOML replacement with explicit format
python bin/fedit_structured_yaml_toml.py -f toml settings.toml "server.port" "3000"
```

## Building from Source

### Prerequisites

- **Rust** 1.82+ (Edition 2024)
- **Python** 3.9 - 3.12
- **Maturin** >= 1.0
- **C compiler/linker** (required for Rust builds)

```bash
# Ubuntu/Debian
sudo apt update && sudo apt install build-essential

# Fedora
sudo dnf install gcc

# Arch Linux
sudo pacman -S base-devel

# macOS (install Xcode Command Line Tools)
xcode-select --install
```

### Build Commands

```bash
# Build Rust library only
cargo build --release

# Build and install Python package in development mode
pip install maturin
maturin develop

# Build distributable wheel
maturin build --release

# Run tests (currently manual)
cargo test  # Rust tests (if any)
```

### Running Without Building

The Python CLI can be run directly without compiling the Rust extension:

```bash
python bin/fedit.py <path> <search> <replace>
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | No matches found / Multiple matches without `--multiple` |
| 2 | File not found / Invalid arguments |
| 3 | Write error |
| 4 | Encoding error |

## Project Structure

```
fedit/
├── bin/                    # CLI scripts
│   ├── fedit.py            # Main CLI
│   ├── fedit_structured_json.py
│   └── fedit_structured_yaml_toml.py
├── src/
│   ├── lib.rs              # Rust library with PyO3 bindings
│   ├── api.rs              # Core replacement engine
│   └── fedit/              # Python package
├── Cargo.toml              # Rust configuration
├── pyproject.toml          # Python configuration
└── README.md
```

## Manpage

A generated manpage for the fedit CLI is produced via the `bin/gen_man.sh` script and included in Linux/macOS packages. After installation, run:

```bash
man fedit
```

## Contributing

See [AGENTS.md](AGENTS.md) for development guidelines and feature tracking.

## License

See LICENSE file for details.
