//! Content-addressed cache for `vize build --format stats` compiles.

use std::sync::Mutex;

use vize_carton::{FxHashMap, String};

use crate::commands::build::config::ErrorPhase;

/// Fingerprint for sharing stats-only SFC compiles across repeated file bodies.
///
/// The stats formatter never writes generated code, so it only needs stable
/// aggregate properties: output byte count and block sizes. Keeping a full copy
/// of every source in the key would bring back the memory pressure this path is
/// trying to avoid, so the key uses the existing fast hash plus source length.
///
/// `component_name_len` is part of the key because generated `__name` text
/// affects byte counts even when the actual component name is otherwise
/// irrelevant to stats. The actual name is deliberately not stored in the key;
/// `should_cache_stats_compile` rejects cases where the source can observe the
/// specific component name through self-component resolution.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct StatsCompileCacheKey {
    /// Fast content fingerprint used to group identical generated benchmark bodies.
    pub(super) source_hash: u64,
    /// Guards the fingerprint and keeps same-hash, different-length sources apart.
    pub(super) source_len: usize,
    /// Captures byte-size changes from the filename-derived `__name` field.
    pub(super) component_name_len: usize,
    /// Compact representation of output-affecting CLI compile options.
    pub(super) settings: u8,
}

/// Cached result of a stats-only compile.
///
/// Success entries hold only the values needed to reproduce aggregate stats and
/// file-profile metadata. Failure entries are cached as well so repeated invalid
/// inputs do not re-run the parser/compiler only to report the same phase.
#[derive(Clone)]
pub(super) enum StatsCompileCacheEntry {
    Success {
        output_bytes: usize,
        template_size: usize,
        script_size: usize,
        style_count: usize,
    },
    Failure {
        phase: ErrorPhase,
        message: String,
    },
}

/// Shared cache for one `vize build --format stats` invocation.
///
/// A single mutex is enough here because the expensive work is parse/compile,
/// not the lookup. If two worker threads miss the same key at the same time they
/// may both compile once; the `entry(...).or_insert_with(...)` write keeps the
/// cache deterministic, and correctness does not depend on suppressing that
/// benign race.
#[derive(Default)]
pub(super) struct StatsCompileCache {
    pub(super) entries: Mutex<FxHashMap<StatsCompileCacheKey, StatsCompileCacheEntry>>,
}

/// Returns whether a source can reuse another file's stats-only compile result.
///
/// Filename-derived output is mostly byte-count stable: generated scope IDs are
/// fixed-width hashes, and different component names only matter by length.
/// Self-component resolution is the exception. If the template mentions its own
/// component name, changing the filename can change whether that tag is treated
/// as a component, which can alter helper usage and generated code shape. Those
/// cases are compiled normally.
pub(super) fn should_cache_stats_compile(source: &str, component_name: &str) -> bool {
    if component_name.is_empty() {
        return true;
    }

    !source.contains(component_name)
        && !source.contains(component_name_to_kebab_case(component_name).as_str())
}

/// Converts a PascalCase filename stem to the kebab-case spelling Vue templates use.
///
/// This is a conservative guard for self-component detection, not a general Vue
/// name normalizer. Exact stem matching is checked separately, and non-ASCII
/// names fall back to that exact spelling path.
fn component_name_to_kebab_case(component_name: &str) -> String {
    let mut out = String::with_capacity(component_name.len());
    for (index, ch) in component_name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index != 0 {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}
