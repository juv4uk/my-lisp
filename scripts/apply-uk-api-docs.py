#!/usr/bin/env python3
from pathlib import Path

TEST_RS = 'use std::collections::{BTreeMap, BTreeSet};\n\nconst COVERAGE: &str = include_str!("../../../lib/surface/uk-sa-coverage.wsm");\nconst DOCS_INDEX: &str = include_str!("../../../lib/surface/uk-docs.wsm");\nconst DOCS_MD: &str = include_str!("../../../docs/ukrainian-api.md");\nconst UK_SURFACE: &str = include_str!("../../../lib/surface/uk.my");\n\nfn stable_pairs() -> BTreeSet<(String, String)> {\n    COVERAGE\n        .lines()\n        .filter_map(|line| {\n            let fields = line.split_whitespace().collect::<Vec<_>>();\n            if fields.first() != Some(&"(entry") || fields.get(5) != Some(&"stable") {\n                return None;\n            }\n            Some((fields[2].to_string(), fields[3].to_string()))\n        })\n        .collect()\n}\n\nfn documented() -> BTreeMap<(String, String), String> {\n    DOCS_INDEX\n        .lines()\n        .filter_map(|line| {\n            let fields = line.split_whitespace().collect::<Vec<_>>();\n            if fields.first() != Some(&"(doc") {\n                return None;\n            }\n            assert!(\n                fields.len() >= 5,\n                "рядок документації має містити category EN UK kind: {line}"\n            );\n            Some(((fields[2].to_string(), fields[3].to_string()), fields[4].to_string()))\n        })\n        .collect()\n}\n\n#[test]\nfn vsi_stable_ukrainski_nazvy_maiut_zapys_u_dovidnyku() {\n    let coverage = stable_pairs();\n    let docs = documented();\n\n    assert_eq!(coverage.len(), 140, "stable-покриття змінилося");\n    assert_eq!(docs.len(), 140, "довідник мусить мати рівно 140 записів");\n\n    let doc_pairs = docs.keys().cloned().collect::<BTreeSet<_>>();\n    assert_eq!(\n        coverage, doc_pairs,\n        "stable surface і машинний документаційний індекс розійшлися"\n    );\n\n    for (_, uk) in &coverage {\n        assert!(\n            DOCS_MD.contains(&format!("| `{uk}` |")),\n            "публічне українське ім\'я відсутнє у Markdown-довіднику: {uk}"\n        );\n    }\n}\n\n#[test]\nfn znak_pytannia_tochno_vidpovidaie_predykatam() {\n    let docs = documented();\n    let mut predicates = 0usize;\n\n    for ((_en, uk), kind) in docs {\n        let question_name = uk.ends_with(\'?\');\n        let predicate = kind == "predicate";\n\n        assert_eq!(\n            question_name, predicate,\n            "українська назва {uk}: знак ? і класифікація predicate мусять збігатися"\n        );\n        predicates += usize::from(predicate);\n    }\n\n    assert_eq!(predicates, 30, "змінився каталог публічних предикатів");\n}\n\n#[test]\nfn znak_oklyku_tochno_vidpovidaie_mutatsii() {\n    let docs = documented();\n    let mut mutations = 0usize;\n\n    for ((_en, uk), kind) in docs {\n        let mutation = kind == "mutation";\n        assert_eq!(\n            uk.ends_with(\'!\'),\n            mutation,\n            "українська назва {uk}: знак ! зарезервований для мутації"\n        );\n        if mutation {\n            mutations += 1;\n            assert_eq!(uk, "встановити-вектор!");\n        }\n    }\n\n    assert_eq!(mutations, 1, "публічний каталог мутацій змінився");\n}\n\n#[test]\nfn stari_nazvy_dvokh_predykativ_lyshaiutsia_aliasamy_symisnosti() {\n    for binding in [\n        "(define конфлікт? check-conflict)",\n        "(define перевірити-конфлікт check-conflict)",\n        "(define змінна-зустрічається? occurs-check)",\n        "(define перевірити-зустрічання occurs-check)",\n    ] {\n        assert!(\n            UK_SURFACE.contains(binding),\n            "відсутній очікуваний stable/compatibility alias: {binding}"\n        );\n    }\n}\n\n#[test]\nfn dovidnyk_poiasniuie_ne_predykaty_shcho_mozhut_povernuty_pustyi_spysok() {\n    for name in ["карта-отримати", "підтримуючий-доказ", "та", "або"] {\n        let kind = documented()\n            .into_iter()\n            .find_map(|((_en, uk), kind)| (uk == name).then_some(kind))\n            .unwrap_or_else(|| panic!("немає документаційного запису для {name}"));\n        assert_ne!(kind, "predicate", "{name} повертає дані/значення, а не лише t/()");\n        assert!(!name.ends_with(\'?\'));\n    }\n}\n'


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one exact marker, found {count}")
    p.write_text(text.replace(old, new), encoding="utf-8")

