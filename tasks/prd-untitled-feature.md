# FEdit – Exact File Edit Toolkit

## 1. Introduction/Overview

FEdit is a cross-platform Rust library and CLI tool that enables robust, safe file edits via exact-match search-and-replace operations. It addresses the common problem of making precise, predictable text modifications while preserving file integrity, line endings, and encodings.

The tool supports multiple search strategies (exact, fuzzy, structured keys), provides optional Python bindings, and includes comprehensive documentation automation. It is designed for developers and DevOps teams who need a reliable, portable solution for codebase maintenance across Linux, macOS, and Windows.

## 2. Goals

- Provide a safe, predictable edit tool that requires exactly one match by default (preventing accidental mass changes)
- Preserve file integrity including line endings, encodings, and overall file structure
- Support exact-match, fuzzy, and structured-key search modes
- Deliver atomic write operations to prevent file corruption
- Enable cross-platform operation on Linux, macOS, and Windows
- Provide both a Rust library API and a CLI interface
- Offer optional Python bindings for broader ecosystem integration
- Automate documentation updates (README, AGENTS.md, manpage) as features are implemented
- Produce distributable packages (wheels, deb/rpm, MSI) for easy installation

## 3. Quality Gates

These commands must pass for every user story:

- `cargo build --release` - Successful build
- `cargo test` - All tests pass
- `cargo clippy -- -D warnings` - No linter warnings
- `cargo fmt --check` - Code formatting check

For CLI stories, also include:
- Manual verification of CLI behavior with test files

For Python binding stories, also include:
- `maturin build` - Python wheel builds successfully
- `pytest` - Python binding tests pass

## 4. User Stories

### US-001: Single Exact-Match Replacement

**Description:** As a developer, I want to replace exactly one occurrence of a search string in a file so that I can make precise edits without accidentally modifying multiple locations.

**Acceptance Criteria:**
- [ ] CLI accepts `fedit <target> <search-str> <replace-str>` arguments
- [ ] When exactly one match exists, the replacement is performed
- [ ] When zero matches exist, an error message states "No matches found for: [search string]"
- [ ] When multiple matches exist, an error message states "Multiple matches found ([count]); use -M to replace all"
- [ ] Original file is unchanged when an error occurs
- [ ] Exit code is 0 on success, non-zero on error

### US-002: Multiple Match Replacement Mode

**Description:** As a developer, I want to optionally replace all occurrences of a search string so that I can perform bulk replacements when intended.

**Acceptance Criteria:**
- [ ] CLI accepts `-M` or `--mult` flag to enable multiple replacements
- [ ] When `-M` is provided, all occurrences are replaced
- [ ] Output displays the count of replacements made
- [ ] When `-M` is provided and zero matches exist, an error message is displayed

### US-003: Atomic File Write

**Description:** As a developer, I want file writes to be atomic so that power failures or crashes don't leave files in a corrupted state.

**Acceptance Criteria:**
- [ ] Replacement writes to a temporary file first
- [ ] Temporary file is flushed to disk before replacing the original
- [ ] Original file is atomically replaced using rename operation
- [ ] If the write fails, the original file remains unchanged
- [ ] Temporary files are cleaned up on both success and failure

### US-004: Encoding Support

**Description:** As a developer, I want to specify file encoding so that I can edit files that aren't UTF-8.

**Acceptance Criteria:**
- [ ] CLI accepts `--encoding` or `-e` argument
- [ ] Default encoding is UTF-8 when not specified
- [ ] Common encodings are supported (UTF-8, UTF-16, ISO-8859-1, Windows-1252)
- [ ] Clear error message when encoding cannot decode the file
- [ ] Output file uses the same encoding as the input

### US-005: Line Ending Preservation

**Description:** As a developer, I want line endings to be preserved so that files maintain their platform-specific format.

**Acceptance Criteria:**
- [ ] CRLF line endings remain CRLF after editing
- [ ] LF line endings remain LF after editing
- [ ] Mixed line endings are preserved in their original positions
- [ ] Replacement strings containing newlines use the file's dominant line ending style

### US-006: Dry Run Mode

**Description:** As a developer, I want to preview changes without modifying the file so that I can verify the edit before committing.

**Acceptance Criteria:**
- [ ] CLI accepts `--dry-run` flag
- [ ] When `--dry-run` is provided, no file modifications occur
- [ ] Output shows what would be changed (before/after preview)
- [ ] Output indicates the line number(s) where changes would occur
- [ ] Exit code reflects whether the operation would succeed

### US-007: CLI Help and Usage

**Description:** As a user, I want comprehensive help text so that I can understand all available options and see usage examples.

