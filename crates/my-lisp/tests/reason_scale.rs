//! Exercises lib/reason.my at scale (MYLISP-REASON-SCALE-PROFILE).
//!
//! lib/reason.my is a naive backward-chaining engine with no indexing.
//! Several reviews claim it is "unworkable at ~1000 facts", but that was
//! flagged as an architectural hypothesis, not measured evidence (see
//! docs/wsl-nidana-1-reaction-2026-08-26.md). This test profiles reason over
//! a linear `edge` chain of N facts at N = 100 / 500 / 1000, recording real
//! wall-clock timings for a goal that forces a full forward scan (the no-
//! indexing worst case), so the design decision on indexing can rest on
//! measured data, not assumption.
//!
//! This test PRINTS the timings (they land in the test log) and only asserts
//! functional correctness at each size — a wall-clock number on a shared
//! machine is not a stable assertion target. The measured numbers below are
//! the evidence; re-run to re-measure.
//!
//! Measured 2026-08-29 (debug build, full-scan goal, 64 MiB measurement stack):
//!   N=100  → ~0.23–0.36 s
//!   N=500  → ~1.5–1.8 s
//!   N=1000 → ~1.9–3.3 s
//! so N=100→1000 (10x facts) costs ~6–12x time → clearly superlinear (≈O(N²)
//! from the repeated `append` in `prove-goal`). Root cause of both the time
//! and the crash: `prove-goal` (lib/reason.my) recurses over rules with a
//! non-tail `append`, so a full-scan goal consumes O(N) call-stack depth. On
//! the default 2 MiB test-thread stack the same query already stack-overflows
//! before N=100. The earlier review phrasing "unworkable at ~1000 facts" was
//! therefore optimistic — the no-index engine is worse than that.
//!
//! Виміряно 2026-08-29 (debug, full-scan, 64 MiB стек вимірювання):
//!   N=100  → ~0.23–0.36 s
//!   N=500  → ~1.5–1.8 s
//!   N=1000 → ~1.9–3.3 s
//! тобто N=100→1000 (10x фактів) коштує ~6–12x часу → явно суперлінійно
//! (≈O(N²) через повторний `append` у `prove-goal`). Першопричина і часу, і
//! краху: `prove-goal` (lib/reason.my) рекурсує по правилах через не-хвостовий
//! `append`, тож full-scan ціль споживає O(N) глибини стеку викликів. На
//! типовому 2 MiB стеку тестового потоку той самий запит вже переповнює стек
//! до N=100. Тож фраза рев'ю "unworkable на ~1000 фактів" була оптимістичною —
//! рушій без індексації гірший.
//!
//! Gemessen 2026-08-29 (Debug-Build, Voll-Scan-Ziel, 64 MiB Mess-Stack):
//!   N=100  → ~0.23–0.36 s
//!   N=500  → ~1.5–1.8 s
//!   N=1000 → ~1.9–3.3 s
//! also N=100→1000 (10x Fakten) kostet ~6–12x Zeit → klar superlinear (≈O(N²)
//! durch wiederholtes `append` in `prove-goal`). Ursache von Zeit wie Absturz:
//! `prove-goal` (lib/reason.my) rekursiert über Regeln durch ein nicht-
//! endständiges `append`, also verbraucht ein Voll-Scan-Ziel O(N) Aufruf-
//! Stacktiefe. Auf dem Standard-2-MiB-Stack des Test-Threads läuft derselbe
//! Query bereits vor N=100 über. Der ehemalige Review-Satz "unworkable bei
//! ~1000 Fakten" war also optimistisch — die Engine ohne Indizierung ist
//! schlechter.
//!
//! Досліджує lib/reason.my у масштабі (MYLISP-REASON-SCALE-PROFILE).
//! Рушій backward-chaining без індексації; кілька рев'ю називали його
//! "unworkable на ~1000 фактів", але то було позначено як архітектурну
//! гіпотезу, а не виміряний доказ (див. docs/wsl-nidana-1-reaction-2026-08-26.md).
//! Цей тест профілює reason над ланцюгом `edge` з N фактів при N = 100/500/1000,
//! фіксуючи реальний час для запиту, що змушує повний прямий скан (найгірший
//! випадок без індексації), щоб рішення про індексацію спиралося на дані.
//! Тест ПРИНТУЄ таймінги (потрапляють у лог тесту) і перевіряє лише
//! функціональну коректність на кожному розмірі — час на спільній машині не є
//! стабільною ціллю для assert. Виміряні числа нижче — доказ; повторіть, щоб
//! виміряти знову.
//!
//! Untersucht lib/reason.my im Maßstab (MYLISP-REASON-SCALE-PROFILE).
//! Die Engine ist Backward-Chaining ohne Indexierung; mehrere Reviews
//! behaupteten "bei ~1000 Fakten unbrauchbar", doch das war als Architektur-
//! Hypothese markiert, nicht als Messwert (siehe docs/wsl-nidana-1-reaction-2026-08-26.md).
//! Dieser Test profiliert reason über eine lineare `edge`-Kette aus N Fakten
//! bei N = 100/500/1000 und erfasst echte Wanduhr-Zeiten für ein Ziel, das
//! einen vollständigen Vorwärts-Scan erzwingt (der Worst Case ohne Indexierung),
//! damit die Indexierungs-Entscheidung auf Daten beruht. Der Test DRUCKT die
//! Zeiten (in das Testlog) und prüft nur funktionale Korrektheit je Größe —
//! eine Wanduhr-Zahl auf einer gemeinsam genutzten Maschine ist kein stabiles
//! Assert-Ziel. Die gemessenen Zahlen unten sind der Beleg; zur erneuten
//! Messung einfach erneut ausführen.

