from pathlib import Path

path = Path("crates/swarm-node/src/journal.rs")
text = path.read_text()

old = '''    /// Wholesale-replaces the on-disk log and in-memory event list — used
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
'''

new = '''    /// Wholesale-replaces the on-disk log and in-memory event list — used
    /// by compaction (`compact.rs`) to swap the full history for a smaller
    /// equivalent set. Callers are responsible for the replacement events
    /// being derivation-equivalent to what they replace. Publication is
    /// transactional: the live journal is never truncated in place.
    pub fn replace_all(&mut self, new_events: Vec<Event>) -> std::io::Result<()> {
        self.replace_all_with_before_publish(new_events, |_| Ok(()))
    }

    /// Transactional implementation with a deterministic fault-injection
    /// seam immediately before the atomic publish point. Any error before
    /// `rename` leaves both the live file and `self.events` unchanged.
    fn replace_all_with_before_publish<F>(
        &mut self,
        new_events: Vec<Event>,
        before_publish: F,
    ) -> std::io::Result<()>
    where
        F: FnOnce(&Path) -> std::io::Result<()>,
    {
        let temp_path = self.path.with_extension("log.tmp");

        match fs::remove_file(&temp_path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err),
        }

        let result = (|| {
            let mut temp_file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp_path)?;
            for ev in &new_events {
                writeln!(temp_file, "{}", ev.to_sexp().to_text())?;
            }
            temp_file.sync_all()?;

            before_publish(&temp_path)?;

            // Open the future append handle before publication so there is no
            // fallible reopen step after the atomic rename. The handle keeps
            // referring to the same file once that file becomes `events.log`.
            let append_file = OpenOptions::new().append(true).open(&temp_path)?;
            fs::rename(&temp_path, &self.path)?;

            self.file = append_file;
            self.events = new_events;
            Ok(())
        })();

        if result.is_err() {
            let _ = fs::remove_file(&temp_path);
        }
        result
    }
'''

if text.count(old) != 1:
    raise SystemExit(f"replace_all block count={text.count(old)}, expected 1")
text = text.replace(old, new, 1)

anchor = '''    #[test]
    fn journal_open_rejects_structurally_invalid_event_without_skipping_it() {
        let dir = test_dir("invalid-event");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("events.log");
        let before = "(event (node n) (seq 1) (type x))\\n";
        fs::write(&path, before).unwrap();

        let err = Journal::open(&dir).err().expect("invalid event must fail closed");

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("event missing lamport"));
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
        let _ = fs::remove_dir_all(dir);
    }
'''

tests = anchor + '''
    #[test]
    fn replace_all_failure_before_publish_preserves_live_file_and_memory() {
        let dir = test_dir("replace-failure");
        let mut journal = Journal::open(&dir).unwrap();
        journal.append(event("old", Some("AAA"), 1)).unwrap();
        journal.append(event("old", Some("AAA"), 2)).unwrap();

        let live_path = dir.join("events.log");
        let before_bytes = fs::read(&live_path).unwrap();
        let before_ids: Vec<String> = journal.events.iter().map(Event::id).collect();
        let replacement = vec![event("new", Some("BBB"), 1)];

        let err = journal
            .replace_all_with_before_publish(replacement, |temp_path| {
                assert!(temp_path.exists());
                Err(std::io::Error::other("injected failure before publish"))
            })
            .expect_err("injected pre-publish failure must abort replacement");

        assert_eq!(err.kind(), std::io::ErrorKind::Other);
        assert_eq!(fs::read(&live_path).unwrap(), before_bytes);
        assert_eq!(
            journal.events.iter().map(Event::id).collect::<Vec<_>>(),
            before_ids
        );
        assert!(!live_path.with_extension("log.tmp").exists());

        let replay = Journal::open(&dir).unwrap();
        assert_eq!(
            replay.events.iter().map(Event::id).collect::<Vec<_>>(),
            before_ids
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn replace_all_publishes_complete_log_and_keeps_append_handle_live() {
        let dir = test_dir("replace-success");
        let mut journal = Journal::open(&dir).unwrap();
        journal.append(event("old", Some("AAA"), 1)).unwrap();

        let replacement = vec![
            event("new", Some("BBB"), 1),
            event("new", Some("BBB"), 2),
        ];
        let expected_replacement_ids: Vec<String> = replacement.iter().map(Event::id).collect();
        journal.replace_all(replacement).unwrap();

        assert_eq!(
            journal.events.iter().map(Event::id).collect::<Vec<_>>(),
            expected_replacement_ids
        );

        let appended = event("new", Some("BBB"), 3);
        let appended_id = appended.id();
        journal.append(appended).unwrap();

        let replay = Journal::open(&dir).unwrap();
        let replay_ids: Vec<String> = replay.events.iter().map(Event::id).collect();
        assert_eq!(replay_ids.len(), 3);
        assert_eq!(&replay_ids[..2], expected_replacement_ids.as_slice());
        assert_eq!(replay_ids[2], appended_id);
        assert!(!replay_ids.iter().any(|id| id.starts_with("old:")));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn journal_open_ignores_stale_partial_replacement_temp_file() {
        let dir = test_dir("stale-replacement-temp");
        let mut journal = Journal::open(&dir).unwrap();
        let authoritative = event("live", Some("AAA"), 1);
        let authoritative_id = authoritative.id();
        journal.append(authoritative).unwrap();
        drop(journal);

        let live_path = dir.join("events.log");
        let temp_path = live_path.with_extension("log.tmp");
        fs::write(&temp_path, "(event (node partial)\\n").unwrap();

        let replay = Journal::open(&dir).expect("stale temp must not become authority");
        assert_eq!(replay.events.len(), 1);
        assert_eq!(replay.events[0].id(), authoritative_id);
        assert!(temp_path.exists(), "open must not confuse or publish the temp file");
        let _ = fs::remove_dir_all(dir);
    }
'''

if text.count(anchor) != 1:
    raise SystemExit(f"test anchor count={text.count(anchor)}, expected 1")
text = text.replace(anchor, tests, 1)
path.write_text(text)
