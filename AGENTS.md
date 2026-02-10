US-001: Single Exact-Match Replacement

- Objective: Add a CLI mode to replace exactly one matching string in a file when exactly one match exists.
- Behavior:
  - Accepts positional arguments: fedit <target> <search-str> <replace-str>
  - If zero matches: exit code 1 and print "No matches found for: [search-str]" to stderr.
  - If one match: perform replacement and exit 0.
  - If more than one match: exit code 1 with message "Multiple matches found ([count]); use --multiple to replace all" unless --multiple is provided, in which case replace all and exit 0.
  - Atomic writes when updating the file to avoid corruption on failure.
- Error semantics:
  - Non-zero exit codes on error paths; 0 on success.
- File encodings:
  - Support common encodings via --encoding; defaults to utf-8.
- User experience:
  - Ensure original file remains unchanged if error occurs.
- Notes:
  - Update README.md with usage example and AGENTS.md reflecting this feature.

US-002: Multiple Match Replacement Mode

- Objective: Provide an option to replace all occurrences of a search string when -m/--multiple is provided.
- Behavior:
  - CLI accepts -m or --multiple to enable multiple replacements
- When -m is provided, all occurrences are replaced
- Output: display the count of replacements made
- When -m is provided and zero matches exist, print an error message
- Notes:
  - Atomic writes are used to update the file to avoid corruption on failure.
- Update README.md with usage example and AGENTS.md reflecting this feature.
