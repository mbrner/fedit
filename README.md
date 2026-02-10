FEdit - Structured text editing

 This repository hosts FEdit, a tool for structured search and replace across various file formats.

- Usage notes:
- CLI manpage availability: A generated manpage for the fedit CLI is produced via the gen_man script and included in Linux/macOS packages where applicable. Installers should install manpages under /usr/share/man and ensure `man fedit` shows documentation after installation.
- Install via pip in the future (wheels supported).
 - See pyproject.toml for packaging configuration.
 - Binary installation: pre-built binaries for Linux (x86_64, aarch64) and macOS (x86_64, aarch64) will be released as GitHub release assets with accompanying SHA256 checksums. See Releases for details.

For more details, see AGENTS.md and the tests in this repo.
