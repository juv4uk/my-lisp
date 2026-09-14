#!/usr/bin/env python3
"""Fail-closed semantic-authority guard for host-authored tests (#115).

The guard does not try to understand all Rust syntax and does not ban assertions.
It enforces the machine-readable authority classification from #114:

- mechanism / observer: allowed host-owned checks;
- semantic-authority / mixed: forbidden as host authority;
- unknown files: forbidden until classified.

Lisp-owned witness data remains the only place that may define normative Lisp
values/errors for migrated semantics. See #112/#113.
"""

from __future__ import annotations

import argparse
import csv
import sys
from pathlib import Path

ALLOWED = {"mechanism", "observer"}
FORBIDDEN = {"semantic-authority", "mixed"}
KNOWN = ALLOWED | FORBIDDEN


def load_inventory(path: Path) -> dict[str, set[str]]:
    try:
        with path.open(newline="", encoding="utf-8") as handle:
            reader = csv.DictReader(handle, delimiter="\t")
            required = {"path", "class"}
            if not reader.fieldnames or not required.issubset(reader.fieldnames):
                raise ValueError("inventory must contain path and class columns")
            result: dict[str, set[str]] = {}
            for row in reader:
                item_path = (row.get("path") or "").strip()
                item_class = (row.get("class") or "").strip()
                if not item_path or item_class not in KNOWN:
                    raise ValueError(f"invalid inventory row: {row}")
                result.setdefault(item_path, set()).add(item_class)
            return result
    except (OSError, ValueError) as exc:
        print(f"semantic authority guard: invalid inventory: {exc}", file=sys.stderr)
        raise SystemExit(2) from exc


def repo_relative(path: Path, inventory: Path) -> str:
    repo_root = inventory.resolve().parent.parent
    try:
        return path.resolve().relative_to(repo_root).as_posix()
    except ValueError:
        print(
            f"semantic authority guard: checked file is outside repository: {path}",
            file=sys.stderr,
        )
        raise SystemExit(2)


def check_file(inventory: dict[str, set[str]], relative: str) -> int:
    classes = inventory.get(relative)
    if not classes:
        print(
            f"semantic authority violation: unclassified host test {relative}; "
            "classify it in tests/authority-inventory.tsv. Lisp-owned witnesses "
            "from #113 own normative semantic truth.",
            file=sys.stderr,
        )
        return 1

    if classes & FORBIDDEN:
        joined = ",".join(sorted(classes))
        print(
            f"semantic authority violation: {relative} is classified {joined}; "
            "host code may observe execution but must obtain expected Lisp meaning "
            "from the Lisp-owned witness corpus established by #113.",
            file=sys.stderr,
        )
        return 1

    if not classes <= ALLOWED:
        print(
            f"semantic authority guard: unsupported classification for {relative}: {classes}",
            file=sys.stderr,
        )
        return 2
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--inventory", type=Path, required=True)
    parser.add_argument("--check-file", type=Path, required=True)
    args = parser.parse_args()

    inventory = load_inventory(args.inventory)
    relative = repo_relative(args.check_file, args.inventory)
    return check_file(inventory, relative)


if __name__ == "__main__":
    raise SystemExit(main())
