from pathlib import Path

path = Path("crates/swarm-node/src/main.rs")
text = path.read_text()

replacements = [
    (
        '''            if let Some(ev) = journal.events.iter().find(|e| &e.id() == event_id).cloned() {
''',
        '''            if let Some(ev) = journal
                .events
                .iter()
                .find(|e| &e.id() == event_id)
                .cloned()
            {
''',
    ),
    (
        '''            warn!(
                "swarm-node: write to peer {id} failed/errored -- dropping and queueing for retry"
            );
''',
        '''            warn!("swarm-node: write to peer {id} failed/errored -- dropping and queueing for retry");
''',
    ),
    (
        '''            Sexp::list(
                std::iter::once(Sexp::atom("work-state"))
                    .chain(fields)
                    .collect(),
            )
''',
        '''            Sexp::list(std::iter::once(Sexp::atom("work-state")).chain(fields).collect())
''',
    ),
    (
        '''                    Sexp::list(vec![
                        Sexp::atom("count"),
                        Sexp::atom(queued_len.to_string()),
                    ]),
''',
        '''                    Sexp::list(vec![Sexp::atom("count"), Sexp::atom(queued_len.to_string())]),
''',
    ),
]

for old, new in replacements:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected exactly one formatting hunk, found {count}: {old!r}")
    text = text.replace(old, new, 1)

path.write_text(text)
