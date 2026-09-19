//! Mechanical index/traversal for the experimental Lisp ground graph.
//!
//! Source model: experiments/ground-graph.lisp
//! Generic edge shape: (endpoint relation endpoint).
//! Relation identities are opaque 8-bit patterns. Rust does not assign meaning.

use std::collections::{HashMap, HashSet, VecDeque};
use wsm_kernel_c_abi::WsmKernelKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BitPattern8(pub u8);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CallableId {
    pub kernel: WsmKernelKind,
    pub local_id: BitPattern8,
}

impl CallableId {
    pub const fn new(kernel: WsmKernelKind, local_id: u8) -> Self {
        Self { kernel, local_id: BitPattern8(local_id) }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GraphEdge {
    pub left: CallableId,
    pub relation: BitPattern8,
    pub right: CallableId,
}

#[derive(Default)]
pub struct CallGraph {
    nodes: HashSet<CallableId>,
    edges: HashSet<GraphEdge>,
    outgoing: HashMap<CallableId, Vec<GraphEdge>>,
}

impl CallGraph {
    pub fn new() -> Self { Self::default() }

    pub fn register(&mut self, id: CallableId) -> bool {
        self.nodes.insert(id)
    }

    pub fn resolve(&self, id: &CallableId) -> Option<&CallableId> {
        self.nodes.get(id)
    }

    pub fn connect(&mut self, left: CallableId, relation: BitPattern8, right: CallableId) -> bool {
        if !self.nodes.contains(&left) || !self.nodes.contains(&right) {
            return false;
        }
        let edge = GraphEdge { left: left.clone(), relation, right };
        if !self.edges.insert(edge.clone()) {
            return false;
        }
        self.outgoing.entry(left).or_default().push(edge);
        true
    }

    pub fn disconnect(&mut self, edge: &GraphEdge) -> bool {
        if !self.edges.remove(edge) { return false; }
        if let Some(list) = self.outgoing.get_mut(&edge.left) {
            list.retain(|candidate| candidate != edge);
        }
        true
    }

    pub fn neighbors(&self, left: &CallableId, relation: BitPattern8) -> Vec<&CallableId> {
        self.outgoing
            .get(left)
            .into_iter()
            .flat_map(|edges| edges.iter())
            .filter(|edge| edge.relation == relation)
            .map(|edge| &edge.right)
            .collect()
    }

    pub fn has_route(&self, from: &CallableId, to: &CallableId, relation: BitPattern8) -> bool {
        if from == to { return self.nodes.contains(from); }
        if !self.nodes.contains(from) || !self.nodes.contains(to) { return false; }

        let mut seen = HashSet::new();
        let mut queue = VecDeque::from([from.clone()]);
        seen.insert(from.clone());

        while let Some(current) = queue.pop_front() {
            for next in self.neighbors(&current, relation) {
                if next == to { return true; }
                if seen.insert(next.clone()) { queue.push_back(next.clone()); }
            }
        }
        false
    }

    pub fn nodes(&self) -> impl Iterator<Item = &CallableId> { self.nodes.iter() }
    pub fn edges(&self) -> impl Iterator<Item = &GraphEdge> { self.edges.iter() }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Opaque experimental relation identities. Their meanings live in Lisp
    // experiment data, never in the Rust type system.
    const RELATION_A: BitPattern8 = BitPattern8(0b1111_0000);
    const RELATION_B: BitPattern8 = BitPattern8(0b1111_0001);

    fn id(kernel: WsmKernelKind, local: u8) -> CallableId { CallableId::new(kernel, local) }

    #[test]
    fn four_islands_resolve_through_one_index_without_one_root() {
        let mut g = CallGraph::new();
        let lisp = id(WsmKernelKind::Lisp, 0b0100_1101);
        let prolog = id(WsmKernelKind::Prolog, 0);
        let clips = id(WsmKernelKind::Clips, 0);
        let datalog = id(WsmKernelKind::Datalog, 0);
        for node in [&lisp, &prolog, &clips, &datalog] {
            assert!(g.register(node.clone()));
            assert!(g.resolve(node).is_some());
        }
        assert_eq!(g.nodes().count(), 4);
    }

    #[test]
    fn graph_uses_opaque_ternary_edges_like_ground_graph_lisp() {
        let mut g = CallGraph::new();
        let lisp = id(WsmKernelKind::Lisp, 0b0100_1101);
        let prolog = id(WsmKernelKind::Prolog, 0);
        g.register(lisp.clone());
        g.register(prolog.clone());
        assert!(g.connect(lisp.clone(), RELATION_A, prolog.clone()));
        assert_eq!(g.neighbors(&lisp, RELATION_A), vec![&prolog]);
        assert!(g.neighbors(&lisp, RELATION_B).is_empty());
    }

    #[test]
    fn removing_relation_evidence_does_not_remove_endpoint_identities() {
        let mut g = CallGraph::new();
        let lisp = id(WsmKernelKind::Lisp, 1);
        let prolog = id(WsmKernelKind::Prolog, 1);
        g.register(lisp.clone());
        g.register(prolog.clone());
        let edge = GraphEdge { left: lisp.clone(), relation: RELATION_A, right: prolog.clone() };
        assert!(g.connect(edge.left.clone(), edge.relation, edge.right.clone()));
        assert!(g.disconnect(&edge));
        assert!(g.resolve(&lisp).is_some());
        assert!(g.resolve(&prolog).is_some());
    }

    #[test]
    fn identical_local_bit_patterns_in_different_kernels_do_not_collide() {
        let mut g = CallGraph::new();
        let lisp = id(WsmKernelKind::Lisp, 0b0000_0100);
        let prolog = id(WsmKernelKind::Prolog, 0b0000_0100);
        assert!(g.register(lisp.clone()));
        assert!(g.register(prolog.clone()));
        assert_ne!(lisp, prolog);
        assert_eq!(g.nodes().count(), 2);
    }

    #[test]
    fn cycles_and_absent_routes_are_observations_not_errors() {
        let mut g = CallGraph::new();
        let lisp = id(WsmKernelKind::Lisp, 1);
        let prolog = id(WsmKernelKind::Prolog, 1);
        let datalog = id(WsmKernelKind::Datalog, 1);
        for node in [&lisp, &prolog, &datalog] { g.register(node.clone()); }
        g.connect(lisp.clone(), RELATION_A, prolog.clone());
        g.connect(prolog.clone(), RELATION_A, lisp.clone());
        assert!(g.has_route(&lisp, &prolog, RELATION_A));
        assert!(g.has_route(&prolog, &lisp, RELATION_A));
        assert!(!g.has_route(&lisp, &datalog, RELATION_A));
    }
}
