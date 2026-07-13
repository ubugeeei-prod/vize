use glob::glob;
use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use rayon::prelude::*;
use std::{fs, time::Instant};
use vize_atlas::Compilation;
use vize_carton::{FxHashMap, hash::hash_str};
use vize_relief::VueDialectInput;

mod group;

use super::{
    experimentals::ExperimentalTemplateOptions,
    thread_pool::BatchThreadPool,
    types::{BatchCompileOptionsNapi, BatchCompileResultNapi},
};
use crate::{artifact_graph::resolve_vue_version, template_syntax::resolve_template_syntax};
use group::{
    BatchCompileJob, BatchCompileKey, BatchStats, batch_options_bits, parent_cache_parts,
    should_cache_batch_compile,
};

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
    let opts = options.unwrap_or_default();
    BatchThreadPool::new(opts.threads)?.install(|| compile_sfc_batch_inner(pattern, opts))
}

fn compile_sfc_batch_inner(
    pattern: String,
    opts: BatchCompileOptionsNapi,
) -> Result<BatchCompileResultNapi> {
    use vize_atelier_sfc::{
        ScriptCompileOptions, SfcCompileOptions, SfcCompileProduct, SfcCompileRequest,
        SfcCompileSettings, SfcParseOptions, StyleCompileOptions, TemplateCompileOptions,
    };

    let dialect = resolve_vue_version(opts.vue_version.as_deref())
        .map_err(|message| Error::new(Status::InvalidArg, message))?;
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
    let experimentals = ExperimentalTemplateOptions::from_batch(&opts);
    let template_syntax = resolve_template_syntax(opts.template_syntax.as_deref())
        .map_err(|message| Error::new(Status::InvalidArg, message))?;
    let standalone = opts.mode.as_deref() == Some("function");
    let start = Instant::now();
    // This named guard must stay alive until the parallel reduce has joined.
    #[deny(let_underscore_drop)]
    let _type_resolution_batch = vize_atelier_sfc::begin_type_resolution_batch();
    let option_bits = batch_options_bits(
        ssr,
        vapor,
        is_ts,
        template_syntax,
        standalone,
        experimentals.bits(),
    );
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
    let mut compilation = Compilation::new();
    crate::artifact_graph::register_sfc_compile_providers(&mut compilation).map_err(|error| {
        Error::new(
            Status::GenericFailure,
            format!("Atlas setup failed: {error}"),
        )
    })?;
    let mut settings = SfcCompileSettings::default();
    let mut source_ids = Vec::with_capacity(jobs.len());
    for job in &jobs {
        let filename: vize_carton::CompactString = job.path.to_string_lossy().as_ref().into();
        let source_id = compilation
            .add_source(filename.as_str(), job.source.as_str())
            .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))?;
        let compile_options = SfcCompileOptions {
            parse: SfcParseOptions {
                filename: filename.clone(),
                ..Default::default()
            },
            script: ScriptCompileOptions {
                id: Some(filename.clone()),
                inline_template: standalone,
                is_ts,
                ..Default::default()
            },
            template: TemplateCompileOptions {
                id: Some(filename.clone()),
                ssr,
                is_ts,
                dialect,
                compiler_options: Some(experimentals.dom_options()),
                ..Default::default()
            },
            style: StyleCompileOptions {
                id: filename,
                ..Default::default()
            },
            vapor,
            scope_id: None,
        };
        settings.insert(
            source_id,
            SfcCompileRequest::new(compile_options, template_syntax)
                .with_runtime_names(
                    opts.runtime_module_name.as_deref().unwrap_or("vue"),
                    opts.runtime_global_name.as_deref().unwrap_or("Vue"),
                )
                .with_inferred_scoped_from_descriptor(),
        );
        source_ids.push(source_id);
    }
    settings
        .install(&mut compilation)
        .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))?;
    compilation
        .set_input::<VueDialectInput>(dialect)
        .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))?;
    let snapshot = compilation.snapshot();
    let compile_stats = jobs
        .into_par_iter()
        .zip(source_ids.into_par_iter())
        .map(|(job, source_id)| {
            let source_len = job.input_bytes;
            let mut session = snapshot.query_session();
            let compile_result = session.query::<SfcCompileProduct>(source_id);

            match compile_result {
                Ok(outcome) => BatchStats {
                    success: job.repeats,
                    input_bytes: source_len,
                    output_bytes: outcome.value().code.len() * job.repeats,
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
