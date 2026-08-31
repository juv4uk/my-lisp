//! Durable candidate intake for the WSM Guard reference bureau.
//! Довговічний прийом кандидатів до довідкового бюро WSM Guard.
//!
//! This tool never promotes an answer to authority. It appends a bounded,
//! provenance-bearing candidate record for later review into the curated WSM
//! directory. The destination path is always explicit.

use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_FIELD_BYTES: usize = 64 * 1024;

#[derive(Debug, PartialEq)]
struct Candidate {
    topic: String,
    question: String,
    answer: String,
    source: String,
    route: String,
    evidence: String,
}

fn sexpr_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn validate(candidate: &Candidate) -> Result<(), String> {
    for (name, value) in [
        ("topic", &candidate.topic),
        ("question", &candidate.question),
        ("answer", &candidate.answer),
        ("source", &candidate.source),
        ("evidence", &candidate.evidence),
    ] {
        if value.trim().is_empty() {
            return Err(format!("{name} must not be empty"));
        }
        if value.len() > MAX_FIELD_BYTES {
            return Err(format!("{name} exceeds {MAX_FIELD_BYTES} bytes"));
        }
    }
    if !matches!(
        candidate.route.as_str(),
        "ask-agent" | "ask-owner" | "research-web"
    ) {
        return Err("route must be ask-agent, ask-owner, or research-web".into());
    }
    Ok(())
}

fn render(candidate: &Candidate, recorded_at: u64) -> String {
    format!(
        "(reference-candidate (schema guard-reference-candidate/1) (status pending-review) (recorded-at-unix {recorded_at}) (route {}) (topic {}) (question {}) (answer {}) (source {}) (evidence {}))",
        candidate.route,
        sexpr_string(&candidate.topic),
        sexpr_string(&candidate.question),
        sexpr_string(&candidate.answer),
        sexpr_string(&candidate.source),
        sexpr_string(&candidate.evidence),
    )
}

fn field(args: &[String], name: &str) -> Result<String, String> {
    let position = args
        .iter()
        .position(|arg| arg == name)
        .ok_or_else(|| format!("missing {name}"))?;
    args.get(position + 1)
        .cloned()
        .ok_or_else(|| format!("missing value after {name}"))
}

fn propose(args: &[String]) -> Result<(), String> {
    let inbox = PathBuf::from(field(args, "--inbox")?);
    if !inbox.is_absolute() {
        return Err("--inbox must be an absolute path".into());
    }
    let candidate = Candidate {
        topic: field(args, "--topic")?,
        question: field(args, "--question")?,
        answer: field(args, "--answer")?,
        source: field(args, "--source")?,
        route: field(args, "--route")?,
        evidence: field(args, "--evidence")?,
    };
    validate(&candidate)?;
    let recorded_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&inbox)
        .map_err(|error| format!("cannot open {}: {error}", inbox.display()))?;
    writeln!(file, "{}", render(&candidate, recorded_at)).map_err(|error| error.to_string())?;
    file.sync_data().map_err(|error| error.to_string())?;
    println!("recorded pending-review candidate in {}", inbox.display());
    Ok(())
}

fn list(args: &[String]) -> Result<(), String> {
    let inbox = PathBuf::from(field(args, "--inbox")?);
    let file = std::fs::File::open(&inbox)
        .map_err(|error| format!("cannot open {}: {error}", inbox.display()))?;
    for line in io::BufReader::new(file).lines() {
        let line = line.map_err(|error| error.to_string())?;
        if !line.trim().is_empty() && !line.trim_start().starts_with(';') {
            println!("{line}");
        }
    }
    Ok(())
}

fn usage() {
    eprintln!(
        "guard-reference propose --inbox ABS --topic T --question Q --answer A --source S --route ask-agent|ask-owner|research-web --evidence E\n\
         guard-reference list --inbox ABS"
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("propose") => propose(&args[1..]),
        Some("list") => list(&args[1..]),
        _ => {
            usage();
            Err("expected propose or list".into())
        }
    };
    if let Err(error) = result {
        eprintln!("guard-reference: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(route: &str) -> Candidate {
        Candidate {
            topic: "guix".into(),
            question: "How do I build a relocatable pack?".into(),
            answer: "Use the reviewed Guix workflow.".into(),
            source: "agent:sakshi".into(),
            route: route.into(),
            evidence: "docs/VIVEKA-FINDINGS-2026-08-24.md#5".into(),
        }
    }

    #[test]
    fn renders_data_without_evaluating_or_losing_provenance() {
        let record = render(&candidate("ask-agent"), 42);
        assert!(record.starts_with("(reference-candidate"));
        assert!(record.contains("(status pending-review)"));
        assert!(record.contains("(recorded-at-unix 42)"));
        assert!(record.contains("(route ask-agent)"));
        assert!(record.contains("(source \"agent:sakshi\")"));
    }

    #[test]
    fn accepts_only_the_three_unknown_routes() {
        assert!(validate(&candidate("ask-agent")).is_ok());
        assert!(validate(&candidate("ask-owner")).is_ok());
        assert!(validate(&candidate("research-web")).is_ok());
        assert!(validate(&candidate("guess")).is_err());
    }

    #[test]
    fn escapes_candidate_text_as_data() {
        assert_eq!(sexpr_string("a\n\"b\"\\c"), "\"a\\n\\\"b\\\"\\\\c\"");
    }
}
