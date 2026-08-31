//! Guard Reference Schema Quality Gate / Гарантна перевірка якості схеми довідника.
//!
//! Fail-closed validation of guard-reference.wsm:
//! - Duplicate topics
//! - Missing authority files
//! - Empty authority/verify
//! - Unreviewed inbox entries (pending-review)

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TOPIC_LENGTH: usize = 64;

#[derive(Debug, Clone)]
struct TopicEntry {
    topic: String,
    summary: String,
    authority: Vec<String>,
    how_to: Vec<String>,
    verify: Vec<String>,
    lifecycle: String,
    provenance: String,
    unknown_route: String,
}

#[derive(Debug)]
struct ValidationResult {
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    fn add_error(&mut self, msg: String) {
        self.errors.push(msg);
    }
    fn add_warning(&mut self, msg: String) {
        self.warnings.push(msg);
    }
    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    fn print(&self) {
        for w in &self.warnings {
            eprintln!("WARN: {}", w);
        }
        for e in &self.errors {
            eprintln!("ERROR: {}", e);
        }
    }
}

fn parse_guard_reference(file_path: &Path) -> Result<Vec<TopicEntry>, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("cannot read {}: {}", file_path.display(), e))?;

    let mut entries = Vec::new();
    let mut depth = 0;
    let mut current: Option<TopicEntry> = None;
    let mut current_field = String::new();
    let mut current_list_field: Option<String> = None;
    let mut buffer = String::new();
    let mut in_string = false;
    let mut escape = false;

    for ch in content.chars() {
        if escape {
            buffer.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape = true;
            buffer.push(ch);
            continue;
        }
        if ch == '"' && !escape {
            in_string = !in_string;
            buffer.push(ch);
            continue;
        }
        if in_string {
            buffer.push(ch);
            continue;
        }

        if ch == '(' {
            depth += 1;
            // Start of reference entry at depth 4
            let token = buffer.trim();
            if depth == 4 && token == "reference" {
                current = Some(TopicEntry {
                    topic: String::new(),
                    summary: String::new(),
                    authority: Vec::new(),
                    how_to: Vec::new(),
                    verify: Vec::new(),
                    lifecycle: String::new(),
                    provenance: String::new(),
                    unknown_route: String::new(),
                });
                current_list_field = None;
                current_field.clear();
            }
            buffer.clear();
        } else if ch == ')' {
            // Handle field values at depth 5 (simple fields) and depth 6 (list items)
            let token = buffer.trim();
            if !token.is_empty() && current.is_some() {
                if depth == 5 {
                    // Simple field value: (field value) - token is the value
                    if let Some(ref mut entry) = current {
                        let value = token.trim_matches('"').to_string();
                        match current_field.as_str() {
                            "topic" => entry.topic = value,
                            "summary" => entry.summary = value,
                            "lifecycle" => entry.lifecycle = value,
                            "provenance" => entry.provenance = value,
                            "unknown-route" => entry.unknown_route = value,
                            _ => {}
                        }
                    }
                    current_field.clear();
                } else if depth == 6 {
                    // List item: (field (item1 item2)) - each item at depth 6
                    if let Some(ref mut entry) = current {
                        let value = token.trim_matches('"').to_string();
                        if let Some(list_field) = &current_list_field {
                            match list_field.as_str() {
                                "authority" => entry.authority.push(value),
                                "how-to" => entry.how_to.push(value),
                                "verify" => entry.verify.push(value),
                                _ => {}
                            }
                        }
                    }
                }
            }

            // End of reference entry at depth 4
            if depth == 4 && current.is_some() && token == "reference" {
                if let Some(entry) = current.take() {
                    if !entry.topic.is_empty() {
                        entries.push(entry);
                    }
                }
                current_list_field = None;
            }

            depth -= 1;
            buffer.clear();
        } else if ch.is_whitespace() {
            let token = buffer.trim();
            if !token.is_empty() {
                if depth == 5 && current.is_some() {
                    // Field name at depth 5
                    if current_field.is_empty() {
                        current_field = token.to_string();
                        if matches!(token.as_ref(), "authority" | "how-to" | "verify") {
                            current_list_field = Some(token.to_string());
                        } else {
                            current_list_field = None;
                        }
                    }
                }
            }
            buffer.clear();
        } else {
            buffer.push(ch);
        }
    }

    if let Some(entry) = current {
        if !entry.topic.is_empty() {
            entries.push(entry);
        }
    }

    if entries.is_empty() {
        return Err("no reference entries found".into());
    }
    Ok(entries)
}

