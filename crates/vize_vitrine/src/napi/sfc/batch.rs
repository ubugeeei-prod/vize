use glob::glob;
use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use vize_carton::{FxHashMap, hash::hash_str};

use super::types::{BatchCompileOptionsNapi, BatchCompileResultNapi};

/// Aggregate counters for the native batch stats surface.
///
/// `compileSfcBatch` reports totals instead of per-file code, so each compiled
/// job can represent many logical input files. These counters must therefore be
/// updated by the repeat count, not by the number of physical compile jobs.
#[derive(Default)]
struct BatchStats {
    success: usize,
    failed: usize,
    input_bytes: usize,
    output_bytes: usize,
}

impl BatchStats {
    fn failed() -> Self {
        Self {
            failed: 1,
            ..Default::default()
        }
    }

    fn add(mut self, other: Self) -> Self {
        self.success += other.success;
        self.failed += other.failed;
        self.input_bytes += other.input_bytes;
        self.output_bytes += other.output_bytes;
        self
    }
}

/// Fingerprint used to collapse repeated batch inputs before compiling.
///
/// The native aggregate API does not return code for each file. It can group
/// repeated sources, compile one representative, and multiply the resulting
/// counters by `repeats`. The key includes only fields that can affect those
/// counters:
///
/// - source hash and length identify the repeated SFC body without storing a
///   second owned copy of the source in the map key.
/// - parent hash and length prevent grouping across directories, where
///   relative type imports inside `<script setup>` can resolve differently.
/// - component name length preserves output byte counts for the generated
///   `__name` field, while `should_cache_batch_compile` filters out cases where
///   the actual name can alter compilation through self-component resolution.
/// - option bits separate SSR, vapor, TypeScript, and parser-quirk modes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct BatchCompileKey {
    source_hash: u64,
    source_len: usize,
    parent_hash: u64,
    parent_len: usize,
    component_name_len: usize,
    options: u8,
}

/// One physical compile job, possibly standing in for many logical files.
///
/// `source` is kept once per unique key. `input_bytes` is the sum for all
/// grouped files so the API still reports the bytes the caller supplied, not
/// just the bytes actually compiled.
struct BatchCompileJob {
    path: PathBuf,
    source: String,
    repeats: usize,
    input_bytes: usize,
}

impl BatchCompileJob {
    fn single(path: PathBuf, source: String) -> Self {
        let input_bytes = source.len();
        Self {
            path,
            source,
            repeats: 1,
            input_bytes,
        }
    }
}

/// Packs options that can change aggregate compile results into the cache key.
///
/// This mirrors the CLI stats cache. N-API options that are not consumed by this
/// aggregate function are intentionally absent so they do not fragment groups.
fn batch_options_bits(ssr: bool, vapor: bool, is_ts: bool, vue_parser_quirks: bool) -> u8 {
    u8::from(ssr)
        | (u8::from(vapor) << 1)
        | (u8::from(is_ts) << 2)
        | (u8::from(vue_parser_quirks) << 3)
}

/// Returns whether a repeated source body is safe to group for aggregate stats.
///
/// Different filenames normally only change fixed-width scope IDs or the length
/// of `__name`, both of which the key accounts for. A source that mentions its
/// own component name is different: self-component resolution can change helper
/// usage and code shape depending on the representative filename. Such files are
/// left ungrouped.
fn should_cache_batch_compile(source: &str, component_name: &str) -> bool {
    if component_name.is_empty() {
        return true;
    }

    !source.contains(component_name)
        && !source.contains(component_name_to_kebab_case(component_name).as_str())
}

/// Converts a PascalCase filename stem to the kebab-case spelling used in templates.
///
/// This is only a cheap guard for the common ASCII component-name case. It does
/// not need to be a complete Vue name canonicalizer because exact stem matching
/// is checked separately.
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

/// Returns parent-directory fingerprint parts for the batch grouping key.
///
/// The N-API batch path passes the full file path into SFC compilation. That
/// lets script setup type imports resolve relative to the file's directory, so
/// two identical source strings in different directories cannot always share a
/// compile result. Grouping by parent keeps the optimization useful for
/// generated corpora while preserving that path-sensitive behavior.
fn parent_cache_parts(path: &Path) -> (u64, usize) {
    let Some(parent) = path.parent() else {
        return (hash_str(""), 0);
    };
    let parent = parent.to_string_lossy();
    (hash_str(parent.as_ref()), parent.len())
}

