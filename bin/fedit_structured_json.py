#!/usr/bin/env python3
import argparse
import json
import os
import re
import sys
import tempfile


class EditError(Exception):
    pass


class InvalidKeyPath(EditError):
    pass


class AmbiguousPath(EditError):
    pass


def parse_path(path_str: str):
    # Supports keys like root.child[0].name
    steps = []
    i = 0
    while i < len(path_str):
        c = path_str[i]
        if c == ".":
            i += 1
            continue
        if c == "[":
            j = path_str.find("]", i)
            if j == -1:
                raise InvalidKeyPath(path_str)
            idx = int(path_str[i + 1 : j])
            steps.append(("idx", idx))
            i = j + 1
            continue
        # key name until '.' or '['
        j = i
        while j < len(path_str) and path_str[j] not in ".[":
            j += 1
        key = path_str[i:j]
        steps.append(("key", key))
        i = j
    return steps


def navigate_parent(data, steps):
    cur = data
    for st in steps[:-1]:
        if st[0] == "key":
            if isinstance(cur, dict) and st[1] in cur:
                cur = cur[st[1]]
            else:
                raise InvalidKeyPath()
        else:
            if isinstance(cur, list) and 0 <= st[1] < len(cur):
                cur = cur[st[1]]
            else:
                raise InvalidKeyPath()
    last = steps[-1]
    return cur, last


def set_value(parent, last, new_value, path_str):
    if last[0] == "key":
        if isinstance(parent, dict) and last[1] in parent:
            parent[last[1]] = new_value
        else:
            raise InvalidKeyPath(path_str)
    else:
        if isinstance(parent, list) and 0 <= last[1] < len(parent):
            parent[last[1]] = new_value
        else:
            raise InvalidKeyPath(path_str)


def detect_indent(text: str) -> int:
    for line in text.splitlines():
        if not line.strip():
            continue
        m = re.match(r"^( +|\t+)", line)
        if m:
            return len(m.group(1))
        break
    return 2


def main():
    parser = argparse.ArgumentParser(
        prog="fedit-structured-json",
        description="Structured JSON key-path replacement mode",
    )
    parser.add_argument("target", help="Target JSON file")
    parser.add_argument(
        "path", help="Key path to replace (e.g., config.port or items[0].name)"
    )
    parser.add_argument("replace", help="New value (JSON value) to set at the path")
    parser.add_argument(
        "-s", "--structured", action="store_true", help="Enable structured-key mode"
    )
    parser.add_argument(
        "-e", "--encoding", default="utf-8", help="File encoding (default utf-8)"
    )
    parser.add_argument(
        "--multiple",
        action="store_true",
        help="Replace all matches (not used in this minimal prototype)",
    )
    args = parser.parse_args()

    target = args.target
    path = args.path
    replace_str = args.replace
    encoding = args.encoding

    # Non-structured mode falls back to a simple replace (not the focus of this story)
    if not args.structured:
        try:
            with open(target, "r", encoding=encoding) as f:
                content = f.read()
        except FileNotFoundError:
            print(f"No such file: {target}", file=sys.stderr)
            sys.exit(1)
        new_content = content.replace(path, replace_str)
        if new_content == content:
            print(f"No matches found for: {path}", file=sys.stderr)
            sys.exit(1)
        # Atomic write
        dirn = os.path.dirname(os.path.abspath(target)) or "."
        fd, tmp_path = tempfile.mkstemp(dir=dirn)
        try:
            with os.fdopen(fd, "w", encoding=encoding) as tmpf:
                tmpf.write(new_content)
                tmpf.flush()
                os.fsync(tmpf.fileno())
            os.replace(tmp_path, target)
        except Exception as e:
            if os.path.exists(tmp_path):
                try:
                    os.remove(tmp_path)
                except Exception:
                    pass
            print(f"Error writing file: {e}", file=sys.stderr)
            sys.exit(1)
        sys.exit(0)

    # Structured JSON path replacement
    try:
        with open(target, "r", encoding=encoding) as f:
            original_text = f.read()
        data = json.loads(original_text)
    except FileNotFoundError:
        print(f"No such file: {target}", file=sys.stderr)
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Invalid JSON content in {target}: {e}", file=sys.stderr)
        sys.exit(1)

    try:
        steps = parse_path(path)
        parent, last = navigate_parent(data, steps)
        new_value = json.loads(replace_str)
        set_value(parent, last, new_value, path)
    except InvalidKeyPath:
        print(f"InvalidKeyPath: {path}", file=sys.stderr)
        sys.exit(1)
    except AmbiguousPath:
        print(f"AmbiguousPath: {path}", file=sys.stderr)
        sys.exit(1)
    except json.JSONDecodeError:
        print(f"Invalid JSON value for replacement: {replace_str}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error applying structured replacement: {e}", file=sys.stderr)
        sys.exit(1)

    indent = detect_indent(original_text)
    try:
        new_text = json.dumps(data, indent=indent, ensure_ascii=False)
    except Exception:
        new_text = json.dumps(data, indent=2, ensure_ascii=False)

    # Preserve final newline if present
    if original_text.endswith("\n") and not new_text.endswith("\n"):
        new_text += "\n"

    dirn = os.path.dirname(os.path.abspath(target)) or "."
    fd, tmp_path = tempfile.mkstemp(dir=dirn)
    try:
        with os.fdopen(fd, "w", encoding=encoding) as tmpf:
            tmpf.write(new_text)
            tmpf.flush()
            os.fsync(tmpf.fileno())
        os.replace(tmp_path, target)
    except Exception as e:
        if os.path.exists(tmp_path):
            try:
                os.remove(tmp_path)
            except Exception:
                pass
        print(f"Error writing file: {e}", file=sys.stderr)
        sys.exit(1)

    sys.exit(0)


if __name__ == "__main__":
    main()
