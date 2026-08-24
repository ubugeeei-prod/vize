//! Host-side writers for `davinci-opt`'s P2-13 outputs: the `--folio-dir`
//! page files and the `--timing-json` profile export.
//!
//! IO lives here and only here - the library half ([`FolioDump`], the timing
//! observer) is `no_std + alloc` and produces data, and this module turns it
//! into files. The timing export reuses `vize_s0::profiler`'s P0-11
//! export wholesale (`export_report` + `to_json`), so there is exactly one
//! producer of `profile-export.schema.json` documents in the tree.

use std::path::Path;

use vize_davinci::folio::dump::FolioDump;
use vize_davinci::folio::feed::SpolveroFeed;
use vize_s0::profiler::{ProfileExportBudget, ProfileExportOptions, global_profiler};
use vize_s0::{String, cstr};

/// The Spolvero feed's file name inside a `--folio-dir` directory (P2-18).
/// Pages end in `.folio`, so the name cannot collide with a page.
const FEED_FILE: &str = "spolvero.json";

/// Write every collected page into `dir`, creating it first, plus the
/// directory's Spolvero feed (`spolvero.json`, P2-18): the same pages as
/// one schema-versioned JSON document, `SpolveroFeed::of_dump` over the
/// same [`FolioDump`] - the directory and the feed cannot disagree.
///
/// The directory is created even when the dump is empty (a fully hash-gated
/// run over no-op passes), and the feed is written even then, with zero
/// pages - so "the gate emitted nothing" is observable in both artifacts
/// rather than indistinguishable from "the flag was ignored".
///
/// # Errors
///
/// Returns a formatted message naming the path that could not be created or
/// written.
pub fn write_dump(dir: &Path, dump: &FolioDump) -> Result<(), String> {
    std::fs::create_dir_all(dir)
        .map_err(|error| cstr!("--folio-dir: cannot create {}: {error}", dir.display()))?;
    for page in &dump.pages {
        let path = dir.join(page.name.as_str());
        std::fs::write(&path, page.text.as_bytes())
            .map_err(|error| cstr!("--folio-dir: cannot write {}: {error}", path.display()))?;
    }
    let feed_path = dir.join(FEED_FILE);
    let feed = SpolveroFeed::of_dump("davinci-opt", dump);
    std::fs::write(&feed_path, feed.to_json().as_bytes())
        .map_err(|error| cstr!("--folio-dir: cannot write {}: {error}", feed_path.display()))
}

/// Write the enabled global profiler's export - the timing JSON - to `path`.
///
/// `allocation` is `None` because `davinci-opt` does not install the
/// profiling allocator as its global allocator; the schema admits `null`
/// there explicitly.
///
/// # Errors
///
/// Returns a formatted message naming the path that could not be written.
pub fn write_timing(path: &Path) -> Result<(), String> {
    let export = global_profiler().export_report(&ProfileExportOptions {
        command: "davinci-opt",
        allocation: None,
        budget: ProfileExportBudget::default(),
    });
    std::fs::write(path, export.to_json().as_bytes())
        .map_err(|error| cstr!("--timing-json: cannot write {}: {error}", path.display()))
}
