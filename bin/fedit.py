#!/usr/bin/env python3
"""
Single Exact-Match Replacement CLI

- Replaces exactly one occurrence of a search string in a file when exactly one exists.
- If zero matches: exit with error and message: No matches found for: [search string]
- If multiple matches: exit with error and message: Multiple matches found ([count]); use -M to replace all
- If -M is provided, all matches are replaced.
- Exits: 0 on success, non-zero on error.
"""

import argparse
import sys


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Single exact-match replacement in a file"
    )
    parser.add_argument("--path", required=True, help="Path to the target file")
    parser.add_argument(
        "--search", required=True, help="Search string to replace (exact match)"
    )
    parser.add_argument("--replace", required=True, help="Replacement string")
    parser.add_argument(
        "-M",
        "--replace-all",
        action="store_true",
        help="Replace all occurrences when multiple matches exist",
    )

    args = parser.parse_args()

    path = args.path
    search = args.search
    replacement = args.replace

    try:
        with open(path, "r", encoding="utf-8") as f:
            content = f.read()
    except FileNotFoundError:
        print(f"No such file: {path}", file=sys.stderr)
        return 2
    except Exception as e:
        print(f"Error reading file: {e}", file=sys.stderr)
        return 2

    # Locate exact non-overlapping matches
    indices = []
    start = 0
    while True:
        idx = content.find(search, start)
        if idx == -1:
            break
        indices.append(idx)
        start = idx + len(search)

    count = len(indices)

    if count == 0:
        print(f"No matches found for: {search}", file=sys.stderr)
        return 1

    if count > 1 and not args.replace_all:
        print(
            f"Multiple matches found ({count}); use -M to replace all", file=sys.stderr
        )
        return 1

    # Perform replacement
    try:
        if count == 1:
            idx = indices[0]
            new_content = content[:idx] + replacement + content[idx + len(search) :]
        else:
            # Replace all occurrences
            new_content = content.replace(search, replacement)
        with open(path, "w", encoding="utf-8") as f:
            f.write(new_content)
        return 0
    except Exception as e:
        print(f"Error writing file: {e}", file=sys.stderr)
        return 3


if __name__ == "__main__":
    sys.exit(main())
