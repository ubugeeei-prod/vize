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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct BatchCompileKey {
    source_hash: u64,
    source_len: usize,
    parent_hash: u64,
    parent_len: usize,
    component_name_len: usize,
    options: u8,
}

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

fn batch_options_bits(ssr: bool, vapor: bool, is_ts: bool, vue_parser_quirks: bool) -> u8 {
    u8::from(ssr)
        | (u8::from(vapor) << 1)
        | (u8::from(is_ts) << 2)
        | (u8::from(vue_parser_quirks) << 3)
}

fn should_cache_batch_compile(source: &str, component_name: &str) -> bool {
    if component_name.is_empty() {
        return true;
    }

    !source.contains(component_name)
        && !source.contains(component_name_to_kebab_case(component_name).as_str())
}

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

fn parent_cache_parts(path: &Path) -> (u64, usize) {
    let Some(parent) = path.parent() else {
        return (hash_str(""), 0);
    };
    let parent = parent.to_string_lossy();
    (hash_str(parent.as_ref()), parent.len())
}

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
