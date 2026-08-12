//! Derived task-ownership state (M0.2): computed by folding the event log,
//! never transmitted directly — "same facts -> same reducer -> same state".

use crate::journal::{Event, Journal};

#[derive(Debug, Clone, Default)]
pub struct TaskState {
    pub generation: u64,
    pub holder: Option<String>,
    pub completed: bool,
}

/// Folds `claim-committed` / `claim-released` / `task-completed` events (in
/// journal order, which is insertion order — append-only, never rewritten)
/// into current ownership per task. `claim-committed` at a lower generation
/// than what's already recorded is a stale/duplicate proposal replay and is
/// ignored, which is what makes fencing hold even if events arrive out of
/// the original real-time order during anti-entropy sync.
pub fn task_state(journal: &Journal, task: &str) -> TaskState {
    let mut state = TaskState::default();
    for ev in &journal.events {
        if event_task(ev).as_deref() != Some(task) {
            continue;
        }
        let generation: u64 = ev
            .payload
            .field_atom("generation")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        match ev.typ.as_str() {
            "claim-committed" if generation > state.generation || (generation == state.generation && state.holder.is_none()) => {
                state.generation = generation;
                state.holder = ev.payload.field_atom("agent").map(|s| s.to_string());
                state.completed = false;
            }
            "claim-released" if generation >= state.generation => {
                state.generation = generation.max(state.generation);
                state.holder = None;
            }
            "task-completed" if generation >= state.generation => {
                state.generation = generation.max(state.generation);
                state.completed = true;
                state.holder = None;
            }
            _ => {}
        }
    }
    state
}

pub fn all_task_ids(journal: &Journal) -> Vec<String> {
    let mut ids: Vec<String> = journal
        .events
        .iter()
        .filter_map(event_task)
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

fn event_task(ev: &Event) -> Option<String> {
    match ev.typ.as_str() {
        "claim-committed" | "claim-released" | "task-completed" => {
            ev.payload.field_atom("task").map(|s| s.to_string())
        }
        _ => None,
    }
}