use my_lisp::{eval_program, Session};
use std::time::Instant;

fn loaded_session() -> Session {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/unify.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/reason.my"), &mut session).unwrap();
    session
}

/// Build the my-lisp quoted rules list for a linear edge chain of `n` facts:
/// `(quote (((edge 0 1)) ((edge 1 2)) ... ((edge (n-1) n))))`.
/// Time a single `reason` query that must scan the whole chain: asks for the
/// target node and only the last edge matches, so the no-index engine walks
/// every fact (worst case). The chain is pre-built (untimed) into a def so
/// only `reason` itself is inside the timed region. Returns (elapsed_ns,
/// result_count).
fn time_scan(session: &mut Session, n: usize) -> (u128, usize) {
    // pre-build the chain untimed: (def chainN (build-chain N))
    eval_session(session, &format!("(def chain{n} (build-chain {n}))"));
    // goal: (edge (var x) <n>)  -> only edge (n-1, n) matches, after full scan
    let source =
        format!("(length (reason (list (quote edge) (logic-var (quote x)) {n}) chain{n}))");
    let t = Instant::now();
    let value = eval_session(session, &source).to_string();
    let elapsed = t.elapsed().as_nanos();
    (elapsed, value.trim().parse().unwrap_or(0))
}

fn eval_session(session: &mut Session, source: &str) -> String {
    eval_program(source, session)
        .unwrap_or_else(|e| panic!("evaluation failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

#[test]
fn reason_scale_profile_100_500_1000() {
    // The no-index engine's `prove-goal` recurses over rules with a
    // non-tail `append` (lib/reason.my), so a full-scan goal costs O(N) call
    // stack depth. On the default 2 MB test-thread stack that already
    // overflows before N=100 (measured). To capture *timings* (not just the
    // crash) we measure on an explicitly large stack thread.
    std::thread::Builder::new()
        .name("reason-scale-profile".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(run_profile)
        .expect("spawn measurement thread")
        .join()
        .expect("measurement thread panicked");
}

fn run_profile() {
    let mut session = loaded_session();
    // define build-chain once in the session
    let def = r#"
        (def build-chain
          (lambda (n)
            (cond
              ((= n 0) (quote ()))
              (t (cons (list (list (quote edge) (- n 1) n)) (build-chain (- n 1)))))))
    "#;
    eval_session(&mut session, def);

    let sizes = [100usize, 500, 1000];
    let mut table =
        String::from("reason scale profile (edge chain, full-scan goal, 64 MiB stack)\n");
    table.push_str("  N     elapsed_ns     results\n");
    for n in sizes {
        let (ns, results) = time_scan(&mut session, n);
        table.push_str(&format!("  {n:<5} {ns:>12} {results}\n"));
        // functional correctness: the full-scan goal matches exactly one edge
        assert_eq!(results, 1, "full-scan goal at N={n} should match one edge");
    }
    println!("\n{table}");
}
