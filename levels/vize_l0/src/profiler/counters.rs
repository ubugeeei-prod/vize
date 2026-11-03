//! Counter and filesystem-syscall recording on the profiler.
//!
//! Split out of `core.rs` to keep that file focused on the span/timer core;
//! these are `impl Profiler` methods like the rest of the recording surface.

use super::allocation::pause_allocation_tracking;
use super::core::Profiler;

impl Profiler {
    /// Record a non-duration counter sample.
    pub fn record_counter(&self, name: &'static str, value: u64) {
        if !self.is_enabled() {
            return;
        }

        self.record_counter_enabled(name, value);
    }

    /// Record a counter after the caller has already checked profiling.
    #[doc(hidden)]
    pub fn record_counter_enabled(&self, name: &'static str, value: u64) {
        let _allocation_tracking = pause_allocation_tracking();
        let mut counters = self.counters_write(Self::shard_index(name));
        counters.entry(name).or_default().record(value);
    }

    /// Record a successful `std::fs::read_to_string` call.
    pub fn record_fs_read_to_string(&self, bytes: usize) {
        if !self.is_enabled() {
            return;
        }

        self.record_counter_enabled("io.read.calls", 1);
        self.record_counter_enabled("io.read.bytes", bytes as u64);
        self.record_counter_enabled("syscall.fs.read_to_string.calls", 1);
    }

    /// Record a failed `std::fs::read_to_string` call.
    pub fn record_fs_read_to_string_failure(&self) {
        if !self.is_enabled() {
            return;
        }

        self.record_counter_enabled("io.read.calls", 1);
        self.record_counter_enabled("io.read.failures", 1);
        self.record_counter_enabled("syscall.fs.read_to_string.calls", 1);
        self.record_counter_enabled("syscall.fs.read_to_string.failures", 1);
    }

    /// Record a successful `std::fs::write` call.
    pub fn record_fs_write(&self, bytes: usize) {
        if !self.is_enabled() {
            return;
        }

        self.record_counter_enabled("io.write.calls", 1);
        self.record_counter_enabled("io.write.attempted_bytes", bytes as u64);
        self.record_counter_enabled("io.write.bytes", bytes as u64);
        self.record_counter_enabled("syscall.fs.write.calls", 1);
    }

    /// Record a failed `std::fs::write` call.
    pub fn record_fs_write_failure(&self, bytes: usize) {
        if !self.is_enabled() {
            return;
        }

        self.record_counter_enabled("io.write.calls", 1);
        self.record_counter_enabled("io.write.attempted_bytes", bytes as u64);
        self.record_counter_enabled("io.write.failures", 1);
        self.record_counter_enabled("syscall.fs.write.calls", 1);
        self.record_counter_enabled("syscall.fs.write.failures", 1);
    }

    /// Record a successful `std::fs::create_dir_all` call.
    pub fn record_fs_create_dir_all(&self) {
        if self.is_enabled() {
            self.record_counter_enabled("syscall.fs.create_dir_all.calls", 1);
        }
    }

    /// Record a failed `std::fs::create_dir_all` call.
    pub fn record_fs_create_dir_all_failure(&self) {
        if !self.is_enabled() {
            return;
        }

        self.record_counter_enabled("syscall.fs.create_dir_all.calls", 1);
        self.record_counter_enabled("syscall.fs.create_dir_all.failures", 1);
    }

    /// Record a successful `std::fs::remove_dir_all` call.
    pub fn record_fs_remove_dir_all(&self) {
        if self.is_enabled() {
            self.record_counter_enabled("syscall.fs.remove_dir_all.calls", 1);
        }
    }

    /// Record a failed `std::fs::remove_dir_all` call.
    pub fn record_fs_remove_dir_all_failure(&self) {
        if !self.is_enabled() {
            return;
        }

        self.record_counter_enabled("syscall.fs.remove_dir_all.calls", 1);
        self.record_counter_enabled("syscall.fs.remove_dir_all.failures", 1);
    }
}
