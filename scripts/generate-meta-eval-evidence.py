#!/usr/bin/env python3
"""Generate docs/meta-eval-evidence.md from the authoritative WSM matrix.

`knowledge/meta-eval-evidence.wsm` owns row status and claim vocabulary. This
script owns only the human-readable Markdown projection. The generated document
must never become a second semantic/status authority.
"""

from __future__ import annotations

import argparse
import shlex
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MATRIX = ROOT / "knowledge" / "meta-eval-evidence.wsm"
OUTPUT = ROOT / "docs" / "meta-eval-evidence.md"


@dataclass(frozen=True)
class Row:
    key: str
    required: str
    status: str
    reference: str
    meta: str
    divergence: str
    missing: str


@dataclass(frozen=True)
class Claim:
    name: str
    state: str
    rule: str


def forms() -> list[list[str]]:
    parsed: list[list[str]] = []
    for number, raw in enumerate(MATRIX.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith(";"):
            continue
        if not (line.startswith("(") and line.endswith(")")):
            raise ValueError(f"{MATRIX}:{number}: expected one complete form")
        try:
            tokens = shlex.split(line[1:-1], comments=False, posix=True)
        except ValueError as error:
            raise ValueError(f"{MATRIX}:{number}: {error}") from error
        if tokens:
            parsed.append(tokens)
    return parsed


def load() -> tuple[str, list[Row], list[Claim]]:
    as_of = "unknown"
    rows: list[Row] = []
    claims: list[Claim] = []

    for tokens in forms():
        if tokens[0] == "as-of" and len(tokens) == 2:
            as_of = tokens[1]
        elif tokens[0] == "row" and len(tokens) == 8:
            _, key, required, status, reference, meta, divergence, missing = tokens
            rows.append(Row(key, required, status, reference, meta, divergence, missing))
        elif tokens[0] == "claim" and len(tokens) == 4:
            _, name, state, rule = tokens
            claims.append(Claim(name, state, rule))

    if not rows:
        raise ValueError("evidence matrix has no rows")
    if not claims:
        raise ValueError("evidence matrix has no claim vocabulary")
    return as_of, rows, claims


def cell(text: str) -> str:
    return text.replace("|", "\\|").replace("\n", "<br>")


def evidence_paths(text: str) -> str:
    items = [item for item in text.split(";") if item and item != "-"]
    if not items:
        return "—"
    return "<br>".join(f"`{cell(item)}`" for item in items)


def render(as_of: str, rows: list[Row], claims: list[Claim]) -> str:
    counts = Counter(row.status for row in rows)
    required = [row for row in rows if row.required == "yes"]
    unresolved = [row.key for row in required if row.status != "confirmed"]

    lines = [
        "# Межа self-hosting для `lib/meta-eval.my`",
        "",
        "> **GENERATED FILE — DO NOT EDIT BY HAND.**",
        "> Джерело статусів і claim vocabulary: `knowledge/meta-eval-evidence.wsm`.",
        "> Генератор: `scripts/generate-meta-eval-evidence.py`.",
        "",
        "Цей Markdown є лише людською проєкцією machine-readable evidence matrix. Він не створює нової семантичної влади: змінювати статуси треба в `.wsm`, після чого перегенерувати цей файл.",
        "",
        f"**Стан на:** `{as_of}`  ",
        f"**Required rows:** {len(required)} · **confirmed:** {sum(row.status == 'confirmed' for row in required)} · **unresolved:** {len(unresolved)}  ",
        "**Усі статуси:** " + ", ".join(f"`{name}`={counts[name]}" for name in ("confirmed", "partial", "broken", "unknown")),
        "",
    ]
    if unresolved:
        lines.append("**Нерозв'язані required rows:** " + ", ".join(f"`{key}`" for key in unresolved))
    else:
        lines.append("**Нерозв'язані required rows:** немає.")

    lines.extend(
        [
            "",
            "## Evidence matrix",
            "",
            "| Поведінка | Required | Статус | Reference evidence | Meta evidence | Розбіжність / межа | Найменший відсутній доказ |",
            "|---|:---:|:---:|---|---|---|---|",
        ]
    )
    for row in rows:
        lines.append(
            "| "
            + " | ".join(
                [
                    f"`{cell(row.key)}`",
                    cell(row.required),
                    f"**{cell(row.status)}**",
                    evidence_paths(row.reference),
                    evidence_paths(row.meta),
                    cell(row.divergence),
                    cell(row.missing),
                ]
            )
            + " |"
        )

    lines.extend(
        [
            "",
            "## Claim vocabulary",
            "",
            "| Твердження | Стан | Правило |",
            "|---|:---:|---|",
        ]
    )
    for claim in claims:
        lines.append(
            f"| `{cell(claim.name)}` | **{cell(claim.state)}** | {cell(claim.rule)} |"
        )

    lines.extend(
        [
            "",
            "## Named errors: correspondence, а не текстова тотожність",
            "",
            "У paired proofs reference-side `ErrorKind` перевіряється як контрактна категорія, а Lisp meta-evaluator повертає data-level observation. Поточні mappings у covered corpus:",
            "",
            "```text",
            "UnknownSymbol ↔ unbound-symbol",
            "Type          ↔ not-callable",
            "Arity         ↔ arity",
            "InvalidForm   ↔ invalid-form",
            "```",
            "",
            "ADR-011 фіксує межу detail parity: Rust `message`/`span` і meta Lisp `detail` є діагностичними поверхнями, доки окремий майбутній контракт не ратифікує спільну structured-detail schema. Їх не можна мовчки нормалізувати в удавану семантичну рівність і не треба видаляти лише заради зовнішньої схожості.",
            "",
            "## Правило інтерпретації",
            "",
            "Differential mismatch — це **finding**, а не автоматично «meta-eval неправильний». Якщо reference runtime суперечить `language-contract.my` або ратифікованому ADR, під підозрою reference implementation. Сила self-hosting тверджень обмежується machine matrix і окремим claim vocabulary вище.",
            "",
            "Після редагування `knowledge/meta-eval-evidence.wsm` запустіть:",
            "",
            "```bash",
            "python3 scripts/generate-meta-eval-evidence.py",
            "python3 scripts/generate-meta-eval-evidence.py --check",
            "```",
            "",
        ]
    )
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail if the committed Markdown differs from the WSM projection",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        generated = render(*load())
        if args.check:
            current = OUTPUT.read_text(encoding="utf-8")
            if current != generated:
                print(
                    f"{OUTPUT} is stale; run scripts/generate-meta-eval-evidence.py",
                    file=sys.stderr,
                )
                return 1
            print(f"meta-eval evidence projection is current: {OUTPUT}")
            return 0

        OUTPUT.write_text(generated, encoding="utf-8")
        print(f"wrote {OUTPUT}")
        return 0
    except (OSError, ValueError) as error:
        print(f"meta-eval evidence generation failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
