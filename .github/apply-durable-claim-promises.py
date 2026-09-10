from pathlib import Path

path = Path("crates/swarm-node/src/main.rs")
text = path.read_text()

replacements = []

replacements.append((
'''mod compact;
mod journal;
''',
'''mod claim_promise;
mod compact;
mod journal;
'''))

replacements.append((
'''const VOTE_TIMEOUT: Duration = Duration::from_millis(1500);
/// How long a voter's "I promised generation N to someone" holds before it
/// expires and can be re-promised. Must exceed `VOTE_TIMEOUT` with margin
/// so a proposer that's still legitimately waiting on votes doesn't get
/// undercut by its own promise expiring first; bounds how long a task can
/// get stuck if a proposer dies mid-vote without completing or retrying.
const PROMISE_TTL: Duration = Duration::from_secs(5);
''',
'''const VOTE_TIMEOUT: Duration = Duration::from_millis(1500);
'''))

replacements.append((
'''    /// Per-task voting promises: `task -> (generation we last voted yes
    /// for, when)`. Closes the concurrent-proposal gap noted as deferred
    /// in M0.2 — without this, two proposers racing for the same task
    /// could each collect yes votes from disjoint voter sets (e.g. across
    /// a network partition) and both reach quorum on the same generation.
    /// A voter now refuses to vote yes again for a task/generation it's
    /// already promised, until that promise expires (`PROMISE_TTL`).
    promised: Mutex<HashMap<String, (u64, Instant)>>,
''',
'''    /// Durable per-task voting promises. A YES is published to
    /// `<data-dir>/claim-promises.my` before it can leave this process, so
    /// crash/restart cannot erase a same-generation single-vote fence.
    claim_promises: Mutex<claim_promise::ClaimPromiseStore>,
'''))

replacements.append((
'''    let identity = journal::load_or_init_identity(&args.data_dir, &args.node_id)?;
    let journal = Journal::open(&args.data_dir)?;
    let lamport_start = journal.max_lamport();
''',
'''    let identity = journal::load_or_init_identity(&args.data_dir, &args.node_id)?;
    let journal = Journal::open(&args.data_dir)?;
    // Safety-critical local vote state is recovered before the node can
    // answer any claim proposal. Corruption fails startup closed instead of
    // silently forgetting a prior YES.
    let claim_promises = claim_promise::ClaimPromiseStore::open(&args.data_dir)?;
    let lamport_start = journal.max_lamport();
'''))

replacements.append((
'''        caught_up_with: Mutex::new(HashSet::new()),
        promised: Mutex::new(HashMap::new()),
        last_seen: Mutex::new(HashMap::new()),
''',
'''        caught_up_with: Mutex::new(HashSet::new()),
        claim_promises: Mutex::new(claim_promises),
        last_seen: Mutex::new(HashMap::new()),
'''))

replacements.append((
'''/// Atomically reserves this voter's YES vote for one task generation.
/// Local self-votes and remote proposal votes must pass through this exact
/// gate; otherwise two simultaneous proposers can each count themselves
/// while also voting YES for the competitor and both reach quorum.
fn try_acquire_claim_promise(node: &Arc<Node>, task: &str, generation: u64) -> bool {
    let mut promises = node
        .promised
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let promise_free = match promises.get(task) {
        Some((promised_gen, at)) => *promised_gen < generation || at.elapsed() > PROMISE_TTL,
        None => true,
    };
    if promise_free {
        promises.insert(task.to_string(), (generation, Instant::now()));
    }
    promise_free
}
''',
'''/// Durably reserves this voter's YES vote for one proposal identity.
/// Local self-votes and remote proposal votes pass through the same store.
/// `Ok(true)` means the fence was already durable (same proposer retry) or
/// has just been atomically published; an I/O error must therefore be
/// treated as NO/fail-closed and can never leak a YES onto the network.
fn try_acquire_claim_promise(
    node: &Arc<Node>,
    task: &str,
    generation: u64,
    proposer: &str,
) -> std::io::Result<bool> {
    node.claim_promises
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .acquire(task, generation, proposer)
}
'''))

