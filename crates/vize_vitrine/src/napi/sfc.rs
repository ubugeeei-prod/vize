//! NAPI bindings for SFC parsing and compilation.
//!
//! Provides parseSfc, compileSfc, compileSfcBatch, and
//! compileSfcBatchWithResults functions for Vue Single File Components.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use glob::glob;
use napi::bindgen_prelude::{Env, Error, Object, Result, Status};
use napi_derive::napi;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};
use vize_carton::cstr;

/// SFC parse options for NAPI
#[napi(object)]
#[derive(Default)]
pub struct SfcParseOptionsNapi {
    pub filename: Option<String>,
}

/// SFC compile options for NAPI
#[napi(object)]
#[derive(Default)]
pub struct SfcCompileOptionsNapi {
    pub filename: Option<String>,
    pub source_map: Option<bool>,
    pub ssr: Option<bool>,
    /// Preserve TypeScript in output when true
    pub is_ts: Option<bool>,
    /// Scope ID for scoped CSS (e.g., "data-v-abc123")
    pub scope_id: Option<String>,
}

/// SFC compile result for NAPI
#[napi(object)]
pub struct SfcCompileResultNapi {
    /// Generated JavaScript code
    pub code: String,
    /// Generated CSS (if any)
    pub css: Option<String>,
    /// Compilation errors
    pub errors: Vec<String>,
    /// Compilation warnings
    pub warnings: Vec<String>,
    /// Hash of template content (for HMR)
    pub template_hash: Option<String>,
    /// Hash of style content (for HMR)
    pub style_hash: Option<String>,
    /// Hash of script content (for HMR)
    pub script_hash: Option<String>,
}

/// Batch compile options for NAPI
#[napi(object)]
#[derive(Default)]
pub struct BatchCompileOptionsNapi {
    pub ssr: Option<bool>,
    /// Preserve TypeScript in output when true
    pub is_ts: Option<bool>,
    pub threads: Option<u32>,
}

/// Batch compile result for NAPI
#[napi(object)]
pub struct BatchCompileResultNapi {
    /// Number of files compiled successfully
    pub success: u32,
    /// Number of files that failed
    pub failed: u32,
    /// Total input bytes
    pub input_bytes: u32,
    /// Total output bytes
    pub output_bytes: u32,
    /// Compilation time in milliseconds
    pub time_ms: f64,
}

/// Input file for batch compilation with results
#[napi(object)]
pub struct BatchFileInputNapi {
    /// File path
    pub path: String,
    /// Source code
    pub source: String,
}

/// Per-file result from batch compilation
#[napi(object)]
pub struct BatchFileResultNapi {
    /// File path
    pub path: String,
    /// Generated JavaScript code
    pub code: String,
    /// Generated CSS (if any)
    pub css: Option<String>,
    /// Scope ID for scoped styles
    pub scope_id: String,
    /// Whether the file has scoped styles
    pub has_scoped: bool,
    /// Compilation errors
    pub errors: Vec<String>,
    /// Compilation warnings
    pub warnings: Vec<String>,
    /// Hash of template content (for HMR)
    pub template_hash: Option<String>,
    /// Hash of style content (for HMR)
    pub style_hash: Option<String>,
    /// Hash of script content (for HMR)
    pub script_hash: Option<String>,
}

/// Batch compile result with per-file results
#[napi(object)]
pub struct BatchCompileResultWithFilesNapi {
    /// Per-file compilation results
    pub results: Vec<BatchFileResultNapi>,
    /// Number of files compiled successfully
    pub success_count: u32,
    /// Number of files that failed
    pub failed_count: u32,
    /// Compilation time in milliseconds
    pub time_ms: f64,
}

