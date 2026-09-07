from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one anchor, found {count}")
    p.write_text(text.replace(old, new), encoding="utf-8")


# diagnose_impl лишається зручним канонічним wrapper лише для наявних unit-тестів.
replace_once(
    "crates/my-lisp-wasm/src/lib.rs",
    "fn diagnose_impl(source: &str, is_literate: bool) -> Vec<Diagnostic> {\n    diagnose_impl_with_presentation(source, is_literate, PresentationLanguage::Canonical)\n}\n",
    "#[cfg(test)]\nfn diagnose_impl(source: &str, is_literate: bool) -> Vec<Diagnostic> {\n    diagnose_impl_with_presentation(source, is_literate, PresentationLanguage::Canonical)\n}\n",
)

# Нові коментарі в коді мають бути українською UTF-8 згідно з policy репозиторію.
replace_once(
    "crates/my-lisp/src/presentation.rs",
    "//! Presentation-only rendering for interactive human-facing surfaces.\n//!\n//! Canonical `Value::Display` and `LanguageError::render` stay unchanged: they\n//! are used by conformance, machine protocols and source round-tripping.  This\n//! module changes only what an interactive surface shows to a person.\n",
    "//! Людське подання значень і діагностик для інтерактивних поверхонь.\n//!\n//! Канонічні `Value::Display` і `LanguageError::render` не змінюються: ними\n//! користуються conformance-перевірки, машинні протоколи й точне відтворення\n//! джерела. Цей модуль змінює лише те, що інтерактивна поверхня показує людині.\n",
)
replace_once(
    "lib/surface/uk.my",
    "; Українські імена тих самих канонічних значень істини й хиби.\n; Це не нові Boolean-об'єкти: істина є тим самим символом t, а хиба — тим\n; самим Canon 0 `()`. Presentation-layer показує t як `істина` лише в UK REPL.\n",
    "; Українські імена тих самих канонічних значень істини й хиби.\n; Це не нові логічні об'єкти: істина є тим самим символом t, а хиба — тим\n; самим Canon 0 `()`. Шар подання показує t як `істина` лише в українському REPL.\n",
)

print("uk presentation zero-warning fix applied")
