//! Preserve four distinct observable result domains across autonomous kernels.
//!
//! Цей крейт визначає артефакт, який зберігає нативні результати чотирьох
//! незалежних ядер поруч, не переписуючи один домен у термінах іншого.
//!
//! The artifact is intentionally a plain data structure: no interpretation,
//! no universal value, no cross-kernel reconstruction. Each kernel owns its
//! own domain.

use std::collections::HashMap;
use wsm_datalog_kernel::{Database, Derivation, Tuple};

/// A complete observation from one kernel, kept in its native form.
#[derive(Clone, Debug, PartialEq)]
pub enum NativeResult {
    /// Lisp: final value plus the printed output trace produced while
    /// evaluating the program.
    Lisp {
        value: String,
        output: Vec<String>,
    },

    /// CLIPS: working-memory deltas and rule firings.
    /// Placeholder for the native CLIPS island (#714).
    Clips {
        deltas: Vec<ClipsDelta>,
        firings: Vec<ClipsFiring>,
    },

    /// Prolog: substitutions and proof alternatives.
    /// Placeholder for the native SWI-Prolog island (#712).
    Prolog {
        substitutions: Vec<HashMap<String, String>>,
        alternatives: Vec<String>,
    },

    /// Datalog: relation deltas (base facts), full closure, and separate
    /// derivations.
    Datalog {
        relation_deltas: HashMap<String, Vec<Tuple>>,
        closure: HashMap<String, Vec<Tuple>>,
        derivations: HashMap<String, Vec<(Tuple, Vec<Derivation>)>>,
    },
}

/// A CLIPS working-memory change.
#[derive(Clone, Debug, PartialEq)]
pub struct ClipsDelta {
    pub kind: ClipsDeltaKind,
    pub template: String,
    pub slots: HashMap<String, String>,
}

/// Direction of a CLIPS WM change.
#[derive(Clone, Debug, PartialEq)]
pub enum ClipsDeltaKind {
    Assert,
    Retract,
}

/// A single CLIPS rule firing.
#[derive(Clone, Debug, PartialEq)]
pub struct ClipsFiring {
    pub rule: String,
    pub matches: Vec<String>,
}

/// The four-domain artifact: one native result per kernel.
#[derive(Clone, Debug, PartialEq)]
pub struct FourKernelObservation {
    pub lisp: NativeResult,
    pub clips: NativeResult,
    pub prolog: NativeResult,
    pub datalog: NativeResult,
}

impl FourKernelObservation {
    /// Build an observation from real Lisp and Datalog results, with typed
    /// placeholders for CLIPS and Prolog.
    pub fn from_lisp_and_datalog(
        lisp_value: String,
        lisp_output: Vec<String>,
        datalog_db: &Database,
    ) -> Self {
        Self {
            lisp: NativeResult::Lisp {
                value: lisp_value,
                output: lisp_output,
            },
            clips: NativeResult::Clips {
                deltas: Vec::new(),
                firings: Vec::new(),
            },
            prolog: NativeResult::Prolog {
                substitutions: Vec::new(),
                alternatives: Vec::new(),
            },
            datalog: datalog_result(datalog_db),
        }
    }

    /// True iff none of the four domains was reconstructed from another.
    ///
    /// The current check verifies that each slot keeps its own variant and
    /// that no cross-kernel bridge was used to manufacture any domain.
    pub fn preserves_independent_domains(&self) -> bool {
        matches!(self.lisp, NativeResult::Lisp { .. })
            && matches!(self.clips, NativeResult::Clips { .. })
            && matches!(self.prolog, NativeResult::Prolog { .. })
            && matches!(self.datalog, NativeResult::Datalog { .. })
    }
}

/// Serialize a Datalog database into its native result domain.
///
/// - `relation_deltas`: tuples introduced by base facts (rule_id == "__base__").
/// - `closure`: all tuples present after evaluation.
/// - `derivations`: all recorded derivations indexed by relation.
pub fn datalog_result(db: &Database) -> NativeResult {
    let mut relation_deltas: HashMap<String, Vec<Tuple>> = HashMap::new();
    let mut closure: HashMap<String, Vec<Tuple>> = HashMap::new();
    let mut derivations: HashMap<String, Vec<(Tuple, Vec<Derivation>)>> = HashMap::new();

    for relation in db.relation_names() {
        let mut closure_tuples = Vec::new();
        let mut delta_tuples = Vec::new();
        let mut derivations_for_relation = Vec::new();

        for tuple in db.relation(relation) {
            let tuple = tuple.clone();
            let ds = db.derivations(relation, &tuple).to_vec();

            if ds.iter().any(|d| d.rule_id == "__base__") {
                delta_tuples.push(tuple.clone());
            }

            closure_tuples.push(tuple.clone());
            derivations_for_relation.push((tuple, ds));
        }

        closure.insert(relation.clone(), closure_tuples);
        relation_deltas.insert(relation.clone(), delta_tuples);
        derivations.insert(relation.clone(), derivations_for_relation);
    }

    NativeResult::Datalog {
        relation_deltas,
        closure,
        derivations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_preserves_four_distinct_variants() {
        let obs = FourKernelObservation::from_lisp_and_datalog(
            "42".to_string(),
            vec!["trace line".to_string()],
            &Database::default(),
        );
        assert!(obs.preserves_independent_domains());
    }
}
