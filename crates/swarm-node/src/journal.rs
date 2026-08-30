//! Durable event journal + node identity, per the M0.1 scope in
//! docs/swarm-mesh-v2.md: append-first, ack-after, restart-safe.

use crate::sexpr::{parse, Sexp};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Event {
    pub node: String,
    /// Durable identity lifetime this event was issued under. `None` for
    /// legacy events written before M1.1a (and for events from pre-M1.1a
    /// peers); those keep the old `(node, seq)` dedup semantics.
    pub incarnation: Option<String>,
    pub seq: u64,
    pub lamport: u64,
    pub typ: String,
    pub payload: Sexp,
}

impl Event {
    pub fn id(&self) -> String {
        match &self.incarnation {
            Some(inc) => format!("{}:{}:{}", self.node, inc, self.seq),
            None => format!("{}:{}", self.node, self.seq),
        }
    }

    pub fn to_sexp(&self) -> Sexp {
        // Flat shape, same as pre-M1.1a: (event (id ..) (node ..) [(incarnation ..)] (seq ..) ...).
        let mut fields = vec![
            Sexp::list(vec![Sexp::atom("id"), Sexp::atom(self.id())]),
            Sexp::list(vec![Sexp::atom("node"), Sexp::atom(&self.node)]),
        ];
        if let Some(inc) = &self.incarnation {
            fields.push(Sexp::list(vec![Sexp::atom("incarnation"), Sexp::atom(inc)]));
        }
        fields.push(Sexp::list(vec![
            Sexp::atom("seq"),
            Sexp::atom(self.seq.to_string()),
        ]));
        fields.push(Sexp::list(vec![
            Sexp::atom("lamport"),
            Sexp::atom(self.lamport.to_string()),
        ]));
        fields.push(Sexp::list(vec![Sexp::atom("type"), Sexp::atom(&self.typ)]));
        fields.push(Sexp::list(vec![
            Sexp::atom("payload"),
            self.payload.clone(),
        ]));
        Sexp::list(std::iter::once(Sexp::atom("event")).chain(fields).collect())
    }

    pub fn from_sexp(s: &Sexp) -> Result<Event, String> {
        let node = s
            .field_atom("node")
            .ok_or("event missing node")?
            .to_string();
        // "-" is the legacy-namespace sentinel in sync-hello v2 maps; a
        // peer sending `(incarnation -)` would alias it (review finding
        // F2), so reject it at parse time.
        let incarnation = match s.field_atom("incarnation") {
            Some("-") => return Err("invalid incarnation `-`".to_string()),
            Some(inc) => Some(inc.to_string()),
            None => None,
        };
        let seq: u64 = s
            .field_atom("seq")
            .ok_or("event missing seq")?
            .parse()
            .map_err(|_| "event seq not a number".to_string())?;
        let lamport: u64 = s
            .field_atom("lamport")
            .ok_or("event missing lamport")?
            .parse()
            .map_err(|_| "event lamport not a number".to_string())?;
        let typ = s
            .field_atom("type")
            .ok_or("event missing type")?
            .to_string();
        let payload = s
            .field("payload")
            .and_then(|f| f.first())
            .cloned()
            .unwrap_or(Sexp::List(vec![]));
        Ok(Event {
            node,
            incarnation,
            seq,
            lamport,
            typ,
            payload,
        })
    }
}

/// Stable node-id + restart-counting epoch + durable incarnation id,
/// persisted at `<data-dir>/node.my`.
///
/// Identity model (M1.1a):
/// - `node_id` — logical actor name, stable across lifetimes;
/// - `incarnation` — opaque id generated once when the identity store is
///   first created. Survives normal restarts; a LOST data-dir produces a
///   NEW incarnation under the same node-id, so old and new event streams
///   never collide even though both restart their seq at 1;
/// - `epoch` — process-restart counter *within* one incarnation.
pub struct Identity {
    pub node_id: String,
    pub epoch: u64,
    pub incarnation: String,
}

