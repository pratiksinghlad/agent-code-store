# /// script
# requires-python = ">=3.12"
# ///
"""Read JSON and print it with sorted keys. Write it atomically with --out."""

from __future__ import annotations

import io
import json
import os
import sys
import tempfile
from pathlib import Path

HELP = """\
usage: json [--path <file>] [--json <text>] [--out <file>]

Read JSON and print it with sorted keys.
Write it atomically when --out is set.
--json overrides --path. With neither, read stdin.
"""


def main(argv: list[str] | None = None) -> int:
    arguments = sys.argv[1:] if argv is None else argv
    if "--help" in arguments or "-h" in arguments:
        sys.stdout.write(HELP)
        return 0
    configure_stdio()
    try:
        return execute(arguments)
    except (OSError, ValueError) as error:
        print(str(error), file=sys.stderr)
        return 1


def configure_stdio() -> None:
    for stream in (sys.stdout, sys.stderr):
        if isinstance(stream, io.TextIOWrapper):
            stream.reconfigure(encoding="utf-8", newline="\n")


def execute(argv: list[str]) -> int:
    path, raw, out = parse_args(argv)
    text = load_text(path, raw)
    body = canonical(parse_json(text, source_label(path, raw)))
    if out is None:
        sys.stdout.write(body)
        return 0
    atomic_write(out, body)
    return 0


def parse_args(argv: list[str]) -> tuple[Path | None, str | None, Path | None]:
    found = collect_flags(argv)
    return (
        optional_path(found.get("--path")),
        found.get("--json"),
        optional_path(found.get("--out")),
    )


def collect_flags(argv: list[str]) -> dict[str, str]:
    found: dict[str, str] = {}
    index = 0
    while index < len(argv):
        flag = argv[index]
        if flag not in {"--path", "--json", "--out"}:
            raise ValueError(f"unknown argument: {flag}")
        found[flag] = flag_value(argv, index, flag)
        index += 2
    return found


def flag_value(argv: list[str], index: int, flag: str) -> str:
    next_index = index + 1
    if next_index >= len(argv) or argv[next_index].startswith("--"):
        raise ValueError(f"missing value for {flag}")
    return argv[next_index]


def optional_path(value: str | None) -> Path | None:
    if value is None:
        return None
    return Path(value)


def source_label(path: Path | None, raw: str | None) -> str:
    if raw is not None:
        return "--json"
    if path is not None:
        return str(path)
    return "stdin"


def load_text(path: Path | None, raw: str | None) -> str:
    if raw is not None:
        return raw
    if path is not None:
        return read_text(path)
    if sys.stdin.isatty():
        raise ValueError("missing json input")
    text = sys.stdin.read()
    if text == "":
        raise ValueError("missing json input")
    return text


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError as error:
        raise ValueError(f"file is not utf-8: {path}") from error


def parse_json(text: str, label: str) -> object:
    try:
        return json.loads(text)
    except json.JSONDecodeError as error:
        raise ValueError(f"invalid json: {label} line {error.lineno}") from error


def canonical(value: object) -> str:
    try:
        text = json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False)
    except (TypeError, ValueError) as error:
        raise ValueError("value is not json") from error
    return f"{text}\n"


def atomic_write(path: Path, content: str) -> None:
    parent = path.parent if path.parent.as_posix() != "" else Path(".")
    parent.mkdir(parents=True, exist_ok=True)
    temporary = write_temp(parent, content)
    replace_file(temporary, path)
    fsync_dir(parent)


def write_temp(parent: Path, content: str) -> Path:
    descriptor, name = tempfile.mkstemp(prefix=".json-", dir=parent)
    temporary = Path(name)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8", newline="\n") as handle:
            handle.write(content)
            handle.flush()
            os.fsync(handle.fileno())
    except OSError:
        temporary.unlink(missing_ok=True)
        raise
    return temporary


def replace_file(temporary: Path, path: Path) -> None:
    try:
        os.replace(temporary, path)
    except OSError:
        temporary.unlink(missing_ok=True)
        raise


def fsync_dir(directory: Path) -> None:
    if os.name == "nt":
        return
    descriptor = os.open(directory, os.O_RDONLY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


if __name__ == "__main__":
    raise SystemExit(main())
