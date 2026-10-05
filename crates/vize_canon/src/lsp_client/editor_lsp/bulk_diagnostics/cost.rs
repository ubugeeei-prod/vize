//! Default-off observation within the existing diagnostic operations.

use serde_json::{Value, json};
use std::{cell::RefCell, time::Instant};

pub(super) struct Cost {
    start: Option<Instant>,
    observations: RefCell<Vec<Value>>,
    outcome: &'static str,
}

impl Cost {
    pub(super) fn new() -> Self {
        Self {
            start: (std::env::var_os("VIZE_CANON_BULK_COST_TRACE").as_deref()
                == Some(std::ffi::OsStr::new("1")))
            .then(Instant::now),
            observations: RefCell::new(Vec::new()),
            outcome: "incomplete",
        }
    }

    pub(super) fn tick(&self) -> Option<Instant> {
        self.start.map(|_| Instant::now())
    }

    pub(super) fn duration(&self, name: &str, start: Option<Instant>) {
        if let Some(start) = start {
            self.observations
                .borrow_mut()
                .push(json!({"operation":name,"microseconds":start.elapsed().as_micros()}));
        }
    }

    pub(super) fn count(&self, name: &str, count: usize) {
        if self.start.is_some() {
            self.observations
                .borrow_mut()
                .push(json!({"counter":name,"value":count}));
        }
    }

    pub(super) fn enabled(&self) -> bool {
        self.start.is_some()
    }

    pub(super) fn outcome(&mut self, outcome: &'static str) {
        self.outcome = outcome;
    }
}

impl Drop for Cost {
    fn drop(&mut self) {
        if let Some(start) = self.start {
            eprintln!(
                "bulk-cost-observation {}",
                json!({
                    "schemaVersion":1,"outcome":self.outcome,
                    "totalMicroseconds":start.elapsed().as_micros(),
                    "observations":self.observations.get_mut(),
                    "scope":"Opt-in instrumentation only; no matched speed or default claim.",
                })
            );
        }
    }
}
