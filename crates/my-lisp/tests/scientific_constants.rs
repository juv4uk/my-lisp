use my_lisp::{eval_program, Session};

fn eval_science(source: &str) -> String {
    let mut session = Session::default();
    for library in [
        include_str!("../../../lib/core.my"),
        include_str!("../../../lib/quantity.my"),
        include_str!("../../../lib/si.my"),
    ] {
        eval_program(library, &mut session).expect("science library should load");
    }
    eval_program(source, &mut session)
        .unwrap_or_else(|e| panic!("science expression failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

#[test]
fn bare_number_quantity_and_scientific_constant_are_distinct_data_levels() {
    let source = r#"
        (list
          (quantity? 299792458)
          (quantity?
            (scientific-constant-quantity si:speed-of-light-constant))
          (scientific-constant? si:speed-of-light-constant)
          (scientific-constant? 299792458))
    "#;
    assert_eq!(eval_science(source), "(() t t ())");
}

#[test]
fn speed_of_light_record_keeps_value_unit_status_kind_system_and_source() {
    let source = r#"
        (list
          (scientific-constant-name si:speed-of-light-constant)
          (scientific-constant-value si:speed-of-light-constant)
          (unit-dimensions
            (scientific-constant-unit si:speed-of-light-constant))
          (scientific-constant-status si:speed-of-light-constant)
          (scientific-constant-kind si:speed-of-light-constant)
          (scientific-constant-system si:speed-of-light-constant)
          (scientific-constant-source si:speed-of-light-constant))
    "#;
    assert_eq!(
        eval_science(source),
        "(si:speed-of-light 299792458 ((metre 1) (second -1)) exact-by-definition physical-defining si (science-source/1 bipm-si-brochure-9 2019))"
    );
}

#[test]
fn all_seven_si_records_are_valid_exact_defining_constants() {
    let source = r#"
        (map
          (lambda (constant)
            (list
              (scientific-constant? constant)
              (scientific-constant-status constant)
              (scientific-constant-kind constant)
              (scientific-constant-system constant)))
          (list
            si:cesium-frequency-constant
            si:speed-of-light-constant
            si:planck-constant-record
            si:elementary-charge-constant
            si:boltzmann-constant-record
            si:avogadro-constant-record
            si:luminous-efficacy-constant))
    "#;
    assert_eq!(
        eval_science(source),
        "((t exact-by-definition physical-defining si) (t exact-by-definition physical-defining si) (t exact-by-definition physical-defining si) (t exact-by-definition physical-defining si) (t exact-by-definition physical-defining si) (t exact-by-definition physical-defining si) (t exact-by-definition physical-defining si))"
    );
}

#[test]
fn established_numeric_si_surface_is_derived_from_the_authoritative_records() {
    let source = r#"
        (list
          (= si:cesium-frequency
             (scientific-constant-value si:cesium-frequency-constant))
          (= si:speed-of-light
             (scientific-constant-value si:speed-of-light-constant))
          (= si:planck-constant
             (scientific-constant-value si:planck-constant-record))
          (= si:elementary-charge
             (scientific-constant-value si:elementary-charge-constant))
          (= si:boltzmann-constant
             (scientific-constant-value si:boltzmann-constant-record))
          (= si:avogadro-constant
             (scientific-constant-value si:avogadro-constant-record))
          (= si:luminous-efficacy
             (scientific-constant-value si:luminous-efficacy-constant)))
    "#;
    assert_eq!(eval_science(source), "(t t t t t t t)");
}

#[test]
fn exact_rational_values_remain_unchanged_after_structuring() {
    assert_eq!(
        eval_science("si:planck-constant"),
        "132521403/200000000000000000000000000000000000000000"
    );
    assert_eq!(
        eval_science("si:elementary-charge"),
        "801088317/5000000000000000000000000000"
    );
    assert_eq!(
        eval_science("si:boltzmann-constant"),
        "1380649/100000000000000000000000000000"
    );
    assert_eq!(
        eval_science("si:avogadro-constant"),
        "602214076000000000000000"
    );
}

#[test]
fn si_source_does_not_reintroduce_a_second_literal_authority_for_numeric_views() {
    let si = include_str!("../../../lib/si.my");

    for forbidden in [
        "(def si:cesium-frequency 9192631770)",
        "(def si:speed-of-light 299792458)",
        "(def si:avogadro-constant 602214076000000000000000)",
        "(def si:luminous-efficacy 683)",
    ] {
        assert!(
            !si.contains(forbidden),
            "numeric view must be derived from its scientific-constant record: {forbidden}"
        );
    }

    for derived in [
        "(def si:cesium-frequency (si:constant-value si:cesium-frequency-constant))",
        "(def si:speed-of-light (si:constant-value si:speed-of-light-constant))",
        "(def si:planck-constant (si:constant-value si:planck-constant-record))",
        "(def si:elementary-charge (si:constant-value si:elementary-charge-constant))",
        "(def si:boltzmann-constant (si:constant-value si:boltzmann-constant-record))",
        "(def si:avogadro-constant (si:constant-value si:avogadro-constant-record))",
        "(def si:luminous-efficacy (si:constant-value si:luminous-efficacy-constant))",
    ] {
        assert!(si.contains(derived), "missing derived SI numeric view: {derived}");
    }
}
