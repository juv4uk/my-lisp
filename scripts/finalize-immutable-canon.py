#!/usr/bin/env python3
from pathlib import Path

# 1. Authoritative executable contract: Contract 2.1 allowed Canon shadowing;
# Contract 6.0 rejects the same program as InvalidForm.
path = Path("tests/fixtures/conformance.my")
lines = path.read_text().splitlines()
needle = '(let ((car (lambda (x) (quote shadowed)))) (car (quote (1 2))))'
hits = [i for i, line in enumerate(lines) if needle in line]
if len(hits) != 1:
    raise SystemExit(f"expected exactly one old Canon-shadow fixture, found {len(hits)}")
lines[hits[0]] = '((expr . "(let ((car (lambda (x) (quote shadowed)))) (car (quote (1 2))))") (error . "InvalidForm") (tier . 1) (since-contract . (6 0)) (requires . (immutable-canon)) (axioms . (G4 S2)) (role . "constitutive") (note . "Contract 6.0: Canon 0+7 spellings are reserved and unshadowable. let lowers through a lambda binder, so attempting to bind car is rejected as InvalidForm. Ordinary non-Canon builtins remain first-class and shadowable."))'
path.write_text("\n".join(lines) + "\n")

# 2. ADR-004 keeps the historical decision, but points to the newer authority.
path = Path("docs/adr/ADR-004-CLOSED-MCCARTHY7-CORE.md")
text = path.read_text()
marker = "## 8. Closed: Canon Access Architecture (2026-09-06)\n## 8. Закрито: Архітектура доступу до канону (2026-09-06)\n"
if "Superseded for Canon 0+7 binding semantics by ADR-006" not in text:
    if marker not in text:
        raise SystemExit("ADR-004 section-8 marker not found")
    note = marker + "\n> **Superseded for Canon 0+7 binding semantics by ADR-006 / Contract 6.0 (2026-09-08).** The text below is retained as historical evidence of the earlier Variant-A decision; it is no longer normative where it permits shadowing Canon spellings.\n>\n> **Замінено для семантики зв’язування Canon 0+7 рішенням ADR-006 / Contract 6.0 (2026-09-08).** Текст нижче збережено як історію попереднього Variant A; дозвіл затінювати канонічні написання більше не є нормативним.\n"
    path.write_text(text.replace(marker, note, 1))

# 3. Bootstrap/value comments must describe Contract 6.0 precisely.
path = Path("crates/my-lisp/src/eval/builtins.rs")
text = path.read_text()
old = """//! Contract 2.1 bootstrap: registers primitive operations into the root
//! environment as first-class `Value::Builtin` values (docs/
//! PROPOSAL-FIRST-CLASS-BUILTINS.md). After this runs, the evaluator's
//! symbol match no longer owns these names -- the environment is the
//! single runtime authority; the registry in this file is
//! bootstrap-description only.
"""
new = """//! Bootstrap registry for first-class `Value::Builtin` mechanisms. Contract
//! 2.1 introduced ordinary callable builtin values; Contract 6.0 narrows the
//! lookup rule for Canon 0+7 only. Non-Canon builtins remain ordinary lexical
//! bindings. Historical Canon bootstrap entries may still exist in the root
//! environment as implementation detail, but they are never semantic authority:
//! the immutable Canon resolver wins before `Environment` lookup for every
//! reserved Canon spelling.
"""
if old not in text:
    raise SystemExit("builtins.rs old authority comment not found")
path.write_text(text.replace(old, new, 1))

