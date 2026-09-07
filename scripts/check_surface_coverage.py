#!/usr/bin/env python3
"""Surface coverage drift checker.

Verifies that every eligible public English-facing name in the my-lisp
runtime is classified in the Ukrainian+Sanskrit coverage table.

Eligibility: names that an ordinary user must write in a program —
canon surfaces, necessary forms, root builtins, and public core.my
definitions. Internal helpers (accumulator workers, algorithm steps,
bootstrap mechanisms) are excluded.

A name may legitimately be `missing` or `candidate`, but it must not be
invisible. If a new English public symbol appears in the runtime without
being classified, this script exits non-zero.

Usage:
    python3 scripts/check_surface_coverage.py

Exit codes:
    0 — all eligible names are classified
    1 — unclassified names found (drift detected)
    2 — error reading sources
"""

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
COVERAGE_FILE = REPO_ROOT / "lib" / "surface" / "uk-sa-coverage.wsm"
INVENTORY_FILE = REPO_ROOT / "lib" / "surface" / "uk-inventory.wsm"
CORE_LIB = REPO_ROOT / "lib" / "core.my"


def extract_coverage_names(path: Path) -> set[str]:
    """Extract all English names from the coverage table."""
    text = path.read_text(encoding="utf-8")
    # Match: (entry (category EN ...))
    names = set()
    for m in re.finditer(r'\(entry\s+\(\S+\s+(\S+)\s', text):
        names.add(m.group(1))
    # Also extract compatibility-only entries: (entry (category EN — — compat compat ...))
    for m in re.finditer(r'\(entry\s+\(\S+\s+(\S+)\s+—\s+—\s+compatibility-only', text):
        names.add(m.group(1))
    return names


def extract_inventory_names(path: Path) -> set[str]:
    """Extract all classified names from the inventory file."""
    text = path.read_text(encoding="utf-8")
    names = set()
    # The inventory lists names in groups: (root-builtins ...), (canon ...), etc.
    # Extract all parenthesized word groups after public markers
    for m in re.finditer(r'\((canon|necessary-forms|language-macros|compatibility-forms|root-builtins|core-library)\s+', text):
        # Find the closing paren for this group
        start = m.end()
        depth = 1
        i = start
        while i < len(text) and depth > 0:
            if text[i] == '(':
                depth += 1
            elif text[i] == ')':
                depth -= 1
            i += 1
        group_text = text[start:i-1]
        for word in group_text.split():
            word = word.strip('()')
            if word and not word.startswith(';'):
                names.add(word)
    return names


def extract_core_public_names(path: Path) -> set[str]:
    """Extract public (def ...) names from core.my, excluding internal helpers."""
    text = path.read_text(encoding="utf-8")
    names = set()
    internal_markers = {'-onto', '-iter', '-step', 'make-', '-helper', '-aux'}
    for m in re.finditer(r'\(def\s+(\S+)\s+', text):
        name = m.group(1)
        if not any(marker in name for marker in internal_markers):
            names.add(name)
    # Also capture defmacro names
    for m in re.finditer(r'\(defmacro\s+(\S+)\s+', text):
        names.add(m.group(1))
    return names


def extract_root_builtins_from_inventory(path: Path) -> set[str]:
    """Extract the root-builtins group specifically."""
    text = path.read_text(encoding="utf-8")
    # Find (root-builtins ... ) and extract all names
    m = re.search(r'\(root-builtins\s+', text)
    if not m:
        return set()
    start = m.end()
    depth = 1
    i = start
    while i < len(text) and depth > 0:
        if text[i] == '(':
            depth += 1
        elif text[i] == ')':
            depth -= 1
        i += 1
    group_text = text[start:i-1]
    names = set()
    for word in group_text.split():
        word = word.strip('()')
        if word:
            names.add(word)
    return names


def main():
    if not COVERAGE_FILE.exists():
        print(f"ERROR: coverage file not found: {COVERAGE_FILE}", file=sys.stderr)
        return 2
    if not INVENTORY_FILE.exists():
        print(f"ERROR: inventory file not found: {INVENTORY_FILE}", file=sys.stderr)
        return 2
    if not CORE_LIB.exists():
        print(f"ERROR: core library not found: {CORE_LIB}", file=sys.stderr)
        return 2

    coverage_names = extract_coverage_names(COVERAGE_FILE)
    inventory_names = extract_inventory_names(INVENTORY_FILE)
    core_names = extract_core_public_names(CORE_LIB)

    # The set of all eligible public names
    all_eligible = inventory_names | core_names

    # Check: every eligible name must appear in the coverage table
    unclassified = all_eligible - coverage_names
    # Filter out internal names that might have slipped through
    internal_markers = {'-onto', '-iter', '-step', 'make-', '-helper', '-aux'}
    unclassified = {n for n in unclassified if not any(m in n for m in internal_markers)}

    if unclassified:
        print("SURFACE DRIFT DETECTED")
        print("=" * 60)
        print(f"The following eligible public names are NOT classified")
        print(f"in lib/surface/uk-sa-coverage.wsm:")
        print()
        for name in sorted(unclassified):
            print(f"  - {name}")
        print()
        print(f"Total unclassified: {len(unclassified)}")
        print()
        print("Action required: add entries for these names in the")
        print("coverage table with appropriate status (stable, candidate,")
        print("missing, or compatibility-only).")
        return 1

    print("OK: all eligible public names are classified in the coverage table.")
    print(f"  Coverage table entries: {len(coverage_names)}")
    print(f"  Inventory names: {len(inventory_names)}")
    print(f"  Core public names: {len(core_names)}")
    print(f"  Total eligible: {len(all_eligible)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
