use crate::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone, Debug, Default)]
struct Limits {
    max_numeric_bits: Option<usize>,
    max_cons_cells: Option<usize>,
    cons_cells_used: usize,
    fs_read_roots: Option<Vec<PathBuf>>,
    fs_write_roots: Option<Vec<PathBuf>>,
    tcp_connect_allow: Option<Vec<(IpAddr, u16)>>,
    tcp_listen_allow: Option<Vec<(IpAddr, u16)>>,
    process_allowlist: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
struct Frame {
    bindings: HashMap<String, Value>,
    parent: Option<Environment>,
}

#[derive(Clone, Debug)]
pub struct Environment(
    Rc<RefCell<Frame>>,
    Rc<RefCell<Vec<String>>>,
    Rc<RefCell<Limits>>,
);

impl Environment {
    pub fn root() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert("t".into(), Value::Symbol(Rc::from("t")));
        Self(
            Rc::new(RefCell::new(Frame {
                bindings,
                parent: None,
            })),
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(RefCell::new(Limits::default())),
        )
    }

    pub fn child(&self) -> Self {
        Self(
            Rc::new(RefCell::new(Frame {
                bindings: HashMap::new(),
                parent: Some(self.clone()),
            })),
            self.1.clone(),
            self.2.clone(),
        )
    }

