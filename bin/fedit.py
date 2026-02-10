#!/usr/bin/env python3
"""FEdit: Single exact-match replacement with encoding support.

Usage:
  fedit <path> <search> <replace> [--encoding <ENC>] [--multiple]

Behavior:
- Replaces exactly one occurrence when there is a single exact-match.
- If there are zero matches, prints an error and exits non-zero.
- If there are multiple matches, errors unless --multiple is provided, in which
  case all matches are replaced.
"""

import argparse
import os
import sys
import tempfile


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Single exact-match replacement in a file with encoding support"
    )
    # Positional arguments for the core task
    parser.add_argument("path", help="Path to the target file")
    parser.add_argument("search", help="Search string to replace (exact match)")
    parser.add_argument("replace", help="Replacement string")

    # Optional arguments
    parser.add_argument(
        "-e",
        "--encoding",
        dest="encoding",
        default="utf-8",
        choices=["utf-8", "utf-16", "iso-8859-1", "windows-1252"],
        help="File encoding to use (default: UTF-8)",
    )
    parser.add_argument(
        "-m",
        "--multiple",
        dest="multiple",
        action="store_true",
        help="Replace all occurrences when multiple matches exist",
    )
    args = parser.parse_args()

    path = args.path
    search = args.search
    replacement = args.replace
    enc = args.encoding

    # Read input with specified encoding
    try:
        with open(path, "r", encoding=enc) as f:
            content = f.read()
    except FileNotFoundError:
        print(f"No such file: {path}", file=sys.stderr)
        return 2
    except UnicodeDecodeError:
        print(
            f"EncodingError: Could not decode input file '{path}' using encoding '{enc}'",
            file=sys.stderr,
        )
        return 4
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

    if count > 1 and not args.multiple:
        print(
            f"Multiple matches found ({count}); use --multiple to replace all",
            file=sys.stderr,
        )
        return 1

    # Perform replacement
    if count == 1:
        idx = indices[0]
        new_content = content[:idx] + replacement + content[idx + len(search) :]
    else:
        new_content = content.replace(search, replacement)

    # Atomic write via temp file
    dirn = os.path.dirname(path) or "."
    tmp_path = None
    try:
        fd, tmp_path = tempfile.mkstemp(
            prefix=".fedit.tmp.", suffix="." + os.path.basename(path), dir=dirn
        )
        with os.fdopen(fd, "w", encoding=enc) as f:
            f.write(new_content)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp_path, path)
        print(f"Replaced {count} occurrence{'s' if count != 1 else ''} in {path}")
        return 0
    except Exception as e:
        if tmp_path and os.path.exists(tmp_path):
            try:
                os.remove(tmp_path)
            except Exception:
                pass
        print(f"Error writing file: {e}", file=sys.stderr)
        return 3
    finally:
        if tmp_path and os.path.exists(tmp_path):
            try:
                os.remove(tmp_path)
            except Exception:
                pass


if __name__ == "__main__":
    raise SystemExit(main())