/// Parse SFC (.vue file) - returns lightweight result for speed
#[napi(js_name = "parseSfc")]
pub fn parse_sfc(env: Env, source: String, options: Option<SfcParseOptionsNapi>) -> Result<Object> {
    use vize_atelier_sfc::{parse_sfc as sfc_parse, SfcParseOptions};

    let opts = options.unwrap_or_default();
    let parse_opts = SfcParseOptions {
        filename: opts
            .filename
            .unwrap_or_else(|| "anonymous.vue".to_string())
            .into(),
        ..Default::default()
    };

    match sfc_parse(&source, parse_opts) {
        Ok(descriptor) => {
            // Build JS object directly for speed (avoid JSON serialization)
            let mut obj = env.create_object()?;

            obj.set("filename", descriptor.filename.as_ref())?;
            obj.set("source", descriptor.source.as_ref())?;

            // Template
            if let Some(ref template) = descriptor.template {
                let mut tpl_obj = env.create_object()?;
                tpl_obj.set("content", template.content.as_ref())?;
                tpl_obj.set("lang", template.lang.as_deref())?;
                obj.set("template", tpl_obj)?;
            } else {
                obj.set("template", env.get_null()?)?;
            }

            // Script
            if let Some(ref script) = descriptor.script {
                let mut scr_obj = env.create_object()?;
                scr_obj.set("content", script.content.as_ref())?;
                scr_obj.set("lang", script.lang.as_deref())?;
                scr_obj.set("setup", script.setup)?;
                obj.set("script", scr_obj)?;
            } else {
                obj.set("script", env.get_null()?)?;
            }

            // Script Setup
            if let Some(ref script_setup) = descriptor.script_setup {
                let mut scr_obj = env.create_object()?;
                scr_obj.set("content", script_setup.content.as_ref())?;
                scr_obj.set("lang", script_setup.lang.as_deref())?;
                scr_obj.set("setup", script_setup.setup)?;
                obj.set("scriptSetup", scr_obj)?;
            } else {
                obj.set("scriptSetup", env.get_null()?)?;
            }

            // Styles
            let mut styles_arr = env.create_array(descriptor.styles.len() as u32)?;
            for (i, style) in descriptor.styles.iter().enumerate() {
                let mut style_obj = env.create_object()?;
                style_obj.set("content", style.content.as_ref())?;
                style_obj.set("lang", style.lang.as_deref())?;
                style_obj.set("scoped", style.scoped)?;
                style_obj.set("module", style.module.as_deref())?;
                styles_arr.set(i as u32, style_obj)?;
            }
            obj.set("styles", styles_arr)?;

            // Custom blocks
            let mut customs_arr = env.create_array(descriptor.custom_blocks.len() as u32)?;
            for (i, block) in descriptor.custom_blocks.iter().enumerate() {
                let mut block_obj = env.create_object()?;
                block_obj.set("type", block.block_type.as_ref())?;
                block_obj.set("content", block.content.as_ref())?;
                customs_arr.set(i as u32, block_obj)?;
            }
            obj.set("customBlocks", customs_arr)?;

            Ok(obj)
        }
        Err(e) => Err(Error::new(Status::GenericFailure, e.message.to_string())),
    }
}

/// Compile SFC (.vue file) to JavaScript - main use case
#[napi(js_name = "compileSfc")]
pub fn compile_sfc(
    source: String,
    options: Option<SfcCompileOptionsNapi>,
) -> Result<SfcCompileResultNapi> {
    use vize_atelier_sfc::{
        compile_sfc as sfc_compile, parse_sfc as sfc_parse, ScriptCompileOptions,
        SfcCompileOptions, SfcParseOptions, StyleCompileOptions, TemplateCompileOptions,
    };

    let opts = options.unwrap_or_default();
    let filename: vize_carton::CompactString = opts
        .filename
        .unwrap_or_else(|| "anonymous.vue".to_string())
        .into();

    // Parse
    let parse_opts = SfcParseOptions {
        filename: filename.clone(),
        ..Default::default()
    };

    let descriptor = match sfc_parse(&source, parse_opts) {
        Ok(d) => d,
        Err(e) => {
            return Ok(SfcCompileResultNapi {
                code: String::new(),
                css: None,
                errors: vec![e.message.into()],
                warnings: vec![],
                template_hash: None,
                style_hash: None,
                script_hash: None,
            });
        }
    };

    let template_hash: Option<String> = descriptor.template_hash().map(Into::into);
    let style_hash: Option<String> = descriptor.style_hash().map(Into::into);
    let script_hash: Option<String> = descriptor.script_hash().map(Into::into);

    // Compile
    let has_scoped = descriptor.styles.iter().any(|s| s.scoped);
    let is_ts = opts.is_ts.unwrap_or(false);

    // Extract scope_id from options (strip "data-v-" prefix if present)
    let external_scope_id: Option<vize_carton::CompactString> = opts
        .scope_id
        .as_ref()
        .map(|sid| sid.strip_prefix("data-v-").unwrap_or(sid).into());

    // Create compiler options with scope_id for scoped CSS
    let template_compiler_options = if has_scoped {
        external_scope_id
            .as_ref()
            .map(|scope_id| vize_atelier_dom::DomCompilerOptions {
                scope_id: Some(cstr!("data-v-{scope_id}").into()),
                ..Default::default()
            })
    } else {
        None
    };

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
            ssr: opts.ssr.unwrap_or(false),
            is_ts,
            compiler_options: template_compiler_options,
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename,
            scoped: has_scoped,
            ..Default::default()
        },
        scope_id: external_scope_id,
    };

    match sfc_compile(&descriptor, compile_opts) {
        Ok(result) => Ok(SfcCompileResultNapi {
            code: result.code.into(),
            css: result.css.map(Into::into),
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
        }),
        Err(e) => Ok(SfcCompileResultNapi {
            code: String::new(),
            css: None,
            errors: vec![e.message.into()],
            warnings: vec![],
            template_hash,
            style_hash,
            script_hash,
        }),
    }
}

