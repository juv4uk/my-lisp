use my_lisp::{eval_program, load_core_library, parse, Session};
use std::{fs, hint::black_box, time::Instant};

const CASES: &[(&str, &str)] = &[
    ("arithmetic", "benchmarks/arithmetic.my"),
    ("lists", "benchmarks/lists.my"),
    ("recursion", "benchmarks/recursion.my"),
    ("closures", "benchmarks/closures.my"),
];

fn measure(iterations: usize, mut operation: impl FnMut()) -> f64 {
    for _ in 0..50 {
        operation();
    }
    let started = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    started.elapsed().as_nanos() as f64 / iterations as f64
}

// ── warm-session section ─────────────────────────────────────────────
// The fresh-session cases above pay core.my parsing on every operation;
// these measure steady-state interpreter throughput against the two
// hottest shapes from docs/OPTIMIZATION-ANALYSIS-VYASA.md: exact
// rational chains (LCM denominator growth drives bignum cost) and
// vector mutation loops. Machine-specific — compare same-machine runs.

const WARM_SETUP: &str = r#"
(def rat-loop
  (lambda (n acc)
    (cond
      ((= n 0) acc)
      (t (rat-loop (- n 1)
                   (+ acc (/ (* n n) (+ (* n 3) 1))))))))
(def vfill
  (lambda (v n)
    (cond
      ((= n 0) v)
      (t (cons (vector-set! v (mod n 8) n)
               (vfill v (- n 1)))))))
(def v0 (make-vector 8))
"#;

fn warm(name: &str, source: &str, iterations: usize) {
    let mut session = Session::default();
    load_core_library(&mut session).expect("canonical macro + core bootstrap should preload");
    eval_program(WARM_SETUP, &mut session).expect("warm setup");
    for _ in 0..3 {
        black_box(eval_program(source, &mut session).expect("warm-up"));
    }
    let started = Instant::now();
    for _ in 0..iterations {
        black_box(eval_program(black_box(source), &mut session).expect("hot path"));
    }
    let ns = started.elapsed().as_nanos() as f64 / iterations as f64;
    println!("BENCH_RESULT\twarm\t{name}\t{ns:.2}");
}

fn main() {
    let iterations = std::env::var("MY_LISP_BENCH_ITERATIONS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1_000);
    let parser_source = fs::read_to_string("benchmarks/parser.my").expect("read parser benchmark");
    let parser_ns = measure(iterations, || {
        black_box(parse(black_box(&parser_source)).expect("parse benchmark"));
    });
    println!("BENCH_RESULT\trust\tparser\t{parser_ns:.2}");

    for (name, path) in CASES {
        let source = fs::read_to_string(path).expect("read evaluation benchmark");
        let ns = measure(iterations, || {
            let mut session = Session::default();
            black_box(eval_program(black_box(&source), &mut session).expect("evaluate benchmark"));
        });
        println!("BENCH_RESULT\trust\t{name}\t{ns:.2}");
    }

    // ── warm-session steady state ──
    let rat_n = std::env::var("MY_LISP_RAT_N")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100usize);
    let rat = format!("(rat-loop {rat_n} 0)");
    warm("rational-chain-100", &rat, 50.max(iterations / 20));

    let vecs = "(cons (vector-set! v0 (mod 7 8) 42) (vfill v0 500))";
    warm("vector-fill-500", vecs, iterations.min(200));
}
