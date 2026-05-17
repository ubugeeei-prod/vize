use glob::glob;
use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{fs, time::Instant};

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

    fn failed_with_input(input_bytes: usize) -> Self {
        Self {
            failed: 1,
            input_bytes,
            ..Default::default()
        }
    }

    fn success(input_bytes: usize, output_bytes: usize) -> Self {
        Self {
            success: 1,
            input_bytes,
            output_bytes,
            failed: 0,
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

#[napi(js_name = "compileSfcBatch")]
pub fn compile_sfc_batch(
    pattern: String,
    options: Option<BatchCompileOptionsNapi>,
) -> Result<BatchCompileResultNapi> {
    use vize_atelier_sfc::{
        ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, StyleCompileOptions,
        TemplateCompileOptions, compile_sfc as sfc_compile, parse_sfc as sfc_parse,
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
    let start = Instant::now();
    let stats = files
        .par_iter()
        .map(|path| {
            let source = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => return BatchStats::failed(),
            };
            let source_len = source.len();
            let filename: vize_carton::CompactString = path.to_string_lossy().as_ref().into();
            let parse_opts = SfcParseOptions {
                filename: filename.clone(),
                ..Default::default()
            };
            let descriptor = match sfc_parse(&source, parse_opts) {
                Ok(d) => d,
                Err(_) => return BatchStats::failed_with_input(source_len),
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

            match sfc_compile(&descriptor, compile_opts) {
                Ok(result) => BatchStats::success(source_len, result.code.len()),
                Err(_) => BatchStats::failed_with_input(source_len),
            }
        })
        .reduce(BatchStats::default, BatchStats::add);

    Ok(BatchCompileResultNapi {
        success: stats.success as u32,
        failed: stats.failed as u32,
        input_bytes: stats.input_bytes as u32,
        output_bytes: stats.output_bytes as u32,
        time_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}
