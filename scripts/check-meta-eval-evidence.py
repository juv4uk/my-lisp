#!/usr/bin/env python3
"""Перевіряє machine-readable meta-eval evidence matrix для issue #26."""

from __future__ import annotations

import shlex
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MATRIX = ROOT / "knowledge" / "meta-eval-evidence.wsm"
STATUSES = {"confirmed", "partial", "broken", "unknown"}
CLAIM_STATES = {"allowed", "forbidden"}

REQUIRED_ROWS = {
    "canon-resolution-precedence",
    "canon-binding-rejection",
    "quote",
    "cond-short-circuit",
    "symbol-lookup-unknown-symbol",
    "lambda-construction",
    "fixed-arity",
    "variadic-bare-symbol-lambda",
    "dotted-rest-lambda",
    "lexical-capture",
    "ordinary-noncanon-shadowing",
    "function-application-order",
    "primitive-identity-bridge",
    "arithmetic-comparison-bridge",
    "chained-comparisons",
    "def-compatibility",
    "define-form",
    "macro-recognition-expansion",
    "macro-arity-error",
    "self-recursive-definitions",
    "finite-mutual-recursion",
    "three-member-mutual-recursion",
    "recursive-group-forward-references",
    "recursive-group-member-shadowing",
    "recursive-group-captured-environment",
    "nested-closures-in-recursive-functions",
    "adjacent-nonrecursive-defs-not-false-grouped",
    "malformed-recursive-group",
    "arbitrary-later-binding-visibility",
    "error-kind-parity",
    "error-detail-parity",
    "data-to-code-boundary",
    "first-class-evaluation",
}
REQUIRED_CLAIMS = {
    "metacircular-evaluator-exists",
    "self-hosting-witness",
    "partial-self-hosting",
    "complete-self-hosting",
}


def forms() -> list[list[str]]:
    out: list[list[str]] = []
    for number, raw in enumerate(MATRIX.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith(";"):
            continue
        if not (line.startswith("(") and line.endswith(")")):
            raise ValueError(f"{MATRIX}:{number}: очікувалася одна завершена форма")
        try:
            tokens = shlex.split(line[1:-1], comments=False, posix=True)
        except ValueError as error:
            raise ValueError(f"{MATRIX}:{number}: {error}") from error
        if tokens:
            out.append(tokens)
    return out


def paths(text: str) -> list[str]:
    return [item for item in text.split(";") if item and item != "-"]


def require_existing(items: list[str], key: str, side: str) -> None:
    for item in items:
        if not (ROOT / item).exists():
            raise ValueError(f"{key}: {side} evidence path не існує: {item}")


def main() -> int:
    try:
        rows: dict[str, tuple[str, str]] = {}
        claims: dict[str, str] = {}
        status_counts: Counter[str] = Counter()

        for tokens in forms():
            tag = tokens[0]
            if tag in {"schema", "as-of"}:
                continue
            if tag == "row":
                if len(tokens) != 8:
                    raise ValueError(f"row {tokens[1] if len(tokens) > 1 else '?'}: очікувалося 8 полів")
                _, key, required, status, reference, meta, divergence, missing = tokens
                if key in rows:
                    raise ValueError(f"duplicate row: {key}")
                if required not in {"yes", "no"}:
                    raise ValueError(f"{key}: REQUIRED мусить бути yes/no")
                if status not in STATUSES:
                    raise ValueError(f"{key}: невідомий status {status}")
                ref_paths = paths(reference)
                meta_paths = paths(meta)
                require_existing(ref_paths, key, "reference")
                require_existing(meta_paths, key, "meta")
                if status == "confirmed":
                    if not ref_paths or not meta_paths:
                        raise ValueError(f"{key}: confirmed row потребує evidence з обох боків")
                    if missing != "немає":
                        raise ValueError(f"{key}: confirmed row не може мати missing proof: {missing}")
                else:
                    if missing == "немає":
                        raise ValueError(f"{key}: {status} row мусить назвати smallest missing proof")
                    if not divergence:
                        raise ValueError(f"{key}: unresolved row мусить описати divergence")
                rows[key] = (required, status)
                status_counts[status] += 1
                continue
            if tag == "claim":
                if len(tokens) != 4:
                    raise ValueError(f"claim {tokens[1] if len(tokens) > 1 else '?'}: очікувалося 4 поля")
                _, name, state, _rule = tokens
                if name in claims:
                    raise ValueError(f"duplicate claim: {name}")
                if state not in CLAIM_STATES:
                    raise ValueError(f"{name}: claim state мусить бути allowed/forbidden")
                claims[name] = state
                continue
            raise ValueError(f"невідома top-level форма: {tag}")

        missing_rows = REQUIRED_ROWS - rows.keys()
        extra_required = {key for key, (required, _) in rows.items() if required == "yes"} - REQUIRED_ROWS
        if missing_rows:
            raise ValueError(f"відсутні required evidence rows: {sorted(missing_rows)}")
        if extra_required:
            raise ValueError(f"нові required rows мусять бути явно додані до checker scope: {sorted(extra_required)}")

        missing_claims = REQUIRED_CLAIMS - claims.keys()
        if missing_claims:
            raise ValueError(f"відсутні claim vocabulary entries: {sorted(missing_claims)}")

        unresolved_required = sorted(
            key for key, (required, status) in rows.items()
            if required == "yes" and status != "confirmed"
        )
        if unresolved_required and claims.get("complete-self-hosting") != "forbidden":
            raise ValueError("complete-self-hosting мусить бути forbidden, поки є unresolved required rows")
        if unresolved_required and claims.get("partial-self-hosting") != "allowed":
            raise ValueError("partial-self-hosting має лишатися allowed vocabulary для чесного current state")
        if claims.get("self-hosting-witness") != "allowed":
            raise ValueError("self-hosting-witness має лишатися дозволеним лише для proven subset")

        print(
            "meta-eval evidence: "
            f"{len(rows)} rows; "
            + ", ".join(f"{name}={status_counts[name]}" for name in sorted(STATUSES))
            + f"; unresolved-required={len(unresolved_required)}; "
            f"complete-self-hosting={claims['complete-self-hosting']}"
        )
        if unresolved_required:
            print("unresolved required rows: " + ", ".join(unresolved_required))
        return 0
    except (OSError, ValueError) as error:
        print(f"meta-eval evidence check failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
