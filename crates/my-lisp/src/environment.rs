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
    /// `None` (typovo, cherez `root()`) oznachaie, shcho `process-run` zavzhdy
    /// provaliuietsia nazvano — protylezhnyi typovyi stan do
    /// `cons_limit`/`numeric_bit_limit` (ti typovo *neobmezheni*, tsei typovo
    /// *vymknenyi*), bo vykonannia protsesu — katehoriino bilsha mozhlyvist,
    /// nizh mezha resursu: razom iz vkhidnoiu merezheiu, yaku daie sesii
    /// `tcp-accept` (PLAN.md, punkt 21), neobmezhenyi `process-run` dav by
    /// viddalenomu uchasnyku shliakh do dovilnoho vykonannia komand cherez
    /// my-lisp-prohramu. `Some(programs)` vmykaie dlia sesii zapusk lyshe tsykh
    /// tochnykh imen prohram, nikoly ne riadok shell — khost (napr.
    /// `my-lisp-cli --allow-process=git`) vyrishuie allowlist, my-lisp-prohrama
    /// sama nikoly ne mozhe dozvolyty tse sobi.
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

    /// Opts this session into `process-run`, restricted to exactly the
    /// program names in `programs` — see `Limits::process_allowlist`'s own
    /// comment for why this defaults to fully disabled rather than
    /// unbounded. Only a host embedding the interpreter calls this (e.g.
    /// `my-lisp-cli`'s `--allow-process` flag); nothing in the language
    /// itself can reach it.
    /// Vmykaie dlia sesii `process-run`, obmezhenyi tochno imenamy prohram u
    /// `programs` — dyv. vlasnyi komentar `Limits::process_allowlist` pro
    /// te, chomu tse typovo povnistiu vymkneno, ne neobmezheno. Vyklykaie lyshe
    /// khost, shcho vbudovuie interpretator (napr. prapor `--allow-process` u
    /// `my-lisp-cli`); nichoho v samii movi ne mozhe do tsoho dotiahnutys.
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

    /// `false` unless this session called `with_process_allowlist` *and*
    /// `program` is exactly one of the names it listed — no substring
    /// match, no path resolution tricks, an allowed name must match in
    /// full.
    /// `false`, yakshcho tsia sesiia ne vyklykala `with_process_allowlist` *abo*
    /// `program` ne ye tochno odnym z perelichenykh tam imen — bez chastkovoho
    /// zbihu, bez khytroshchiv iz rozdilnoiu zdatnistiu shliakhu, dozvolene imia
    /// maie zbihatys povnistiu.
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
        self.1.borrow_mut().push(line);
    }

    /// A snapshot of everything `print` has produced so far in this session.
    /// Znimok usoho, shcho `print` uzhe vyviv u tsii sesii.
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
