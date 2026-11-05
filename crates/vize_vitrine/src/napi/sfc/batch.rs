use super::codegen_options::sfc_codegen_options;
use glob::glob;
use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::time::Instant;
use vize_l0::FxHashMap;

use super::vue_version::resolve_sfc_vue_version;
use super::{
    batch_helpers::{
        BatchCompileCandidateKey, BatchCompileJob, BatchStats, batch_compile_candidate_key,
        batch_options_bits, should_cache_batch_compile,
    },
    experimentals::ExperimentalTemplateOptions,
    thread_pool::BatchThreadPool,
    types::{BatchCompileOptionsNapi, BatchCompileResultNapi},
};
use crate::template_syntax::resolve_template_syntax;
use crate::whitespace::resolve_whitespace;

#[cfg(test)]
mod stats_tests;

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
    let vue_version = resolve_sfc_vue_version(opts.vue_version.as_deref())?;
    BatchThreadPool::new(opts.threads)?
        .install(|| compile_sfc_batch_inner(pattern, opts, vue_version))
}

fn compile_sfc_batch_inner(
    pattern: String,
    opts: BatchCompileOptionsNapi,
    vue_version: vize_l0::config::VueVersion,
) -> Result<BatchCompileResultNapi> {
    use vize_atelier_sfc::{
        ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
        StyleCompileOptions, TemplateCompileOptions, compile_sfc_for_adapter_with_project,
        parse_sfc as sfc_parse,
    };
    let type_projects = vize::type_projects::ImportedTypeProjects::new(
        opts.type_resolution_fallback_tsconfig.as_deref(),
    );

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
    let custom_renderer = opts.custom_renderer.unwrap_or(false);
    let custom_elements = vize_atelier_core::options::CustomElementMatcher::from_patterns(
        crate::types::custom_element_patterns(opts.custom_elements.as_deref()),
    );
    let experimentals = ExperimentalTemplateOptions::from_batch(&opts);
    let template_syntax = resolve_template_syntax(opts.template_syntax.as_deref())
        .map_err(|message| Error::new(Status::InvalidArg, message))?;
    let whitespace = resolve_whitespace(opts.whitespace.as_deref())
        .map_err(|message| Error::new(Status::InvalidArg, message))?;
    let shape = super::batch_helpers::sfc_output_shape(
        opts.mode.as_deref(),
        opts.inline_template,
        opts.is_prod,
    );
    let script_output = if shape.inline_render {
        SfcScriptOutputMode::InlineTemplate
    } else {
        SfcScriptOutputMode::SeparateTemplate
    };
    let start = Instant::now();
    // This named guard must stay alive until the parallel reduce has joined.
    #[deny(let_underscore_drop)]
    let _type_resolution_batch = vize_atelier_sfc::begin_type_resolution_batch();
    let option_bits = batch_options_bits(
        ssr,
        vapor,
        is_ts,
        template_syntax,
        whitespace,
        shape.standalone,
        shape.inline_render,
        shape.is_prod,
        experimentals.bits(),
    );
    let read_inputs: Vec<_> = files
        .par_iter()
        .map(|path| match vize_carton::source_io::read_to_string(path) {
            Ok(source) => Ok((path.clone(), source)),
            Err(_) => Err(()),
        })
        .collect();

    let mut stats = BatchStats::default();
    let mut grouped = FxHashMap::<BatchCompileCandidateKey, usize>::default();
    let mut jobs = Vec::<BatchCompileJob>::new();

    for (input_index, input) in read_inputs.into_iter().enumerate() {
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
            let key = batch_compile_candidate_key(&path, &source, component_name, option_bits);
            if let Some(job) = grouped.get(&key).and_then(|&index| jobs.get_mut(index)) {
                job.repeats += 1;
                job.input_bytes += source.len();
                job.remaining.push(input_index);
                continue;
            }

            grouped.insert(key, jobs.len());
        }

        jobs.push(BatchCompileJob::single(path, source));
    }

    let compile_one = |path: &std::path::Path, source: &str| {
        let source_len = source.len();
        let filename: vize_l0::CompactString = path.to_string_lossy().as_ref().into();
        let parse_opts = SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        };
        let descriptor = match sfc_parse(source, parse_opts) {
            Ok(d) => d,
            Err(_) => {
                return (
                    BatchStats {
                        failed: 1,
                        input_bytes: source_len,
                        ..Default::default()
                    },
                    true,
                );
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
                inline_template: shape.standalone,
                is_ts,
                ..Default::default()
            },
            template: TemplateCompileOptions {
                id: Some(filename.clone()),
                scoped: has_scoped,
                ssr,
                is_ts,
                is_prod: shape.is_prod,
                custom_renderer,
                dialect: vue_version,
                compiler_options: {
                    let mut dom_options = experimentals.dom_options();
                    if let Some(value) = opts.template_cache_handlers {
                        dom_options.cache_handlers = value;
                    }
                    if let Some(value) = opts.template_comments {
                        dom_options.comments = value;
                    }
                    if let Some(value) = opts.template_hoist_static {
                        dom_options.hoist_static = value;
                    }
                    if let Some(value) = opts.template_prefix_identifiers {
                        dom_options.prefix_identifiers = value;
                    }
                    Some(dom_options)
                },
                ..Default::default()
            },
            style: StyleCompileOptions {
                id: filename,
                scoped: has_scoped,
                trim: opts.style_trim.unwrap_or(false),
                ..Default::default()
            },
            vapor,
            scope_id: None,
        };

        let compile_result =
            type_projects
                .for_descriptor(&descriptor, path)
                .and_then(|type_project| {
                    let contextual = type_project.cache_identity(path).is_some();
                    whitespace.apply(|| {
                        compile_sfc_for_adapter_with_project(
                            &descriptor,
                            compile_opts,
                            template_syntax,
                            custom_elements.clone(),
                            sfc_codegen_options(
                                false,
                                opts.runtime_module_name.as_deref(),
                                opts.runtime_global_name.as_deref(),
                            ),
                            script_output,
                            experimentals.sfc_options(),
                            opts.nuxt_page_meta.unwrap_or(false),
                            type_project.as_deref(),
                        )
                        .map(|result| (type_project.track(result), contextual))
                    })
                });

        match compile_result {
            Ok((result, contextual)) => (
                BatchStats {
                    success: 1,
                    input_bytes: source_len,
                    output_bytes: result.code.len(),
                    failed: 0,
                },
                contextual,
            ),
            Err(_) => (
                BatchStats {
                    failed: 1,
                    input_bytes: source_len,
                    ..Default::default()
                },
                true,
            ),
        }
    };

    // Candidate buckets are only source-equivalence hints. The representative
    // compile records its real AST facts; contextual and failed inputs always
    // execute again with each complete importer and its selected authority.
    let compile_stats = jobs
        .par_iter()
        .map(|job| {
            let (first, contextual) = compile_one(&job.path, &job.source);
            if contextual {
                let remaining = job
                    .remaining
                    .par_iter()
                    .map(|&index| compile_one(&files[index], &job.source).0)
                    .reduce(BatchStats::default, BatchStats::add);
                first.add(remaining)
            } else {
                BatchStats {
                    success: first.success * job.repeats,
                    failed: first.failed * job.repeats,
                    input_bytes: job.input_bytes,
                    output_bytes: first.output_bytes * job.repeats,
                }
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
