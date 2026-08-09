use crate::Value;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// **Known Risk:** Dropping a deeply nested `Environment` chain (thousands of levels)
/// could cause a stack overflow because `Rc<RefCell<Frame>>` uses Rust's recursive `Drop`.
/// This is not currently an issue since we only have one child level from root in most usage,
/// but it could appear if deep nesting of `let` or currying patterns emerges.
#[derive(Clone, Debug)]
pub struct Environment(Rc<RefCell<Frame>>, Rc<RefCell<Vec<String>>>, Rc<RefCell<Limits>>);

#[derive(Debug)]
struct Frame {
    values: HashMap<Rc<str>, Value>,
    parent: Option<Environment>,
}

/// Opt-in resource caps for one session, shared across every `Environment`
/// in its lexical tree (same sharing pattern as the output transcript) —
/// added 2026-08-09 so S1/S3's own named examples (`NumericOverflow`,
/// `OutOfMemory`) are real, testable categories, not just words in a
/// document. `None` (the default, via `root()`) means unbounded — the
/// Rust reference implementation's own choice, not a claim every future
/// implementation must match (see S1's open note).
/// Опційні межі ресурсу для однієї сесії, спільні для кожного
/// `Environment` у її лексичному дереві (той самий патерн, що й у
/// транскрипту виводу) — додано 2026-08-09, щоб власні названі приклади
/// S1/S3 (`NumericOverflow`, `OutOfMemory`) стали реальними, перевіряними
/// категоріями, не лише словами в документі. `None` (типово, через
/// `root()`) означає необмежено — власний вибір Rust-реалізації, не
/// твердження, що кожна майбутня реалізація мусить це повторити (див.
/// відкриту примітку в S1).
#[derive(Debug, Default)]
struct Limits {
    cons_limit: Option<usize>,
    cons_count: usize,
    numeric_bit_limit: Option<usize>,
    /// `None` (the default, via `root()`) means `process-run` always fails
    /// named — the opposite default from `cons_limit`/`numeric_bit_limit`
    /// (those default to *unbounded*, this defaults to *disabled*), because
    /// process execution is a categorically bigger capability than a
    /// resource cap: combined with the inbound networking `tcp-accept`
    /// gives a session (PLAN.md item 21), an unrestricted `process-run`
    /// would let a remote peer reach arbitrary command execution through a
    /// my-lisp program. `Some(programs)` opts a session into running only
    /// those exact program names, never a shell string — the host (e.g.
    /// `my-lisp-cli --allow-process=git`) decides the allowlist, a my-lisp
    /// program itself can never grant itself this.
    /// `None` (типово, через `root()`) означає, що `process-run` завжди
    /// провалюється названо — протилежний типовий стан до
    /// `cons_limit`/`numeric_bit_limit` (ті типово *необмежені*, цей типово
    /// *вимкнений*), бо виконання процесу — категорійно більша можливість,
    /// ніж межа ресурсу: разом із вхідною мережею, яку дає сесії
    /// `tcp-accept` (PLAN.md, пункт 21), необмежений `process-run` дав би
    /// віддаленому учаснику шлях до довільного виконання команд через
    /// my-lisp-програму. `Some(programs)` вмикає для сесії запуск лише цих
    /// точних імен програм, ніколи не рядок shell — хост (напр.
    /// `my-lisp-cli --allow-process=git`) вирішує allowlist, my-lisp-програма
    /// сама ніколи не може дозволити це собі.
    process_allowlist: Option<Vec<String>>,
}

impl Environment {
    pub fn root() -> Self {
        let environment = Self(
            Rc::new(RefCell::new(Frame {
                values: HashMap::new(),
                parent: None,
            })),
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(RefCell::new(Limits::default())),
        );
        environment.define("t", Value::Bool(true));
        environment
    }

    /// Opts this session into a maximum `cons` allocation count — past it,
    /// `cons` returns `ErrorKind::OutOfMemory` instead of succeeding.
    /// Simulates a genuinely bounded heap (an FPGA with 4096 cons cells,
    /// S3's own example) without needing real hardware to test the claim
    /// "bounded implementations fail named, never silently redefine `cons`."
    /// Вмикає для цієї сесії максимальну кількість `cons`-виділень — понад
    /// неї `cons` повертає `ErrorKind::OutOfMemory` замість успіху. Імітує
    /// справді обмежену купу (FPGA з 4096 cons-комірками, власний приклад
    /// S3) без потреби в реальному залізі, щоб перевірити твердження
    /// "обмежені реалізації провалюються названо, ніколи не переозначають
    /// сенс `cons` мовчки".
    pub fn with_cons_limit(self, limit: usize) -> Self {
        self.2.borrow_mut().cons_limit = Some(limit);
        self
    }

