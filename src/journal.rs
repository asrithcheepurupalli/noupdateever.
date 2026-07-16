//! The journal: an append-only JSONL record of every change Stasis makes to a
//! machine, with the undo data needed to reverse it exactly. Open format,
//! documented in FORMATS.md, readable with a text editor. `thaw` replays it
//! in reverse; nothing Stasis does is off the books.

use crate::action::{Action, Undo};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub ts_unix: u64,
    pub session: String,
    pub op: Op,
    pub recipe_id: String,
    pub action: Action,
    pub undo: Undo,
    /// False means the action failed and changed nothing; kept for honesty.
    pub applied: bool,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    Apply,
    Revert,
}

pub fn append(path: &Path, entry: &Entry) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    let line = serde_json::to_string(entry).map_err(|e| e.to_string())?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("cannot open journal {}: {e}", path.display()))?;
    writeln!(f, "{line}").map_err(|e| format!("cannot write journal: {e}"))?;
    Ok(())
}

pub fn read_all(path: &Path) -> Result<Vec<Entry>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let f = fs::File::open(path)
        .map_err(|e| format!("cannot open journal {}: {e}", path.display()))?;
    let mut out = Vec::new();
    for (i, line) in BufReader::new(f).lines().enumerate() {
        let line = line.map_err(|e| format!("journal read error: {e}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let entry: Entry = serde_json::from_str(&line)
            .map_err(|e| format!("journal line {}: {e}", i + 1))?;
        out.push(entry);
    }
    Ok(out)
}

/// Undos that are still outstanding: applied entries whose undo has not been
/// reverted by a later entry, newest first (the order `thaw` must replay).
pub fn outstanding(entries: &[Entry]) -> Vec<Entry> {
    let mut live: Vec<Entry> = Vec::new();
    for e in entries {
        match e.op {
            Op::Apply => {
                if e.applied && e.undo != Undo::Nothing {
                    live.push(e.clone());
                }
            }
            Op::Revert => {
                // A revert cancels the most recent live entry with the same undo.
                if let Some(pos) = live.iter().rposition(|l| l.undo == e.undo) {
                    live.remove(pos);
                }
            }
        }
    }
    live.reverse();
    live
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, Undo};

    fn entry(op: Op, undo: Undo) -> Entry {
        Entry {
            ts_unix: 1,
            session: "s".into(),
            op,
            recipe_id: "chrome".into(),
            action: Action::DisableService { service: "gupdate".into() },
            undo,
            applied: true,
            note: String::new(),
        }
    }

    #[test]
    fn round_trip_and_outstanding() {
        let dir = std::env::temp_dir().join(format!("stasis-test-{}", std::process::id()));
        let path = dir.join("journal.jsonl");
        let u1 = Undo::ServiceStart { service: "gupdate".into(), prior_start: 2 };
        let u2 = Undo::ServiceStart { service: "gupdatem".into(), prior_start: 3 };
        append(&path, &entry(Op::Apply, u1.clone())).unwrap();
        append(&path, &entry(Op::Apply, u2.clone())).unwrap();
        append(&path, &entry(Op::Revert, u2)).unwrap();

        let all = read_all(&path).unwrap();
        assert_eq!(all.len(), 3);
        let live = outstanding(&all);
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].undo, u1);
        std::fs::remove_dir_all(&dir).ok();
    }
}