/// Batch compile SFC files matching a glob pattern (native multithreading)
#[napi(js_name = "compileSfcBatch")]
pub fn compile_sfc_batch(
    pattern: String,
    options: Option<BatchCompileOptionsNapi>,
) -> Result<BatchCompileResultNapi> {
    use std::time::Instant;
    use vize_atelier_sfc::{
        compile_sfc as sfc_compile, parse_sfc as sfc_parse, ScriptCompileOptions,
        SfcCompileOptions, SfcParseOptions, StyleCompileOptions, TemplateCompileOptions,
    };

    let opts = options.unwrap_or_default();
    let ssr = opts.ssr.unwrap_or(false);
    let is_ts = opts.is_ts.unwrap_or(false);

    // Configure thread pool if specified
    if let Some(threads) = opts.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads as usize)
            .build_global()
            .ok(); // Ignore if already configured
    }

    // Collect files matching the pattern
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

    let success = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    let input_bytes = AtomicUsize::new(0);
    let output_bytes = AtomicUsize::new(0);

    let start = Instant::now();

    // Compile files in parallel using rayon
    files.par_iter().for_each(|path| {
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                failed.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };

        input_bytes.fetch_add(source.len(), Ordering::Relaxed);

        let filename: vize_carton::CompactString = path.to_string_lossy().as_ref().into();

        // Parse
        let parse_opts = SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        };

        let descriptor = match sfc_parse(&source, parse_opts) {
            Ok(d) => d,
            Err(_) => {
                failed.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };

        // Compile
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
            scope_id: None,
        };

        match sfc_compile(&descriptor, compile_opts) {
            Ok(result) => {
                success.fetch_add(1, Ordering::Relaxed);
                output_bytes.fetch_add(result.code.len(), Ordering::Relaxed);
            }
            Err(_) => {
                failed.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    let elapsed = start.elapsed();

    Ok(BatchCompileResultNapi {
        success: success.load(Ordering::Relaxed) as u32,
        failed: failed.load(Ordering::Relaxed) as u32,
        input_bytes: input_bytes.load(Ordering::Relaxed) as u32,
        output_bytes: output_bytes.load(Ordering::Relaxed) as u32,
        time_ms: elapsed.as_secs_f64() * 1000.0,
    })
}

/// Batch compile SFC files with per-file results (in-memory, native multithreading)
#[napi(js_name = "compileSfcBatchWithResults")]
pub fn compile_sfc_batch_with_results(
    files: Vec<BatchFileInputNapi>,
    options: Option<BatchCompileOptionsNapi>,
) -> Result<BatchCompileResultWithFilesNapi> {
    use std::sync::Mutex;
    use std::time::Instant;
    use vize_atelier_sfc::{
        compile_sfc as sfc_compile, parse_sfc as sfc_parse, ScriptCompileOptions,
        SfcCompileOptions, SfcParseOptions, StyleCompileOptions, TemplateCompileOptions,
    };

    let opts = options.unwrap_or_default();
    let ssr = opts.ssr.unwrap_or(false);
    let is_ts = opts.is_ts.unwrap_or(false);

    // Configure thread pool if specified
    if let Some(threads) = opts.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads as usize)
            .build_global()
            .ok(); // Ignore if already configured
    }

    let results: Mutex<Vec<BatchFileResultNapi>> = Mutex::new(Vec::with_capacity(files.len()));
    let success_count = AtomicUsize::new(0);
    let failed_count = AtomicUsize::new(0);

    let start = Instant::now();

    // Compile files in parallel using rayon
    files.par_iter().for_each(|file| {
        let filename = &file.path;
        let source = &file.source;

        // Generate scope ID from filename using SHA-256 (matching JS-side generateScopeId)
        let scope_id = {
            use sha2::{Digest, Sha256};
            let hash = Sha256::digest(filename.as_bytes());
            // Take first 8 hex chars of the SHA-256 hash (same as JS: hash.slice(0, 8))
            cstr!(
                "{:02x}{:02x}{:02x}{:02x}",
                hash[0],
                hash[1],
                hash[2],
                hash[3]
            )
        };

        let has_scoped = source.contains("<style") && source.contains("scoped");

        let filename_cs: vize_carton::CompactString = filename.clone().into();

        // Parse
        let parse_opts = SfcParseOptions {
            filename: filename_cs.clone(),
            ..Default::default()
        };

        let descriptor = match sfc_parse(source, parse_opts) {
            Ok(d) => d,
            Err(e) => {
                failed_count.fetch_add(1, Ordering::Relaxed);
                let mut guard = results.lock().unwrap();
                guard.push(BatchFileResultNapi {
                    path: filename.clone(),
                    code: String::new(),
                    css: None,
                    scope_id: scope_id.clone().into(),
                    has_scoped,
                    errors: vec![e.message.into()],
                    warnings: vec![],
                    template_hash: None,
                    style_hash: None,
                    script_hash: None,
                });
                return;
            }
        };

        // Compute hashes for HMR
        let template_hash: Option<String> = descriptor.template_hash().map(Into::into);
        let style_hash: Option<String> = descriptor.style_hash().map(Into::into);
        let script_hash: Option<String> = descriptor.script_hash().map(Into::into);

        // Compile
        // Preserve TypeScript in output - let Vite/esbuild handle TS transformation
        let actual_has_scoped = descriptor.styles.iter().any(|s| s.scoped);
        // Create compiler options with scope_id for scoped CSS
        let template_compiler_options = if actual_has_scoped {
            Some(vize_atelier_dom::DomCompilerOptions {
                scope_id: Some(cstr!("data-v-{scope_id}").into()),
                ..Default::default()
            })
        } else {
            None
        };

        let compile_opts = SfcCompileOptions {
            parse: SfcParseOptions {
                filename: filename_cs.clone(),
                ..Default::default()
            },
            script: ScriptCompileOptions {
                id: Some(filename_cs.clone()),
                is_ts,
                ..Default::default()
            },
            template: TemplateCompileOptions {
                id: Some(filename_cs.clone()),
                scoped: actual_has_scoped,
                ssr,
                is_ts,
                compiler_options: template_compiler_options,
                ..Default::default()
            },
            style: StyleCompileOptions {
                id: filename_cs,
                scoped: actual_has_scoped,
                ..Default::default()
            },
            scope_id: Some(scope_id.clone()),
        };

        match sfc_compile(&descriptor, compile_opts) {
            Ok(result) => {
                success_count.fetch_add(1, Ordering::Relaxed);
                let mut guard = results.lock().unwrap();
                guard.push(BatchFileResultNapi {
                    path: filename.clone(),
                    code: result.code.into(),
                    css: result.css.map(Into::into),
                    scope_id: scope_id.into(),
                    has_scoped: actual_has_scoped,
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
                });
            }
            Err(e) => {
                failed_count.fetch_add(1, Ordering::Relaxed);
                let mut guard = results.lock().unwrap();
                guard.push(BatchFileResultNapi {
                    path: filename.clone(),
                    code: String::new(),
                    css: None,
                    scope_id: scope_id.into(),
                    has_scoped: actual_has_scoped,
                    errors: vec![e.message.into()],
                    warnings: vec![],
                    template_hash,
                    style_hash,
                    script_hash,
                });
            }
        }
    });

    let elapsed = start.elapsed();
    let final_results = results.into_inner().unwrap();

    Ok(BatchCompileResultWithFilesNapi {
        results: final_results,
        success_count: success_count.load(Ordering::Relaxed) as u32,
        failed_count: failed_count.load(Ordering::Relaxed) as u32,
        time_ms: elapsed.as_secs_f64() * 1000.0,
    })
}
