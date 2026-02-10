# FEdit – Exact File Edit Toolkit

## 1. Introduction/Overview

FEdit is a POSIX-focused Rust library and CLI tool that enables robust, safe file edits via exact-match search-and-replace operations. It addresses the common problem of making precise, predictable text modifications while preserving file integrity, line endings, and encodings.

The tool supports exact-match and structured-key search modes with optional whitespace and Unicode normalization. Python bindings are provided via maturin as the `fedit` package. The core is written in Rust for performance and safety.

**CLI Syntax:**
```
fedit <target> <search-str> <replace-str> [options]
```

## 2. Goals

- Provide a safe, predictable edit tool that requires exactly one match by default (preventing accidental mass changes)
- Preserve file integrity including line endings, encodings, and overall file structure
- Support exact-match and structured-key search modes, with optional whitespace/Unicode normalization
- Deliver atomic write operations to prevent file corruption
- Detect and reject binary files to prevent accidental corruption
- Enable operation on POSIX systems (Linux and macOS)
- Provide both a Rust library API and a CLI interface
- Offer Python bindings via maturin (`fedit` package) for ecosystem integration
- Maintain up-to-date AGENTS.md and README.md documentation

## 3. Quality Gates

These commands must pass for every user story:

- `cargo build --release` - Successful build
- `cargo test` - All tests pass
- `cargo clippy -- -D warnings` - No linter warnings
- `cargo fmt --check` - Code formatting check

For Python binding stories, also include:
- `maturin develop` - Python bindings build successfully
- `pytest` - Python binding tests pass

## 4. User Stories

### US-001: Single Exact-Match Replacement

**Description:** As a developer, I want to replace exactly one occurrence of a search string in a file so that I can make precise edits without accidentally modifying multiple locations.

**Acceptance Criteria:**
- [ ] CLI accepts positional arguments: `fedit <target> <search-str> <replace-str>`
- [ ] When exactly one match exists, the replacement is performed
- [ ] When zero matches exist, an error message states "No matches found for: [search-str]"
- [ ] When multiple matches exist, an error message states "Multiple matches found ([count]); use --multiple to replace all"
- [ ] Original file is unchanged when an error occurs
- [ ] Exit code is 0 on success, non-zero on error
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with usage example

### US-002: Multiple Match Replacement Mode

**Description:** As a developer, I want to optionally replace all occurrences of a search string so that I can perform bulk replacements when intended.

**Acceptance Criteria:**
- [ ] CLI accepts `-m` or `--multiple` flag to enable multiple replacements
- [ ] When `-m` is provided, all occurrences are replaced
- [ ] Output displays the count of replacements made
- [ ] When `-m` is provided and zero matches exist, an error message is displayed
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with usage example

### US-003: Atomic File Write

**Description:** As a developer, I want file writes to be atomic so that power failures or crashes don't leave files in a corrupted state.

**Acceptance Criteria:**
- [ ] Replacement writes to a temporary file first
- [ ] Temporary file is flushed to disk before replacing the original
- [ ] Original file is atomically replaced using rename operation
- [ ] If the write fails, the original file remains unchanged
- [ ] Temporary files are cleaned up on both success and failure
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with safety guarantees section

### US-003b: Binary File Detection

**Description:** As a developer, I want the tool to detect and reject binary files so that I don't accidentally corrupt non-text files.

**Acceptance Criteria:**
- [ ] Tool scans first 8KB of file for null bytes
- [ ] If null bytes are found, operation is aborted with error "Binary file detected: [filename]"
- [ ] Error message suggests using `--force` flag to override (flag not implemented in v1.0)
- [ ] Exit code is non-zero when binary file is detected
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with binary file handling documentation

### US-003c: Large File Warning

**Description:** As a developer, I want to be warned when editing large files so that I'm aware of potential memory usage.

**Acceptance Criteria:**
- [ ] Files larger than 5MB trigger a warning to stderr
- [ ] Warning states: "Warning: Large file ([size]MB). Memory usage will be proportional to file size."
- [ ] Operation proceeds after warning (no confirmation required)
- [ ] Warning can be suppressed with `--quiet` or `-q` flag
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with large file handling documentation

### US-004: Encoding Support

**Description:** As a developer, I want to specify file encoding so that I can edit files that aren't UTF-8.

**Acceptance Criteria:**
- [ ] CLI accepts `--encoding` or `-e` argument
- [ ] Default encoding is UTF-8 when not specified
- [ ] Common encodings are supported (UTF-8, UTF-16, ISO-8859-1)
- [ ] Clear error message when encoding cannot decode the file
- [ ] Output file uses the same encoding as the input
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with encoding documentation

### US-005: Line Ending Preservation

**Description:** As a developer, I want line endings to be preserved so that files maintain their format.

