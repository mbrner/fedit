"""FEdit: Exact File Edit Toolkit - structured search-and-replace operations.

This module provides Python bindings to the Rust-based FEdit library.
"""

import sys
from typing import Optional, Tuple

# Import from Rust core module
from fedit._core import (
    Encoding,
    EditResult,
    edit,
    replace_in_string,
    read,
    detect_line_ending,
)

__all__ = [
    "Encoding",
    "EditResult",
    "edit",
    "replace_in_string",
    "read",
    "detect_line_ending",
    "edit_file",
    "main",
]


def edit_file(
    path: str,
    search: str,
    replace: str,
    *,
    multiple: bool = False,
    ignore_whitespace: bool = False,
    encoding: str = "utf-8",
    dry_run: bool = False,
) -> EditResult:
    """Edit a file in place.

    Args:
        path: Path to the file to edit
        search: The string to search for
        replace: The replacement string
        multiple: If True, replace all occurrences
        ignore_whitespace: If True, treat consecutive whitespace as equivalent
        encoding: File encoding (default: "utf-8")
        dry_run: If True, don't actually modify the file

    Returns:
        EditResult with replacement count and detected line ending

    Raises:
        FileNotFoundError: If the file does not exist
        ValueError: If no matches found, multiple matches without multiple=True,
                   or encoding error
    """
    return edit(
        path,
        search,
        replace,
        multiple=multiple,
        ignore_whitespace=ignore_whitespace,
        encoding=encoding,
        dry_run=dry_run,
    )


def main() -> int:
    """CLI entry point - delegates to Rust binary or provides Python fallback."""
    import argparse

    parser = argparse.ArgumentParser(
        prog="fedit",
        description="FEdit: Whitespace-insensitive search and replace with encoding and line ending preservation",
        epilog="""Examples:
  Replace a single exact match: fedit <path> <search> <replace>
  Replace all matches: fedit <path> <search> <replace> --multiple
  Whitespace-insensitive search: fedit <path> <search> <replace> -w
  Use a specific encoding: fedit <path> <search> <replace> -e utf-16
""",
    )
    parser.add_argument("path", help="Path to the target file")
    parser.add_argument(
        "search", help="Search string to replace (may contain whitespace)"
    )
    parser.add_argument("replace", help="Replacement string")
    parser.add_argument(
        "-e",
        "--encoding",
        default="utf-8",
        choices=["utf-8", "utf-16", "iso-8859-1", "windows-1252"],
        help="File encoding to use (default: UTF-8)",
    )
    parser.add_argument(
        "-m",
        "--multiple",
        action="store_true",
        help="Replace all occurrences when multiple matches exist",
    )
    parser.add_argument(
        "-n",
        "--dry-run",
        action="store_true",
        help="Preview changes without modifying the file",
    )
    parser.add_argument(
        "-w",
        "--ignore-whitespace",
        action="store_true",
        help="Whitespace-insensitive search (treats consecutive whitespace as equivalent)",
    )
    parser.add_argument(
        "-s",
        "--structured",
        action="store_true",
        help="Structured mode: exact key-path matching (for JSON/YAML/TOML)",
    )

    args = parser.parse_args()

    # Structured mode not yet in Rust - fallback message
    if args.structured:
        print(
            "Structured mode is not yet implemented in the Python CLI.",
            file=sys.stderr,
        )
        print(
            "Use the Python scripts for structured JSON/YAML/TOML editing:",
            file=sys.stderr,
        )
        print(
            "  python bin/fedit_structured_json.py -s file.json 'path.to.key' 'value'",
            file=sys.stderr,
        )
        return 2

    try:
        result = edit(
            args.path,
            args.search,
            args.replace,
            multiple=args.multiple,
            ignore_whitespace=args.ignore_whitespace,
            encoding=args.encoding,
            dry_run=args.dry_run,
        )

        if args.dry_run:
            print(
                f"Dry-run: would replace {result.replacements} occurrence(s) in {args.path}"
            )
        else:
            s = "s" if result.replacements != 1 else ""
            print(f"Replaced {result.replacements} occurrence{s} in {args.path}")

        return 0

    except FileNotFoundError as e:
        print(f"No such file: {args.path}", file=sys.stderr)
        return 2
    except ValueError as e:
        msg = str(e)
        if "No matches found" in msg:
            print(msg, file=sys.stderr)
            return 1
        elif "Multiple matches found" in msg:
            print(msg, file=sys.stderr)
            return 1
        elif "Encoding error" in msg:
            print(msg, file=sys.stderr)
            return 4
        else:
            print(msg, file=sys.stderr)
            return 2
    except IOError as e:
        print(f"Error writing file: {e}", file=sys.stderr)
        return 3
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
