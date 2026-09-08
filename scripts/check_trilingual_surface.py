#!/usr/bin/env python3
"""Validate and report EN/UK/SA public-surface parity.

The current machine registry still has the historical filename
``lib/surface/uk-sa-coverage.wsm``.  ADR-005 defines how to read it as a
trilingual registry: the canonical/EN column is both the semantic identity and
the English human-surface name.  English is therefore not the core authority;
it is one peer surface whose spelling currently coincides with canonical IDs.

Default mode checks structural parity and prints the current matrix.
``--require-complete`` is the release gate: it fails until every
translation-eligible identity is stable in EN, UK and SA.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
REGISTRY = REPO_ROOT / "lib" / "surface" / "uk-sa-coverage.wsm"
ALLOWED_STATUSES = {"stable", "candidate", "missing", "compatibility-only"}


@dataclass(frozen=True)
class Entry:
    category: str
    canonical: str
    en: str
    uk: str
    sa: str
    en_status: str
    uk_status: str
    sa_status: str


def parse_entries(source: str) -> list[Entry]:
    entries: list[Entry] = []
    for line_number, line in enumerate(source.splitlines(), start=1):
        fields = line.split()
        if fields[:1] != ["(entry"]:
            continue
        if len(fields) < 7:
            raise ValueError(f"line {line_number}: malformed registry entry")

        category = fields[1].removeprefix("(")
        canonical, uk, sa = fields[2], fields[3], fields[4]
        uk_status, sa_status = fields[5], fields[6]
        for language, status in (("uk", uk_status), ("sa", sa_status)):
            if status not in ALLOWED_STATUSES:
                raise ValueError(
                    f"line {line_number}: unknown {language} status {status!r}"
                )

        # The legacy table predates an explicit EN status column. ADR-005 makes
        # the English surface explicit without pretending it is the core:
        # translation-eligible canonical names are stable EN names; rows that
        # were intentionally outside the denominator stay compatibility-only.
        en_status = (
            "compatibility-only"
            if uk_status == "compatibility-only"
            else "stable"
        )

        if uk_status in {"stable", "candidate"} and uk == "—":
            raise ValueError(
                f"line {line_number}: {uk_status} UK entry {canonical!r} has no name"
            )
        if uk_status == "missing" and uk != "—":
            raise ValueError(
                f"line {line_number}: missing UK entry {canonical!r} unexpectedly has {uk!r}"
            )
        if sa_status in {"stable", "candidate"} and sa == "—":
            raise ValueError(
                f"line {line_number}: {sa_status} SA entry {canonical!r} has no name"
            )
        if sa_status == "missing" and sa != "—":
            raise ValueError(
                f"line {line_number}: missing SA entry {canonical!r} unexpectedly has {sa!r}"
            )

        entries.append(
            Entry(
                category=category,
                canonical=canonical,
                en=canonical,
                uk=uk,
                sa=sa,
                en_status=en_status,
                uk_status=uk_status,
                sa_status=sa_status,
            )
        )

    if not entries:
        raise ValueError("surface registry contains no entries")
    return entries


def counts(entries: list[Entry], language: str) -> dict[str, int]:
    status_field = f"{language}_status"
    result = {status: 0 for status in ALLOWED_STATUSES}
    for entry in entries:
        result[getattr(entry, status_field)] += 1
    return result


def public_denominator(entries: list[Entry]) -> int:
    # A compatibility-only EN row is deliberately outside the selected public
    # translation surface. All other rows are obligations for every language.
    return sum(entry.en_status != "compatibility-only" for entry in entries)


def stable_percent(language_counts: dict[str, int], denominator: int) -> float:
    if denominator == 0:
        return 100.0
    return 100.0 * language_counts["stable"] / denominator


def is_release_complete(entries: list[Entry]) -> bool:
    public = [entry for entry in entries if entry.en_status != "compatibility-only"]
    return all(
        entry.en_status == entry.uk_status == entry.sa_status == "stable"
        for entry in public
    )


def render_report(entries: list[Entry]) -> str:
    denominator = public_denominator(entries)
    by_language = {language: counts(entries, language) for language in ("en", "uk", "sa")}

    lines = [
        "Trilingual surface parity",
        f"public semantic identities: {denominator}",
        "",
        "surface  stable  candidate  missing  compatibility  stable/public",
    ]
    for language in ("en", "uk", "sa"):
        current = by_language[language]
        lines.append(
            f"{language.upper():<7}"
            f"{current['stable']:>7}"
            f"{current['candidate']:>11}"
            f"{current['missing']:>9}"
            f"{current['compatibility-only']:>15}"
            f"{stable_percent(current, denominator):>13.1f}%"
        )

    trilingual_stable = sum(
        entry.en_status == entry.uk_status == entry.sa_status == "stable"
        for entry in entries
        if entry.en_status != "compatibility-only"
    )
    lines.extend(
        [
            "",
            f"trilingual stable identities: {trilingual_stable}/{denominator}",
            f"release parity: {'CONFIRMED' if is_release_complete(entries) else 'OPEN'}",
        ]
    )
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--require-complete",
        action="store_true",
        help="fail until EN, UK and SA are all stable for every public identity",
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