    /// Opts this session into a maximum bit width for exact arithmetic
    /// results — past it, `+`/`-`/`*`/`/` return `ErrorKind::NumericOverflow`
    /// instead of continuing to compute (never falling back to an inexact
    /// approximation — that would violate S1, not satisfy it).
    /// Вмикає для цієї сесії максимальну ширину в бітах для результатів
    /// точної арифметики — понад неї `+`/`-`/`*`/`/` повертають
    /// `ErrorKind::NumericOverflow` замість продовження обчислення (ніколи
    /// не відкочуючись до неточного наближення — це порушило б S1, не
    /// задовольнило б його).
    pub fn with_numeric_bit_limit(self, limit: usize) -> Self {
        self.2.borrow_mut().numeric_bit_limit = Some(limit);
        self
    }

    /// Opts this session into `process-run`, restricted to exactly the
    /// program names in `programs` — see `Limits::process_allowlist`'s own
    /// comment for why this defaults to fully disabled rather than
    /// unbounded. Only a host embedding the interpreter calls this (e.g.
    /// `my-lisp-cli`'s `--allow-process` flag); nothing in the language
    /// itself can reach it.
    /// Вмикає для сесії `process-run`, обмежений точно іменами програм у
    /// `programs` — див. власний коментар `Limits::process_allowlist` про
    /// те, чому це типово повністю вимкнено, не необмежено. Викликає лише
    /// хост, що вбудовує інтерпретатор (напр. прапор `--allow-process` у
    /// `my-lisp-cli`); нічого в самій мові не може до цього дотягнутись.
    pub fn with_process_allowlist(self, programs: Vec<String>) -> Self {
        self.2.borrow_mut().process_allowlist = Some(programs);
        self
    }

    /// Called by `cons` before allocating; `Err(())` means the configured
    /// limit (if any) is already reached. No-op (always `Ok`) when this
    /// session never opted into a limit.
    /// Викликається `cons` перед виділенням; `Err(())` означає, що
    /// налаштована межа (якщо є) уже досягнута. Нічого не робить (завжди
    /// `Ok`), якщо ця сесія ніколи не вмикала межу.
    pub(crate) fn try_alloc_cons(&self) -> Result<(), ()> {
        let mut limits = self.2.borrow_mut();
        if let Some(limit) = limits.cons_limit {
            if limits.cons_count >= limit {
                return Err(());
            }
        }
        limits.cons_count += 1;
        Ok(())
    }

    /// The configured numeric bit-width cap, if this session opted into one.
    /// Налаштована межа ширини числа в бітах, якщо ця сесія її ввімкнула.
    pub(crate) fn numeric_bit_limit(&self) -> Option<usize> {
        self.2.borrow().numeric_bit_limit
    }

    /// `false` unless this session called `with_process_allowlist` *and*
    /// `program` is exactly one of the names it listed — no substring
    /// match, no path resolution tricks, an allowed name must match in
    /// full.
    /// `false`, якщо ця сесія не викликала `with_process_allowlist` *або*
    /// `program` не є точно одним з перелічених там імен — без часткового
    /// збігу, без хитрощів із роздільною здатністю шляху, дозволене ім'я
    /// має збігатись повністю.
    pub(crate) fn is_process_allowed(&self, program: &str) -> bool {
        match &self.2.borrow().process_allowlist {
            Some(programs) => programs.iter().any(|allowed| allowed == program),
            None => false,
        }
    }