**Acceptance Criteria:**
- [ ] Tool detects the dominant line ending style (LF or CRLF) in the file
- [ ] All line endings in the output match the detected dominant style
- [ ] Replacement strings containing `\n` are converted to the file's line ending style
- [ ] Files with no line endings are written without adding any
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with line ending behavior

### US-006: Dry Run Mode

**Description:** As a developer, I want to preview changes without modifying the file so that I can verify the edit before committing.

**Acceptance Criteria:**
- [ ] CLI accepts `--dry-run` or `-n` flag
- [ ] When `--dry-run` is provided, no file modifications occur
- [ ] Output shows what would be changed (before/after preview)
- [ ] Output indicates the line number(s) where changes would occur
- [ ] Exit code reflects whether the operation would succeed
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with dry-run example

### US-007: CLI Help and Usage

**Description:** As a user, I want comprehensive help text so that I can understand all available options and see usage examples.

**Acceptance Criteria:**
- [ ] `fedit --help` displays all available flags and arguments
- [ ] Each flag includes a description of its purpose
- [ ] Usage examples are provided for common scenarios
- [ ] Help text fits within 80-column terminal width
- [ ] Short and long flag variants are documented (e.g., `-m` and `--multiple`)
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with complete CLI reference

### US-008: Rust Library API - Core Function

**Description:** As a Rust developer, I want a library function to perform replacements so that I can integrate FEdit into my applications.

**Acceptance Criteria:**
- [ ] Public function accepts file content as string and replacement parameters
- [ ] Function returns `Result<EditResult, EditError>`
- [ ] `EditResult` contains the modified content and count of replacements
- [ ] Options struct allows enabling/disabling multiple replacements
- [ ] Function is documented with rustdoc including examples
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with library usage example

### US-009: Error Types and Handling

**Description:** As a Rust developer, I want well-defined error types so that I can handle different failure modes appropriately.

**Acceptance Criteria:**
- [ ] `EditError::NotFound` is returned when search string has no matches
- [ ] `EditError::MultipleFound(count)` is returned when uniqueness is required but multiple matches exist
- [ ] `EditError::IoError` wraps underlying I/O errors with context
- [ ] `EditError::EncodingError` is returned for encoding issues
- [ ] `EditError::InvalidKeyPath(path)` is returned for invalid structured mode paths
- [ ] `EditError::KeyNotFound(path)` is returned when structured mode path doesn't exist in document
- [ ] All error types implement `std::error::Error` and `Display`
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with error handling documentation

### US-010: Whitespace-Insensitive Search

**Description:** As a developer, I want to match text regardless of whitespace differences so that I can find content even when spacing varies.

**Acceptance Criteria:**
- [ ] CLI accepts `--ignore-whitespace` or `-w` flag
- [ ] When enabled, consecutive whitespace in search matches any whitespace sequence in file
- [ ] Original whitespace in the file is preserved after replacement
- [ ] Position mapping tracks original byte offsets for accurate replacement
- [ ] Works correctly with multi-line search strings
- [ ] Whitespace flag is ignored when structured mode (`-s`) is active (key paths are exact)
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with whitespace-insensitive search example

### US-010b: Unicode Normalization for Search

**Description:** As a developer, I want Unicode-normalized matching so that visually identical text matches regardless of Unicode representation.

**Acceptance Criteria:**
- [ ] CLI accepts `--normalize` flag with optional value (NFC, NFD, NFKC, NFKD)
- [ ] Default normalization form is NFC when flag is provided without value
- [ ] Search string and file content are normalized for comparison only
- [ ] Original Unicode representation in the file is preserved after replacement
- [ ] Combining characters and precomposed forms match when normalized
- [ ] Normalization flags are ignored when structured mode (`-s`) is active (key paths are exact)
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with Unicode normalization documentation

### US-011: Structured Key Mode - JSON

**Description:** As a developer, I want to specify a JSON path to locate and replace a value so that I can edit structured data safely.

**Acceptance Criteria:**
- [ ] CLI accepts `--structured` or `-s` flag to enable structured-key mode
- [ ] In structured mode, `<search-str>` is interpreted as a key path (e.g., `root.child.field`)
- [ ] In structured mode, `<replace-str>` is the new value for the resolved path
- [ ] Path syntax supports nested keys: `config.database.host`
- [ ] Path syntax supports array indices: `items[0].name`
- [ ] Replacement updates only the resolved value's content, not its quotes or structure
- [ ] JSON formatting (indentation, spacing) is preserved via string manipulation
- [ ] Error `InvalidKeyPath` is returned for paths that don't exist
- [ ] Error `AmbiguousPath` is returned if path matches multiple locations (malformed JSON)
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with structured mode examples showing `fedit -s file.json "config.port" "8080"`

### US-012: Structured Key Mode - YAML and TOML

