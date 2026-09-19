use my_lisp::{eval_program, Session};

const EXPERIMENT: &str = include_str!("../../../experiments/empty-list-unfold.lisp");

#[test]
fn empty_list_ground_unfolds_one_table_row_as_plain_lisp_data() {
    let mut session = Session::default();

    let source = format!("{EXPERIMENT}\nempty-list-unfold-result");
    let result = eval_program(&source, &mut session)
        .expect("the empty-list unfolding witness must execute as ordinary Lisp");

    assert_eq!(
        result.value.to_string(),
        "((00000000 (en ()) (uk ()) (ukr ()) (sa ()) (sym ())))"
    );
}

#[test]
fn unfolding_is_structurally_idempotent() {
    let mut session = Session::default();

    let source = format!(
        "{EXPERIMENT}\n(eq (quote ((00000000 (en ()) (uk ()) (ukr ()) (sa ()) (sym ())))) empty-list-unfold-result)"
    );
    let result = eval_program(&source, &mut session)
        .expect("shape-preserving traversal must execute");

    // Do not assume universal t/nil: preserve the post-revolution explicit
    // identity relation produced by EQ.
    assert_eq!(result.value.to_string(), "(identity-relation same)");
}

#[test]
fn empty_list_recovers_its_opaque_identity_from_lisp_data() {
    let mut session = Session::default();

    let source = format!("{EXPERIMENT}\nempty-list-identity");
    let result = eval_program(&source, &mut session)
        .expect("empty list must recover its identity from the Lisp witness");

    assert_eq!(result.value.to_string(), "00000000");
}

#[test]
fn eight_bit_zero_is_not_numeric_zero() {
    let mut session = Session::default();

    let bits = eval_program("(quote 00000000)", &mut session)
        .expect("8-bit pattern must survive quote as data");
    assert_eq!(bits.value.to_string(), "00000000");

    let relation = eval_program("(eq (quote 00000000) 0)", &mut session)
        .expect("8-bit pattern and number are both atoms and may be compared");
    assert_eq!(relation.value.to_string(), "(identity-relation distinct)");
}

#[test]
fn eight_bit_pattern_survives_read_write_round_trip() {
    let mut session = Session::default();
    let result = eval_program("(write-to-string (read \"00000000\"))", &mut session)
        .expect("8-bit pattern must survive reader and printer without decimal reinterpretation");

    assert_eq!(result.value.to_string(), "\"00000000\"");
}

const GROUND_GRAPH: &str = include_str!("../../../experiments/ground-graph.lisp");

#[test]
fn ground_graph_supports_bidirectional_equivalence_and_transition() {
    let mut session = Session::default();
    let result = eval_program(GROUND_GRAPH, &mut session)
        .expect("ground graph experiment must execute as ordinary Lisp data");

    assert_eq!(
        result.value.to_string(),
        "((eclass-forward 00000000) (eclass-backward ()) (graph-equivalence-forward 00000000) (graph-equivalence-backward ()) (graph-transition-forward 00000001) (graph-transition-backward 00000000))"
    );
}


#[test]
fn graph_result_envelope_keeps_empty_list_distinct_from_absence() {
    let mut session = Session::default();
    let source = format!(
        "{GROUND_GRAPH}\n(graph-neighbor-result ground-graph ground-equivalence-relation 00000000)"
    );
    let result = eval_program(&source, &mut session)
        .expect("graph lookup must preserve () as a legitimate found endpoint");

    assert_eq!(
        result.value.to_string(),
        "(graph-result found ())"
    );
}

#[test]
fn graph_supports_unresolved_late_binding_backlinks_and_bounded_observation() {
    let mut session = Session::default();
    let source = format!("{GROUND_GRAPH}\n(nodum-kernel-lessons-witness)");
    let result = eval_program(&source, &mut session)
        .expect("Nodum-inspired graph mechanisms must execute as ordinary Lisp apparatus");

    assert_eq!(
        result.value.to_string(),
        "((resolution-evidence-before (graph-result absent)) (unresolved-before (graph-result found 00000100)) (resolved-after (graph-result found 00000101)) (asserted-edge-still (graph-result found 00000100)) (derived-backlink (graph-result found 00000001)) (bounded-one-hop (00001001 00001010)) (bounded-two-hop (00001001 00001010 00001011)))"
    );
}


#[test]
fn eclass_can_hold_multiple_peers_and_graph_growth_preserves_prior_generation() {
    let mut session = Session::default();
    let source = format!("{GROUND_GRAPH}\n(graph-growth-witness)");
    let result = eval_program(&source, &mut session)
        .expect("expanded e-class and persistent graph growth must execute");

    assert_eq!(
        result.value.to_string(),
        "((expanded-class-member (graph-result found (00001101))) (previous-generation ((() 00000010 00000000))) (previous-generation-preserved (identity-relation same)))"
    );
}


#[test]
fn relation_lifecycle_requires_evidence_before_assertion_and_resolves_late() {
    let mut session = Session::default();
    let source = format!("{GROUND_GRAPH}\n(relation-lifecycle-witness)");
    let result = eval_program(&source, &mut session)
        .expect("relation lifecycle experiment must execute");

    assert_eq!(
        result.value.to_string(),
        "((candidate-visible (graph-result found 00010011)) (assertion-before-evidence (graph-result absent)) (assertion-after-evidence (graph-result found (00010010 00010000 00010011))) (unresolved-assertion (graph-result found (00010010 00010000 00010011))) (resolved-assertion (graph-result found (00010010 00010000 00010100))) (candidate-preserved (graph-result found 00010011)))"
    );
}


