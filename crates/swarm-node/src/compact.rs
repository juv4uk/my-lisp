//! M0.8: journal compaction.
//!
//! Every event type this system has (`task-defined`, `claim-committed`,
//! `claim-released`, `task-completed`, `agent-joined`, `agent-left`) exists
//! only to be folded into derived state — "same facts -> same reducer ->
//! same state" — and the fold functions in `state.rs` are all last-write /
//! monotonic-generation, meaning a task or agent's *entire* history can be
//! losslessly replaced by a small set of terminal facts that fold to the
//! identical derived state. Compaction computes that minimal set and
//! rewrites the journal to hold only it.
//!
//! Safety argument (this matters because other live nodes' correctness
//! depends on it, not just this node's disk usage):
//! - The replacement events are freshly appended under *this* node's own
//!   identity, using sequence numbers strictly greater than any this node
//!   has ever issued (`next_seq` after the fold, before truncation) — so
//!   they can never collide with a sequence number some peer has already
//!   observed from this node, regardless of what got compacted away.
//! - A peer catching up via `sync-hello`/`sync-events` only ever asks for
//!   "events after the highest sequence I've seen per node" and folds
//!   whatever it receives with the same monotonic reducers — it has no way
//!   to observe *how many* events it took to reach a given state, only the
//!   state itself. Sending the compacted set instead of the original raw
//!   history produces an identical derived world for that peer.
//! - What this does NOT do: touch or renumber any event this node did not
//!   just emit, and it never deletes another node's ability to serve its
//!   own full raw history to someone else — only *this* node's local copy
//!   is compacted.

use crate::journal::{Event, Journal};
use crate::sexpr::Sexp;
use crate::state;

/// Rewrites `journal` to hold only the minimal set of facts needed to
/// reconstruct the current derived state (task definitions + ownership,
/// membership), replacing however much raw history led there. Returns
/// `(events_before, events_after)`.
pub fn compact(journal: &mut Journal, self_node_id: &str) -> std::io::Result<(usize, usize)> {
    let before = journal.events.len();
    let mut next_seq = journal.next_seq(self_node_id);
    let mut next_lamport = journal.max_lamport() + 1;
    let mut fresh = || {
        let seq = next_seq;
        next_seq += 1;
        seq
    };
    let mut fresh_lamport = || {
        let l = next_lamport;
        next_lamport += 1;
        l
    };

    let mut new_events = Vec::new();

    for task in state::all_task_ids(journal) {
        if let Some(def) = state::task_def(journal, &task) {
            let payload = Sexp::list(vec![
                Sexp::list(vec![Sexp::atom("task"), Sexp::atom(&task)]),
                Sexp::list(vec![Sexp::atom("priority"), Sexp::atom(def.priority.to_string())]),
                Sexp::list(vec![Sexp::atom("capabilities"), Sexp::list(def.capabilities.iter().map(Sexp::atom).collect())]),
                Sexp::list(vec![Sexp::atom("depends-on"), Sexp::list(def.depends_on.iter().map(Sexp::atom).collect())]),
            ]);
            new_events.push(Event { node: self_node_id.to_string(), seq: fresh(), lamport: fresh_lamport(), typ: "task-defined".to_string(), payload });
        }

        let ts = state::task_state(journal, &task);
        if ts.generation > 0 {
            let typ = if ts.completed {
                "task-completed"
            } else if ts.holder.is_some() {
                "claim-committed"
            } else {
                "claim-released"
            };
            let mut fields = vec![
                Sexp::list(vec![Sexp::atom("task"), Sexp::atom(&task)]),
                Sexp::list(vec![Sexp::atom("generation"), Sexp::atom(ts.generation.to_string())]),
            ];
            if let Some(holder) = &ts.holder {
                fields.push(Sexp::list(vec![Sexp::atom("agent"), Sexp::atom(holder)]));
            }
            new_events.push(Event { node: self_node_id.to_string(), seq: fresh(), lamport: fresh_lamport(), typ: typ.to_string(), payload: Sexp::list(fields) });
        }
    }

    let members = state::membership(journal);
    let mut member_ids: Vec<&String> = members.keys().collect();
    member_ids.sort();
    for id in member_ids {
        let m = &members[id];
        let payload = Sexp::list(vec![
            Sexp::list(vec![Sexp::atom("node"), Sexp::atom(id)]),
            Sexp::list(vec![Sexp::atom("epoch"), Sexp::atom("0")]),
            Sexp::list(vec![Sexp::atom("capabilities"), Sexp::list(m.capabilities.iter().map(Sexp::atom).collect())]),
            Sexp::list(vec![Sexp::atom("roles"), Sexp::list(m.roles.iter().map(Sexp::atom).collect())]),
        ]);
        new_events.push(Event { node: self_node_id.to_string(), seq: fresh(), lamport: fresh_lamport(), typ: "agent-joined".to_string(), payload });
        if !m.present {
            let leave_payload = Sexp::list(vec![
                Sexp::list(vec![Sexp::atom("node"), Sexp::atom(id)]),
                Sexp::list(vec![Sexp::atom("epoch"), Sexp::atom("0")]),
            ]);
            new_events.push(Event { node: self_node_id.to_string(), seq: fresh(), lamport: fresh_lamport(), typ: "agent-left".to_string(), payload: leave_payload });
        }
    }

    let after = new_events.len();
    journal.replace_all(new_events)?;
    Ok((before, after))
}