**Acceptance Criteria:**
- [ ] `fedit --help` displays all available flags and arguments
- [ ] Each flag includes a description of its purpose
- [ ] Usage examples are provided for common scenarios
- [ ] Help text fits within 80-column terminal width

### US-008: Rust Library API - Core Function

**Description:** As a Rust developer, I want a library function to perform replacements so that I can integrate FEdit into my applications.

**Acceptance Criteria:**
- [ ] Public function accepts file content as string and replacement parameters
- [ ] Function returns `Result<EditResult, EditError>` 
- [ ] `EditResult` contains the modified content and count of replacements
- [ ] Options struct allows enabling/disabling multiple replacements
- [ ] Function is documented with rustdoc including examples

### US-009: Error Types and Handling

**Description:** As a Rust developer, I want well-defined error types so that I can handle different failure modes appropriately.

**Acceptance Criteria:**
- [ ] `EditError::NotFound` is returned when search string has no matches
- [ ] `EditError::MultipleFound(count)` is returned when uniqueness is required but multiple matches exist
- [ ] `EditError::IoError` wraps underlying I/O errors with context
- [ ] `EditError::EncodingError` is returned for encoding issues
- [ ] All error types implement `std::error::Error` and `Display`

### US-010: Normalization for Search

**Description:** As a developer, I want whitespace and Unicode normalization so that searches are more flexible while preserving original formatting.

**Acceptance Criteria:**
- [ ] Search normalizes both input and file text for comparison
- [ ] Original byte offsets are preserved via position mapping
- [ ] Replacement is performed in original (non-normalized) space
- [ ] Multiple consecutive whitespace can match single space in search (optional flag)
- [ ] Unicode normalization form can be specified (NFC, NFD)

### US-011: Structured Key Mode - JSON

**Description:** As a developer, I want to specify a JSON path to locate and replace a value so that I can edit structured data safely.

**Acceptance Criteria:**
- [ ] CLI accepts `-S` flag to enable structured-key mode
- [ ] Path syntax supports nested keys: `root.child.field`
- [ ] Path syntax supports array indices: `items[0].name`
- [ ] Replacement updates only the resolved value
- [ ] JSON structure (formatting, other fields) is preserved
- [ ] Error is returned for invalid key paths with specific message

### US-012: Structured Key Mode - YAML and TOML

**Description:** As a developer, I want structured-key mode to work with YAML and TOML files so that I can edit configuration files safely.

**Acceptance Criteria:**
- [ ] YAML files are detected by extension (.yml, .yaml) or explicit flag
- [ ] TOML files are detected by extension (.toml) or explicit flag
- [ ] Same path syntax works across all supported formats
- [ ] Comments in YAML/TOML are preserved when possible
- [ ] Formatting and indentation are preserved

### US-013: Python Bindings - Core API

**Description:** As a Python developer, I want to use FEdit from Python so that I can integrate it into Python-based tooling.

**Acceptance Criteria:**
- [ ] Python bindings are available behind `python-bindings` cargo feature
- [ ] `fedit.replace_exact(content, search, replace)` function exists
- [ ] `fedit.replace_exact(content, search, replace, allow_multiple=True)` works
- [ ] Python exceptions mirror Rust error types (NotFoundException, etc.)
- [ ] Function accepts and returns Python strings

### US-014: Python Wheel Packaging

**Description:** As a Python developer, I want to install FEdit via pip so that I can easily add it to my projects.

**Acceptance Criteria:**
- [ ] Wheels are built for Linux (manylinux), macOS, and Windows
- [ ] Wheels support Python 3.9, 3.10, 3.11, and 3.12
- [ ] `pip install fedit` installs the package successfully
- [ ] Package includes type stubs for IDE support
- [ ] README is included in package metadata

### US-015: CLI Manpage Generation

**Description:** As a Unix user, I want a manpage so that I can access documentation via the standard `man` command.

**Acceptance Criteria:**
- [ ] Manpage is generated from CLI definition (via clap_mangen or similar)
- [ ] Manpage includes all commands, flags, and options
- [ ] Manpage includes examples section
- [ ] Manpage is included in Linux/macOS packages
- [ ] `man fedit` displays the documentation after installation

### US-016: README Automation

**Description:** As a maintainer, I want the README to be automatically updated so that documentation stays in sync with features.

**Acceptance Criteria:**
- [ ] README includes overview and installation instructions
- [ ] README includes usage examples for all implemented modes
- [ ] README sections are generated from code/config where possible
- [ ] CI validates README is up-to-date
- [ ] Version number in README matches crate version

### US-017: Cross-Platform Binary Distribution

**Description:** As a user, I want pre-built binaries for my platform so that I don't need a Rust toolchain to install FEdit.

