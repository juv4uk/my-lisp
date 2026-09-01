use crate::Value;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// Dropping a deeply nested `Environment` chain (thousands of `let`/currying
/// levels) would otherwise recurse through `Rc<RefCell<Frame>>`'s default
/// `Drop` one stack frame per level and could overflow the stack. `Environment`
/// has its own `Drop` below (mirroring `Value::Pair`'s iterative Drop in
/// `value.rs`) that walks the parent chain with an explicit worklist instead,
/// so this is fixed, not just documented as a live risk.
/// Session-wide print transcript plus a consumer cursor, so hot hosts
/// (REPL, LSP, swarm TCP) can take only the lines appended since their
/// last read instead of re-cloning the whole history per evaluation.
#[derive(Debug)]
pub struct Transcript {
    lines: Vec<String>,
    taken: usize,
}

#[derive(Clone, Debug)]
pub struct Environment(
    Rc<RefCell<Frame>>,
    Rc<RefCell<Transcript>>,
    Rc<RefCell<Limits>>,
);

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
/// Optsiini mezhi resursu dlia odniiei sesii, spilni dlia kozhnoho
/// `Environment` u yii leksychnomu derevi (toi samyi patern, shcho y u
/// transkryptu vyvodu) — dodano 2026-08-09, shchob vlasni nazvani pryklady
/// S1/S3 (`NumericOverflow`, `OutOfMemory`) staly realnymy, perevirianymy
/// katehoriiamy, ne lyshe slovamy v dokumenti. `None` (typovo, cherez
/// `root()`) oznachaie neobmezheno — vlasnyi vybir Rust-realizatsii, ne
/// tverdzhennia, shcho kozhna maibutnia realizatsiia musyt tse povtoryty (dyv.
/// vidkrytu prymitku v S1).
#[derive(Debug, Default)]
struct Limits {
    cons_limit: Option<usize>,
    cons_count: usize,
    numeric_bit_limit: Option<usize>,
    /// `None` (the default, via `root()`) means unrestricted named-program
    /// execution once a native host installs `process-run`. This is the
    /// trusted Lisp-machine profile: programs can compose the host instead
    /// of requiring a per-executable grant. `Some(programs)` remains an
    /// embedding boundary for untrusted entry points such as the loopback
    /// TCP oracle; an empty list disables process execution there. Commands
    /// still never pass through a shell.
    process_allowlist: Option<Vec<String>>,
}

impl Environment {
    pub fn root() -> Self {
        let environment = Self(
            Rc::new(RefCell::new(Frame {
                values: HashMap::new(),
                parent: None,
            })),
            Rc::new(RefCell::new(Transcript {
                lines: Vec::new(),
                taken: 0,
            })),
            Rc::new(RefCell::new(Limits::default())),
        );
        environment.define("t", Value::Bool(true));
        // contract 2.1: primitives enter the root environment as
        // first-class builtin values -- one runtime authority, no
        // head-only registry (docs/PROPOSAL-FIRST-CLASS-BUILTINS.md).
        crate::eval::builtins::install(&environment);
        environment
    }

    /// Opts this session into a maximum `cons` allocation count — past it,
    /// `cons` returns `ErrorKind::OutOfMemory` instead of succeeding.
    /// Simulates a genuinely bounded heap (an FPGA with 4096 cons cells,
    /// S3's own example) without needing real hardware to test the claim
    /// "bounded implementations fail named, never silently redefine `cons`."
    /// Vmykaie dlia tsiiei sesii maksymalnu kilkist `cons`-vydilen — ponad
    /// nei `cons` povertaie `ErrorKind::OutOfMemory` zamist uspikhu. Imituie
    /// spravdi obmezhenu kupu (FPGA z 4096 cons-komirkamy, vlasnyi pryklad
    /// S3) bez potreby v realnomu zalizi, shchob pereviryty tverdzhennia
    /// "obmezheni realizatsii provaliuiutsia nazvano, nikoly ne pereoznachaiut
    /// sens `cons` movchky".
    pub fn with_cons_limit(self, limit: usize) -> Self {
        self.2.borrow_mut().cons_limit = Some(limit);
        self
    }