/// Generates an opaque incarnation id. Post-review (finding F4): 8 bytes
/// from the OS entropy pool via /dev/urandom, hex-encoded — no
/// deterministic-hash reasoning burden. Falls back to a time/pid hash only
/// if /dev/urandom is unreadable (non-Linux/testing environments).
pub fn fresh_incarnation() -> String {
    use std::io::Read;
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        let mut buf = [0u8; 8];
        if f.read_exact(&mut buf).is_ok() {
            return buf.iter().map(|b| format!("{b:02x}")).collect();
        }
    }
    // Fallback (non-Linux/testing): time + pid hash.
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
        .hash(&mut h);
    std::process::id().hash(&mut h);
    format!("{:016x}", h.finish())
}

pub fn load_or_init_identity(data_dir: &Path, node_id: &str) -> std::io::Result<Identity> {
    fs::create_dir_all(data_dir)?;
    let path = data_dir.join("node.my");
    let (epoch, stored_incarnation) = if path.exists() {
        let text = fs::read_to_string(&path)?;
        let parsed = parse(&text).unwrap_or(Sexp::List(vec![]));
        if let Some(stored_id) = parsed.field_atom("id") {
            if stored_id != node_id {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "data-dir identity mismatch: stored node-id `{stored_id}`, requested `{node_id}`"
                    ),
                ));
            }
        }
        let e = parsed
            .field_atom("epoch")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0)
            + 1;
        // A pre-M1.1a node.my has no incarnation: generate one now and
        // persist it — this upgrade keeps the journal's legacy events
        // (incarnation-less) distinct from everything this process will
        // emit from here on.
        let inc = match parsed.field_atom("incarnation") {
            Some(s) => s.to_string(),
            None => fresh_incarnation(),
        };
        (e, inc)
    } else {
        (0, fresh_incarnation())
    };
    let doc = Sexp::list(vec![
        Sexp::atom("node"),
        Sexp::list(vec![Sexp::atom("id"), Sexp::atom(node_id)]),
        Sexp::list(vec![Sexp::atom("epoch"), Sexp::atom(epoch.to_string())]),
        Sexp::list(vec![
            Sexp::atom("incarnation"),
            Sexp::atom(&stored_incarnation),
        ]),
    ]);
    fs::write(&path, doc.to_text())?;
    Ok(Identity {
        node_id: node_id.to_string(),
        epoch,
        incarnation: stored_incarnation,
    })
}

/// Append-only durable log at `<data-dir>/events.log`, one event per line.
pub struct Journal {
    path: PathBuf,
    file: File,
    pub events: Vec<Event>,
}

