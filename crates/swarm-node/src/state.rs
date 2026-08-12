//! Derived task-ownership state (M0.2) and task definitions + scheduling
//! (M0.3): computed by folding the event log, never transmitted directly —
//! "same facts -> same reducer -> same state".

use crate::journal::{Event, Journal};
use crate::sexpr::Sexp;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct TaskState {
    pub generation: u64,
    pub holder: Option<String>,
    pub completed: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TaskDef {
    pub priority: f64,
    pub capabilities: Vec<String>,
    pub depends_on: Vec<String>,
    pub description: Option<String>,
}

/// Folds `task-defined` events for `task`, last one wins (a redefinition
/// replaces the previous one wholesale). A concurrent conflicting redefine
/// from two agents is a known, deliberately unhandled edge case — task
/// metadata churns far less than claims, so it wasn't worth a generation
/// scheme on top of the one claims already have.
pub fn task_def(journal: &Journal, task: &str) -> Option<TaskDef> {
    let mut def = None;
    for ev in &journal.events {
        if ev.typ != "task-defined" || ev.payload.field_atom("task") != Some(task) {
            continue;
        }
        let priority: f64 = ev.payload.field_atom("priority").and_then(|s| s.parse().ok()).unwrap_or(1.0);
        let capabilities = string_list(&ev.payload, "capabilities");
        let depends_on = string_list(&ev.payload, "depends-on");
        let description = ev.payload.field_atom("description").map(|s| s.to_string());
        def = Some(TaskDef { priority, capabilities, depends_on, description });
    }
    def
}

fn string_list(payload: &Sexp, key: &str) -> Vec<String> {
    match payload.field(key).and_then(|f| f.first()) {
        Some(Sexp::List(items)) => items
            .iter()
            .filter_map(|i| match i {
                Sexp::Atom(s) | Sexp::Str(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
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
        "claim-committed" | "claim-released" | "task-completed" | "task-defined" => {
            ev.payload.field_atom("task").map(|s| s.to_string())
        }
        _ => None,
    }
}

/// A schedulable candidate: has a definition, isn't claimed or completed,
/// and everything it depends on is completed.
struct Candidate {
    task: String,
    score: f64,
}

/// `next-best-action`: mirrors the scoring already documented for the
/// `:9999` oracle (score = priority * (1 + unblock_impact)), with
/// capability match as a hard gate rather than a down-rank — an agent
/// missing a required capability should never be offered a task it can't
/// actually do. Ties broken by task id for determinism across nodes.
pub fn next_best_action(journal: &Journal, capabilities: &[String]) -> Option<(String, TaskDef, TaskState)> {
    let ids = all_task_ids(journal);
    let defs: Vec<(String, TaskDef)> = ids.iter().filter_map(|id| task_def(journal, id).map(|d| (id.clone(), d))).collect();

    let is_done = |task: &str| -> bool {
        defs.iter().find(|(id, _)| id == task).map(|_| task_state(journal, task).completed).unwrap_or(false)
    };

    let mut candidates: Vec<Candidate> = Vec::new();
    for (id, def) in &defs {
        let ts = task_state(journal, id);
        if ts.completed || ts.holder.is_some() {
            continue;
        }
        if !def.capabilities.iter().all(|c| capabilities.contains(c)) {
            continue;
        }
        if !def.depends_on.iter().all(|dep| is_done(dep)) {
            continue;
        }
        let unblock_impact = defs
            .iter()
            .filter(|(_, other)| other.depends_on.contains(id))
            .count() as f64;
        candidates.push(Candidate { task: id.clone(), score: def.priority * (1.0 + unblock_impact) });
    }

    candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal).then(a.task.cmp(&b.task)));
    let best = candidates.into_iter().next()?;
    let def = task_def(journal, &best.task)?;
    let ts = task_state(journal, &best.task);
    Some((best.task, def, ts))
}

/// M0.4: membership as facts, not a mutable table. `agent-joined` records
/// (or updates) a node's declared capabilities/roles; `agent-left` marks it
/// absent without erasing its history. Distinct from `presence` (main.rs),
/// which is live connection state — membership is "has this node ever
/// declared itself part of the swarm and what can it do", independent of
/// whether it happens to be connected to *this* node right now.
#[derive(Debug, Clone, Default)]
pub struct Member {
    pub capabilities: Vec<String>,
    pub roles: Vec<String>,
    pub present: bool,
}

pub fn is_voter(member: &Member) -> bool {
    member.roles.iter().any(|r| r == "voter")
}

pub fn membership(journal: &Journal) -> HashMap<String, Member> {
    let mut members: HashMap<String, Member> = HashMap::new();
    for ev in &journal.events {
        match ev.typ.as_str() {
            "agent-joined" => {
                let Some(node) = ev.payload.field_atom("node") else { continue };
                let capabilities = string_list(&ev.payload, "capabilities");
                let mut roles = string_list(&ev.payload, "roles");
                if roles.is_empty() {
                    roles.push("worker".to_string());
                }
                members.insert(node.to_string(), Member { capabilities, roles, present: true });
            }
            "agent-left" => {
                let Some(node) = ev.payload.field_atom("node") else { continue };
                if let Some(m) = members.get_mut(node) {
                    m.present = false;
                }
            }
            _ => {}
        }
    }
    members
}
