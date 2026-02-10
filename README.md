Usage
- Run the CLI with: `fedit <path> <search-str> <replace-str> [--encoding <ENC>] [--multiple]`

- Examples
- Replace a single exact match:
  - fedit example.txt "old" "new"
- Replace all matches (requires --multiple):
  - fedit example.txt "dup" "dup2" --multiple

- Notes
- Exit code 0 on success, non-zero on error.
- When zero matches exist: prints "No matches found for: <search-str>" and exits 1.
- When multiple matches exist: prints "Multiple matches found (<count>); use --multiple to replace all" and exits 1.
- Original file is unchanged on error.
- Safety guarantees
- - Atomic writes ensure partial writes don't corrupt files: writing to a temporary file, flushing to disk, and atomically replacing the target file.
- - If a write fails, the original file remains unchanged.
- - Temporary files are cleaned up on both success and failure.