    /// Opts this session into a maximum bit width for exact arithmetic
    /// results — past it, `+`/`-`/`*`/`/` return `ErrorKind::NumericOverflow`
    /// instead of continuing to compute (never falling back to an inexact
    /// approximation — that would violate S1, not satisfy it).
    /// Vmykaie dlia tsiiei sesii maksymalnu shyrynu v bitakh dlia rezultativ
    /// tochnoi aryfmetyky — ponad nei `+`/`-`/`*`/`/` povertaiut
    /// `ErrorKind::NumericOverflow` zamist prodovzhennia obchyslennia (nikoly
    /// ne vidkochuiuchys do netochnoho nablyzhennia — tse porushylo b S1, ne
    /// zadovolnylo b yoho).
    pub fn with_numeric_bit_limit(self, limit: usize) -> Self {
        self.2.borrow_mut().numeric_bit_limit = Some(limit);
        self
    }

    /// Restricts `process-run` to the exact program names in `programs`.
    /// This is an embedding/network policy, not the native machine default.
    pub fn with_process_allowlist(self, programs: Vec<String>) -> Self {
        self.2.borrow_mut().process_allowlist = Some(programs);
        self
    }

    /// Called by `cons` before allocating; `Err(())` means the configured
    /// limit (if any) is already reached. No-op (always `Ok`) when this
    /// session never opted into a limit.
    /// Vyklykaietsia `cons` pered vydilenniam; `Err(())` oznachaie, shcho
    /// nalashtovana mezha (yakshcho ye) uzhe dosiahnuta. Nichoho ne robyt (zavzhdy
    /// `Ok`), yakshcho tsia sesiia nikoly ne vmykala mezhu.
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
    /// Nalashtovana mezha shyryny chysla v bitakh, yakshcho tsia sesiia yii vvimknula.
    pub(crate) fn numeric_bit_limit(&self) -> Option<usize> {
        self.2.borrow().numeric_bit_limit
    }

    /// Native root sessions are unrestricted (`None`). An embedding can set
    /// an exact allowlist; an empty allowlist is an explicit deny-all policy.
    pub fn is_process_allowed(&self, program: &str) -> bool {
        match &self.2.borrow().process_allowlist {
            Some(programs) => programs.iter().any(|allowed| allowed == program),
            None => true,
        }
    }

    /// A child frame is the future lexical boundary captured by a closure.
    /// It shares the parent's output sink (the second field, cloned as the
    /// same `Rc`, not reinitialized) so `print` inside a closure body still
    /// lands in the one session-wide transcript rather than a per-call one.
    /// Dochirnii freim stane maibutnoiu leksychnoiu mezheiu, yaku zberihatyme
    /// zamykannia. Vin dilyt sink vyvodu batka (druhe pole, klonuietsia yak
    /// toi samyi `Rc`, ne pereinitsializuietsia), tozh `print` useredyni tila
    /// zamykannia vse odno potrapliaie v odyn spilnyi na sesiiu transkrypt.
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
    /// Dodaie riadok do transkryptu vyvodu, spilnoho na vsiu sesiiu — yoho
    /// podiliaiut usi `Environment` u leksychnomu derevi tsiiei sesii.
    /// Hängt eine Zeile an das sitzungsweite Ausgabetranskript an, das sich
    /// jede `Environment` im lexikalischen Baum dieser Sitzung teilt.
    pub fn print(&self, line: String) {
        self.1.borrow_mut().lines.push(line);
    }

    /// A snapshot of everything `print` has produced so far in this session.
    /// Znimok usoho, shcho `print` uzhe vyviv u tsii sesii.
    /// Ein Schnappschuss von allem, was `print` in dieser Sitzung bisher ausgegeben hat.
    pub fn output_snapshot(&self) -> Vec<String> {
        self.1.borrow().lines.clone()
    }

