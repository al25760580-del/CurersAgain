//! Explicit opt-in server transport. Blocking I/O runs outside the SDL thread.
//! Linux prototype uses a tightly scoped Python stdlib helper, not Steam auth files.
use crate::scenes::scores::Score;
use serde::Deserialize;
use std::{
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};
#[derive(Clone, Debug, PartialEq)]
pub struct Query {
    pub stage: String,
    pub character: String,
    pub daily: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reply {
    schema: u32,
    records: Vec<Score>,
}
pub enum Poll {
    Waiting,
    Ready(Vec<Score>),
    Failed(String),
}
pub struct Server {
    helper: PathBuf,
    config: PathBuf,
    pending: Option<(Query, Receiver<Result<Vec<Score>, String>>)>,
    loaded: Option<Query>,
    last_start: Option<Instant>,
}
impl Server {
    pub fn new(helper: PathBuf, config: PathBuf) -> Result<Self, String> {
        if !helper.is_file() || !config.is_file() {
            return Err("Missing read-only helper or public client config".into());
        }
        Ok(Self {
            helper,
            config,
            pending: None,
            loaded: None,
            last_start: None,
        })
    }
    pub fn refresh(&mut self) {
        self.loaded = None;
    }
    pub fn poll(&mut self, query: Query) -> Option<Poll> {
        if let Some((old, rx)) = &self.pending {
            match rx.try_recv() {
                Ok(result) => {
                    let matches = *old == query;
                    self.pending = None;
                    if matches {
                        self.loaded = Some(query);
                        return Some(match result {
                            Ok(rows) => Poll::Ready(rows),
                            Err(e) => Poll::Failed(e),
                        });
                    }
                }
                Err(mpsc::TryRecvError::Empty) => return Some(Poll::Waiting),
                Err(_) => {
                    self.pending = None;
                    self.loaded = Some(query);
                    return Some(Poll::Failed("WORKER_DISCONNECTED".into()));
                }
            }
        }
        if self.loaded.as_ref() == Some(&query) {
            return None;
        }
        if self
            .last_start
            .is_some_and(|t| t.elapsed() < Duration::from_secs(5))
        {
            return Some(Poll::Waiting);
        }
        let helper = self.helper.clone();
        let config = self.config.clone();
        let wanted = query.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(fetch(helper, config, wanted));
        });
        self.pending = Some((query, rx));
        self.last_start = Some(Instant::now());
        Some(Poll::Waiting)
    }
}
fn fetch(helper: PathBuf, config: PathBuf, q: Query) -> Result<Vec<Score>, String> {
    let mut child = Command::new("python3")
        .arg(helper)
        .arg(config)
        .arg(q.stage)
        .arg(q.character)
        .arg(if q.daily { "daily" } else { "allTime" })
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "CANNOT_START_READ_ONLY_CLIENT")?;
    let mut bytes = Vec::new();
    let read = child
        .stdout
        .take()
        .ok_or("NO_CLIENT_OUTPUT")?
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes);
    if read.is_err() || bytes.len() > 4 * 1024 * 1024 {
        let _ = child.kill();
        let _ = child.wait();
        return Err("CLIENT_OUTPUT_LIMIT".into());
    }
    if !child.wait().map_err(|_| "CLIENT_WAIT_FAILED")?.success() {
        return Err(
            "SERVER_READ_FAILED: inspect configuration, supported filters or server permissions"
                .into(),
        );
    }
    let r: Reply = serde_json::from_slice(&bytes).map_err(|_| "INVALID_SERVER_REPLY")?;
    if r.schema != 1 || r.records.len() > 1000 {
        return Err("INVALID_SERVER_REPLY".into());
    }
    Ok(r.records)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_helper_is_rejected() {
        assert!(Server::new("/nonexistent-helper".into(), "/nonexistent-config".into()).is_err());
    }
    #[test]
    fn typed_wire_records_accept_real_integer_counts() {
        let r:Reply=serde_json::from_str(r#"{"schema":1,"records":[{"username":"Example","stage":"STAGE 1","character":"ame","score":10,"duration_seconds":60,"level":2}]}"#).unwrap();
        assert_eq!(r.records[0].score, 10);
        assert!(!r.records[0].is_player);
    }
}