    /// A child frame is the future lexical boundary captured by a closure.
    /// It shares the parent's output sink (the second field, cloned as the
    /// same `Rc`, not reinitialized) so `print` inside a closure body still
    /// lands in the one session-wide transcript rather than a per-call one.
    /// Дочірній фрейм стане майбутньою лексичною межею, яку зберігатиме
    /// замикання. Він ділить sink виводу батька (друге поле, клонується як
    /// той самий `Rc`, не переініціалізується), тож `print` усередині тіла
    /// замикання все одно потрапляє в один спільний на сесію транскрипт.
    /// Ein untergeordneter Frame ist die künftige lexikalische Grenze einer
    /// Closure. Er teilt sich die Ausgabesenke des Elternteils (das zweite
    /// Feld, als derselbe `Rc` geklont, nicht neu initialisiert), sodass
    /// `print` im Rumpf einer Closure weiterhin im einen sitzungsweiten
    /// Transkript landet statt in einem pro Aufruf.
    pub fn child(&self) -> Self {
        Self(
            Rc::new(RefCell::new(Frame {
                values: HashMap::new(),
                parent: Some(self.clone()),
            })),
            self.1.clone(),
            self.2.clone(),
        )
    }

    /// Appends a line to the session-wide output transcript, shared by every
    /// `Environment` in this session's lexical tree (root and all closures).
    /// Додає рядок до транскрипту виводу, спільного на всю сесію — його
    /// поділяють усі `Environment` у лексичному дереві цієї сесії.
    /// Hängt eine Zeile an das sitzungsweite Ausgabetranskript an, das sich
    /// jede `Environment` im lexikalischen Baum dieser Sitzung teilt.
    pub fn print(&self, line: String) {
        self.1.borrow_mut().push(line);
    }

    /// A snapshot of everything `print` has produced so far in this session.
    /// Знімок усього, що `print` уже вивів у цій сесії.
    /// Ein Schnappschuss von allem, was `print` in dieser Sitzung bisher ausgegeben hat.
    pub fn output_snapshot(&self) -> Vec<String> {
        self.1.borrow().clone()
    }

    pub fn define(&self, name: impl Into<Rc<str>>, value: Value) {
        self.0.borrow_mut().values.insert(name.into(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        let mut current = Some(self.clone());
        while let Some(env) = current {
            if let Some(value) = env.0.borrow().values.get(name) {
                return Some(value.clone());
            }
            current = env.0.borrow().parent.clone();
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    pub environment: Environment,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            environment: Environment::root(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Exactness;

    #[test]
    fn root_predefines_t_as_true() {
        let root = Environment::root();
        assert_eq!(root.get("t"), Some(Value::Bool(true)));
    }

    #[test]
    fn define_then_get_returns_the_value() {
        let root = Environment::root();
        root.define("x", Value::Number(1.0, Exactness::Exact));
        assert_eq!(root.get("x"), Some(Value::Number(1.0, Exactness::Exact)));
    }

    #[test]
    fn get_on_unknown_name_returns_none() {
        let root = Environment::root();
        assert_eq!(root.get("does-not-exist"), None);
    }

    #[test]
    fn child_reads_bindings_from_its_parent() {
        let root = Environment::root();
        root.define("x", Value::Number(1.0, Exactness::Exact));
        let child = root.child();
        assert_eq!(child.get("x"), Some(Value::Number(1.0, Exactness::Exact)));
    }

    #[test]
    fn child_definitions_do_not_leak_into_the_parent() {
        // Lexical scoping requires that a child frame's bindings stay local:
        // a closure's parameters must never become visible outside its call.
        // Лексичний скоуп вимагає, щоб зв’язування дочірнього фрейму лишались
        // локальними: параметри замикання не повинні ставати видимими зовні виклику.
        // Lexikalischer Scope verlangt, dass Bindungen eines Kind-Frames lokal
        // bleiben: Parameter einer Closure dürfen außerhalb ihres Aufrufs nie sichtbar werden.
        let root = Environment::root();
        let child = root.child();
        child.define("local", Value::Number(2.0, Exactness::Exact));
        assert_eq!(root.get("local"), None);
    }

    #[test]
    fn child_binding_shadows_the_parent_without_mutating_it() {
        let root = Environment::root();
        root.define("x", Value::Number(1.0, Exactness::Exact));
        let child = root.child();
        child.define("x", Value::Number(2.0, Exactness::Exact));
        assert_eq!(child.get("x"), Some(Value::Number(2.0, Exactness::Exact)));
        assert_eq!(root.get("x"), Some(Value::Number(1.0, Exactness::Exact)));
    }

    #[test]
    fn redefining_in_the_same_frame_overwrites_the_previous_value() {
        let root = Environment::root();
        root.define("x", Value::Number(1.0, Exactness::Exact));
        root.define("x", Value::Number(2.0, Exactness::Exact));
        assert_eq!(root.get("x"), Some(Value::Number(2.0, Exactness::Exact)));
    }
}