    pub fn define(&self, name: impl Into<String>, value: Value) {
        self.0.borrow_mut().bindings.insert(name.into(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        let mut current = Some(self.clone());
        while let Some(environment) = current {
            let frame = environment.0.borrow();
            if let Some(value) = frame.bindings.get(name) {
                return Some(value.clone());
            }
            current = frame.parent.clone();
        }
        None
    }

    pub fn record_output(&self, line: String) {
        self.1.borrow_mut().push(line);
    }

    pub fn output_snapshot(&self) -> Vec<String> {
        self.1.borrow().clone()
    }

    pub fn output_take_new(&self) -> Vec<String> {
        std::mem::take(&mut *self.1.borrow_mut())
    }

    /// Opt in to a maximum exact-integer/rational numerator/denominator size.
    /// `None` (the default) preserves the language's unbounded arithmetic
    /// contract. The limit is a resource guard, not a numeric semantic change.
    pub fn with_max_numeric_bits(self, max_bits: usize) -> Self {
        self.2.borrow_mut().max_numeric_bits = Some(max_bits);
        self
    }

    pub fn max_numeric_bits(&self) -> Option<usize> {
        self.2.borrow().max_numeric_bits
    }

    /// Opt in to a maximum number of cons cells allocated through the evaluator.
    /// `None` (the default) keeps the historical unbounded behavior.
    pub fn with_max_cons_cells(self, max_cells: usize) -> Self {
        self.2.borrow_mut().max_cons_cells = Some(max_cells);
        self
    }

    pub fn max_cons_cells(&self) -> Option<usize> {
        self.2.borrow().max_cons_cells
    }

    pub(crate) fn reserve_cons_cell(&self) -> bool {
        let mut limits = self.2.borrow_mut();
        let Some(max_cells) = limits.max_cons_cells else {
            return true;
        };
        if limits.cons_cells_used >= max_cells {
            return false;
        }
        limits.cons_cells_used += 1;
        true
    }

    /// Restrict filesystem reads (`read-file`, `read-dir`, `load`, raw byte
    /// reads) to these roots. `None` remains the trusted native default;
    /// `Some([])` is explicit deny-all.
    pub fn with_fs_read_roots(self, roots: Vec<PathBuf>) -> Self {
        self.2.borrow_mut().fs_read_roots = Some(roots);
        self
    }

    /// Restrict filesystem writes (`write-file`, raw byte writes) to these
    /// roots. `None` remains unrestricted; `Some([])` is deny-all.
    pub fn with_fs_write_roots(self, roots: Vec<PathBuf>) -> Self {
        self.2.borrow_mut().fs_write_roots = Some(roots);
        self
    }

    pub fn fs_read_roots(&self) -> Option<Vec<PathBuf>> {
        self.2.borrow().fs_read_roots.clone()
    }

    pub fn fs_write_roots(&self) -> Option<Vec<PathBuf>> {
        self.2.borrow().fs_write_roots.clone()
    }

    /// Restrict outbound TCP connects to exact IP/port pairs. `None` is the
    /// trusted native default; `Some([])` is deny-all.
    pub fn with_tcp_connect_allow(self, destinations: Vec<(IpAddr, u16)>) -> Self {
        self.2.borrow_mut().tcp_connect_allow = Some(destinations);
        self
    }

    /// Restrict TCP listeners to exact IP/port pairs. `None` is the trusted
    /// native default; `Some([])` is deny-all.
    pub fn with_tcp_listen_allow(self, destinations: Vec<(IpAddr, u16)>) -> Self {
        self.2.borrow_mut().tcp_listen_allow = Some(destinations);
        self
    }

    pub fn tcp_connect_allow(&self) -> Option<Vec<(IpAddr, u16)>> {
        self.2.borrow().tcp_connect_allow.clone()
    }

    pub fn tcp_listen_allow(&self) -> Option<Vec<(IpAddr, u16)>> {
        self.2.borrow().tcp_listen_allow.clone()
    }

    /// Restrict process execution to exact program names. `None` keeps the
    /// trusted native profile unrestricted; `Some([])` is explicit deny-all.
    pub fn with_process_allowlist(self, programs: Vec<String>) -> Self {
        self.2.borrow_mut().process_allowlist = Some(programs);
        self
    }

    pub fn process_allowlist(&self) -> Option<Vec<String>> {
        self.2.borrow().process_allowlist.clone()
    }

    pub fn is_process_allowed(&self, program: &str) -> bool {
        self.2
            .borrow()
            .process_allowlist
            .as_ref()
            .map(|programs| programs.iter().any(|allowed| allowed == program))
            .unwrap_or(true)
    }

    pub fn is_tcp_connect_allowed(&self, ip: IpAddr, port: u16) -> bool {
        self.2
            .borrow()
            .tcp_connect_allow
            .as_ref()
            .map(|allowed| allowed.iter().any(|entry| *entry == (ip, port)))
            .unwrap_or(true)
    }

    pub fn is_tcp_listen_allowed(&self, ip: IpAddr, port: u16) -> bool {
        self.2
            .borrow()
            .tcp_listen_allow
            .as_ref()
            .map(|allowed| allowed.iter().any(|entry| *entry == (ip, port)))
            .unwrap_or(true)
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        if Rc::strong_count(&self.0) != 1 {
            return;
        }

        let taken = std::mem::replace(
            self,
            Environment(
                Rc::new(RefCell::new(Frame {
                    bindings: HashMap::new(),
                    parent: None,
                })),
                Rc::new(RefCell::new(Vec::new())),
                Rc::new(RefCell::new(Limits::default())),
            ),
        );

        let mut worklist = vec![taken];
        while let Some(env) = worklist.pop() {
            let env = std::mem::ManuallyDrop::new(env);
            // SAFETY: `env` is ManuallyDrop, so each field is read exactly once.
            let frame_rc = unsafe { std::ptr::read(&env.0) };
            let transcript_rc = unsafe { std::ptr::read(&env.1) };
            let limits_rc = unsafe { std::ptr::read(&env.2) };
            drop(transcript_rc);
            drop(limits_rc);

            if let Ok(cell) = Rc::try_unwrap(frame_rc) {
                let mut frame = cell.into_inner();
                if let Some(parent) = frame.parent.take() {
                    worklist.push(parent);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    pub environment: Environment,
}

impl Default for Session {
    fn default() -> Self {
        let mut session = Self {
            environment: Environment::root(),
        };
        crate::load_macro_library(&mut session)
            .expect("embedded lib/macro.my must bootstrap a default Session");
        session
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Exactness;

    #[test]
    fn root_predefines_t_as_the_self_evaluating_truth_symbol() {
        let root = Environment::root();
        assert_eq!(root.get("t"), Some(Value::Symbol(Rc::from("t"))));
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

    #[test]
    fn host_policies_are_unrestricted_by_default_and_shared_with_children() {
        let root = Environment::root()
            .with_fs_read_roots(vec![PathBuf::from("/tmp/read")])
            .with_fs_write_roots(vec![PathBuf::from("/tmp/write")])
            .with_tcp_connect_allow(vec![("127.0.0.1".parse().unwrap(), 9999)])
            .with_tcp_listen_allow(vec![("127.0.0.1".parse().unwrap(), 9100)])
            .with_process_allowlist(vec!["git".into()]);
        let child = root.child();
        assert_eq!(child.fs_read_roots(), root.fs_read_roots());
        assert_eq!(child.fs_write_roots(), root.fs_write_roots());
        assert_eq!(child.tcp_connect_allow(), root.tcp_connect_allow());
        assert_eq!(child.tcp_listen_allow(), root.tcp_listen_allow());
        assert_eq!(child.process_allowlist(), root.process_allowlist());
    }
}