# Спочатку перевіряємо щойно доданий машинний каталог, а вже тоді міняємо surface.
doc_lines = [
    line.split()
    for line in Path("lib/surface/uk-docs.wsm").read_text(encoding="utf-8").splitlines()
    if line.strip().startswith("(doc ")
]
if len(doc_lines) != 140:
    raise SystemExit(f"expected 140 doc records, got {len(doc_lines)}")
predicates = [fields for fields in doc_lines if fields[4] == "predicate"]
if len(predicates) != 30:
    raise SystemExit(f"expected 30 predicates, got {len(predicates)}")
for fields in doc_lines:
    uk = fields[3]
    kind = fields[4]
    if uk.endswith("?") != (kind == "predicate"):
        raise SystemExit(f"predicate naming drift: {uk} / {kind}")
    if uk.endswith("!") != (kind == "mutation"):
        raise SystemExit(f"mutation naming drift: {uk} / {kind}")

Path("crates/my-lisp/tests/ukrainian_api_docs.rs").write_text(TEST_RS, encoding="utf-8")

replace_once(
    "lib/surface/uk.my",
    "(define логічний-висновок reason-in)\n(define перевірити-конфлікт check-conflict)\n(define модуль-відомий? module-known?)",
    """(define логічний-висновок reason-in)
; Публічний предикат читається як питання; стара дієслівна назва лишається
; compatibility alias, щоб наявні українські програми не ламалися.
(define конфлікт? check-conflict)
(define перевірити-конфлікт check-conflict)
(define модуль-відомий? module-known?)""",
)
replace_once(
    "lib/surface/uk.my",
    "(define відшукати walk)\n(define перевірити-зустрічання occurs-check)",
    """(define відшукати walk)
; Occurs-check є питанням t/(), тому preferred-назва має ?. Стара назва
; збережена як compatibility alias.
(define змінна-зустрічається? occurs-check)
(define перевірити-зустрічання occurs-check)""",
)

uk_path = Path("lib/surface/uk.my")
uk_text = uk_path.read_text(encoding="utf-8")
candidate_count = uk_text.count("(status: candidate)")
if candidate_count != 16:
    raise SystemExit(f"uk.my: expected 16 stale candidate headings, got {candidate_count}")
uk_text = uk_text.replace("(status: candidate)", "(status: stable)")
uk_text = uk_text.replace(
    "; vector-set! is a mutation — keep the ! suffix in Ukrainian too.",
    "; vector-set! є мутацією — тому знак ! зберігається і в українській назві.",
)
uk_path.write_text(uk_text, encoding="utf-8")

replace_once(
    "lib/surface/uk-sa-coverage.wsm",
    "(entry (knowledge check-conflict перевірити-конфлікт virodha-parīkṣā stable candidate",
    "(entry (knowledge check-conflict конфлікт? virodha-parīkṣā stable candidate",
)
replace_once(
    "lib/surface/uk-sa-coverage.wsm",
    "(entry (unification occurs-check перевірити-зустрічання parivṛtti-parīkṣā stable candidate",
    "(entry (unification occurs-check змінна-зустрічається? parivṛtti-parīkṣā stable candidate",
)

replace_once(
    "docs/ukrainian-surface-inventory.md",
    "Предикат на своїй припустимій області повертає лише t або (). Усі нові\nукраїнські імена таких операцій мусять закінчуватися знаком ?. Історичні\nзнаки atom, eq, <, =, > цим правилом не перейменовуються.\n",
    "Предикат на своїй припустимій області повертає лише `t` або `()`. Українське\nпублічне ім'я такого питання закінчується знаком `?`: `атом?`, `менше?`,\n`містить?`, `модуль-відомий?`. Знак `?` є частиною ідентифікатора, а не\nокремим оператором. Повна класифікація всіх 140 stable назв лежить у\n`lib/surface/uk-docs.wsm`, а пояснений довідник — у `docs/ukrainian-api.md`.\n\nФункція, що може повернути `()` або корисні дані, не стає від цього\nпредикатом: наприклад, `карта-отримати` повертає форму maybe, а\n`підтримуючий-доказ` — evidence-запис. Окремо `!` зарезервовано за\nукраїнськими назвами мутацій; зараз це `встановити-вектор!`.\n",
)

