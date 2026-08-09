#!/usr/bin/env python3
"""Regenerate my-lisp-constitution.json from its actual sources of truth.

my-lisp-constitution.json is a *projection*, not a second source of truth —
the same pattern lib/knowledge.my's *knowledge-journal* uses (one append-only
log, current state computed on demand), applied here to documentation instead
of runtime state. Never hand-edit the generated file; edit one of its three
real inputs instead, then rerun this script:

  1. tests/fixtures/conformance.json        — the facts (expr/expected/error),
                                               append-only, values never edited
  2. tests/fixtures/conformance-tier-map.json — the tags (tier/axioms/role),
                                               index-aligned with (1)
  3. This script's own PRINCIPLES/AXIOMS text below — kept here, not
     duplicated from docs/language-core-axioms.md, so there is exactly one
     place that owns the canonical one-line wording

Usage:
    python3 scripts/build-constitution.py
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONFORMANCE = ROOT / "tests" / "fixtures" / "conformance.json"
TIER_MAP = ROOT / "tests" / "fixtures" / "conformance-tier-map.json"
OUTPUT = ROOT / "my-lisp-constitution.json"

PRINCIPLES = [
    {
        "n": 1,
        "en": "Write about possibilities, not limitations.",
        "uk": "Писати про можливості, не про обмеження.",
    },
    {
        "n": 2,
        "en": "Be Lisp in the full sense of the word — homoiconicity and a minimal, closed core that grows the rest of the language from inside itself, not the surface syntax of any one historical dialect.",
        "uk": "Бути Lisp-ом у повному розумінні цього слова — гомоіконність і мінімальне, замкнене ядро, що вирощує решту мови зсередини себе, не поверхневий синтаксис якогось одного історичного діалекту.",
    },
    {
        "n": 3,
        "en": "Build the reasoning machine — McCarthy's documented 1958 Advice Taker goal, extended by the author's own hybrid neural/symbolic vision (private/lisp-to-knowledge.md).",
        "uk": "Реалізувати розумну машину — задокументована ціль МакКарті 1958 року (Advice Taker), продовжена власним гібридним нейро-символьним баченням автора (private/lisp-to-knowledge.md).",
    },
    {
        "n": 4,
        "en": "Cross-platform-ness, or more simply: universality — the falsifiability test for G6/G7; my-lisp commits to real, physically different substrates (Rust, a future C core, fpga-lisp), not just one implementation asserting conformance.",
        "uk": "Кросплатформеність, або простіше — універсальність — тест на фальсифіковність для G6/G7; my-lisp зобов'язується перед реально різними фізичними субстратами (Rust, майбутнє C-ядро, fpga-lisp), не лише однією реалізацією, що заявляє конформність.",
    },
    {
        "n": 5,
        "en": "Maximum awareness of today's technology, applied to symbolic AI — classical symbolic AI is not a museum piece; modern tooling and modern LLMs (as the fuzzy natural-language interface, not a competitor to the precise symbolic core) are part of building it.",
        "uk": "Максимальна обізнаність у сьогоднішніх технологіях, застосована до символьного ШІ — класичний символьний AI не музейний експонат; сучасні інструменти й сучасні LLM (як нечіткий інтерфейс природної мови, не конкурент точному символьному ядру) — частина його побудови.",
    },
]

AXIOM_TEXT = {
    "G1": ("A value's meaning can be fully defined by observable behavior.",
           "Значення value може бути повністю визначене спостережуваною поведінкою."),
    "G2": ("Every value can be built from just two things: atoms and pairs.",
           "Кожне значення можна побудувати лише з двох речей: атомів і пар."),
    "G3": ("Program structure can be inspected, transformed, and built like any other value.",
           "Структуру програми можна оглядати, трансформувати й будувати, як і будь-яке інше значення."),
    "G4": ("A minimal core can grow an entire language inside itself.",
           "Мінімальне ядро може вирощувати всю мову всередині себе."),
    "G5": ("Anything expressible within the language can live above the implementation boundary.",
           "Усе, що виразне мовою, може жити над межею реалізації."),
    "G6": ("Conformance can be defined purely by observable behavior.",
           "Конформність можна визначити суто спостережуваною поведінкою."),
    "G7": ("The same expression can mean the same thing everywhere.",
           "Той самий вираз може означати те саме всюди."),
    "G8": ("The absence of any element and the absence of truth can be the same value.",
           "Відсутність будь-якого елемента й відсутність істини можуть бути тим самим значенням."),
    "S1": ("Never silently turn an exact value into an approximation.",
           "Ніколи мовчки не перетворювати точне значення на наближення."),
    "S2": ("Never fail silently — every failure is a named, observable outcome.",
           "Ніколи не провалюватись мовчки — кожен провал є названим, спостережуваним результатом."),
    "S3": ("Never let a resource limit silently redefine an operation's meaning.",
           "Ніколи не дозволяти обмеженню ресурсу мовчки переозначити сенс операції."),
}


def build():
    conformance = json.loads(CONFORMANCE.read_text(encoding="utf-8"))
    tier_map = json.loads(TIER_MAP.read_text(encoding="utf-8"))
    if len(conformance) != len(tier_map):
        raise SystemExit(
            f"conformance.json has {len(conformance)} fixtures but "
            f"conformance-tier-map.json has {len(tier_map)} tags — they must "
            f"stay index-aligned. Add/remove a tag entry to match, in the "
            f"same position as the fixture it describes."
        )

    fixtures = []
    for fact, tags in zip(conformance, tier_map):
        entry = dict(tags)  # tier, axioms, and optionally role/layer/note
        entry.update(fact)  # expr, expected/error, and optionally mode
        fixtures.append(entry)

    generative = [{"id": k, "en": v[0], "uk": v[1]} for k, v in AXIOM_TEXT.items() if k.startswith("G")]
    safety = [{"id": k, "en": v[0], "uk": v[1]} for k, v in AXIOM_TEXT.items() if k.startswith("S")]

    doc = {
        "$about": (
            "my-lisp-constitution.json — the executable proof of docs/language-core-axioms.md's "
            "project principles and axioms (G1-G8 generative, S1-S3 safety). Each fixture is one "
            "of the observable claims from tests/fixtures/conformance.json, tagged with the tier "
            "(1 CORE SEMANTICS, 2 LANGUAGE CONTRACT, 3 ECOSYSTEM CONFORMANCE, or null for the "
            "optional literate-Markdown layer) and, where one applies, the axiom(s) it is "
            "evidence for. Symbolic-reasoning fixtures (tier 3, unify/reason) carry no axiom tag "
            "on purpose — they are evidence for project principle 3, not the G/S axiom list."
        ),
        "$status": "draft — not yet ratified; will become read-only once ratified",
        "$generated": (
            "This file is GENERATED — do not hand-edit it. It is a projection over two real "
            "sources of truth (tests/fixtures/conformance.json for facts, "
            "tests/fixtures/conformance-tier-map.json for tags), the same "
            "one-source-plus-projection shape lib/knowledge.my's *knowledge-journal* uses for "
            "runtime state. Edit one of those two files (or scripts/build-constitution.py's own "
            "principle/axiom text), then run: python3 scripts/build-constitution.py"
        ),
        "$selfContained": (
            "principles and axioms below are the canonical one-line statements from "
            "docs/language-core-axioms.md, kept here so this file can be read and understood "
            "on its own; docs/language-core-axioms.md remains the single source of the full "
            "prose rationale, examples, and open questions — not duplicated here, to avoid two "
            "sources of truth for the same wording drifting apart"
        ),
        "$roleField": (
            "Each fixture may carry \"role\": \"constitutive\" — meaning it directly invokes "
            "one of McCarthy's seven original primitives (quote, atom, eq, car, cdr, cons, "
            "cond), including their documented error paths. A constitutive fixture doesn't "
            "just provide evidence that an axiom holds; it's one of the acts that makes the "
            "axiom true in the first place — remove the primitive and the axiom becomes "
            "false, not just unproven. Every other fixture (role omitted, i.e. \"derived\") "
            "demonstrates a consequence built on top of the constitutive primitives — "
            "removing it could in principle still leave the axiom provable some other way."
        ),
        "principlesDocument": "docs/language-core-axioms.md",
        "tierMap": "docs/conformance-tier-map.md",
        "principles": PRINCIPLES,
        "axioms": {
            "generative": generative,
            "safety": safety,
        },
        "tiers": {
            "1": "CORE SEMANTICS — every conforming implementation must have this",
            "2": "LANGUAGE CONTRACT — every conforming implementation must have this",
            "3": "ECOSYSTEM CONFORMANCE — an implementation can be my-lisp without this loaded yet; tests a library, not the language itself",
            "literate": "LITERATE MARKDOWN LAYER — not one of the three tiers; optional, skippable by an implementation with no literate-Markdown support",
        },
        "fixtures": fixtures,
    }

    OUTPUT.write_text(json.dumps(doc, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {OUTPUT.relative_to(ROOT)} ({len(fixtures)} fixtures)")


if __name__ == "__main__":
    build()
