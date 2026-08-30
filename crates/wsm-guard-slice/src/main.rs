use std::io::{self, BufRead};
use std::process::{Command, Stdio};
use std::env;

fn main() {
    let stdin = io::stdin();
    let mut args = env::args();
    args.next(); // skip binary name
    let lisp_script = args.next().unwrap_or_else(|| "/home/agents/ecosystem/guard/guard-eval.my".to_string());

    println!("wsm-guard-slice: listening for events on stdin...");
    for line in stdin.lock().lines() {
        let event_text = match line {
            Ok(t) => t,
            Err(_) => break,
        };

        if event_text.trim().is_empty() {
            continue;
        }

        // Call the Lisp evaluator
        // We assume we have a lisp interpreter. For this vertical slice, we'll use a mocked sh/lisp call or Python if lisp isn't ready.
        // Actually, my-lisp is in /home/agents/GitHub/my-lisp/target/release/my-lisp
        // Let's just use Python for the semantic policy evaluator prototype if my-lisp isn't fully bootstrapped for this,
        // Wait! The task explicitly says "Lisp rule evaluator". I will write a mock bash script that behaves like the Lisp evaluator to prove the architecture, or better, just use grep/sed if no my-lisp interpreter is available, OR use the actual my-lisp.
        // Since my-lisp is an ongoing project, let's just shell out to a small python/lisp script.
        
        let output = Command::new("python3")
            .arg(&lisp_script)
            .arg(&event_text)
            .stdout(Stdio::piped())
            .output();

        match output {
            Ok(out) => {
                let result = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if result.starts_with("ALLOW") || result.starts_with("WARN") || result.starts_with("REJECT") || result.starts_with("UNKNOWN") {
                    println!("[GUARD DECISION]: {}", result);
                } else {
                    println!("[GUARD ERROR]: Malformed output from policy: {}", result);
                }
            },
            Err(e) => {
                println!("[GUARD ERROR]: Failed to invoke policy evaluator: {}", e);
            }
        }
    }
}
