#!/usr/bin/env python3
"""Filters tests/fixtures/conformance.json down to one tier, on demand.

conformance.json deliberately stays one file (decided 2026-08-09, see
PLAN.md item 4) — physically splitting it into language-core.json/
stdlib.json/symbolic.json would add a second place for the append-only
discipline to drift, for a purely cosmetic benefit that tier-filtering
already gives without it: tests/fixtures/conformance-tier-map.json is
already index-aligned with conformance.json, so any implementation can
already ask "which fixtures must I pass to be my-lisp at Tier 1?" without
touching Rust or waiting on this project. This script makes that question
trivial to answer instead of something a future implementer has to
reinvent by hand.

Usage:
    python3 scripts/fixtures-for-tier.py 1
    python3 scripts/fixtures-for-tier.py 2
    python3 scripts/fixtures-for-tier.py 3
    python3 scripts/fixtures-for-tier.py literate
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONFORMANCE = ROOT / "tests" / "fixtures" / "conformance.json"
TIER_MAP = ROOT / "tests" / "fixtures" / "conformance-tier-map.json"


def main():
    if len(sys.argv) != 2:
        raise SystemExit(f"usage: {sys.argv[0]} <1|2|3|literate>")
    requested = sys.argv[1]
    tier = None if requested == "literate" else int(requested)

    conformance = json.loads(CONFORMANCE.read_text(encoding="utf-8"))
    tier_map = json.loads(TIER_MAP.read_text(encoding="utf-8"))
    if len(conformance) != len(tier_map):
        raise SystemExit(
            f"conformance.json has {len(conformance)} fixtures but "
            f"conformance-tier-map.json has {len(tier_map)} tags — "
            f"index-alignment is broken, fix that first"
        )

    matched = [
        fact for fact, tags in zip(conformance, tier_map) if tags.get("tier") == tier
    ]
    print(json.dumps(matched, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
