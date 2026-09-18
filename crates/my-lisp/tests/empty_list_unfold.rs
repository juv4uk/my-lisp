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
