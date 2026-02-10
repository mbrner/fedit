"""FEdit: Safe, atomic file editing with search-and-replace.

Main API:
    edit(path, old, new)           - Replace text in a file
    edit_structured(path, key, value) - Edit JSON/YAML/TOML by key path

Example:
    >>> import fedit
    >>> fedit.edit("config.py", "DEBUG = True", "DEBUG = False")
    >>> fedit.edit_structured("config.json", "server.port", 8080)
"""

import sys
from pathlib import Path
from typing import Any, Optional, Union

# Import from Rust core module
from fedit._core import (
    # Result types
    EditResultWithDiff,
    StructuredEditResult,
    # Core functions
    edit_fuzzy as _edit_fuzzy,
    edit_structured_file as _edit_structured_file,
)

__all__ = [
    "edit",
    "edit_structured",
    "EditResultWithDiff",
    "StructuredEditResult",
]


def edit(
    path: Union[str, Path],
    old_text: str,
    new_text: str,
    *,
    multiple: bool = False,
    encoding: str = "utf-8",
    dry_run: bool = False,
) -> EditResultWithDiff:
    """Replace text in a file.

    Uses fuzzy matching by default: tries exact match first, then falls back
    to matching with normalized smart quotes, dashes, and whitespace.

    Args:
        path: Path to the file to edit
        old_text: Text to find and replace
        new_text: Replacement text
        multiple: Replace all occurrences (default: False, error if multiple found)
        encoding: File encoding (default: "utf-8")
        dry_run: Preview changes without modifying the file

    Returns:
        EditResultWithDiff with content, replacements, diff, and line_ending

    Raises:
        FileNotFoundError: If file does not exist
        ValueError: If text not found, or multiple matches without multiple=True

    Example:
        >>> result = fedit.edit("app.py", "v1.0", "v2.0")
        >>> print(f"Made {result.replacements} replacement(s)")
        >>> print(result.diff)
    """
    return _edit_fuzzy(
        str(path),
        old_text,
        new_text,
        multiple=multiple,
        encoding=encoding,
        dry_run=dry_run,
    )


def edit_structured(
    path: Union[str, Path],
    key_path: str,
    value: Any,
    *,
    format: Optional[str] = None,
    encoding: str = "utf-8",
    dry_run: bool = False,
) -> StructuredEditResult:
    """Edit a structured file (JSON, YAML, TOML) by key path.

    Args:
        path: Path to the file to edit
        key_path: Dot-separated path (e.g., "server.port", "users[0].name")
        value: New value (str, int, float, bool, or JSON string for complex values)
        format: Force format ("json", "jsonc", "json5", "toml", "yaml"), or auto-detect
        encoding: File encoding (default: "utf-8")
        dry_run: Preview changes without modifying the file

    Returns:
        StructuredEditResult with content, key_path, old_value, new_value

    Raises:
        FileNotFoundError: If file does not exist
        ValueError: If key path invalid, key not found, or parse error

    Example:
        >>> fedit.edit_structured("config.json", "server.port", 8080)
        >>> fedit.edit_structured("config.yaml", "database.host", "localhost")
    """
    # Convert value to string
    if isinstance(value, bool):
        value_str = "true" if value else "false"
    elif isinstance(value, (int, float)):
        value_str = str(value)
    elif isinstance(value, str):
        value_str = value
    else:
        import json

        value_str = json.dumps(value)

    return _edit_structured_file(
        str(path),
        key_path,
        value_str,
        format=format,
        encoding=encoding,
        dry_run=dry_run,
    )


def main() -> int:
    """CLI entry point."""
    import argparse

    parser = argparse.ArgumentParser(
        prog="fedit",
        description="Safe, atomic file editing with search-and-replace",
        epilog="""Examples:
  fedit file.txt "old" "new"              # Replace text
  fedit file.txt "old" "new" -m           # Replace all occurrences
  fedit config.json -s server.port 8080   # Edit JSON by key path
  fedit config.yaml -s db.host localhost  # Edit YAML by key path
""",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("path", help="File to edit")
    parser.add_argument("search", help="Text to find (or key path with -s)")
    parser.add_argument("replace", help="Replacement text (or new value with -s)")
    parser.add_argument(
        "-s",
        "--structured",
        action="store_true",
        help="Structured mode: edit by key path (JSON/YAML/TOML)",
    )
    parser.add_argument(
        "-f",
        "--format",
        choices=["json", "jsonc", "json5", "toml", "yaml"],
        help="Force file format (with -s)",
    )
    parser.add_argument(
        "-m", "--multiple", action="store_true", help="Replace all occurrences"
    )
    parser.add_argument(
        "-e", "--encoding", default="utf-8", help="File encoding (default: utf-8)"
    )
    parser.add_argument(
        "-n",
        "--dry-run",
        action="store_true",
        help="Preview changes without modifying file",
    )
    parser.add_argument("-d", "--diff", action="store_true", help="Show diff output")

    args = parser.parse_args()

    try:
        if args.structured:
            result = edit_structured(
                args.path,
                args.search,
                args.replace,
                format=args.format,
                encoding=args.encoding,
                dry_run=args.dry_run,
            )
            action = "Would set" if args.dry_run else "Set"
            print(f"{action} {result.key_path} = {result.new_value}")
            if result.old_value:
                print(f"  (was: {result.old_value})")
        else:
            result = edit(
                args.path,
                args.search,
                args.replace,
                multiple=args.multiple,
                encoding=args.encoding,
                dry_run=args.dry_run,
            )
            action = "Would replace" if args.dry_run else "Replaced"
            s = "s" if result.replacements != 1 else ""
            print(f"{action} {result.replacements} occurrence{s}")
            if args.diff and result.diff:
                print("\n" + result.diff)

        return 0

    except FileNotFoundError:
        print(f"Error: File not found: {args.path}", file=sys.stderr)
        return 2
    except ValueError as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