#[test]
fn mismatched_evidence_does_not_assert_candidate_relation() {
    let mut session = Session::default();
    let source = format!(
        "{GROUND_GRAPH}\n(def mismatched-evidence (list (list lifecycle-source lifecycle-evidence-relation 00010101)))\n(derive-asserted-edge lifecycle-candidate-graph mismatched-evidence lifecycle-source)"
    );
    let result = eval_program(&source, &mut session)
        .expect("mismatched evidence query must execute");

    assert_eq!(result.value.to_string(), "(graph-result absent)");
}


const FUNCTION_GENEALOGY: &str = include_str!("../../../experiments/function-genealogy.lisp");
const UNIFY_LIBRARY: &str = include_str!("../../../lib/unify.lisp");

#[test]
fn registry_function_genealogy_reproduces_core_accessors_from_smaller_seeds() {
    let mut session = Session::default();
    let source = format!("{FUNCTION_GENEALOGY}\n(function-genealogy-observation)");
    let result = eval_program(&source, &mut session)
        .expect("function genealogy experiment must execute");

    assert_eq!(
        result.value.to_string(),
        "((from-cons (00101110)) (from-car (00101111 00110000 00110001 00110010 00110011 00110100)) (from-cdr (00101111 00110000 00110001 00110010 00110100 00110101)) (reproductions ((00101110 reproduced) (00101111 reproduced) (00110000 reproduced) (00110001 reproduced) (00110010 reproduced) (00110011 reproduced) (00110100 reproduced) (00110101 reproduced))))"
    );
}


#[test]
fn function_laboratory_finds_overlap_candidates_and_unification_dependencies() {
    let mut session = Session::default();
    let source = format!(
        "{UNIFY_LIBRARY}\n{FUNCTION_GENEALOGY}\n(function-laboratory-witness)"
    );
    let result = eval_program(&source, &mut session)
        .expect("function laboratory witness must execute");

    assert_eq!(
        result.value.to_string(),
        "((structural-genealogy ((from-cons (00101110)) (from-car (00101111 00110000 00110001 00110010 00110011 00110100)) (from-cdr (00101111 00110000 00110001 00110010 00110100 00110101)) (reproductions ((00101110 reproduced) (00101111 reproduced) (00110000 reproduced) (00110001 reproduced) (00110010 reproduced) (00110011 reproduced) (00110100 reproduced) (00110101 reproduced))))) (overlap-candidates ((second-vs-cadr (structural-relation same)) (fourth-vs-cadddr (structural-relation same)) (pair-vs-list-two-args (structural-relation same)))) (unification-island ((walk-needs (10001001)) (occurs-check-needs (10001011 10001001 00100010 00101111 00000101 00000110)) (apply-subst-needs (10001011 00000100 00000101 00000110)) (unify-needs (10001011 10001001 10001100 00100010)) (unify-produces-for-apply-subst bob))))"
    );
}


const REALITY_LEDGER: &str = include_str!("../../../experiments/reality-ledger.lisp");

#[test]
fn reality_ledger_separates_observation_from_ontology_claims() {
    let mut session = Session::default();
    let result = eval_program(REALITY_LEDGER, &mut session)
        .expect("reality-ledger experiment must execute");

    assert_eq!(
        result.value.to_string(),
        "((introduced apparatus atom-car-cdr-cons-equal) (introduced vocabulary (structural-kind atom pair empty-list)) (observations ((observed symbol-sample (input radio raw-result (structural-kind atom))) (observed empty-list-sample (input () raw-result (structural-kind empty-list))) (observed pair-sample (input (radio antenna) raw-result (structural-kind pair))))) (independent-pair-probes ((observed car-projection (input (radio . antenna) output radio)) (observed cdr-projection (input (radio . antenna) output antenna)) (derived reconstruction (observed-value (radio . antenna) comparison (structural-relation same))))) (callable-probe (observed application (argument probe output probe))) (not-claimed ontology atom-is-fundamental-kind-of-reality) (not-claimed ontology every-callable-is-a-fundamental-function-kind) (not-claimed identity result-label-proves-its-own-semantic-truth))"
    );
}


#[test]
fn semantic_identity_ledger_separates_grouping_behavior_and_ontology() {
    let mut session = Session::default();
    let source = format!("{REALITY_LEDGER}\n(semantic-identity-reality-ledger)");
    let result = eval_program(&source, &mut session)
        .expect("semantic identity reality-ledger experiment must execute");

    assert_eq!(
        result.value.to_string(),
        "((grouped-surfaces ((observed en-car (output radio)) (observed uk-car (output radio)) (observed sa-car (output radio)) (observed sym-car (output radio)) (derived sampled-behavioral-agreement (en-vs-uk (structural-relation same) en-vs-sa (structural-relation same) en-vs-sym (structural-relation same))) (introduced registry-grouping (00000101 car перше ādi :п)) (not-claimed ontology grouped-spellings-are-literally-one-entity) (not-claimed portability shared-host-representation-is-required-by-reality))) (distinct-id-overlap ((introduced registry-distinction (00101111 second 00110100 cadr)) (observed second-output b) (observed cadr-output b) (derived sampled-behavioral-agreement (structural-relation same)) (not-claimed identity different-registry-id-implies-different-behavior) (not-claimed identity same-behavior-implies-same-registry-id))) (not-claimed ontology semantic-identity-is-fundamental-kind-of-reality) (not-claimed authority registry-row-proves-meaning-by-itself))"
    );
}
