#!/usr/bin/env python3
"""Validate semantic ownership inventory and deterministically render its report."""

from __future__ import annotations

import argparse
import collections
import re
import shlex
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MAP_PATH = ROOT / "knowledge" / "semantic-ownership.wsm"
REPORT_PATH = ROOT / "docs" / "semantic-ownership-report.md"

CLASSES = {
    "canon-ground",
    "canon-operation",
    "necessary-form",
    "host-observation",
    "host-authorization",
    "host-mechanism",
    "lisp-owned",
    "derived-tooling",
    "unknown",
}
STATUSES = {"confirmed", "partial", "broken", "unknown"}
LAYERS = {
    "canon",
    "bootstrap",
    "stdlib",
    "reasoning",
    "knowledge",
    "self-hosting",
    "host-capability",
    "tooling",
}
HEX40 = re.compile(r"^[0-9a-f]{40}$")


@dataclass(frozen=True)
class Ownership:
    key: str
    semantic_id: str
    owner_class: str
    layer: str
    status: str
    policy_candidate: str
    behavior: str
    implementation_paths: str
    evidence_paths: str
    previous_owner: str
    migration_ref: str


@dataclass(frozen=True)
class Migration:
    key: str
    from_owner: str
    to_owner: str
    status: str
    commit: str
    behavior: str
    evidence_paths: str


def forms(path: Path) -> list[list[str]]:
    parsed: list[list[str]] = []
    for number, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith(";"):
            continue
        if not (line.startswith("(") and line.endswith(")")):
            raise ValueError(f"{path}:{number}: expected one complete form per line")
        try:
            tokens = shlex.split(line[1:-1], comments=False, posix=True)
        except ValueError as error:
            raise ValueError(f"{path}:{number}: {error}") from error
        if tokens:
            parsed.append(tokens)
    return parsed


def split_paths(text: str) -> list[str]:
    if text == "-":
        return []
    return [item for item in text.split(";") if item]


def require_paths(paths: list[str], context: str) -> None:
    for relative in paths:
        target = ROOT / relative
        if not target.exists():
            raise ValueError(f"{context}: referenced path does not exist: {relative}")


def load() -> tuple[list[Ownership], list[Migration]]:
    ownership: list[Ownership] = []
    migrations: list[Migration] = []
    seen_keys: set[str] = set()
    seen_semantic: dict[str, str] = {}

    for tokens in forms(MAP_PATH):
        tag = tokens[0]
        if tag in {"schema", "as-of", "scope"}:
            continue
        if tag == "ownership":
            if len(tokens) != 12:
                raise ValueError(
                    f"ownership row {tokens[1] if len(tokens) > 1 else '?'}: "
                    f"expected 12 fields, got {len(tokens)}"
                )
            row = Ownership(*tokens[1:])
            if row.key in seen_keys:
                raise ValueError(f"duplicate ownership key: {row.key}")
            seen_keys.add(row.key)
            if row.owner_class not in CLASSES:
                raise ValueError(f"{row.key}: unknown ownership class {row.owner_class}")
            if row.layer not in LAYERS:
                raise ValueError(f"{row.key}: unknown layer {row.layer}")
            if row.status not in STATUSES:
                raise ValueError(f"{row.key}: unknown status {row.status}")
            if row.policy_candidate not in {"yes", "no"}:
                raise ValueError(f"{row.key}: policy candidate must be yes/no")
            if row.semantic_id != "-":
                previous = seen_semantic.get(row.semantic_id)
                if previous is not None:
                    raise ValueError(
                        f"semantic identity {row.semantic_id} claimed by both "
                        f"{previous} and {row.key}; split layered meaning explicitly"
                    )
                seen_semantic[row.semantic_id] = row.key
            require_paths(split_paths(row.implementation_paths), row.key)
            require_paths(split_paths(row.evidence_paths), row.key)
            if row.status == "confirmed" and not split_paths(row.evidence_paths):
                raise ValueError(f"{row.key}: confirmed row requires executable evidence")
            if (row.previous_owner == "-") != (row.migration_ref == "-"):
                raise ValueError(
                    f"{row.key}: previous_owner and migration_ref must be present together"
                )
            if row.migration_ref != "-" and not HEX40.fullmatch(row.migration_ref):
                raise ValueError(f"{row.key}: migration_ref must be a full commit SHA")
            ownership.append(row)
        elif tag == "migration":
            if len(tokens) != 8:
                raise ValueError(
                    f"migration row {tokens[1] if len(tokens) > 1 else '?'}: "
                    f"expected 8 fields, got {len(tokens)}"
                )
            row = Migration(*tokens[1:])
            if row.key in seen_keys:
                raise ValueError(f"duplicate key across ownership/migration rows: {row.key}")
            seen_keys.add(row.key)
            if row.status not in STATUSES:
                raise ValueError(f"{row.key}: unknown status {row.status}")
            if row.status == "confirmed" and not HEX40.fullmatch(row.commit):
                raise ValueError(f"{row.key}: confirmed migration requires a full commit SHA")
            require_paths(split_paths(row.evidence_paths), row.key)
            if row.status == "confirmed" and not split_paths(row.evidence_paths):
                raise ValueError(f"{row.key}: confirmed migration requires current evidence")
            migrations.append(row)
        else:
            raise ValueError(f"unknown top-level form: {tag}")

    if not ownership:
        raise ValueError("ownership inventory is empty")
    return ownership, migrations