replacements.append((
'''/// exactly the next one after what we've derived locally, task not already
/// held/completed) and the promise check (we haven't already voted yes for
/// this task at this-or-higher generation within `PROMISE_TTL`). The promise
/// is what actually excludes concurrent proposals — the fencing check alone
''',
'''/// exactly the next one after what we've derived locally, task not already
/// held/completed) and the durable promise check (we haven't already voted
/// yes for a competing proposer at this-or-higher generation). The promise
/// is what actually excludes concurrent proposals — the fencing check alone
'''))

replacements.append((
'''    let promise_free = if fencing_ok {
        try_acquire_claim_promise(node, &task, generation)
    } else {
        false
    };
''',
'''    let promise_free = if fencing_ok {
        match try_acquire_claim_promise(node, &task, generation, &agent) {
            Ok(granted) => granted,
            Err(error) => {
                // The durability write happens before any YES.  Disk failure
                // therefore degrades to NO, never to an unsafe ephemeral vote.
                warn!(
                    "swarm-node: refusing claim YES for task={task} agent={agent} generation={generation}: durable promise failed: {error}"
                );
                false
            }
        }
    } else {
        false
    };
'''))

replacements.append((
'''    // Counting our own vote must acquire the same promise as a remote YES.
    // If this node already promised the competing proposal, abort instead
    // of self-voting (or trying to commit a conflicting local proposal).
    if self_votes == 1 && !try_acquire_claim_promise(node, task, generation) {
        send(
            stream,
            &Sexp::list(vec![
                Sexp::atom("error"),
                Sexp::string(format!(
                    "claim conflict for `{task}` generation {generation}: local voter already promised a competing proposal"
                )),
            ]),
        );
        return;
    }
''',
'''    // Counting our own vote must acquire the same durable promise as a
    // remote YES.  A failed persistence operation is a hard local NO: the
    // vote cannot be counted before its restart-safe fence exists.
    if self_votes == 1 {
        match try_acquire_claim_promise(
            node,
            task,
            generation,
            &node.identity.node_id,
        ) {
            Ok(true) => {}
            Ok(false) => {
                send(
                    stream,
                    &Sexp::list(vec![
                        Sexp::atom("error"),
                        Sexp::string(format!(
                            "claim conflict for `{task}` generation {generation}: local voter already promised a competing proposal"
                        )),
                    ]),
                );
                return;
            }
            Err(error) => {
                send(
                    stream,
                    &Sexp::list(vec![
                        Sexp::atom("error"),
                        Sexp::string(format!(
                            "cannot durably reserve local claim vote for `{task}` generation {generation}: {error}"
                        )),
                    ]),
                );
                return;
            }
        }
    }
'''))

replacements.append((
'''    let mut yes_votes = self_votes;
    let mut counted_responses = 0;
    let deadline = Instant::now() + VOTE_TIMEOUT;
''',
'''    let mut yes_votes = self_votes;
    let mut counted_responses = 0;
    // Same-proposer retries are allowed to re-send the already-durable YES,
    // so the proposer must count each voter identity at most once.
    let mut responded_voters = HashSet::new();
    let deadline = Instant::now() + VOTE_TIMEOUT;
'''))

replacements.append((
'''        match rx.recv_timeout(remaining) {
            Ok((voter, vote)) if voting_peers.contains(&voter) => {
                counted_responses += 1;
                if vote {
                    yes_votes += 1;
                }
            }
''',
'''        match rx.recv_timeout(remaining) {
            Ok((voter, vote))
                if voting_peers.contains(&voter) && responded_voters.insert(voter.clone()) =>
            {
                counted_responses += 1;
                if vote {
                    yes_votes += 1;
                }
            }
'''))

changed = 0
for old, new in replacements:
    count = text.count(old)
    if count == 1:
        text = text.replace(old, new, 1)
        changed += 1
    elif count == 0 and new in text:
        # Idempotent second workflow run after the patch commit.
        continue
    else:
        raise SystemExit(f"anchor count={count}, expected exactly one: {old[:120]!r}")

path.write_text(text)
print(f"durable claim promise wiring: {changed} replacement(s) applied")