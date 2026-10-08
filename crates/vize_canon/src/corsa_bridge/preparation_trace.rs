//! Opt-in, numeric-only observation of existing editor preparation work.

use std::sync::{
    OnceLock,
    atomic::{AtomicU64, Ordering},
};
use std::time::Instant;

static ENABLED: OnceLock<bool> = OnceLock::new();
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Disabled sessions do not read clocks or record inputs, paths or payloads.
/// A dropped unfinished phase also records cancellation/error unwinding.
pub(crate) struct Phase {
    name: &'static str,
    id: u64,
    started: Option<Instant>,
    completed: bool,
}

impl Phase {
    pub(crate) fn start(name: &'static str, items: usize) -> Self {
        let enabled = *ENABLED.get_or_init(|| {
            std::env::var("VIZE_TRACE_EDITOR_PREPARATION").is_ok_and(|value| value == "1")
        });
        let id = if enabled {
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        } else {
            0
        };
        let started = enabled.then(Instant::now);
        if enabled {
            tracing::info!(target: "vize_editor_preparation", phase = name, id, items, event = "begin");
        }
        Self {
            name,
            id,
            started,
            completed: false,
        }
    }

    pub(crate) fn finish(mut self) {
        self.completed = true;
    }
}

impl Drop for Phase {
    fn drop(&mut self) {
        if let Some(started) = self.started {
            tracing::info!(target: "vize_editor_preparation", phase = self.name, id = self.id,
                elapsed_ms = started.elapsed().as_secs_f64() * 1000.0,
                completed = self.completed, event = "end");
        }
    }
}
