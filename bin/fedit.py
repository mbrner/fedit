#!/usr/bin/env python3
"""FEdit: Whitespace-insensitive search and replace with encoding and line ending preservation."""

import argparse
import os
import sys
import tempfile
import re
from typing import Optional


def _detect_line_endings(raw_bytes: bytes) -> Optional[str]:
    crlf = raw_bytes.count(b"\r\n")
    lf_only = raw_bytes.count(b"\n") - crlf
    if crlf == 0 and lf_only == 0:
        return None
    if crlf >= lf_only:
        return "crlf"
    return "lf"


def _detect_target_ending(variant: Optional[str]) -> Optional[str]:
    if variant == "crlf":
        return "\r\n"
    if variant == "lf":
        return "\n"
    return None


def _build_ws_pattern(search: str) -> str:
    parts = []
    i = 0
    while i < len(search):
        ch = search[i]
        if ch.isspace():
            j = i
            while j < len(search) and search[j].isspace():
                j += 1
            parts.append(r"\s+")
            i = j
        else:
            parts.append(re.escape(ch))
            i += 1
    return "".join(parts)


class FEditHelpFormatter(argparse.HelpFormatter):
    def __init__(self, prog):
        super().__init__(prog, width=80)


def main() -> int:
    epilog = (
        "Examples:\n"
        "  - Replace a single exact match: fedit <path> <search> <replace>\n"
        "  - Replace all matches: fedit <path> <search> <replace> --multiple\n"
        "  - Use whitespace-insensitive search: fedit <path> <search> <replace> -w\n"
        "  - Use a specific encoding: fedit <path> <search> <replace> -e utf-16\n"
    )
    parser = argparse.ArgumentParser(
        description=(
            "FEdit: Whitespace-insensitive search and replace with encoding and line ending preservation"
        ),
        epilog=epilog,
        formatter_class=FEditHelpFormatter,
    )
    parser.add_argument("path", help="Path to the target file")
    parser.add_argument(
        "search", help="Search string to replace (may contain whitespace)"
    )
    parser.add_argument("replace", help="Replacement string")

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
    parser.add_argument(
        "-n",
        "--dry-run",
        dest="dry_run",
        action="store_true",
        help="Preview changes without modifying the file",
    )
    parser.add_argument(
        "-w",
        "--ignore-whitespace",
        dest="ignore_whitespace",
        action="store_true",
        help="Whitespace-insensitive search (treats consecutive whitespace as equivalent)",
    )
    parser.add_argument(
        "-s",
        "--structured",
        dest="structured",
        action="store_true",
        help="Structured mode: exact key-path matching (ignore whitespace flag)",
    )
    args = parser.parse_args()

    path = args.path
    search = args.search
    replacement = args.replace
    enc = args.encoding
    do_dry_run = bool(args.dry_run)
    ignore_ws = bool(args.ignore_whitespace)
    structured = bool(args.structured)

    try:
        with open(path, "rb") as f:
            raw = f.read()
    except FileNotFoundError:
        print(f"No such file: {path}", file=sys.stderr)
        return 2
    except Exception as e:
        print(f"Error reading file: {e}", file=sys.stderr)
        return 2

    dom = _detect_line_endings(raw)
    line_ending = _detect_target_ending(dom)

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

    text_for_search = text if structured else text
    matches = []

    if structured:
        if search:
            for m in re.finditer(re.escape(search), text_for_search, flags=re.DOTALL):
                matches.append((m.start(), m.end()))
    elif ignore_ws:
        pattern = _build_ws_pattern(search)
        rx = re.compile(pattern, flags=re.DOTALL)
        for m in rx.finditer(text_for_search):
            matches.append((m.start(), m.end()))
    else:
        start = 0
        while True:
            idx = text_for_search.find(search, start)
            if idx == -1:
                break
            matches.append((idx, idx + len(search)))
            start = idx + len(search)

    count = len(matches)
    if count == 0:
        print(f"No matches found for: {search}", file=sys.stderr)
        return 1

    if count > 1 and not args.multiple:
        print(
            f"Multiple matches found ({count}); use --multiple to replace all",
            file=sys.stderr,
        )
        return 1

    rep = replacement
    rep_for_norm = rep.replace("\\n", "\n")
    if line_ending == "crlf":
        rep_translated = rep_for_norm.replace("\n", "\r\n")
    else:
        rep_translated = rep_for_norm

    if ignore_ws:
        new_parts = []
        prev = 0
        for start_idx, end_idx in matches:
            new_parts.append(text[prev:start_idx])
            new_parts.append(rep_translated)
            prev = end_idx
        new_parts.append(text[prev:])
        final_text = "".join(new_parts)
    else:
        if count == 1:
            start_idx, end_idx = matches[0]
            final_text = text[:start_idx] + rep_translated + text[end_idx:]
        else:
            final_text = text.replace(search, rep_translated)

    if do_dry_run:
        print(f"Dry-run: would replace {count} occurrence(s) in {path}")
        return 0

    dirn = os.path.dirname(path) or "."
    tmp_path = None
    try:
        fd, tmp_path = tempfile.mkstemp(
            prefix=".fedit.tmp.", suffix="." + os.path.basename(path), dir=dirn
        )
        with os.fdopen(fd, "w", encoding=enc, newline="") as f:
            f.write(final_text)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp_path, path)
        replaced = count if ignore_ws else (count if count > 0 else 0)
        print(f"Replaced {replaced} occurrence{'s' if replaced != 1 else ''} in {path}")
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
