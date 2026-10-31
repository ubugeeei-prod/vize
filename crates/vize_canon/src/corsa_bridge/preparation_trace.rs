//! Opt-in, numeric-only observation of existing editor preparation work.

use std::future::Future;
use std::sync::{
    OnceLock,
    atomic::{AtomicU64, Ordering},
};
use std::time::Instant;

#[cfg(feature = "profiling")]
use tracing::Instrument;

static ENABLED: OnceLock<bool> = OnceLock::new();
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
#[cfg(feature = "profiling")]
static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);

pub(crate) fn enabled() -> bool {
    *ENABLED.get_or_init(|| {
        std::env::var("VIZE_TRACE_EDITOR_PREPARATION").is_ok_and(|value| value == "1")
    })
}

/// Disabled sessions do not read clocks or record inputs, paths or payloads.
/// A dropped unfinished phase also records cancellation/error unwinding.
pub struct Phase {
    name: &'static str,
    id: u64,
    started: Option<Instant>,
    completed: bool,
    context: tracing::Span,
}

impl Phase {
    /// Observe an existing operation only when editor preparation tracing is enabled.
    pub fn start(name: &'static str, items: usize) -> Self {
        let enabled = enabled();
        let id = if enabled {
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        } else {
            0
        };
        let started = enabled.then(Instant::now);
        let context = Self::context();
        if enabled {
            context.in_scope(|| {
                if let Some(context_id) = context.id() {
                    tracing::info!(target: "vize_editor_preparation", phase = name, id, items,
                        context_span = context_id.into_u64(), event = "begin");
                } else {
                    tracing::info!(target: "vize_editor_preparation", phase = name, id, items, event = "begin");
                }
            });
        }
        Self {
            name,
            id,
            started,
            completed: false,
            context,
        }
    }

    /// Navigation probes require the existing profiling feature as well as the trace flag.
    pub fn navigation(name: &'static str, items: usize) -> Self {
        #[cfg(feature = "profiling")]
        if enabled() {
            return Self::start(name, items);
        }
        let _ = items;
        Self {
            name,
            id: 0,
            started: None,
            completed: false,
            context: tracing::Span::none(),
        }
    }

    /// Capture a tracing context for entry only inside an existing synchronous worker job.
    pub fn context() -> tracing::Span {
        #[cfg(feature = "profiling")]
        if enabled() {
            return tracing::Span::current();
        }
        tracing::Span::none()
    }

    /// Observe the original future without holding an entered span across await.
    pub async fn request<F: Future>(method: &'static str, future: F) -> F::Output {
        #[cfg(feature = "profiling")]
        if enabled() {
            return observed_request(method, future).await;
        }
        let _ = method;
        future.await
    }

    /// Record that the observed operation returned, including a returned error.
    pub fn finish(mut self) {
        self.completed = true;
    }
}

#[cfg(feature = "profiling")]
async fn observed_request<F: Future>(method: &'static str, future: F) -> F::Output {
    let request_id = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed);
    let context = tracing::info_span!(target: "vize_editor_preparation",
        "navigation_request", request_id, method);
    context.in_scope(|| {
        tracing::info!(target: "vize_editor_preparation", request_id, method,
            context_span = context.id().map_or(0, |id| id.into_u64()), event = "request");
    });
    async {
        let phase = Phase::navigation("navigation.request", 1);
        let result = future.await;
        phase.finish();
        result
    }
    .instrument(context)
    .await
}

impl Drop for Phase {
    fn drop(&mut self) {
        if let Some(started) = self.started {
            self.context.in_scope(|| {
                if let Some(context_id) = self.context.id() {
                    tracing::info!(target: "vize_editor_preparation", phase = self.name, id = self.id,
                        context_span = context_id.into_u64(),
                        elapsed_ms = started.elapsed().as_secs_f64() * 1000.0,
                        completed = self.completed, event = "end");
                } else {
                    tracing::info!(target: "vize_editor_preparation", phase = self.name, id = self.id,
                        elapsed_ms = started.elapsed().as_secs_f64() * 1000.0,
                        completed = self.completed, event = "end");
                }
            });
        }
    }
}
