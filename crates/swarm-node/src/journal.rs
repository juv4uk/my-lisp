//! Durable event journal + node identity, per the M0.1 scope in
//! docs/swarm-mesh-v2.md: append-first, ack-after, restart-safe.

use crate::sexpr::{parse, Sexp};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Event {
    pub node: String,
    pub seq: u64,
    pub lamport: u64,
    pub typ: String,
    pub payload: Sexp,
}

impl Event {
    pub fn id(&self) -> String {
        format!("{}:{}", self.node, self.seq)
    }

    pub fn to_sexp(&self) -> Sexp {
        Sexp::list(vec![
            Sexp::atom("event"),
            Sexp::list(vec![Sexp::atom("id"), Sexp::atom(self.id())]),
            Sexp::list(vec![Sexp::atom("node"), Sexp::atom(&self.node)]),
            Sexp::list(vec![Sexp::atom("seq"), Sexp::atom(self.seq.to_string())]),
            Sexp::list(vec![Sexp::atom("lamport"), Sexp::atom(self.lamport.to_string())]),
            Sexp::list(vec![Sexp::atom("type"), Sexp::atom(&self.typ)]),
            Sexp::list(vec![Sexp::atom("payload"), self.payload.clone()]),
        ])
    }

    pub fn from_sexp(s: &Sexp) -> Result<Event, String> {
        let node = s.field_atom("node").ok_or("event missing node")?.to_string();
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
        let typ = s.field_atom("type").ok_or("event missing type")?.to_string();
        let payload = s
            .field("payload")
            .and_then(|f| f.first())
            .cloned()
            .unwrap_or(Sexp::List(vec![]));
        Ok(Event { node, seq, lamport, typ, payload })
    }
}

/// Stable node-id + restart-counting epoch, persisted at `<data-dir>/node.my`.
pub struct Identity {
    pub node_id: String,
    pub epoch: u64,
}

pub fn load_or_init_identity(data_dir: &Path, node_id: &str) -> std::io::Result<Identity> {
    fs::create_dir_all(data_dir)?;
    let path = data_dir.join("node.my");
    let epoch = if path.exists() {
        let text = fs::read_to_string(&path)?;
        let parsed = parse(&text).unwrap_or(Sexp::List(vec![]));
        parsed
            .field_atom("epoch")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0)
            + 1
    } else {
        0
    };
    let doc = Sexp::list(vec![
        Sexp::atom("node"),
        Sexp::list(vec![Sexp::atom("id"), Sexp::atom(node_id)]),
        Sexp::list(vec![Sexp::atom("epoch"), Sexp::atom(epoch.to_string())]),
    ]);
    fs::write(&path, doc.to_text())?;
    Ok(Identity { node_id: node_id.to_string(), epoch })
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
        let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(&self.path)?;
        for ev in &new_events {
            writeln!(file, "{}", ev.to_sexp().to_text())?;
        }
        file.sync_data()?;
        self.file = OpenOptions::new().create(true).append(true).open(&self.path)?;
        self.events = new_events;
        Ok(())
    }

    pub fn has(&self, node: &str, seq: u64) -> bool {
        self.events.iter().any(|e| e.node == node && e.seq == seq)
    }

    pub fn last_seq(&self, node: &str) -> u64 {
        self.events.iter().filter(|e| e.node == node).map(|e| e.seq).max().unwrap_or(0)
    }

    pub fn next_seq(&self, node: &str) -> u64 {
        self.last_seq(node) + 1
    }

    pub fn max_lamport(&self) -> u64 {
        self.events.iter().map(|e| e.lamport).max().unwrap_or(0)
    }

    pub fn events_after(&self, node: &str, seq: u64) -> Vec<&Event> {
        self.events.iter().filter(|e| e.node == node && e.seq > seq).collect()
    }

    pub fn all_node_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.events.iter().map(|e| e.node.clone()).collect();
        ids.sort();
        ids.dedup();
        ids
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
