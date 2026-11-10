//! Imported type modules shared by the transforms of one host burst.

use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use vize_canon::ContentMapperTypeSources;

/// A transform that starts within this long of the last one finishing joins
/// its burst. Hosts pause between transforms to parse their own files, and a
/// fresh snapshot re-resolves every imported type module.
const BURST_IDLE: Duration = Duration::from_secs(1);

/// Oldest snapshot a starting transform may still join, so a server that is
/// never idle still reads the filesystem again.
const MAX_AGE: Duration = Duration::from_secs(10);

/// Imported type modules shared within a host burst. Each dependency is read
/// once per burst; an idle server drops the snapshot and reads again.
pub(super) struct SharedTypeSources {
    state: Mutex<Generation>,
    changed: Condvar,
    idle: Duration,
    max_age: Duration,
}

#[derive(Default)]
struct Generation {
    active: usize,
    current: Option<(ContentMapperTypeSources, Instant)>,
    idle_since: Option<Instant>,
    stopped: bool,
}

impl Default for SharedTypeSources {
    fn default() -> Self {
        Self::with_limits(BURST_IDLE, MAX_AGE)
    }
}

impl SharedTypeSources {
    fn with_limits(idle: Duration, max_age: Duration) -> Self {
        Self {
            state: Mutex::default(),
            changed: Condvar::new(),
            idle,
            max_age,
        }
    }

    fn lock(&self) -> MutexGuard<'_, Generation> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn acquire(&self) -> ContentMapperTypeSources {
        let mut state = self.lock();
        let now = Instant::now();
        let burst_ended = state
            .idle_since
            .is_some_and(|since| now.saturating_duration_since(since) >= self.idle);
        state.active += 1;
        state.idle_since = None;
        if !burst_ended
            && let Some((sources, created)) = &state.current
            && now.saturating_duration_since(*created) < self.max_age
        {
            return sources.clone();
        }
        let sources = ContentMapperTypeSources::default();
        state.current = Some((sources.clone(), now));
        sources
    }

    pub(super) fn release(&self) {
        let mut state = self.lock();
        state.active = state.active.saturating_sub(1);
        if state.active == 0 {
            state.idle_since = Some(Instant::now());
            self.changed.notify_all();
        }
    }

    /// Drop the snapshot, and the files it holds, once the server has been
    /// idle for a burst gap. Returns after [`Self::stop`].
    pub(super) fn expire_idle(&self) {
        let mut state = self.lock();
        while !state.stopped {
            let left = match state.idle_since {
                Some(since) if state.current.is_some() => {
                    let left = self.idle.saturating_sub(since.elapsed());
                    if left.is_zero() {
                        state.current = None;
                        continue;
                    }
                    Some(left)
                }
                _ => None,
            };
            state = match left {
                Some(left) => {
                    self.changed
                        .wait_timeout(state, left)
                        .unwrap_or_else(PoisonError::into_inner)
                        .0
                }
                None => self
                    .changed
                    .wait(state)
                    .unwrap_or_else(PoisonError::into_inner),
            };
        }
    }

    pub(super) fn stop(&self) {
        self.lock().stopped = true;
        self.changed.notify_all();
    }
}

/// Stops [`SharedTypeSources::expire_idle`] when the serving scope ends, on
/// every return path, before the scope joins its threads.
pub(super) struct StopExpiry<'a>(pub(super) &'a SharedTypeSources);

impl Drop for StopExpiry<'_> {
    fn drop(&mut self) {
        self.0.stop();
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::SharedTypeSources;

    const LONG: Duration = Duration::from_secs(3600);

    #[test]
    fn one_burst_shares_a_snapshot_across_short_idle_gaps() {
        let shared = SharedTypeSources::with_limits(LONG, LONG);
        let first = shared.acquire();
        assert_eq!(shared.acquire(), first);
        shared.release();
        shared.release();
        // The host paused between two transforms of the same burst.
        assert_eq!(shared.acquire(), first);
        shared.release();
    }

    #[test]
    fn a_burst_ends_after_its_idle_gap_or_its_age() {
        let idle = SharedTypeSources::with_limits(Duration::ZERO, LONG);
        let first = idle.acquire();
        assert_eq!(idle.acquire(), first);
        idle.release();
        idle.release();
        assert_ne!(idle.acquire(), first);
        idle.release();

        let busy = SharedTypeSources::with_limits(LONG, Duration::ZERO);
        let first = busy.acquire();
        assert_ne!(busy.acquire(), first);
        busy.release();
        busy.release();
    }

    #[test]
    fn an_idle_server_drops_its_snapshot() {
        let shared = SharedTypeSources::with_limits(Duration::from_millis(10), LONG);
        drop(shared.acquire());
        shared.release();
        std::thread::scope(|scope| {
            let _stop = super::StopExpiry(&shared);
            scope.spawn(|| shared.expire_idle());
            let deadline = Instant::now() + Duration::from_secs(5);
            while shared.lock().current.is_some() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(5));
            }
            assert!(shared.lock().current.is_none());
        });
    }
}
