use napi::Result;
use napi_derive::napi;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};
use vize_carton::cstr;

use super::types::{
    BatchCompileOptionsNapi, BatchCompileResultWithFilesNapi, BatchFileInputNapi,
    BatchFileResultNapi, custom_blocks_to_napi, macro_artifacts_to_napi, style_blocks_to_napi,
};

#[napi(js_name = "compileSfcBatchWithResults")]
pub fn compile_sfc_batch_with_results(
    files: Vec<BatchFileInputNapi>,
    options: Option<BatchCompileOptionsNapi>,
) -> Result<BatchCompileResultWithFilesNapi> {
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

    let results: Mutex<Vec<BatchFileResultNapi>> = Mutex::new(Vec::with_capacity(files.len()));
    let success_count = AtomicUsize::new(0);
    let failed_count = AtomicUsize::new(0);
    let ssr = opts.ssr.unwrap_or(false);
    let vapor = opts.vapor.unwrap_or(false);
    let is_ts = opts.is_ts.unwrap_or(false);
    let custom_renderer = opts.custom_renderer.unwrap_or(false);
    let vue_parser_quirks = opts.vue_parser_quirks.unwrap_or(false);
    let standalone = opts.mode.as_deref() == Some("function");
    let start = Instant::now();

    files.par_iter().for_each(|file| {
        let filename = &file.path;
        let source = &file.source;
        let scope_id = vize_atelier_sfc::generate_bundler_scope_id(filename, None, false, None);
        let filename_cs: vize_carton::CompactString = filename.clone().into();
        let descriptor = match sfc_parse(
            source,
            SfcParseOptions {
                filename: filename_cs.clone(),
                ..Default::default()
            },
        ) {
            Ok(descriptor) => descriptor,
            Err(error) => {
                failed_count.fetch_add(1, Ordering::Relaxed);
                push_result(
                    &results,
                    BatchFileResultNapi {
                        path: filename.clone(),
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
                    },
                );
                return;
            }
        };

        let template_hash: Option<String> = descriptor.template_hash().map(Into::into);
        let style_hash: Option<String> = descriptor.style_hash().map(Into::into);
        let script_hash: Option<String> = descriptor.script_hash().map(Into::into);
        let styles = style_blocks_to_napi(&descriptor.styles);
        let custom_blocks = custom_blocks_to_napi(&descriptor.custom_blocks);
        let has_scoped = descriptor.styles.iter().any(|s| s.scoped);
        let template_compiler_options = Some(vize_atelier_dom::DomCompilerOptions {
            scope_id: has_scoped.then(|| cstr!("data-v-{scope_id}")),
            ..Default::default()
        });
        let compile_opts = SfcCompileOptions {
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
                id: Some(filename_cs.clone()),
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

        let compile_result = if vue_parser_quirks {
            sfc_compile_with_vue_parser_quirks(&descriptor, compile_opts)
        } else {
            sfc_compile(&descriptor, compile_opts)
        };

        match compile_result {
            Ok(result) => {
                success_count.fetch_add(1, Ordering::Relaxed);
                push_result(
                    &results,
                    BatchFileResultNapi {
                        path: filename.clone(),
                        code: result.code.into(),
                        css: result.css.map(Into::into),
                        scope_id: scope_id.into(),
                        has_scoped,
                        errors: result
                            .errors
                            .into_iter()
                            .map(|e| e.message.into())
                            .collect(),
                        warnings: result
                            .warnings
                            .into_iter()
                            .map(|e| e.message.into())
                            .collect(),
                        template_hash: template_hash.clone(),
                        style_hash: style_hash.clone(),
                        script_hash: script_hash.clone(),
                        styles,
                        custom_blocks,
                        macro_artifacts: macro_artifacts_to_napi(result.macro_artifacts),
                    },
                );
            }
            Err(error) => {
                failed_count.fetch_add(1, Ordering::Relaxed);
                push_result(
                    &results,
                    BatchFileResultNapi {
                        path: filename.clone(),
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
                );
            }
        }
    });

    let final_results = match results.into_inner() {
        Ok(results) => results,
        Err(poisoned) => poisoned.into_inner(),
    };

    Ok(BatchCompileResultWithFilesNapi {
        results: final_results,
        success_count: success_count.load(Ordering::Relaxed) as u32,
        failed_count: failed_count.load(Ordering::Relaxed) as u32,
        time_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

fn push_result(results: &Mutex<Vec<BatchFileResultNapi>>, result: BatchFileResultNapi) {
    let mut guard = match results.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.push(result);
}