def count_rows(rows: list[Ownership], attr: str) -> list[tuple[str, int]]:
    counts = collections.Counter(getattr(row, attr) for row in rows)
    return sorted(counts.items())


def table(rows: list[tuple[str, int]]) -> str:
    out = ["| category | audited rows |", "|---|---:|"]
    out.extend(f"| `{name}` | {count} |" for name, count in rows)
    return "\n".join(out)


def render(ownership: list[Ownership], migrations: list[Migration]) -> str:
    confirmed_migrations = [row for row in migrations if row.status == "confirmed"]
    host_policy_candidates = [
        row
        for row in ownership
        if row.owner_class == "host-mechanism" and row.policy_candidate == "yes"
    ]
    irreducible_host = [
        row
        for row in ownership
        if row.status == "confirmed"
        and row.policy_candidate == "no"
        and row.owner_class in {"host-observation", "host-authorization", "host-mechanism"}
    ]
    unknown_rows = [row for row in ownership if row.status == "unknown"]

    lines = [
        "# Semantic ownership report",
        "",
        "> Generated deterministically from `knowledge/semantic-ownership.wsm`.",
        "> This report counts **audited behaviors/responsibilities**, not LOC and not total language completeness.",
        "> No number below is a “self-hosting percentage”.",
        "",
        "## Summary",
        "",
        f"- Audited ownership rows: **{len(ownership)}**",
        f"- Confirmed ownership rows: **{sum(row.status == 'confirmed' for row in ownership)}**",
        f"- Partial ownership rows: **{sum(row.status == 'partial' for row in ownership)}**",
        f"- Confirmed migration ledger entries: **{len(confirmed_migrations)}**",
        f"- Remaining host semantic-policy candidates: **{len(host_policy_candidates)}**",
        f"- Confirmed irreducible host mechanism/observation/authorization rows: **{len(irreducible_host)}**",
        f"- Unknown ownership rows: **{len(unknown_rows)}**",
        "",
        "## Ownership classes",
        "",
        table(count_rows(ownership, "owner_class")),
        "",
        "## Layers",
        "",
        table(count_rows(ownership, "layer")),
        "",
        "## Epistemic status",
        "",
        table(count_rows(ownership, "status")),
        "",
        "## Host-policy candidates",
        "",
    ]
    if host_policy_candidates:
        lines.extend(
            f"- `{row.key}` — {row.behavior} (`{row.status}`)"
            for row in sorted(host_policy_candidates, key=lambda item: item.key)
        )
    else:
        lines.append("- none")

    lines.extend(["", "## Confirmed migration ledger", ""])
    if confirmed_migrations:
        lines.extend(
            f"- `{row.key}` — `{row.from_owner}` → `{row.to_owner}` at `{row.commit}`: {row.behavior}"
            for row in sorted(confirmed_migrations, key=lambda item: item.key)
        )
    else:
        lines.append("- none")

    lines.extend(
        [
            "",
            "## Audited behaviors",
            "",
            "| key | semantic id | owner | layer | status | behavior |",
            "|---|---|---|---|---|---|",
        ]
    )
    for row in sorted(ownership, key=lambda item: item.key):
        semantic = row.semantic_id if row.semantic_id != "-" else "—"
        lines.append(
            f"| `{row.key}` | `{semantic}` | `{row.owner_class}` | `{row.layer}` | "
            f"`{row.status}` | {row.behavior} |"
        )

    lines.extend(
        [
            "",
            "## Interpretation rule",
            "",
            "The denominator of every count is the checked-in audited inventory above. "
            "A larger `lisp-owned` count is not automatically progress, and a host-owned "
            "observation or authorization boundary is not automatically debt. Ownership "
            "changes are progress only when they remove duplicate semantic authority or "
            "move policy to the layer that can own it without weakening evidence.",
            "",
        ]
    )
    return "\n".join(lines)


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--check", action="store_true", help="validate map and checked-in report")
    group.add_argument("--write", action="store_true", help="validate map and rewrite report")
    return parser.parse_args()


def main() -> int:
    args = arguments()
    try:
        ownership, migrations = load()
        report = render(ownership, migrations)
        if args.write:
            REPORT_PATH.write_text(report, encoding="utf-8")
        else:
            existing = REPORT_PATH.read_text(encoding="utf-8")
            if existing != report:
                raise ValueError(
                    "semantic ownership report drift: run "
                    "`python3 scripts/semantic-ownership.py --write`"
                )
    except (OSError, ValueError) as error:
        print(f"semantic ownership check failed: {error}", file=sys.stderr)
        return 1

    print(
        f"semantic ownership: {len(ownership)} rows, "
        f"{len(migrations)} migrations, report synchronized"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