**Description:** As a developer, I want structured-key mode to work with YAML and TOML files so that I can edit configuration files safely.

**Acceptance Criteria:**
- [ ] YAML files are detected by extension (.yml, .yaml) or `--format yaml` flag
- [ ] TOML files are detected by extension (.toml) or `--format toml` flag
- [ ] Same path syntax works across all supported formats
- [ ] Replacement uses string manipulation (not parse-serialize) to preserve formatting
- [ ] Comments are preserved (best-effort: may fail on complex nested structures)
- [ ] If formatting cannot be preserved, operation fails with clear error message
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with YAML/TOML examples and limitations

### US-013: Python Bindings via Maturin

**Description:** As a Python developer, I want to use FEdit from Python so that I can integrate it into Python-based tooling.

**Acceptance Criteria:**
- [ ] Python bindings are built using maturin
- [ ] `fedit.replace_exact(content, search, replace)` function exists
- [ ] `fedit.replace_exact(content, search, replace, allow_multiple=True)` works
- [ ] Python exceptions mirror Rust error types (`NotFoundError`, `MultipleFoundError`, etc.)
- [ ] Function accepts and returns Python strings
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with Python usage examples

### US-014: Python Wheel Packaging

**Description:** As a Python developer, I want to install FEdit via pip so that I can easily add it to my projects.

**Acceptance Criteria:**
- [ ] Wheels are built for Linux (manylinux2014) x86_64 and aarch64
- [ ] Wheels are built for macOS x86_64 and aarch64 (universal2)
- [ ] Wheels support Python 3.9, 3.10, 3.11, and 3.12
- [ ] `pip install fedit` installs the package successfully
- [ ] Package exposes type information via `py.typed` marker and inline annotations
- [ ] README is included in package metadata via pyproject.toml
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with pip installation instructions

### US-015: CLI Manpage Generation

**Description:** As a Unix user, I want a manpage so that I can access documentation via the standard `man` command.

**Acceptance Criteria:**
- [ ] Manpage is generated from CLI definition (via clap_mangen or similar)
- [ ] Manpage includes all commands, flags, and options
- [ ] Manpage includes examples section
- [ ] Manpage is included in Linux/macOS packages
- [ ] `man fedit` displays the documentation after installation
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with manpage availability

### US-016: Cross-Platform Binary Distribution (POSIX)

**Description:** As a user, I want pre-built binaries for my platform so that I don't need a Rust toolchain to install FEdit.

**Acceptance Criteria:**
- [ ] Linux binaries are built for x86_64 and aarch64
- [ ] macOS binaries are built for x86_64 and aarch64 (Apple Silicon)
- [ ] Binaries are available as GitHub release assets
- [ ] SHA256 checksums are provided for all binaries
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with binary installation instructions

### US-017: Linux Package Distribution

**Description:** As a Linux user, I want to install FEdit via my package manager so that updates are managed automatically.

**Acceptance Criteria:**
- [ ] .deb package is built for Debian/Ubuntu
- [ ] .rpm package is built for Fedora/RHEL
- [ ] Packages include the binary, manpage, and shell completions
- [ ] Package metadata includes description, license, and homepage
- [ ] AGENTS.md is updated to reflect this feature
- [ ] README.md is updated with package manager instructions

## 5. Functional Requirements

- **FR-1:** The system must accept positional arguments in the form `fedit <target> <search-str> <replace-str>`.
- **FR-2:** The system must replace text in a file based on an exact string match.
- **FR-3:** The system must return an error when the search string matches zero occurrences.
- **FR-4:** The system must return an error when the search string matches more than one occurrence, unless multiple replacement mode is enabled via `-m`.
- **FR-5:** When multiple replacement mode is enabled (`-m`), the system must replace all occurrences.
- **FR-6:** The system must write changes atomically via a temporary file and rename operation.
- **FR-7:** The system must detect and preserve the dominant line ending style (LF or CRLF).
- **FR-8:** The system must preserve the original file's encoding, defaulting to UTF-8.
- **FR-9:** The system must support specifying an alternative encoding via `--encoding`.
- **FR-10:** The system must support dry-run mode (`-n`) that shows changes without modifying the file.
- **FR-11:** The system must support structured-key mode (`-s`) for JSON, YAML, and TOML files.
- **FR-12:** In structured-key mode, `<search-str>` is interpreted as a key path (e.g., `config.db.host`).
- **FR-13:** In structured-key mode, the system must support array indices (e.g., `items[0].name`).
- **FR-14:** The CLI must provide comprehensive help text via `--help`.
- **FR-15:** The Rust library must expose a public API for programmatic use.
- **FR-16:** The system must provide Python bindings built via maturin, published as `fedit` on PyPI.
- **FR-17:** The system must support whitespace-insensitive search via `-w` flag.
- **FR-18:** The system must support Unicode normalization for search via `--normalize` flag.
- **FR-19:** The system must detect binary files (via null byte check) and return an error.
- **FR-20:** The system must warn (to stderr) when processing files larger than 5MB.