path = Path("crates/my-lisp/src/value.rs")
text = path.read_text()
old = """/// A primitive operation as a first-class value (contract 2.1,
/// docs/PROPOSAL-FIRST-CLASS-BUILTINS.md). Registered into the root
/// environment at bootstrap; from then on it is an ordinary callable
/// value indistinguishable -- by the language's own rules -- from a
/// lambda. Arguments arrive pre-evaluated; special forms never become
/// Builtins (they are syntax, not values).
/// Prymityv iak pershoklasne znachennia (kontrakt 2.1). Reiestruietsia
/// v kornevomu seredovyshchi na bootstrapi; pislia tsioho tse zvychaine
/// zastosovne znachennia. Arhumenty nadkhodiat vzhie obchyslenymy;
/// spetsialni formy nikoly ne staiutsia Builtinamy.
"""
new = """/// A primitive operation as a first-class value. Contract 2.1 introduced
/// callable builtin values; Contract 6.0 keeps that value-level property while
/// reserving Canon 0+7 *names*. A Canon callable can still be passed as a value,
/// but its registered EN/UK/SA/symbolic spellings cannot be rebound. Non-Canon
/// builtins remain ordinary lexical values. Arguments arrive pre-evaluated;
/// special forms never become Builtins (they are syntax, not values).
/// Prymityv yak pershoklasne znachennia. Kontrakt 6.0 ne zaboraie peredavaty
/// Canon-callable yak znachennia; vin rezervuie lyshe kanonichni imena vid
/// perevyznachennia. Ne-Canon builtiny zalyshaiutsia zvychainymy lexical values.
"""
if old not in text:
    raise SystemExit("value.rs old Builtin comment not found")
path.write_text(text.replace(old, new, 1))

# 4. Use the Canon surface-value API in production, not only in tests. This
# keeps the resolution boundary in one place and avoids a dead helper that
# duplicates evaluator logic.
path = Path("crates/my-lisp/src/eval/mod.rs")
text = path.read_text()
old = '''        ExprKind::Symbol(symbol) => {
            if let Some(identity) = canon::identity_for_surface(symbol) {
                return canon::value(identity)
                    .map(EvalStep::Value)
                    .ok_or_else(|| {
                        LanguageError::new(
                            ErrorKind::InvalidForm,
                            format!(
                                "canonical special form is syntax-only · канонічна спеціальна форма є лише синтаксисом · kanonische Sonderform ist nur Syntax: {symbol}"
                            ),
                            expression.span,
                        )
                    });
            }
            environment
'''
new = '''        ExprKind::Symbol(symbol) => {
            if let Some(value) = canon::value_for_surface(symbol) {
                return Ok(EvalStep::Value(value));
            }
            if canon::identity_for_surface(symbol).is_some() {
                return Err(LanguageError::new(
                    ErrorKind::InvalidForm,
                    format!(
                        "canonical special form is syntax-only · канонічна спеціальна форма є лише синтаксисом · kanonische Sonderform ist nur Syntax: {symbol}"
                    ),
                    expression.span,
                ));
            }
            environment
'''
if old not in text:
    raise SystemExit("eval/mod.rs Canon symbol-resolution block not found")
path.write_text(text.replace(old, new, 1))

# 5. Value implements Drop, so the stable-handle test must borrow the Rc fields
# instead of moving them out of Value. The proof remains pointer identity.
path = Path("crates/my-lisp/src/eval/canon.rs")
text = path.read_text()
old = '''        let (Value::Builtin(historical), Value::Builtin(ukrainian), Value::Builtin(sanskrit)) =
            (historical, ukrainian, sanskrit)
        else {
            panic!("PRIM_CAR must be a first-class builtin value");
        };
        assert!(Rc::ptr_eq(&historical, &ukrainian));
        assert!(Rc::ptr_eq(&historical, &sanskrit));
'''
new = '''        let (Value::Builtin(historical), Value::Builtin(ukrainian), Value::Builtin(sanskrit)) =
            (&historical, &ukrainian, &sanskrit)
        else {
            panic!("PRIM_CAR must be a first-class builtin value");
        };
        assert!(Rc::ptr_eq(historical, ukrainian));
        assert!(Rc::ptr_eq(historical, sanskrit));
'''
if old not in text:
    raise SystemExit("canon.rs stable-handle test block not found")
path.write_text(text.replace(old, new, 1))
