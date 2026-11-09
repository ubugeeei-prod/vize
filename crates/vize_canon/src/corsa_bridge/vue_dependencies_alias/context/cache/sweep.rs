//! A server-side polling watcher for LSP owners whose client cannot watch.
//!
//! Without a client watcher every request validated every captured disk input
//! of every project member before answering: O(project) syscalls per hover,
//! and one 9P round trip each on a WSL2 Windows mount. A background sweep does
//! that validation off the request path instead. A sweep that found every
//! input unchanged vouches for them for one window from its start; requests
//! inside it skip disk validation as they do under a client watcher. Requests
//! ask for the next sweep once a third of the window has passed, so an active
//! session stays vouched and an idle one costs nothing. A changed input, or no
//! clean sweep within the window, leaves requests on their full validation,
//! which retires the stale revisions exactly as before.

use std::sync::Arc;
use std::time::{Duration, Instant};

use super::SessionCache;
use crate::package_route::stamp::{InputStamp, InputStampValidation};

/// The shortest vouch: how long an external edit can stay unobserved.
const MIN_VOUCH_WINDOW: Duration = Duration::from_secs(1);
/// A slow file system (a WSL2 Windows mount) widens the window to three sweeps.
const SWEEPS_PER_WINDOW: u32 = 3;
/// After a stale sweep, requests retry at most this often.
const SWEEP_SPACING: Duration = Duration::from_millis(100);

#[derive(Default)]
pub(in crate::corsa_bridge) struct DiskSweep {
    enabled: bool,
    running: bool,
    last_started: Option<Instant>,
    /// Start of the last sweep that found every captured input unchanged.
    clean_since: Option<Instant>,
    /// The last sweep's duration times `SWEEPS_PER_WINDOW`.
    window: Duration,
}

/// Captured disk inputs at one instant, validated without the cache lock.
pub(in crate::corsa_bridge) struct SweepInputs {
    started: Instant,
    stamps: Vec<Arc<[InputStamp]>>,
}

impl SweepInputs {
    /// Whether every input still matches its capture; each path is observed once.
    pub(in crate::corsa_bridge) fn unchanged(&self) -> bool {
        let mut observed = InputStampValidation::default();
        self.stamps
            .iter()
            .flat_map(|stamps| stamps.iter())
            .all(|stamp| observed.is_current(stamp, None))
    }
}

impl SessionCache {
    pub(in crate::corsa_bridge) fn enable_disk_sweep(&mut self) {
        self.sweep.enabled = true;
    }

    fn vouch_window(&self) -> Duration {
        self.sweep.window.max(MIN_VOUCH_WINDOW)
    }

    fn vouch_age(&self, now: Instant) -> Option<Duration> {
        self.sweep
            .clean_since
            .map(|start| now.saturating_duration_since(start))
    }

    /// Whether disk inputs need no per-request validation at `now`.
    pub(super) fn disk_inputs_trusted_at(&self, now: Instant) -> bool {
        self.trust_watched_disk_inputs
            || (self.sweep.enabled
                && self
                    .vouch_age(now)
                    .is_some_and(|age| age < self.vouch_window()))
    }

    pub(super) fn disk_inputs_trusted(&self) -> bool {
        self.disk_inputs_trusted_at(Instant::now())
    }

    /// Every captured input when a sweep is due at `now`.
    pub(in crate::corsa_bridge) fn begin_disk_sweep(
        &mut self,
        now: Instant,
    ) -> Option<SweepInputs> {
        let refresh = self.vouch_window() / SWEEPS_PER_WINDOW;
        let sweep = &self.sweep;
        if !sweep.enabled
            || self.trust_watched_disk_inputs
            || sweep.running
            || sweep
                .last_started
                .is_some_and(|last| now.saturating_duration_since(last) < SWEEP_SPACING)
            || self.vouch_age(now).is_some_and(|age| age < refresh)
        {
            return None;
        }
        self.sweep.running = true;
        self.sweep.last_started = Some(now);
        Some(SweepInputs {
            started: now,
            stamps: self.captured_disk_inputs(),
        })
    }

    pub(in crate::corsa_bridge) fn finish_disk_sweep(
        &mut self,
        inputs: &SweepInputs,
        unchanged: bool,
        now: Instant,
    ) {
        self.sweep.running = false;
        self.sweep.window = now.saturating_duration_since(inputs.started) * SWEEPS_PER_WINDOW;
        self.sweep.clean_since = unchanged.then_some(inputs.started);
    }

    /// Inputs that later capture, adopt or extend a revision are captured
    /// fresh, so the cached ones are all a sweep has to observe. Shared stamp
    /// lists keep the cache lock to a few pointer copies.
    fn captured_disk_inputs(&self) -> Vec<Arc<[InputStamp]>> {
        let members = self
            .project_members
            .values()
            .flat_map(|members| members.values())
            .map(|member| &member.fingerprint);
        self.slots
            .values()
            .map(|cached| &cached.fingerprint)
            .chain(members)
            .chain(self.prebuilt_fingerprint())
            .map(super::ContextFingerprint::disk_stamps)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_clean_sweep_vouches_and_only_within_its_window() {
        let mut cache = SessionCache::default();
        let now = Instant::now();
        assert!(cache.begin_disk_sweep(now).is_none(), "disabled by default");
        cache.enable_disk_sweep();
        assert!(!cache.disk_inputs_trusted_at(now));
        let inputs = cache.begin_disk_sweep(now).unwrap();
        assert!(cache.begin_disk_sweep(now).is_none(), "one sweep at a time");
        cache.finish_disk_sweep(&inputs, false, now);
        assert!(
            !cache.disk_inputs_trusted_at(now),
            "a stale sweep never vouches"
        );
        assert!(
            cache.begin_disk_sweep(now + SWEEP_SPACING / 2).is_none(),
            "stale retries are spaced"
        );
        let later = now + SWEEP_SPACING;
        let inputs = cache.begin_disk_sweep(later).unwrap();
        cache.finish_disk_sweep(&inputs, true, later);
        assert!(cache.disk_inputs_trusted_at(later + MIN_VOUCH_WINDOW / 2));
        assert!(!cache.disk_inputs_trusted_at(later + MIN_VOUCH_WINDOW));
        let refresh = MIN_VOUCH_WINDOW / SWEEPS_PER_WINDOW;
        assert!(
            cache.begin_disk_sweep(later + refresh / 2).is_none(),
            "a fresh vouch needs no sweep"
        );
        assert!(cache.begin_disk_sweep(later + refresh).is_some());
    }

    #[test]
    fn a_slow_sweep_widens_the_window_to_three_sweeps() {
        let mut cache = SessionCache::default();
        cache.enable_disk_sweep();
        let now = Instant::now();
        let inputs = cache.begin_disk_sweep(now).unwrap();
        let slow = Duration::from_secs(3);
        cache.finish_disk_sweep(&inputs, true, now + slow);
        assert!(cache.disk_inputs_trusted_at(now + slow * 2));
        assert!(!cache.disk_inputs_trusted_at(now + slow * 3));
    }

    #[test]
    fn a_changed_input_fails_the_sweep() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("Child.vue");
        std::fs::write(&path, "<template>a</template>").unwrap();
        let inputs = SweepInputs {
            started: Instant::now(),
            stamps: vec![Arc::from([InputStamp::capture(&path)])],
        };
        assert!(inputs.unchanged());
        std::fs::write(&path, "<template>b</template>").unwrap();
        assert!(!inputs.unchanged());
    }
}
