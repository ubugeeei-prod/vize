use napi::{Result, Status};
use napi_derive::napi;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};
use vize_atlas::{Compilation, SourceId};
use vize_carton::cstr;
use vize_relief::VueDialectInput;

use super::{
    experimentals::ExperimentalTemplateOptions,
    thread_pool::BatchThreadPool,
    types::{
        BatchCompileOptionsNapi, BatchCompileResultWithFilesNapi, BatchFileInputNapi,
        BatchFileResultNapi, custom_blocks_to_napi, macro_artifacts_to_napi, style_blocks_to_napi,
    },
};
use crate::{artifact_graph::resolve_vue_version, template_syntax::resolve_template_syntax};

#[napi(js_name = "compileSfcBatchWithResults")]
pub fn compile_sfc_batch_with_results(
    files: Vec<BatchFileInputNapi>,
    options: Option<BatchCompileOptionsNapi>,
) -> Result<BatchCompileResultWithFilesNapi> {
    let opts = options.unwrap_or_default();
    BatchThreadPool::new(opts.threads)?
        .install(|| compile_sfc_batch_with_results_inner(files, opts))
}

fn compile_sfc_batch_with_results_inner(
    files: Vec<BatchFileInputNapi>,
    opts: BatchCompileOptionsNapi,
) -> Result<BatchCompileResultWithFilesNapi> {
    use vize_atelier_sfc::{
        ScriptCompileOptions, SfcCompileOptions, SfcCompileProduct, SfcCompileRequest,
        SfcCompileSettings, SfcDescriptorProduct, SfcParseOptions, StyleCompileOptions,
        TemplateCompileOptions,
    };

    let dialect = resolve_vue_version(opts.vue_version.as_deref())
        .map_err(|message| napi::Error::new(Status::InvalidArg, message))?;
    let total_count = files.len();
    let success_count = AtomicUsize::new(0);
    let ssr = opts.ssr.unwrap_or(false);
    let vapor = opts.vapor.unwrap_or(false);
    let source_map = opts.source_map.unwrap_or(false);
    let is_ts = opts.is_ts.unwrap_or(false);
    let custom_renderer = opts.custom_renderer.unwrap_or(false);
    let experimentals = ExperimentalTemplateOptions::from_batch(&opts);
    let template_syntax = resolve_template_syntax(opts.template_syntax.as_deref())
        .map_err(|message| napi::Error::new(Status::InvalidArg, message))?;
    let standalone = opts.mode.as_deref() == Some("function");
    // Heavy/optional payloads are opt-in so the default boundary stays lean:
    // `code`/`css` are always materialized eagerly (fairness), but the per-block
    // `styles` (which re-sends the CSS `css` already carries), custom blocks,
    // macro artifacts and content hashes are skipped unless the caller asks.
    let include_styles = opts.include_styles.unwrap_or(false);
    let include_custom_blocks = opts.include_custom_blocks.unwrap_or(false);
    let include_macro_artifacts = opts.include_macro_artifacts.unwrap_or(false);
    let include_hashes = opts.include_hashes.unwrap_or(false);
    let start = Instant::now();
    // Snapshot the filesystem for this batch: imported-type resolution treats
    // every file it stats as stable for the batch's duration, so the second and
    // later hits of a shared types barrel skip their revalidation syscalls.
    // The named guard must stay alive until the parallel collection has joined.
    #[deny(let_underscore_drop)]
    let _type_resolution_batch = vize_atelier_sfc::begin_type_resolution_batch();

    let mut compilation = Compilation::new();
    crate::artifact_graph::register_sfc_compile_providers(&mut compilation)
        .map_err(|error| napi::Error::new(Status::GenericFailure, error.to_string()))?;
    let mut settings = SfcCompileSettings::default();
    let prepared = files
        .into_par_iter()
        .map(|file| {
            let scope_id =
                vize_atelier_sfc::generate_bundler_scope_id(&file.path, None, false, None);
            (file, scope_id)
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|(file, scope_id)| {
            let filename_cs: vize_carton::CompactString = file.path.as_str().into();
            let template_compiler_options = Some(vize_atelier_dom::DomCompilerOptions {
                scope_id: Some(cstr!("data-v-{scope_id}")),
                source_map,
                ..experimentals.dom_options()
            });
            // `parse.filename` is left empty: compile falls back to `script.id`,
            // which carries the same value, so no per-file clone is needed.
            // `template.id` is never read by the template compiler.
            let compile_options = SfcCompileOptions {
                parse: SfcParseOptions {
                    filename: filename_cs.clone(),
                    ..Default::default()
                },
                script: ScriptCompileOptions {
                    id: Some(filename_cs.clone()),
                    inline_template: standalone,
                    is_ts,
                    ..Default::default()
                },
                template: TemplateCompileOptions {
                    scoped: false,
                    ssr,
                    is_ts,
                    custom_renderer,
                    dialect,
                    compiler_options: template_compiler_options,
                    ..Default::default()
                },
                style: StyleCompileOptions {
                    id: filename_cs,
                    scoped: false,
                    ..Default::default()
                },
                vapor,
                scope_id: Some(scope_id.clone()),
            };
            let source_id = compilation
                .add_source(file.path.as_str(), file.source.as_str())
                .map_err(|error| napi::Error::new(Status::GenericFailure, error.to_string()))?;
            settings.insert(
                source_id,
                SfcCompileRequest::new(compile_options, template_syntax)
                    .with_runtime_names(
                        opts.runtime_module_name.as_deref().unwrap_or("vue"),
                        opts.runtime_global_name.as_deref().unwrap_or("Vue"),
                    )
                    .with_inferred_scoped_from_descriptor(),
            );
            Ok((file, scope_id, source_id))
        })
        .collect::<Result<Vec<_>>>()?;
    settings
        .install(&mut compilation)
        .map_err(|error| napi::Error::new(Status::GenericFailure, error.to_string()))?;
    compilation
        .set_input::<VueDialectInput>(dialect)
        .map_err(|error| napi::Error::new(Status::GenericFailure, error.to_string()))?;
    let snapshot = compilation.snapshot();

    // Each worker gets an isolated query session over one immutable batch graph.
    let results: Vec<BatchFileResultNapi> = prepared
        .into_par_iter()
        .map(|(file, scope_id, source_id): (_, _, SourceId)| {
            let mut session = snapshot.query_session();
            let descriptor_artifact = match session.query::<SfcDescriptorProduct>(source_id) {
                Ok(outcome) => outcome.shared(),
                Err(error) => return failed_file(file.path, scope_id.into(), error.to_string()),
            };
            let descriptor = match descriptor_artifact.as_result() {
                Ok(descriptor) => descriptor,
                Err(error) => {
                    return failed_file(file.path, scope_id.into(), error.message.to_string());
                }
            };
            let (template_hash, style_hash, script_hash) = if include_hashes {
                (
                    descriptor.template_hash().map(Into::into),
                    descriptor.style_hash().map(Into::into),
                    descriptor.script_hash().map(Into::into),
                )
            } else {
                (None, None, None)
            };
            let styles = if include_styles {
                style_blocks_to_napi(&descriptor.styles)
            } else {
                Vec::new()
            };
            let custom_blocks = if include_custom_blocks {
                custom_blocks_to_napi(&descriptor.custom_blocks)
            } else {
                Vec::new()
            };
            let has_scoped = descriptor.styles.iter().any(|style| style.scoped);

            match session.query::<SfcCompileProduct>(source_id) {
                Ok(outcome) => {
                    let result = outcome.shared();
                    success_count.fetch_add(1, Ordering::Relaxed);
                    // Empty diagnostic vectors are the common case; skip the
                    // per-element map/collect (and the empty-array boundary
                    // crossing) when there is nothing to report.
                    let errors = if result.errors.is_empty() {
                        vec![]
                    } else {
                        result
                            .errors
                            .iter()
                            .map(|e| e.message.to_string())
                            .collect()
                    };
                    let warnings = if result.warnings.is_empty() {
                        vec![]
                    } else {
                        result
                            .warnings
                            .iter()
                            .map(|e| e.message.to_string())
                            .collect()
                    };
                    let macro_artifacts = if include_macro_artifacts {
                        macro_artifacts_to_napi(result.macro_artifacts.clone())
                    } else {
                        vec![]
                    };
                    BatchFileResultNapi {
                        path: file.path,
                        code: result.code.to_string(),
                        map: result.map.as_ref().map(ToString::to_string),
                        css: result.css.as_ref().map(ToString::to_string),
                        scope_id: scope_id.into(),
                        has_scoped,
                        errors,
                        warnings,
                        template_hash,
                        style_hash,
                        script_hash,
                        styles,
                        custom_blocks,
                        macro_artifacts,
                    }
                }
                Err(error) => BatchFileResultNapi {
                    path: file.path,
                    code: String::new(),
                    map: None,
                    css: None,
                    scope_id: scope_id.into(),
                    has_scoped,
                    errors: vec![error.to_string()],
                    warnings: vec![],
                    template_hash,
                    style_hash,
                    script_hash,
                    styles,
                    custom_blocks,
                    macro_artifacts: vec![],
                },
            }
        })
        .collect();

    let success = success_count.load(Ordering::Relaxed);

    Ok(BatchCompileResultWithFilesNapi {
        results,
        success_count: success as u32,
        failed_count: (total_count - success) as u32,
        time_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

fn failed_file(path: String, scope_id: String, error: String) -> BatchFileResultNapi {
    BatchFileResultNapi {
        path,
        code: String::new(),
        map: None,
        css: None,
        scope_id,
        has_scoped: false,
        errors: vec![error],
        warnings: vec![],
        template_hash: None,
        style_hash: None,
        script_hash: None,
        styles: vec![],
        custom_blocks: vec![],
        macro_artifacts: vec![],
    }
}