## 6. Non-Goals (Out of Scope)

- **Windows support:** This version focuses on POSIX systems (Linux and macOS) only.
- **Regex support:** This version focuses on exact-match and structured-key modes only; regular expression matching is not included.
- **Fuzzy matching:** Deferred to future release; use whitespace-insensitive (`-w`) and Unicode normalization (`--normalize`) for flexible matching in v1.0.
- **In-place streaming for very large files:** Files are loaded into memory; streaming edits for multi-gigabyte files are not supported.
- **GUI interface:** Only CLI and library interfaces are provided.
- **Remote file editing:** The tool operates on local files only; network protocols (SSH, HTTP, etc.) are not supported.
- **Version control integration:** Automatic git commits or VCS awareness is not included.
- **Backup file creation:** Users are responsible for backups; the tool does not create `.bak` files.
- **Interactive mode:** The tool is non-interactive; all parameters must be specified upfront.
- **Configuration file:** All options are passed via CLI arguments; there is no `.feditrc` or similar config file.
- **Binary file editing:** Binary files are detected and rejected; use dedicated binary tools instead.

## 7. Technical Considerations

### AGENTS.md Structure

The AGENTS.md file provides context and instructions for AI coding agents. It must be updated after each user story is completed. Follow the standard AGENTS.md format:

```markdown
# AGENTS.md

## Project Overview
FEdit is a POSIX-focused Rust CLI and library for exact-match file editing.
Core in Rust, Python bindings via maturin/PyO3.

## Setup Commands
- Install Rust deps: `cargo build`
- Run tests: `cargo test`
- Build Python bindings: `maturin develop`
- Run Python tests: `pytest`

## Build Commands
- `cargo build --release` - Build optimized CLI binary
- `cargo clippy -- -D warnings` - Lint check
- `cargo fmt --check` - Format check
- `maturin build --release` - Build Python wheels

## Code Style
- Rust: follow `rustfmt` defaults, use `clippy` lints
- Error handling: use `thiserror` for error types
- Tests: place unit tests in `#[cfg(test)]` modules, integration tests in `tests/`

## Architecture
- `src/lib.rs` - Core library (EditResult, EditError, replace functions)
- `src/main.rs` - CLI entry point (clap argument parsing)
- `src/python.rs` - Python bindings (PyO3)
- `tests/` - Integration tests
- `tests/fixtures/` - Test files (various encodings, formats)

## Testing Instructions
- Run `cargo test` before committing
- Run `pytest` after `maturin develop` for Python binding tests
- Add tests for new functionality
- Test fixtures go in `tests/fixtures/`

## Implemented Features
- [ ] US-001: Single exact-match replacement
- [ ] US-002: Multiple match mode (-m)
...

## Known Limitations
[Document edge cases and limitations as discovered]
```

### Architecture
- **Core:** Rust library containing all edit logic
- **CLI:** Rust binary using clap for argument parsing
- **Python bindings:** PyO3 + maturin for building wheels

### Dependencies
- **Rust toolchain:** Minimum Rust version TBD (likely 1.70+)
- **clap:** For CLI argument parsing
- **serde/serde_json/serde_yaml/toml:** For structured-key mode parsing
- **PyO3:** For Python bindings
- **maturin:** For building Python wheels

### Performance Requirements
- Linear-time pass over file content for search
- Memory usage proportional to file size (entire file loaded)
- Target: < 100ms for files under 10MB on modern hardware

### Integration Points
- Can be used as a standalone CLI tool
- Can be integrated as a Rust library dependency
- Can be called from Python via maturin-built bindings

### Platform Support
- Linux: x86_64 and aarch64, glibc and musl targets
- macOS: x86_64 (Intel) and aarch64 (Apple Silicon)

### Test Framework

**Rust Tests:**
- Unit tests in `src/*.rs` files using `#[cfg(test)]` modules
- Integration tests in `tests/` directory
- Run with `cargo test`

**Python Tests:**
- pytest for Python binding tests in `tests/python/`
- Run with `pytest` after `maturin develop`

**Test file fixtures:**
- Located in `tests/fixtures/`
- Include various encodings, line endings, and structured data formats

## 8. Success Metrics

- **Correctness:** 100% of test cases pass, including edge cases for encodings and line endings
- **Safety:** Zero instances of file corruption in production use (verified via atomic write tests)
- **Adoption:** Library downloaded 1,000+ times within 6 months of release
- **Documentation:** README and CLI help rated as "clear and complete" by 80%+ of user feedback
- **Cross-platform:** Successful installation and operation confirmed on Linux and macOS
- **Performance:** Sub-100ms operation for files under 10MB