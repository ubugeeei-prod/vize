//! Shared `--profile-json` argument and export writer.
//!
//! Flattened into the build, lint, and check argument structs so every
//! pipeline-running subcommand can emit the machine-readable profile report
//! defined by `davinci-road/plan/profile-export.schema.json`.

use std::path::{Path, PathBuf};

use clap::Args;
use vize_s0::profiler::{ProfileExportBudget, ProfileExportOptions, global_profiler};

use crate::profile_support;

/// Shared machine-readable profile export flags.
#[derive(Args, Debug, Clone, Default)]
pub struct ProfileExportArgs {
    /// Write a machine-readable profiling report as JSON to this path (implies profile collection; schema: davinci-road/plan/profile-export.schema.json)
    #[arg(long, value_name = "PATH")]
    pub profile_json: Option<PathBuf>,
}

impl ProfileExportArgs {
    /// Whether a machine-readable export was requested.
    pub fn is_requested(&self) -> bool {
        self.profile_json.is_some()
    }

    /// Clear and enable the global profiler when this run collects spans,
    /// either for human-readable `--profile` output or for an export.
    pub fn begin(&self, profile: bool) {
        if profile || self.is_requested() {
            let profiler = global_profiler();
            profiler.clear();
            profiler.enable();
        }
    }

    /// Write the export, then release the profiler when only the export was
    /// holding it enabled (`--profile` owns that lifecycle otherwise).
    pub fn finish(&self, command: &'static str, profile: bool) {
        self.write_or_exit(command);
        if self.is_requested() && !profile {
            global_profiler().disable();
        }
    }

    /// Write the export for the still-populated global profiler, exiting the
    /// process with an error message if the file cannot be written.
    ///
    /// No-op when `--profile-json` was not passed. Does not disable the
    /// profiler; callers own that lifecycle.
    pub fn write_or_exit(&self, command: &'static str) {
        let Some(path) = self.profile_json.as_deref() else {
            return;
        };

        if let Err(error) = write_export(path, command) {
            eprintln!(
                "\x1b[31mError:\x1b[0m failed to write profile export {}: {}",
                path.display(),
                error
            );
            std::process::exit(1);
        }
    }
}

fn write_export(path: &Path, command: &'static str) -> std::io::Result<()> {
    let export = global_profiler().export_report(&ProfileExportOptions {
        command,
        allocation: profile_support::allocation_snapshot(),
        budget: ProfileExportBudget::default(),
    });
    std::fs::write(path, export.to_json().as_bytes())
}
