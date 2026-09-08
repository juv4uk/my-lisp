#!/usr/bin/env python3
"""Перевіряє рівноправність UK/EN/SA від numeric semantic authority.

Людські поверхні рахуються від semantic identities, а не від словника EN.
`sym` є спільною немовною нотацією і не зараховується жодній людській мові.
"""

from __future__ import annotations

import argparse
import re
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
REGISTRY = REPO_ROOT / "lib" / "surface" / "semantic-registry.wsm"
ALLOWED_STATUSES = {"stable", "candidate", "missing", "compatibility-only"}
HUMAN_SURFACES = ("uk", "en", "sa")
ENTRY = re.compile(r"^\s*\(([0-9]{4,})\s+(.*)\)\s*$")
SURFACE = re.compile(
    r"\(([A-Za-z][A-Za-z0-9-]*)\s+([^\s()]+)\s+"
    r"(stable|candidate|missing|compatibility-only)\)"
)


@dataclass(frozen=True)
class Entry:
    identity: str
    surfaces: dict[str, tuple[str, str]]


def parse_entries(source: str) -> list[Entry]:
    entries: list[Entry] = []
    seen: set[str] = set()
    for line_number, line in enumerate(source.splitlines(), start=1):
        match = ENTRY.match(line)
        if not match:
            continue
        identity, body = match.groups()
        if identity in seen:
            raise ValueError(f"line {line_number}: duplicate semantic ID {identity}")
        seen.add(identity)

        surfaces: dict[str, tuple[str, str]] = {}
        matches = list(SURFACE.finditer(body))
        residue = SURFACE.sub("", body).strip()
        if not matches or residue:
            raise ValueError(f"line {line_number}: malformed semantic entry {identity}")
        for item in matches:
            language, name, status = item.groups()
            if language in surfaces:
                raise ValueError(f"line {line_number}: duplicate {language} in {identity}")
            if status not in ALLOWED_STATUSES:
                raise ValueError(f"{identity}/{language}: unknown status {status}")
            surfaces[language] = (name, status)
        missing = set(HUMAN_SURFACES) - surfaces.keys()
        if missing:
            raise ValueError(f"{identity}: missing explicit human surfaces {sorted(missing)}")
        entries.append(Entry(identity, surfaces))

    if not entries:
        raise ValueError("numeric semantic registry contains no entries")
    return entries


def status(entry: Entry, language: str) -> str:
    return entry.surfaces[language][1]


def is_public(entry: Entry) -> bool:
    # Нейтральний знаменник: identity є compatibility-only лише тоді, коли
    # ВСІ людські поверхні явно кажуть compatibility-only.
    return not all(status(entry, language) == "compatibility-only" for language in HUMAN_SURFACES)


def counts(entries: list[Entry], language: str) -> dict[str, int]:
    result = {item: 0 for item in ALLOWED_STATUSES}
    for entry in entries:
        result[status(entry, language)] += 1
    return result


def is_release_complete(entries: list[Entry]) -> bool:
    return all(
        all(status(entry, language) == "stable" for language in HUMAN_SURFACES)
        for entry in entries
        if is_public(entry)
    )


def render_report(entries: list[Entry]) -> str:
    public = [entry for entry in entries if is_public(entry)]
    denominator = len(public)
    by_language = {language: counts(entries, language) for language in HUMAN_SURFACES}
    symbolic = sum("sym" in entry.surfaces for entry in entries)

    lines = [
        "Рівноправність людських поверхонь від numeric semantic authority",
        f"public semantic identities: {denominator}",
        f"shared symbolic identities: {symbolic}",
        "",
        "surface  stable  candidate  missing  compatibility  stable/public",
    ]
    for language in HUMAN_SURFACES:
        current = by_language[language]
        percent = 100.0 if denominator == 0 else 100.0 * current["stable"] / denominator
        lines.append(
            f"{language.upper():<7}"
            f"{current['stable']:>7}"
            f"{current['candidate']:>11}"
            f"{current['missing']:>9}"
            f"{current['compatibility-only']:>15}"
            f"{percent:>13.1f}%"
        )

    common_stable = sum(
        all(status(entry, language) == "stable" for language in HUMAN_SURFACES)
        for entry in public
    )
    lines.extend(
        [
            "",
            f"trilingual stable identities: {common_stable}/{denominator}",
            f"release parity: {'CONFIRMED' if is_release_complete(entries) else 'OPEN'}",
        ]
    )
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--require-complete",
        action="store_true",
        help="fail until UK, EN and SA are stable for every public numeric identity",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        entries = parse_entries(REGISTRY.read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        print(f"surface registry error: {error}")
        return 2

    print(render_report(entries))
    if args.require_complete and not is_release_complete(entries):
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