readme_marker = "Повна самоперевірна українська програма є в [`lib/surface/uk-acceptance.my`](lib/surface/uk-acceptance.my).\n"
readme_insert = """Повний україномовний довідник публічного API: [`docs/ukrainian-api.md`](docs/ukrainian-api.md). Він покриває всі 140 stable українських назв, пояснює сигнатури, повернені значення, предикати `?` та мутацію `!`.

### Предикати читаються як питання

Українська назва предиката закінчується `?`. На своїй припустимій області предикат повертає тільки канонічне `t` або `()`:

```lisp
(атом? 'кіт)                         ; t
(менше? 2 5)                         ; t
(містить? 'пес (список 'кіт 'пес))   ; t
```

`?` — частина ідентифікатора, а не окремий оператор. Функції, що можуть повернути дані або `()` (наприклад `карта-отримати`), предикатами не є й `?` не мають.

"""
replace_once("README.md", readme_marker, readme_insert + readme_marker)

functions_marker = "> Regeneration rule: use `scripts/gen-functions.my` for a full refresh. The library sections are a generated snapshot and do not define semantic authority; see [`semantic-authority-map.md`](semantic-authority-map.md).\n\n"
functions_note = "> Для україномовного програмування користуйтеся [`ukrainian-api.md`](ukrainian-api.md): це повний пояснений довідник 140 stable українських публічних назв. Цей файл лишається технічним інвентарем реалізації, включно з внутрішніми helper-ами та compatibility-only іменами.\n\n"
replace_once("docs/FUNCTIONS.md", functions_marker, functions_marker + functions_note)

agents_marker = "Guard також реєструє інструмент `(guard-script (quote uk-latynka))`."
agents_rule = "7. **Публічні українські предикати пишуться як питання із `?`**, а публічні мутації — з `!`. Предикат на своїй припустимій області повертає лише канонічне `t` або `()`. Повний перевірюваний каталог: `lib/surface/uk-docs.wsm`; довідник: `docs/ukrainian-api.md`; Guard-тема: `(guard-reference (quote ukrainian-programming-surface))`.\n"
agents_path = Path("AGENTS.md")
agents_text = agents_path.read_text(encoding="utf-8")
if "Публічні українські предикати пишуться як питання із `?`" in agents_text:
    raise SystemExit("AGENTS.md: predicate rule already present unexpectedly")
if agents_text.count(agents_marker) != 1:
    raise SystemExit("AGENTS.md: guard marker drifted")
agents_text = agents_text.replace(agents_marker, agents_rule + "\n" + agents_marker)
agents_path.write_text(agents_text, encoding="utf-8")

guard_marker = "     (reference\n       (topic language-semantics)"
guard_block = """     (reference
       (topic ukrainian-programming-surface)
       (summary "Український публічний API: stable-імена, довідник, предикати з ? та мутації з !")
       (authority (lib/surface/uk-sa-coverage.wsm lib/surface/uk-docs.wsm docs/ukrainian-api.md lib/surface/uk.my))
       (how-to (choose-stable-ukrainian-name classify-as-form-macro-function-predicate-mutation-or-value suffix-predicate-with-question-mark suffix-mutation-with-exclamation-mark update-api-docs))
       (verify (uk_surface_equivalence ukrainian_api_docs stable-140-of-140 predicate-question-mark-bijection))
       (lifecycle current-surface)
       (provenance "owner directive 2026-09-07; executable documentation gate")
       (unknown-route ask-owner))
"""
guard_path = Path("knowledge/guard-reference.wsm")
guard_text = guard_path.read_text(encoding="utf-8")
if "(topic ukrainian-programming-surface)" in guard_text:
    raise SystemExit("guard-reference: topic already exists unexpectedly")
if guard_text.count(guard_marker) != 1:
    raise SystemExit("guard-reference: language-semantics marker drifted")
guard_text = guard_text.replace(guard_marker, guard_block + guard_marker)
guard_path.write_text(guard_text, encoding="utf-8")

print("uk-api-docs: surface + guard + docs gate synchronized")
