//! Stats-only builds: cache ordinary compiles, observe every requested dump.

use std::{
    path::PathBuf,
    sync::{Mutex, atomic::Ordering},
    time::Duration,
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::super::config::{CompileError, CompileStats, FileProfile};
use super::cache::StatsCompileCache;
use super::capture::compile_planned_file;
use super::compile_stats::compile_file_stats_with_cache;
use super::output::PlannedInput;
use super::settings::CompileFileSettings;

pub(super) struct StatsRun<'a> {
    pub planned_inputs: &'a [PlannedInput],
    pub files: &'a [PathBuf],
    pub settings: &'a CompileFileSettings,
    pub stats: &'a CompileStats,
    pub slow_threshold: Duration,
    pub slow_files: &'a Mutex<Vec<FileProfile>>,
    pub profiles: &'a Mutex<Vec<FileProfile>>,
    pub errors: &'a Mutex<Vec<CompileError>>,
    pub profile: bool,
}

impl StatsRun<'_> {
    pub(super) fn run(&self) {
        if self.settings.davinci.dump_dir.is_some() {
            // A cache hit has no executed compile to observe for this file.
            self.planned_inputs.par_iter().for_each(|input| {
                self.record_result(
                    compile_planned_file(input, self.settings, self.stats)
                        .map(|(output, profile)| (output.code.len(), profile)),
                );
            });
        } else {
            let cache = StatsCompileCache::default();
            self.files.par_iter().for_each(|path| {
                self.record_result(compile_file_stats_with_cache(
                    path,
                    self.settings,
                    self.stats,
                    &cache,
                ));
            });
        }
    }

    fn record_result(&self, result: Result<(usize, FileProfile), CompileError>) {
        match result {
            Ok((output_bytes, profile)) => {
                self.stats.success.fetch_add(1, Ordering::Relaxed);
                self.stats
                    .output_bytes
                    .fetch_add(output_bytes, Ordering::Relaxed);
                if profile.is_slow(self.slow_threshold)
                    && let Ok(mut slow) = self.slow_files.lock()
                {
                    slow.push(profile.clone());
                }
                if self.profile
                    && let Ok(mut profiles) = self.profiles.lock()
                {
                    profiles.push(profile);
                }
            }
            Err(error) => {
                self.stats.failed.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut errors) = self.errors.lock() {
                    errors.push(error);
                }
            }
        }
    }
}