**Acceptance Criteria:**
- [ ] Linux binaries are built for x86_64 and aarch64
- [ ] macOS binaries are built for x86_64 and aarch64 (Apple Silicon)
- [ ] Windows binaries are built for x86_64
- [ ] Binaries are available as GitHub release assets
- [ ] SHA256 checksums are provided for all binaries

### US-018: Linux Package Distribution

**Description:** As a Linux user, I want to install FEdit via my package manager so that updates are managed automatically.

**Acceptance Criteria:**
- [ ] .deb package is built for Debian/Ubuntu
- [ ] .rpm package is built for Fedora/RHEL
- [ ] Packages include the binary, manpage, and shell completions
- [ ] Package metadata includes description, license, and homepage
- [ ] Post-install scripts set up shell completions if applicable

## 5. Functional Requirements

- **FR-1:** The system must replace text in a file based on an exact string match.
- **FR-2:** The system must return an error when the search string matches zero occurrences (unless multiple mode is explicitly enabled).
- **FR-3:** The system must return an error when the search string matches more than one occurrence, unless multiple replacement mode is enabled.
- **FR-4:** When multiple replacement mode is enabled (`-M`), the system must replace all occurrences.
- **FR-5:** The system must write changes atomically via a temporary file and rename operation.
- **FR-6:** The system must preserve the original file's line endings (LF, CRLF, or mixed).
- **FR-7:** The system must preserve the original file's encoding, defaulting to UTF-8.
- **FR-8:** The system must support specifying an alternative encoding via `--encoding`.
- **FR-9:** The system must support dry-run mode that shows changes without modifying the file.
- **FR-10:** The system must support structured-key mode (`-S`) for JSON, YAML, and TOML files.
- **FR-11:** In structured-key mode, the system must support nested key paths (e.g., `root.child.field`).
- **FR-12:** In structured-key mode, the system must support array indices (e.g., `items[0].name`).
- **FR-13:** The CLI must provide comprehensive help text via `--help`.
- **FR-14:** The Rust library must expose a public API for programmatic use.
- **FR-15:** The system must support optional Python bindings behind a feature flag.

## 6. Non-Goals (Out of Scope)

- **Regex support:** This version focuses on exact-match and structured-key modes only; regular expression matching is not included.
- **In-place streaming for very large files:** Files are loaded into memory; streaming edits for multi-gigabyte files are not supported.
- **GUI interface:** Only CLI and library interfaces are provided.
- **Remote file editing:** The tool operates on local files only; network protocols (SSH, HTTP, etc.) are not supported.
- **Version control integration:** Automatic git commits or VCS awareness is not included.
- **Backup file creation:** Users are responsible for backups; the tool does not create `.bak` files.
- **Interactive mode:** The tool is non-interactive; all parameters must be specified upfront.
- **Configuration file:** All options are passed via CLI arguments; there is no `.feditrc` or similar config file.

## 7. Technical Considerations

### Dependencies
- **Rust toolchain:** Minimum Rust version TBD (likely 1.70+)
- **clap:** For CLI argument parsing
- **serde/serde_json/serde_yaml/toml:** For structured-key mode parsing
- **PyO3/maturin:** For Python bindings (optional feature)

### Performance Requirements
- Linear-time pass over file content for search
- Memory usage proportional to file size (entire file loaded)
- Target: < 100ms for files under 10MB on modern hardware

### Integration Points
- Can be used as a standalone CLI tool
- Can be integrated as a Rust library dependency
- Can be called from Python via bindings

### Platform-Specific Considerations
- Windows: Handle CRLF line endings and Windows-specific path separators
- macOS: Support both Intel and Apple Silicon architectures
- Linux: Support glibc and musl targets for broader compatibility

## 8. Success Metrics

- **Correctness:** 100% of test cases pass, including edge cases for encodings and line endings
- **Safety:** Zero instances of file corruption in production use (verified via atomic write tests)
- **Adoption:** Library downloaded 1,000+ times within 6 months of release
- **Documentation:** README and CLI help rated as "clear and complete" by 80%+ of user feedback
- **Cross-platform:** Successful installation and operation confirmed on all target platforms
- **Performance:** Sub-100ms operation for files under 10MB

## 9. Open Questions

1. Should we expose a formal API compatibility shim for Python bindings in v1.0, or defer to a follow-up release?
2. What should the Python binding module namespace be? (`fedit`, `py_fedit`, `fedit_py`?)
3. Should Windows MSI packaging be included in the initial release, or deferred until the cross-platform baseline is stable?
4. Should we support JSON5 or JSONC (JSON with comments) in structured-key mode?
5. What is the minimum supported Rust version (MSRV) for the project?
6. Should fuzzy matching be included in v1.0, or deferred to a later release?
7. How should the tool handle binary files that are accidentally passed as input?