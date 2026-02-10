# AGENTS.md

- US-011: Structured Key Mode - JSON
- US-011: Structured Key Mode - YAML/TOML (not in scope for this patch)

This patch introduces a structured key path mode for JSON files. When -s/--structured is supplied, the search string is treated as a JSON key path (supporting nested keys and array indices) and the replace string becomes the new value for that path. The feature preserves JSON formatting by applying changes via a parsed JSON tree and then writing back with similar indentation. It performs strict path resolution and returns errors for invalid paths or ambiguous paths where the path matches multiple locations.

- Changes touched
  - bin/fedit_structured_json.py: new structured JSON path replacer
  - AGENTS.md: update note for US-011 feature
- How to use (example)
  - fedit -s file.json "config.port" "8080"
- Next steps
  - Expand YAML/T TOML structured support in future PRs
