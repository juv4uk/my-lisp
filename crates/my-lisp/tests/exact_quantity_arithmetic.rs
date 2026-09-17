use my_lisp::{eval_program, Session};

fn eval_exact_quantity(source: &str) -> String {
    let mut session = Session::default();
    for library in [
        include_str!("../../../lib/core.lisp"),
        include_str!("../../../lib/quantity.lisp"),
        include_str!("../../../lib/si.lisp"),
    ] {
        eval_program(library, &mut session).expect("exact quantity library should load");
    }
    eval_program(source, &mut session)
        .unwrap_or_else(|e| panic!("exact quantity expression failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

#[test]
fn planck_times_cesium_frequency_stays_exact_and_becomes_energy_dimension() {
    let source = r#"
        (def energy
          (quantity-product
            (scientific-constant-quantity si:defining-planck-constant)
            (scientific-constant-quantity si:defining-cesium-frequency)))
        (list
          (quantity-value energy)
          (quantity-unit energy)
          (scientific-constant? energy))
    "#;

    assert_eq!(
        eval_exact_quantity(source),
        "(121822045942277331/20000000000000000000000000000000000000000 (unit/1 (dimension/1 kilogram 1) (dimension/1 metre 2) (dimension/1 second -2)) ())"
    );
}
