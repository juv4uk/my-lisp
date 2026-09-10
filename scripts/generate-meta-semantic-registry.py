#!/usr/bin/env python3
"""Generate the Lisp surface→semantic-ID projection used by meta-eval.

The only spelling authority is lib/surface/semantic-registry.wsm. This script
owns projection mechanics only. It admits exactly stable and
compatibility-only surfaces, matching the native semantic registry policy.

The exact apostrophe token (') is intentionally absent from the runtime table:
the reader consumes it as quote syntax before symbol lookup. Apostrophes inside
ordinary identifiers remain valid and are emitted unchanged.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REGISTRY = ROOT / "lib" / "surface" / "semantic-registry.wsm"
OUTPUT = ROOT / "lib" / "generated" / "meta-semantic-registry.my"

ENTRY = re.compile(r"^\s*\(([0-9]{4,})\s+(.*)\)\s*$")
SURFACE = re.compile(
    r"\(([A-Za-z][A-Za-z0-9-]*)\s+([^\s()]+)\s+"
    r"(stable|candidate|missing|compatibility-only)\)"
)
ADMITTED = {"stable", "compatibility-only"}
READER_ONLY = {"'"}


def admitted_entries() -> list[tuple[str, str, str]]:
    entries: list[tuple[str, str, str]] = []
    spelling_to_id: dict[str, str] = {}

    for line_number, line in enumerate(REGISTRY.read_text(encoding="utf-8").splitlines(), 1):
        match = ENTRY.match(line)
        if not match:
            continue
        semantic_id, body = match.groups()
        for surface in SURFACE.finditer(body):
            namespace, spelling, status = surface.groups()
            if status not in ADMITTED or spelling == "—" or spelling in READER_ONLY:
                continue
            if any(ch.isspace() or ch in '();"' for ch in spelling):
                raise ValueError(
                    f"{REGISTRY}:{line_number}: runtime surface {spelling!r} "
                    "cannot be emitted as one Lisp symbol"
                )
            previous = spelling_to_id.get(spelling)
            if previous is not None and previous != semantic_id:
                raise ValueError(
                    f"ambiguous admitted surface {spelling!r}: {previous} vs {semantic_id}"
                )
            if previous is None:
                spelling_to_id[spelling] = semantic_id
                entries.append((semantic_id, namespace, spelling))

    if not entries:
        raise ValueError("numeric semantic registry has no admitted runtime surfaces")
    return entries


def render(entries: list[tuple[str, str, str]]) -> str:
    lines = [
        "; GENERATED FILE — DO NOT EDIT.",
        "; Source authority: lib/surface/semantic-registry.wsm",
        "; Generator: scripts/generate-meta-semantic-registry.py",
        "; stable + compatibility-only runtime surfaces only; exact ' is reader syntax.",
        "",
        "(def my-semantic-surface-registry",
        "  (quote (",
    ]
    for semantic_id, namespace, spelling in entries:
        lines.append(f"    ({spelling} \"{semantic_id}\") ; {namespace}")
    lines.extend(
        [
            "  )))",
            "",
            "(def my-semantic-id-for-surface",
            "  (lambda (name)",
            "    (let ((entry (assoc name my-semantic-surface-registry)))",
            "      (cond",
            "        ((atom entry) (quote ()))",
            "        (t (second entry))))))",
            "",
        ]
    )
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail if the committed projection differs from generated output",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        generated = render(admitted_entries())
        if args.check:
            current = OUTPUT.read_text(encoding="utf-8")
            if current != generated:
                print(
                    f"{OUTPUT} is stale; run scripts/generate-meta-semantic-registry.py",
                    file=sys.stderr,
                )
                return 1
            print(f"meta semantic registry projection is current: {OUTPUT}")
            return 0

        OUTPUT.parent.mkdir(parents=True, exist_ok=True)
        OUTPUT.write_text(generated, encoding="utf-8")
        print(f"wrote {OUTPUT}")
        return 0
    except (OSError, ValueError) as error:
        print(f"meta semantic registry generation failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