    /// Returns and consumes the lines printed since the previous call —
    /// O(new lines), not O(session output). Full-history reads stay
    /// available through `output_snapshot`.
    /// Povertaie i pohlynaie riadky, nadrukovani z poperednoho vyklyku —
    /// O(novykh riadkiv), ne O(usoho vyvodu sesii).
    pub fn output_take_new(&self) -> Vec<String> {
        let mut transcript = self.1.borrow_mut();
        let new = transcript.lines[transcript.taken..].to_vec();
        transcript.taken = transcript.lines.len();
        new
    }

    /// Snapshot of every visible binding, root-first, shadowed names
    /// resolved to their innermost value (contract 2.1: builtins live in
    /// the environment now, so introspection is possible).
    pub fn snapshot(&self) -> Vec<(Rc<str>, Value)> {
        let mut frames = Vec::new();
        let mut current = Some(self.clone());
        while let Some(env) = current {
            frames.push(env.0.clone());
            current = env.0.borrow().parent.clone();
        }
        let mut out: Vec<(Rc<str>, Value)> = Vec::new();
        for frame in frames.iter().rev() {
            let f = frame.borrow();
            for (name, value) in f.values.iter() {
                match out.iter_mut().find(|(n, _)| n == name) {
                    Some(slot) => slot.1 = value.clone(),
                    None => out.push((name.clone(), value.clone())),
                }
            }
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
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

impl Drop for Environment {
    fn drop(&mut self) {
        // Swap this Environment's real content out for a cheap, parentless
        // sentinel so the field-drop that runs after this function returns
        // is O(1), then manually walk the extracted frame chain iteratively
        // instead of letting Rust's recursive Drop walk `Frame.parent`.
        // Mirrors `Value::Pair`'s Drop in value.rs.
        let taken = std::mem::replace(
            self,
            Environment(
                Rc::new(RefCell::new(Frame {
                    values: HashMap::new(),
                    parent: None,
                })),
                Rc::new(RefCell::new(Transcript {
                    lines: Vec::new(),
                    taken: 0,
                })),
                Rc::new(RefCell::new(Limits::default())),
            ),
        );

        let mut worklist = vec![taken];
        while let Some(env) = worklist.pop() {
            let env = std::mem::ManuallyDrop::new(env);
            // SAFETY: `env` is ManuallyDrop, so each field is read out of it
            // exactly once here and nothing double-drops when `env` itself
            // goes out of scope at the end of this iteration (ManuallyDrop's
            // own Drop is a no-op).
            let frame_rc = unsafe { std::ptr::read(&env.0) };
            let transcript_rc = unsafe { std::ptr::read(&env.1) };
            let limits_rc = unsafe { std::ptr::read(&env.2) };
            // Ordinary, non-recursive Rc drops -- Transcript/Limits never
            // nest another Environment.
            drop(transcript_rc);
            drop(limits_rc);

            if let Ok(cell) = Rc::try_unwrap(frame_rc) {
                let mut frame = cell.into_inner();
                // frame.values (HashMap<Rc<str>, Value>) drops normally at
                // the end of this arm -- Value already has its own
                // iterative Drop for long Pair chains, so this is safe
                // regardless of how large a single frame's bindings are.
                if let Some(parent) = frame.parent.take() {
                    worklist.push(parent);
                }
            }
            // Err case: frame_rc is still referenced elsewhere (shared via
            // Environment::clone); dropping this handle just decrements the
            // refcount, no recursion.
        }
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
    fn dropping_a_very_deep_environment_chain_does_not_overflow_the_stack() {
        // Regression for the stack-overflow-on-deep-drop risk this file
        // used to document as live and unmitigated (see Environment's Drop
        // impl above). Each child() holds the only strong reference to its
        // parent's frame by the time this loop finishes reassigning
        // `current`, so dropping the final Environment walks a genuine
        // 300k-level singly-referenced chain through the iterative
        // worklist -- the same order of magnitude as the long-list
        // regression Value::Pair's Drop already guards against.
        let mut current = Environment::root();
        for _ in 0..300_000 {
            current = current.child();
        }
        drop(current);
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
        // Leksychnyi skoup vymahaie, shchob zv’yazuvannia dochirnoho freimu lyshalys
        // lokalnymy: parametry zamykannia ne povynni stavaty vydymymy zovni vyklyku.
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
