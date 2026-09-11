#!/usr/bin/env python3
"""Generate ECO-CANON-1 function table projection from semantic-registry.wsm.

Authority remains lib/surface/semantic-registry.wsm — this script only projects.
Columns (human display order per my-lisp#75):
  semantic-id | formal-action-stub | uk | full-uk | full-uk-status | en | sa | sym | status | authority
"""

from __future__ import annotations

import argparse
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REGISTRY = ROOT / "lib" / "surface" / "semantic-registry.wsm"
OUT_WSM = ROOT / "lib" / "generated" / "function-table.wsm"
OUT_MD = ROOT / "docs" / "generated" / "function-table.md"

ENTRY = re.compile(r"^\s*\(([0-9]{4,})\s+(.*)\)\s*$")
SURFACE = re.compile(
    r"\(([A-Za-z][A-Za-z0-9-]*)\s+([^\s()]+)\s+"
    r"(stable|candidate|missing|compatibility-only)\)"
)


def load():
    rows = []
    for line in REGISTRY.read_text(encoding="utf-8").splitlines():
        m = ENTRY.match(line)
        if not m:
            continue
        sid, body = m.groups()
        surfaces = {}
        for s in SURFACE.finditer(body):
            lang, name, status = s.groups()
            surfaces[lang] = (name, status)
        rows.append((sid, surfaces))
    return rows


def full_uk_projection(uk_name: str, uk_status: str) -> tuple[str, str]:
    """Separate full-uk column; do not invent new names.

    Until a dedicated ratification pass, full-uk mirrors stable/candidate uk.
    """
    if uk_status in ("missing", "compatibility-only") or uk_name == "—":
        return "—", "needs-review" if uk_status == "compatibility-only" else "missing"
    if uk_status == "candidate":
        return uk_name, "candidate"
    return uk_name, "stable"


def formal_stub(sid: str, surfaces: dict) -> str:
    for lang in ("en", "uk", "sa"):
        if lang in surfaces and surfaces[lang][1] != "missing" and surfaces[lang][0] != "—":
            name, _st = surfaces[lang]
            return f"identity:{sid}/surface:{name}"
    return f"identity:{sid}"


def primary_status(surfaces: dict) -> str:
    statuses = [st for _, st in surfaces.values() if st != "missing"]
    if "stable" in statuses:
        return "stable"
    if "candidate" in statuses:
        return "candidate"
    if "compatibility-only" in statuses:
        return "compatibility-only"
    return "missing"


def render_wsm(rows) -> str:
    lines = [
        "; GENERATED — DO NOT EDIT BY HAND",
        "; Authority: lib/surface/semantic-registry.wsm",
        "; Generator: scripts/generate-function-table.py (ECO-CANON-1 / my-lisp#75)",
        "; Schema ft/1: (id formal uk full-uk full-uk-status en sa sym primary-status authority)",
        "; Display order for humans: Українська → Повна українська → English → Sanskrit",
        "; authority = my-lisp (semantic)",
        "",
        "(ft/1",
    ]
    for sid, surfaces in rows:

        def get(lang):
            if lang not in surfaces:
                return "—", "missing"
            return surfaces[lang]

        uk_n, uk_st = get("uk")
        en_n, en_st = get("en")
        sa_n, sa_st = get("sa")
        sym_n, sym_st = get("sym") if "sym" in surfaces else ("—", "missing")
        full_n, full_st = full_uk_projection(uk_n, uk_st)
        formal = formal_stub(sid, surfaces)
        primary = primary_status(surfaces)
        lines.append(
            f"  ({sid} {formal} "
            f"(uk {uk_n} {uk_st}) "
            f"(full-uk {full_n} {full_st}) "
            f"(en {en_n} {en_st}) "
            f"(sa {sa_n} {sa_st}) "
            f"(sym {sym_n} {sym_st}) "
            f"{primary} my-lisp)"
        )
    lines.append(")")
    lines.append("")
    return "\n".join(lines)


def render_md(rows) -> str:
    lines = [
        "# Function table (generated projection)",
        "",
        "**Authority:** `lib/surface/semantic-registry.wsm` — projection only, not a second source of truth.",
        "",
        "| ID | Українська | Повна українська | full-uk status | English | Sanskrit | primary |",
        "|----|------------|------------------|----------------|---------|----------|---------|",
    ]
    for sid, surfaces in rows:

        def g(lang):
            if lang not in surfaces:
                return "—", "missing"
            return surfaces[lang]

        uk_n, uk_st = g("uk")
        en_n, _ = g("en")
        sa_n, _ = g("sa")
        full_n, full_st = full_uk_projection(uk_n, uk_st)
        primary = primary_status(surfaces)
        lines.append(
            f"| `{sid}` | {uk_n} | {full_n} | {full_st} | {en_n} | {sa_n} | {primary} |"
        )
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail if committed projection is stale")
    args = parser.parse_args()
    rows = load()
    if not rows:
        print("no rows", flush=True)
        return 2
    wsm = render_wsm(rows)
    md = render_md(rows)
    if args.check:
        ok = True
        for path, content in ((OUT_WSM, wsm), (OUT_MD, md)):
            if not path.exists() or path.read_text(encoding="utf-8") != content:
                print(f"stale: {path}; run scripts/generate-function-table.py", flush=True)
                ok = False
        if not ok:
            return 1
        print(f"function-table projections current ({len(rows)} identities)")
        return 0
    OUT_WSM.parent.mkdir(parents=True, exist_ok=True)
    OUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUT_WSM.write_text(wsm, encoding="utf-8")
    OUT_MD.write_text(md, encoding="utf-8")
    needs = sum(
        1
        for _, s in rows
        if full_uk_projection(*s.get("uk", ("—", "missing")))[1] == "needs-review"
    )
    missing_full = sum(
        1 for _, s in rows if full_uk_projection(*s.get("uk", ("—", "missing")))[1] == "missing"
    )
    print(f"rows={len(rows)} full-uk needs-review={needs} missing={missing_full}")
    print(f"wrote {OUT_WSM}")
    print(f"wrote {OUT_MD}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
