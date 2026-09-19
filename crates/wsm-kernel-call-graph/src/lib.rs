//! Namespaced callable graph for the four-kernel experiment.
//!
//! The graph records mechanical reachability and explicit bridge topology.
//! It does not claim semantic equivalence between nodes.

use std::collections::{HashMap, HashSet, VecDeque};
use wsm_kernel_c_abi::WsmKernelKind;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CallableId {
    pub kernel: WsmKernelKind,
    pub local_id: String,
}

impl CallableId {
    pub fn new(kernel: WsmKernelKind, local_id: impl Into<String>) -> Self {
        Self { kernel, local_id: local_id.into() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    Call,
    Bridge,
    Observe,
    Transport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallableNode {
    pub id: CallableId,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Edge {
    pub from: CallableId,
    pub to: CallableId,
    pub kind: EdgeKind,
}

#[derive(Default)]
pub struct CallGraph {
    nodes: HashMap<CallableId, CallableNode>,
    edges: HashSet<Edge>,
}

impl CallGraph {
    pub fn new() -> Self { Self::default() }

    pub fn register(&mut self, id: CallableId) -> bool {
        self.nodes.insert(id.clone(), CallableNode { id }).is_none()
    }

    pub fn resolve(&self, id: &CallableId) -> Option<&CallableNode> {
        self.nodes.get(id)
    }

    pub fn connect(&mut self, from: CallableId, to: CallableId, kind: EdgeKind) -> bool {
        if !self.nodes.contains_key(&from) || !self.nodes.contains_key(&to) {
            return false;
        }
        self.edges.insert(Edge { from, to, kind })
    }

    pub fn disconnect(&mut self, from: &CallableId, to: &CallableId, kind: EdgeKind) -> bool {
        self.edges.remove(&Edge { from: from.clone(), to: to.clone(), kind })
    }

    pub fn has_direct_edge(&self, from: &CallableId, to: &CallableId, kind: EdgeKind) -> bool {
        self.edges.contains(&Edge { from: from.clone(), to: to.clone(), kind })
    }

    pub fn has_route(&self, from: &CallableId, to: &CallableId, kind: EdgeKind) -> bool {
        if from == to { return self.nodes.contains_key(from); }
        if !self.nodes.contains_key(from) || !self.nodes.contains_key(to) { return false; }

        let mut seen = HashSet::new();
        let mut queue = VecDeque::from([from.clone()]);
        seen.insert(from.clone());

        while let Some(current) = queue.pop_front() {
            for edge in self.edges.iter().filter(|e| e.kind == kind && e.from == current) {
                if &edge.to == to { return true; }
                if seen.insert(edge.to.clone()) {
                    queue.push_back(edge.to.clone());
                }
            }
        }
        false
    }

    pub fn nodes(&self) -> impl Iterator<Item = &CallableNode> { self.nodes.values() }
    pub fn edges(&self) -> impl Iterator<Item = &Edge> { self.edges.iter() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(kernel: WsmKernelKind, local: &str) -> CallableId {
        CallableId::new(kernel, local)
    }

    #[test]
    fn four_islands_resolve_through_one_index_without_one_root() {
        let mut g = CallGraph::new();
        let lisp = id(WsmKernelKind::Lisp, "eval");
        let prolog = id(WsmKernelKind::Prolog, "query");
        let clips = id(WsmKernelKind::Clips, "run");
        let datalog = id(WsmKernelKind::Datalog, "derive");

        for node in [&lisp, &prolog, &clips, &datalog] {
            assert!(g.register(node.clone()));
            assert!(g.resolve(node).is_some());
        }
        assert_eq!(g.nodes().count(), 4);
    }

    #[test]
    fn removing_lisp_to_prolog_bridge_does_not_remove_either_callable() {
        let mut g = CallGraph::new();
        let lisp = id(WsmKernelKind::Lisp, "eval");
        let prolog = id(WsmKernelKind::Prolog, "query");
        g.register(lisp.clone());
        g.register(prolog.clone());
        assert!(g.connect(lisp.clone(), prolog.clone(), EdgeKind::Bridge));
        assert!(g.has_direct_edge(&lisp, &prolog, EdgeKind::Bridge));
        assert!(g.disconnect(&lisp, &prolog, EdgeKind::Bridge));
        assert!(g.resolve(&lisp).is_some());
        assert!(g.resolve(&prolog).is_some());
        assert!(!g.has_route(&lisp, &prolog, EdgeKind::Bridge));
    }

    #[test]
    fn cycles_are_allowed_and_missing_route_stays_missing() {
        let mut g = CallGraph::new();
        let lisp = id(WsmKernelKind::Lisp, "eval");
        let prolog = id(WsmKernelKind::Prolog, "query");
        let datalog = id(WsmKernelKind::Datalog, "derive");
        for node in [&lisp, &prolog, &datalog] { g.register(node.clone()); }

        g.connect(lisp.clone(), prolog.clone(), EdgeKind::Call);
        g.connect(prolog.clone(), lisp.clone(), EdgeKind::Call);
        assert!(g.has_route(&lisp, &prolog, EdgeKind::Call));
        assert!(g.has_route(&prolog, &lisp, EdgeKind::Call));
        assert!(!g.has_route(&lisp, &datalog, EdgeKind::Call));
    }

    #[test]
    fn identical_local_ids_in_different_kernels_do_not_collide() {
        let mut g = CallGraph::new();
        let lisp = id(WsmKernelKind::Lisp, "0104");
        let prolog = id(WsmKernelKind::Prolog, "0104");
        assert!(g.register(lisp.clone()));
        assert!(g.register(prolog.clone()));
        assert_ne!(lisp, prolog);
        assert_eq!(g.nodes().count(), 2);
    }
}
