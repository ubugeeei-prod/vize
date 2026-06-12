use napi::{Result, Status};
use napi_derive::napi;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};
use vize_carton::cstr;

use super::types::{
    BatchCompileOptionsNapi, BatchCompileResultWithFilesNapi, BatchFileInputNapi,
    BatchFileResultNapi, custom_blocks_to_napi, macro_artifacts_to_napi, style_blocks_to_napi,
};
use crate::template_syntax::resolve_template_syntax;

#[napi(js_name = "compileSfcBatchWithResults")]
pub fn compile_sfc_batch_with_results(
    files: Vec<BatchFileInputNapi>,
    options: Option<BatchCompileOptionsNapi>,
) -> Result<BatchCompileResultWithFilesNapi> {
    use vize_atelier_sfc::{
        ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, StyleCompileOptions,
        TemplateCompileOptions,
        compile_sfc_with_template_syntax as sfc_compile_with_template_syntax,
        parse_sfc as sfc_parse,
    };

    let opts = options.unwrap_or_default();
    if let Some(threads) = opts.threads {
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(threads as usize)
            .build_global();
    }

    let total_count = files.len();
    let success_count = AtomicUsize::new(0);
    let ssr = opts.ssr.unwrap_or(false);
    let vapor = opts.vapor.unwrap_or(false);
    let is_ts = opts.is_ts.unwrap_or(false);
    let custom_renderer = opts.custom_renderer.unwrap_or(false);
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
    vize_atelier_sfc::begin_type_resolution_batch();

    // Indexed parallel map keeps results in input order (deterministic) and
    // collects lock-free, replacing the previous contended `Mutex<Vec>`.
    let results: Vec<BatchFileResultNapi> = files
        .into_par_iter()
        .map(|file| {
            let scope_id =
                vize_atelier_sfc::generate_bundler_scope_id(&file.path, None, false, None);
            let filename_cs: vize_carton::CompactString = file.path.as_str().into();
            let descriptor = match sfc_parse(
                &file.source,
                SfcParseOptions {
                    filename: filename_cs.clone(),
                    ..Default::default()
                },
            ) {
                Ok(descriptor) => descriptor,
                Err(error) => {
                    return BatchFileResultNapi {
                        path: file.path,
                        code: String::new(),
                        css: None,
                        scope_id: scope_id.into(),
                        has_scoped: false,
                        errors: vec![error.message.into()],
                        warnings: vec![],
                        template_hash: None,
                        style_hash: None,
                        script_hash: None,
                        styles: vec![],
                        custom_blocks: vec![],
                        macro_artifacts: vec![],
                    };
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
                vec![]
            };
            let custom_blocks = if include_custom_blocks {
                custom_blocks_to_napi(&descriptor.custom_blocks)
            } else {
                vec![]
            };
            let has_scoped = descriptor.styles.iter().any(|s| s.scoped);
            let template_compiler_options = Some(vize_atelier_dom::DomCompilerOptions {
                scope_id: has_scoped.then(|| cstr!("data-v-{scope_id}")),
                ..Default::default()
            });
            // `parse.filename` is left empty: compile falls back to `script.id`,
            // which carries the same value, so no per-file clone is needed.
            // `template.id` is never read by the template compiler.
            let compile_opts = SfcCompileOptions {
                parse: SfcParseOptions::default(),
                script: ScriptCompileOptions {
                    id: Some(filename_cs.clone()),
                    inline_template: standalone,
                    is_ts,
                    ..Default::default()
                },
                template: TemplateCompileOptions {
                    scoped: has_scoped,
                    ssr,
                    is_ts,
                    custom_renderer,
                    compiler_options: template_compiler_options,
                    ..Default::default()
                },
                style: StyleCompileOptions {
                    id: filename_cs,
                    scoped: has_scoped,
                    ..Default::default()
                },
                vapor,
                scope_id: Some(scope_id.clone()),
            };

            let compile_result =
                sfc_compile_with_template_syntax(&descriptor, compile_opts, template_syntax);

            match compile_result {
                Ok(result) => {
                    success_count.fetch_add(1, Ordering::Relaxed);
                    // Empty diagnostic vectors are the common case; skip the
                    // per-element map/collect (and the empty-array boundary
                    // crossing) when there is nothing to report.
                    let errors = if result.errors.is_empty() {
                        vec![]
                    } else {
                        result
                            .errors
                            .into_iter()
                            .map(|e| e.message.into())
                            .collect()
                    };
                    let warnings = if result.warnings.is_empty() {
                        vec![]
                    } else {
                        result
                            .warnings
                            .into_iter()
                            .map(|e| e.message.into())
                            .collect()
                    };
                    let macro_artifacts = if include_macro_artifacts {
                        macro_artifacts_to_napi(result.macro_artifacts)
                    } else {
                        vec![]
                    };
                    BatchFileResultNapi {
                        path: file.path,
                        code: result.code.into(),
                        css: result.css.map(Into::into),
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
                    css: None,
                    scope_id: scope_id.into(),
                    has_scoped,
                    errors: vec![error.message.into()],
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
