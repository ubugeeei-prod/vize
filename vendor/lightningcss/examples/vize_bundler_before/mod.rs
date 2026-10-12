use lightningcss::bundler::{ResolveResult, SourceProvider};
use serde_json::{json, Value};
use std::{
  collections::HashMap,
  fs::{File, OpenOptions},
  io::{self, Write},
  path::{Path, PathBuf},
  sync::{Condvar, Mutex},
  time::Duration,
};

struct State {
  events: File,
  sequence: usize,
  read_gate: bool,
}

pub struct Provider {
  files: HashMap<PathBuf, String>,
  aliases: HashMap<String, PathBuf>,
  entry: String,
  first: Option<String>,
  read_path: Option<PathBuf>,
  threads: usize,
  state: Mutex<State>,
  wake: Condvar,
}

impl Provider {
  pub fn new(case: &Value, threads: usize, events: PathBuf) -> Self {
    Self {
      files: case["files"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, source)| (PathBuf::from(name), source.as_str().unwrap().to_owned()))
        .collect(),
      aliases: case["aliases"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, path)| (name.clone(), PathBuf::from(path.as_str().unwrap())))
        .collect(),
      entry: case["entry"].as_str().unwrap().to_owned(),
      first: case["forceReadBeforeResolve"]["specifier"].as_str().map(str::to_owned),
      read_path: case["forceReadBeforeResolve"]["path"].as_str().map(PathBuf::from),
      threads,
      state: Mutex::new(State {
        events: OpenOptions::new().create_new(true).write(true).open(events).unwrap(),
        sequence: 0,
        read_gate: false,
      }),
      wake: Condvar::new(),
    }
  }

  fn event(state: &mut State, value: Value) {
    let value = json!({"sequence": state.sequence, "event": value});
    serde_json::to_writer(&mut state.events, &value).unwrap();
    writeln!(state.events).unwrap();
    state.events.flush().unwrap();
    state.sequence += 1;
  }
}

impl SourceProvider for Provider {
  type Error = io::Error;

  fn read<'a>(&'a self, file: &Path) -> Result<&'a str, Self::Error> {
    {
      let mut state = self.state.lock().unwrap();
      Self::event(&mut state, json!({"type": "read", "path": file}));
      if self.read_path.as_deref() == Some(file) {
        state.read_gate = true;
        self.wake.notify_all();
      }
    }
    self
      .files
      .get(file)
      .map(String::as_str)
      .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("fixture missing {}", file.display())))
  }

  fn resolve(&self, specifier: &str, originating_file: &Path) -> Result<ResolveResult, Self::Error> {
    let mut state = self.state.lock().unwrap();
    Self::event(
      &mut state,
      json!({"type": "resolve-start", "specifier": specifier, "origin": originating_file}),
    );
    let gated =
      self.threads > 1 && originating_file == Path::new(&self.entry) && self.first.as_deref() == Some(specifier);
    if gated {
      let (next, timeout) = self
        .wake
        .wait_timeout_while(state, Duration::from_secs(3), |state| !state.read_gate)
        .unwrap();
      state = next;
      if timeout.timed_out() && !state.read_gate {
        Self::event(&mut state, json!({"type": "gate-timeout", "specifier": specifier}));
        return Err(io::Error::new(
          io::ErrorKind::TimedOut,
          "later provider.read did not precede first resolve",
        ));
      }
    }
    Self::event(
      &mut state,
      json!({"type": "resolve-finish", "specifier": specifier, "origin": originating_file, "gated": gated}),
    );
    if specifier.starts_with("https:") {
      return Ok(ResolveResult::External(specifier.to_owned()));
    }
    Ok(
      self
        .aliases
        .get(specifier)
        .cloned()
        .unwrap_or_else(|| originating_file.with_file_name(specifier))
        .into(),
    )
  }
}