impl Journal {
    pub fn open(data_dir: &Path) -> std::io::Result<Journal> {
        fs::create_dir_all(data_dir)?;
        let path = data_dir.join("events.log");
        let mut events = Vec::new();
        if path.exists() {
            let reader = BufReader::new(File::open(&path)?);
            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(sexp) = parse(&line) {
                    if let Ok(ev) = Event::from_sexp(&sexp) {
                        events.push(ev);
                    }
                }
            }
        }
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Journal { path, file, events })
    }

    /// Persists `event` to disk (fsync'd) before it is considered committed —
    /// callers must only ACK/broadcast after this returns Ok.
    pub fn append(&mut self, event: Event) -> std::io::Result<()> {
        let line = event.to_sexp().to_text();
        writeln!(self.file, "{line}")?;
        self.file.sync_data()?;
        self.events.push(event);
        Ok(())
    }

    /// Wholesale-replaces the on-disk log and in-memory event list — used
    /// by compaction (`compact.rs`) to swap the full history for a smaller
    /// equivalent set. Callers are responsible for the replacement events
    /// being derivation-equivalent to what they replace; this method just
    /// does the (fsync'd) file swap safely.
    pub fn replace_all(&mut self, new_events: Vec<Event>) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;
        for ev in &new_events {
            writeln!(file, "{}", ev.to_sexp().to_text())?;
        }
        file.sync_data()?;
        self.file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        self.events = new_events;
        Ok(())
    }

    pub fn has(&self, node: &str, incarnation: Option<&str>, seq: u64) -> bool {
        self.events
            .iter()
            .any(|e| e.node == node && e.incarnation.as_deref() == incarnation && e.seq == seq)
    }

    pub fn last_seq(&self, node: &str, incarnation: Option<&str>) -> u64 {
        self.events
            .iter()
            .filter(|e| e.node == node && e.incarnation.as_deref() == incarnation)
            .map(|e| e.seq)
            .max()
            .unwrap_or(0)
    }

    pub fn next_seq(&self, node: &str, incarnation: Option<&str>) -> u64 {
        self.last_seq(node, incarnation) + 1
    }

    pub fn max_lamport(&self) -> u64 {
        self.events.iter().map(|e| e.lamport).max().unwrap_or(0)
    }

    pub fn events_after(&self, node: &str, incarnation: Option<&str>, seq: u64) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| e.node == node && e.incarnation.as_deref() == incarnation && e.seq > seq)
            .collect()
    }

    /// All distinct `(node, incarnation)` origins present in the journal.
    /// Legacy events (no incarnation) report `None`.
    pub fn all_origins(&self) -> Vec<(String, Option<String>)> {
        let mut origins: Vec<(String, Option<String>)> = self
            .events
            .iter()
            .map(|e| (e.node.clone(), e.incarnation.clone()))
            .collect();
        origins.sort();
        origins.dedup();
        origins
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod incarnation_tests {
    use super::*;

    #[test]
    fn roundtrips_incarnation_through_sexp() {
        let ev = Event {
            node: "a1".to_string(),
            incarnation: Some("abc123".to_string()),
            seq: 7,
            lamport: 9,
            typ: "task-defined".to_string(),
            payload: Sexp::list(vec![Sexp::list(vec![Sexp::atom("task"), Sexp::atom("T1")])]),
        };
        let text = ev.to_sexp().to_text();
        eprintln!("wire: {text}");
        let parsed = parse(&text).unwrap();
        let back = Event::from_sexp(&parsed).unwrap();
        assert_eq!(back.incarnation.as_deref(), Some("abc123"));
        assert_eq!(back.seq, 7);
        assert_eq!(back.node, "a1");
        assert_eq!(back.id(), "a1:abc123:7");
    }

    #[test]
    fn legacy_events_parse_without_incarnation() {
        let line = "(event (id a1:1) (node a1) (seq 1) (lamport 2) (type task-defined) (payload ((task T))))";
        let back = Event::from_sexp(&parse(line).unwrap()).unwrap();
        assert_eq!(back.incarnation, None);
        assert_eq!(back.id(), "a1:1");
    }

    #[test]
    fn has_distinguishes_incarnations() {
        let dir = std::env::temp_dir().join(format!("inc-test-{}", std::process::id()));
        let mut j = Journal::open(&dir).unwrap();
        let mk = |inc: Option<&str>, seq: u64| Event {
            node: "n".to_string(),
            incarnation: inc.map(|s| s.to_string()),
            seq,
            lamport: seq,
            typ: "x".into(),
            payload: Sexp::List(vec![]),
        };
        j.append(mk(Some("AAA"), 1)).unwrap();
        assert!(j.has("n", Some("AAA"), 1));
        assert!(
            !j.has("n", Some("BBB"), 1),
            "different incarnation must not dedup-hit"
        );
        j.append(mk(Some("BBB"), 1)).unwrap();
        assert_eq!(j.next_seq("n", Some("AAA")), 2);
        assert_eq!(j.next_seq("n", Some("BBB")), 2);
        assert_eq!(j.all_origins().len(), 2);
    }
}
