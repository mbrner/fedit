#!/usr/bin/env python3
"""
Structured Key Mode for YAML and TOML (experimental, best-effort).

- Detect YAML/TOML by file extension (.yml/.yaml, .toml) or --format flag.
- Path syntax is the same as JSON mode: dot-separated keys, e.g. "config.port".
- Replacement is done via string manipulation on the target line to preserve
  formatting and comments as much as possible. No full parse-serialize is used.

Limitations:
- Only simple top-down nested mappings supported in YAML (no complex anchors, etc).
- TOML path support covers basic tables and dotted keys; arrays/inline tables not fully supported.
- Multiline or complex values may fail gracefully with a clear error.
"""

import argparse
import re
import sys
from pathlib import Path


def detect_format(args_format: str, file_path: Path) -> str:
    if args_format:
        return args_format.lower()
    p = file_path.suffix.lower()
    if p in {".yml", ".yaml"}:
        return "yaml"
    if p == ".toml":
        return "toml"
    return "yaml"  # default


def replace_yaml_path(lines, path_tokens, replacement):
    stack = []  # list of (key, indent)
    updated = False
    for idx, raw in enumerate(lines):
        m = re.match(r"^(\s*)([A-Za-z0-9_-]+)\s*:\s*(.*)?$", raw)
        if m:
            indent = len(m.group(1))
            key = m.group(2)
            rest = m.group(3) if m.group(3) is not None else ""
            # Prune stack to current indentation
            while stack and stack[-1][1] >= indent:
                stack.pop()
            stack.append((key, indent))
            current_path = [k for k, _ in stack]
            if current_path == path_tokens and rest != "":
                # replace value (preserve inline comment if present)
                # Split off inline comment after '#', but avoid '#' in quotes is ignored for simplicity
                val, sep, comment = rest.partition("#")
                new_line = f"{' ' * indent}{key}: {replacement}"
                if comment:
                    new_line += f" #" + comment
                lines[idx] = new_line + ("\n" if not raw.endswith("\n") else "\n")
                updated = True
                # Do not stop; continue to allow only first match
                break
        else:
            # Non-key line: do nothing
            pass
    if not updated:
        raise ValueError(
            "Structured YAML path not found or unable to replace value with the given path: '"
            + ".".join(path_tokens)
            + "'"
        )
    return lines


def replace_toml_path(lines, path_tokens, replacement):
    current_table = []
    updated = False
    for idx, raw in enumerate(lines):
        tbl = re.match(r"^\s*\[([^\]]+)\]\s*$", raw)
        if tbl:
            table_path = tbl.group(1).strip()
            current_table = table_path.split(".") if table_path else []
            continue
        m = re.match(r"^\s*([A-Za-z0-9_-]+)\s*=\s*(.*)$", raw)
        if m:
            key = m.group(1)
            rest = m.group(2)
            full_path = current_table + [key]
            if full_path == path_tokens:
                value, sep, comment = rest.partition("#")
                new_line = f"{key} = {replacement}"
                if comment:
                    new_line += f" #" + comment
                # preserve original indentation
                indent = raw[: len(raw) - len(raw.lstrip())]
                lines[idx] = (
                    indent + new_line + ("\n" if not raw.endswith("\n") else "\n")
                )
                updated = True
                break
    if not updated:
        raise ValueError(
            "Structured TOML path not found or unable to replace value with the given path: '"
            + ".".join(path_tokens)
            + "'"
        )
    return lines


def main():
    ap = argparse.ArgumentParser(
        prog="fedit_structured_yaml_toml",
        description="Structured Key Mode for YAML/TOML (experimental)",
    )
    ap.add_argument("file", help="Target file path to modify")
    ap.add_argument("path", help="Dot-separated path to the key (e.g. config.port)")
    ap.add_argument("replacement", help="New value as string to set for the path")
    ap.add_argument("-f", "--format", dest="fmt", help="Explicit format: yaml or toml")
    ap.add_argument(
        "--strict",
        action="store_true",
        help="Enable strict path resolution (default: enabled)",
    )
    args = ap.parse_args()

    target = Path(args.file)
    if not target.exists():
        print(f"File not found: {target}", file=sys.stderr)
        sys.exit(2)

    fmt = detect_format(args.fmt, target)
    path_tokens = args.path.split(".") if args.path else []
    if not path_tokens:
        print("Invalid path: empty", file=sys.stderr)
        sys.exit(2)

    content = target.read_text(encoding="utf-8")
    lines = content.splitlines(True)
    try:
        if fmt == "yaml":
            lines = replace_yaml_path(lines, path_tokens, args.replacement)
        elif fmt == "toml":
            lines = replace_toml_path(lines, path_tokens, args.replacement)
        else:
            print("Unsupported format: " + fmt, file=sys.stderr)
            sys.exit(2)
    except ValueError as e:
        print("Error:", str(e), file=sys.stderr)
        sys.exit(1)

    target.write_text("".join(lines), encoding="utf-8")
    print(f"Updated {target}")


if __name__ == "__main__":
    main()