fn validate_entries(entries: &[TopicEntry], repo_root: &Path) -> ValidationResult {
    let mut result = ValidationResult::new();
    let mut seen_topics = HashSet::new();

    for entry in entries {
        if entry.topic.is_empty() {
            result.add_error("topic must not be empty".into());
        } else if entry.topic.len() > MAX_TOPIC_LENGTH {
            result.add_error(format!(
                "topic '{}' exceeds {} chars",
                entry.topic, MAX_TOPIC_LENGTH
            ));
        }
        if !seen_topics.insert(entry.topic.clone()) {
            result.add_error(format!("duplicate topic: {}", entry.topic));
        }
        if entry.summary.trim().is_empty() {
            result.add_error(format!(
                "topic '{}': summary must not be empty",
                entry.topic
            ));
        }
        if entry.authority.is_empty() {
            result.add_error(format!(
                "topic '{}': authority must not be empty",
                entry.topic
            ));
        } else {
            for auth in &entry.authority {
                let path = if auth.starts_with("../") {
                    repo_root.join(auth)
                } else {
                    repo_root.join(auth)
                };
                if !path.exists() {
                    result.add_error(format!(
                        "topic '{}': authority file not found: {} (checked: {})",
                        entry.topic,
                        auth,
                        path.display()
                    ));
                }
            }
        }
        if entry.verify.is_empty() {
            result.add_error(format!("topic '{}': verify must not be empty", entry.topic));
        }
        if entry.lifecycle.is_empty() {
            result.add_error(format!(
                "topic '{}': lifecycle must not be empty",
                entry.topic
            ));
        }
        if entry.provenance.is_empty() {
            result.add_error(format!(
                "topic '{}': provenance must not be empty",
                entry.topic
            ));
        }
        if entry.unknown_route.is_empty() {
            result.add_error(format!(
                "topic '{}': unknown-route must not be empty",
                entry.topic
            ));
        } else if !matches!(
            entry.unknown_route.as_str(),
            "ask-agent" | "ask-owner" | "research-web"
        ) {
            result.add_error(format!(
                "topic '{}': unknown-route must be ask-agent|ask-owner|research-web, got '{}'",
                entry.topic, entry.unknown_route
            ));
        }
    }

    result
}

fn validate_inbox(inbox_path: &Path) -> ValidationResult {
    let mut result = ValidationResult::new();
    if !inbox_path.exists() {
        result.add_warning(format!("inbox not found: {}", inbox_path.display()));
        return result;
    }
    let content = fs::read_to_string(inbox_path)
        .map_err(|e| format!("cannot read inbox: {}", e))
        .unwrap_or_default();
    let pending_count = content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with(';'))
        .filter(|l| l.contains("pending-review"))
        .count();
    if pending_count > 0 {
        result.add_warning(format!(
            "{} unreviewed inbox entries (pending-review)",
            pending_count
        ));
    }
    result
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check_inbox = args.iter().any(|a| a == "--check-inbox");

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent().unwrap().parent().unwrap(); // my-lisp/
    let guard_ref_path = repo_root.join("knowledge/guard-reference.wsm");
    let inbox_path = repo_root.join("../ecosystem/knowledge/guard-reference-inbox.mylog");

    let entries = match parse_guard_reference(&guard_ref_path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(2);
        }
    };

    let mut result = validate_entries(&entries, repo_root);

    if check_inbox {
        let inbox_result = validate_inbox(&inbox_path);
        result.errors.extend(inbox_result.errors);
        result.warnings.extend(inbox_result.warnings);
    }

    result.print();

    if result.has_errors() {
        eprintln!(
            "\nSchema validation FAILED ({} errors)",
            result.errors.len()
        );
        std::process::exit(1);
    } else {
        println!("Schema validation PASSED: {} topics checked", entries.len());
        if !result.warnings.is_empty() {
            println!("Warnings: {}", result.warnings.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn sample_wsm() -> String {
        r#"(def *guard-reference-directory*
  (quote
    ((reference
       (topic test-topic)
       (summary "Test summary")
       (authority (file1.md file2.md))
       (how-to (step1 step2))
       (verify (check1 check2))
       (lifecycle current-contract)
       (provenance commit abc123)
       (unknown-route ask-agent))))"#
            .into()
    }

    #[test]
    fn parses_valid_entry() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(sample_wsm().as_bytes()).unwrap();
        let entries = parse_guard_reference(f.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].topic, "test-topic");
        assert_eq!(entries[0].summary, "Test summary");
        assert_eq!(entries[0].lifecycle, "current-contract");
        assert_eq!(entries[0].unknown_route, "ask-agent");
        assert_eq!(entries[0].authority, vec!["file1.md", "file2.md"]);
        assert_eq!(entries[0].how_to, vec!["step1", "step2"]);
        assert_eq!(entries[0].verify, vec!["check1", "check2"]);
    }

    #[test]
    fn catches_duplicate_topic() {
        let content = format!(
            "{}\n{}",
            sample_wsm(),
            sample_wsm().replace("test-topic", "test-topic")
        );
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        let entries = parse_guard_reference(f.path()).unwrap();
        let repo_root = std::env::current_dir().unwrap();
        let result = validate_entries(&entries, &repo_root);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.contains("duplicate topic")));
    }

    #[test]
    fn catches_missing_authority() {
        let content = sample_wsm().replace("file1.md file2.md", "nonexistent.md");
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        let entries = parse_guard_reference(f.path()).unwrap();
        let repo_root = std::env::current_dir().unwrap();
        let result = validate_entries(&entries, &repo_root);
        assert!(result.has_errors());
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("authority file not found")));
    }

    #[test]
    fn catches_empty_fields() {
        let content = sample_wsm()
            .replace("(summary \"Test summary\")", "(summary \"\")")
            .replace("(lifecycle current-contract)", "(lifecycle \"\")");
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        let entries = parse_guard_reference(f.path()).unwrap();
        let repo_root = std::env::current_dir().unwrap();
        let result = validate_entries(&entries, &repo_root);
        assert!(result.has_errors());
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("summary must not be empty")));
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("lifecycle must not be empty")));
    }

    #[test]
    fn catches_invalid_unknown_route() {
        let content = sample_wsm().replace("(unknown-route ask-agent)", "(unknown-route guess)");
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        let entries = parse_guard_reference(f.path()).unwrap();
        let repo_root = std::env::current_dir().unwrap();
        let result = validate_entries(&entries, &repo_root);
        assert!(result.has_errors());
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("unknown-route must be")));
    }
}
