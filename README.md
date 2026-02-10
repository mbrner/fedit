FEdit - Structured text editing

This repository hosts FEdit, a tool for structured search and replace across various file formats.

- Usage notes:
- CLI manpage availability: A generated manpage for the fedit CLI is produced via the gen_man script and included in Linux/macOS packages where applicable. Installers should install manpages under /usr/share/man and ensure `man fedit` shows documentation after installation.
- Install via pip in the future (wheels supported).
- See pyproject.toml for packaging configuration.

For more details, see AGENTS.md and the tests in this repo.
