#!/usr/bin/env python3
"""FEdit: Single exact-match replacement with encoding and line ending preservation.

Usage:
  fedit <path> <search> <replace> [--encoding <ENC>] [--multiple]

Behavior:
- Replaces exactly one occurrence when there is a single exact-match.
- If there are zero matches, prints an error and exits non-zero.
- If there are multiple matches, errors unless --multiple is provided, in which
  case all matches are replaced.
- Line endings are preserved based on the dominant style in the input file (LF or CRLF).
  Replacements containing the escape sequence "\n" will be translated into the target
  line ending style.
"""

import argparse
import os
import sys
import tempfile
from typing import Optional


def _detect_line_endings(raw_bytes: bytes) -> Optional[str]:
    # Determine dominant line ending style based on content.
    crlf = raw_bytes.count(b"\r\n")
    lf_only = raw_bytes.count(b"\n") - crlf
    if crlf == 0 and lf_only == 0:
        return None  # No line endings detected
    if crlf >= lf_only:
        return "crlf"
    return "lf"


def _detect_target_ending(variant: Optional[str]) -> Optional[str]:
    if variant == "crlf":
        return "\r\n"
    if variant == "lf":
        return "\n"
    return None


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Single exact-match replacement in a file with encoding and line ending preservation"
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

    # Read input as bytes to preserve line ending information
    try:
        with open(path, "rb") as f:
            raw = f.read()
    except FileNotFoundError:
        print(f"No such file: {path}", file=sys.stderr)
        return 2
    except Exception as e:
        print(f"Error reading file: {e}", file=sys.stderr)
        return 2

    # Detect line endings
    dom = _detect_line_endings(raw)
    line_ending = _detect_target_ending(dom)

    # Decode content using the provided encoding
    try:
        text = raw.decode(enc)
    except UnicodeDecodeError:
        print(
            f"EncodingError: Could not decode input file '{path}' using encoding '{enc}'",
            file=sys.stderr,
        )
        return 4
    except Exception as e:
        print(f"Error decoding file: {e}", file=sys.stderr)
        return 2

    # Locate exact non-overlapping matches
    indices = []
    start = 0
    while True:
        idx = text.find(search, start)
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

    # Prepare replacement string:
    # If a dominant line ending exists, convert escaped "\n" sequences to that ending.
    rep = replacement
    if line_ending is not None:
        rep = rep.replace("\\n", line_ending)

    # Perform replacement
    if count == 1:
        idx = indices[0]
        new_text = text[:idx] + rep + text[idx + len(search) :]
    else:
        new_text = text.replace(search, rep)

    # Atomic write via temp file
    dirn = os.path.dirname(path) or "."
    tmp_path = None
    try:
        fd, tmp_path = tempfile.mkstemp(
            prefix=".fedit.tmp.", suffix="." + os.path.basename(path), dir=dirn
        )
        # Write with explicit encoding and no newline translation
        with open(fd, "w", encoding=enc, newline="") as f:
            f.write(new_text)
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