/// Compiles a glob of Vue SFCs and returns aggregate stats for the native API.
///
/// This stats-only surface is intentionally optimized differently from
/// `compileSfcBatchWithResults`: because no per-file code crosses the JS/native
/// boundary, repeated SFC bodies are grouped before parallel compilation. The
/// representative compile produces one output length, and the counters are
/// multiplied by the number of files that shared the same safe key.
#[napi(js_name = "compileSfcBatch")]
pub fn compile_sfc_batch(
    pattern: String,
    options: Option<BatchCompileOptionsNapi>,
) -> Result<BatchCompileResultNapi> {
    use vize_atelier_sfc::{
        ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, StyleCompileOptions,
        TemplateCompileOptions, compile_sfc as sfc_compile,
        compile_sfc_with_vue_parser_quirks as sfc_compile_with_vue_parser_quirks,
        parse_sfc as sfc_parse,
    };

    let opts = options.unwrap_or_default();
    if let Some(threads) = opts.threads {
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(threads as usize)
            .build_global();
    }

    let files: Vec<_> = glob(&pattern)
        .map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Invalid glob pattern: {}", e),
            )
        })?
        .filter_map(|entry| entry.ok())
        .filter(|path| path.extension().is_some_and(|ext| ext == "vue"))
        .collect();

    if files.is_empty() {
        return Err(Error::new(
            Status::GenericFailure,
            "No .vue files found matching the pattern",
        ));
    }

    let ssr = opts.ssr.unwrap_or(false);
    let vapor = opts.vapor.unwrap_or(false);
    let is_ts = opts.is_ts.unwrap_or(false);
    let vue_parser_quirks = opts.vue_parser_quirks.unwrap_or(false);
    let start = Instant::now();
    let option_bits = batch_options_bits(ssr, vapor, is_ts, vue_parser_quirks);
    let read_inputs: Vec<_> = files
        .par_iter()
        .map(|path| match fs::read_to_string(path) {
            Ok(source) => Ok((path.clone(), source)),
            Err(_) => Err(()),
        })
        .collect();

    let mut stats = BatchStats::default();
    let mut grouped = FxHashMap::<BatchCompileKey, usize>::default();
    let mut jobs = Vec::<BatchCompileJob>::new();

    for input in read_inputs {
        let (path, source) = match input {
            Ok(input) => input,
            Err(()) => {
                stats = stats.add(BatchStats::failed());
                continue;
            }
        };

        let component_name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if should_cache_batch_compile(&source, component_name) {
            let (parent_hash, parent_len) = parent_cache_parts(&path);
            let key = BatchCompileKey {
                source_hash: hash_str(&source),
                source_len: source.len(),
                parent_hash,
                parent_len,
                component_name_len: component_name.len(),
                options: option_bits,
            };
            if let Some(index) = grouped.get(&key).copied() {
                let job = &mut jobs[index];
                job.repeats += 1;
                job.input_bytes += source.len();
                continue;
            }

            grouped.insert(key, jobs.len());
        }

        jobs.push(BatchCompileJob::single(path, source));
    }

    let compile_stats = jobs
        .par_iter()
        .map(|job| {
            let source_len = job.input_bytes;
            let filename: vize_carton::CompactString = job.path.to_string_lossy().as_ref().into();
            let parse_opts = SfcParseOptions {
                filename: filename.clone(),
                ..Default::default()
            };
            let descriptor = match sfc_parse(&job.source, parse_opts) {
                Ok(d) => d,
                Err(_) => {
                    return BatchStats {
                        failed: job.repeats,
                        input_bytes: source_len,
                        ..Default::default()
                    };
                }
            };
            let has_scoped = descriptor.styles.iter().any(|s| s.scoped);
            let compile_opts = SfcCompileOptions {
                parse: SfcParseOptions {
                    filename: filename.clone(),
                    ..Default::default()
                },
                script: ScriptCompileOptions {
                    id: Some(filename.clone()),
                    is_ts,
                    ..Default::default()
                },
                template: TemplateCompileOptions {
                    id: Some(filename.clone()),
                    scoped: has_scoped,
                    ssr,
                    is_ts,
                    ..Default::default()
                },
                style: StyleCompileOptions {
                    id: filename,
                    scoped: has_scoped,
                    ..Default::default()
                },
                vapor,
                scope_id: None,
            };

            let compile_result = if vue_parser_quirks {
                sfc_compile_with_vue_parser_quirks(&descriptor, compile_opts)
            } else {
                sfc_compile(&descriptor, compile_opts)
            };

            match compile_result {
                Ok(result) => BatchStats {
                    success: job.repeats,
                    input_bytes: source_len,
                    output_bytes: result.code.len() * job.repeats,
                    failed: 0,
                },
                Err(_) => BatchStats {
                    failed: job.repeats,
                    input_bytes: source_len,
                    ..Default::default()
                },
            }
        })
        .reduce(BatchStats::default, BatchStats::add);
    stats = stats.add(compile_stats);

    Ok(BatchCompileResultNapi {
        success: stats.success as u32,
        failed: stats.failed as u32,
        input_bytes: stats.input_bytes as u32,
        output_bytes: stats.output_bytes as u32,
        time_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}
