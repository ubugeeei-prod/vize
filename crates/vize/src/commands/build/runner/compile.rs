//! Per-file compilation with profiling for the build command.
//!
//! # Arena reuse across files (Davinci P1-11)
//!
//! The batch is file-parallel over rayon and every compile allocates from an
//! arena, which is no longer built per file: the compiler takes it from
//! `vize_s0::pool`, a per-worker free list, and returns it — reset, not
//! freed — when the compile ends. Rayon worker threads outlive the batch, so
//! one arena serves every file a worker takes, and the next file bumps into
//! memory that is already mapped.
//!
//! The pool is acquired where the arena is born (the template/script/style
//! entry points inside `vize_atelier_sfc`), not passed down from here: the
//! birth site is several layers below this function, a `thread_local` is
//! already per rayon worker, and routing a handle through the CLI would pool
//! the CLI's callers only. What the build path owns is the **file boundary**:
//!
//! - every value that crosses it is in its owned form — [`CompileOutput`],
//!   [`CompileError`], [`FileProfile`], the stats cache entries — so nothing
//!   here borrows an arena, and the resident cache in `super::cache` keeps
//!   data that outlives the arena that produced it;
//! - `vize_s0::pool::checked_out()` is asserted to be zero once a file's
//!   artifacts are in hand, here and in `super::compile_stats`. That is the
//!   runtime half of the contract: a pool guard parked anywhere it must not be
//!   would keep an arena pinned across files.

use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    sync::atomic::Ordering,
    time::Instant,
};

use vize_atelier_core::{CodegenOptions, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, StyleCompileOptions,
    TemplateCompileOptions, compile_sfc_with_custom_elements_template_syntax_and_codegen_options,
    parse_sfc,
};
use vize_s0::cstr;
use vize_s0::profile;
use vize_s0::profiler::global_profiler;
use vize_s0::{String, ToCompactString};

use crate::commands::build::ScriptExtension;
use crate::commands::build::config::{
    CompileError, CompileOutput, CompileStats, ErrorPhase, FileProfile,
};
use crate::commands::davinci_ice;

use super::profile_facts::{self, FileProfileFacts, StatsCacheStatus};
use super::settings::CompileFileSettings;

/// The ICE-guarded per-file compile (P2-13, charter #30): an injected panic
/// or a panic caught around the real compile fails **this file** - with a
/// written `repro.folio` and an `internal compiler error` report - while the
/// rest of the batch continues. There is no fallback output on this path:
/// charter #26 forbids degrading to possibly-wrong output, so an ICE'd file
/// emits nothing but its repro.
///
/// Catching is live in every unwind build (dev, test, CI - where TS-23 pins
/// it); the release profile's `panic = "abort"` keeps its abort semantics
/// (see `commands::davinci_ice`).
pub(super) fn compile_file_with_profile(
    path: &PathBuf,
    settings: &CompileFileSettings,
    stats: &CompileStats,
) -> Result<(CompileOutput, FileProfile), CompileError> {
    if let Some(pass) = settings.davinci.injected_pass_for(path) {
        let failure = davinci_ice::run_injected(settings.davinci.plan_string.as_str(), pass)
            .expect_err("the injected pass was validated to be in the plan");
        return Err(ice_error(path, settings, &failure, Some(pass)));
    }
    davinci_ice::silence_panics();
    match catch_unwind(AssertUnwindSafe(|| {
        compile_file_inner(path, settings, stats)
    })) {
        Ok(result) => result,
        Err(payload) => {
            // A panic in the real compile: the driver did not see it, so it
            // carries the plan's stage and no pass (a stated unknown beats a
            // plausible lie).
            let failure = davinci_ice::IceFailure {
                stage: settings.davinci.stage.into(),
                pass: String::default(),
                reason: davinci_ice::panic_reason(payload),
            };
            Err(ice_error(path, settings, &failure, None))
        }
    }
}

/// Write the failing file's repro and fold the failure into the error
/// report: the failure line plus where the repro went.
fn ice_error(
    path: &PathBuf,
    settings: &CompileFileSettings,
    failure: &davinci_ice::IceFailure,
    inject: Option<&str>,
) -> CompileError {
    let source = fs::read_to_string(path).unwrap_or_default();
    let folio = davinci_ice::source_repro(
        settings.davinci.plan_string.as_str(),
        settings.davinci.mode,
        inject,
        failure,
        String::from(source.as_str()),
    );
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("anonymous");
    let error = match davinci_ice::write_repro(&settings.davinci.repro_dir, stem, &folio) {
        Ok(repro_path) => cstr!(
            "internal compiler error: {}\nrepro: {}",
            failure.text(),
            repro_path.display()
        ),
        Err(write_error) => cstr!(
            "internal compiler error: {}\nrepro could not be written: {write_error}",
            failure.text()
        ),
    };
    CompileError {
        path: path.clone(),
        error,
        phase: ErrorPhase::Ice,
    }
}

fn compile_file_inner(
    path: &PathBuf,
    settings: &CompileFileSettings,
    stats: &CompileStats,
) -> Result<(CompileOutput, FileProfile), CompileError> {
    let file_start = Instant::now();

    // Read file
    let source = match profile!("cli.build.file.read", fs::read_to_string(path)) {
        Ok(source) => {
            global_profiler().record_fs_read_to_string(source.len());
            source
        }
        Err(error) => {
            global_profiler().record_fs_read_to_string_failure();
            return Err(CompileError {
                path: path.clone(),
                error: cstr!("Failed to read file: {}", error),
                phase: ErrorPhase::Read,
            });
        }
    };

    let file_size = source.len();
    stats.total_bytes.fetch_add(file_size, Ordering::Relaxed);

    let filename: String = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("anonymous.vue")
        .into();
    let source_id = path.to_string_lossy().as_ref().to_compact_string();

    // Parse
    let parse_start = Instant::now();
    let parse_opts = SfcParseOptions {
        filename: filename.clone(),
        ..Default::default()
    };

    let descriptor =
        profile!("atelier.sfc.parse", parse_sfc(&source, parse_opts)).map_err(|e| {
            CompileError {
                path: path.clone(),
                error: e.message,
                phase: ErrorPhase::Parse,
            }
        })?;
    let parse_time = parse_start.elapsed();
    if settings.record_profile_totals {
        stats.add_parse_time(parse_time);
    }

    let script_lang = descriptor
        .script_setup
        .as_ref()
        .and_then(|s| s.lang.as_deref())
        .or_else(|| descriptor.script.as_ref().and_then(|s| s.lang.as_deref()))
        .unwrap_or("js")
        .to_compact_string();

    // Calculate sizes
    let template_size = descriptor
        .template
        .as_ref()
        .map(|t| t.content.len())
        .unwrap_or(0);
    let script_size = descriptor
        .script
        .as_ref()
        .map(|s| s.content.len())
        .unwrap_or(0)
        + descriptor
            .script_setup
            .as_ref()
            .map(|s| s.content.len())
            .unwrap_or(0);
    let style_count = descriptor.styles.len();

    // Compile
    let compile_start = Instant::now();
    let has_scoped = descriptor.styles.iter().any(|s| s.scoped);
    let is_ts = matches!(settings.script_ext, ScriptExtension::Preserve);
    let custom_elements = CustomElementMatcher::from_patterns(settings.custom_elements.clone());
    let compile_opts = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(source_id),
            is_ts,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            id: Some(filename.clone()),
            scoped: has_scoped,
            ssr: settings.ssr,
            is_ts,
            custom_renderer: settings.custom_renderer,
            compiler_options: Some(vize_atelier_dom::DomCompilerOptions {
                experimental_in_tag_comments: settings.experimental_in_tag_comments,
                experimental_patterned_template: settings.experimental_patterned_template,
                ..Default::default()
            }),
            dialect: settings.dialect,
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename.clone(),
            scoped: has_scoped,
            ..Default::default()
        },
        vapor: settings.vapor,
        scope_id: None,
    };

    let result = profile!(
        "atelier.sfc.compile",
        compile_sfc_with_custom_elements_template_syntax_and_codegen_options(
            &descriptor,
            compile_opts,
            settings.template_syntax,
            custom_elements,
            CodegenOptions::default()
        )
    )
    .map_err(|e| CompileError {
        path: path.clone(),
        error: e.message,
        phase: ErrorPhase::Compile,
    })?;
    let compile_time = compile_start.elapsed();
    if settings.record_profile_totals {
        stats.add_compile_time(compile_time);
    }

    let total_time = file_start.elapsed();

    // P1-11 file boundary: the compile is done and everything below is owned,
    // so this worker must be holding no arena. A non-zero count means a pool
    // guard was parked somewhere it outlives one file.
    debug_assert_eq!(
        vize_s0::pool::checked_out(),
        0,
        "a pooled arena is still checked out after compiling {}",
        path.display()
    );

    let profile = profile_facts::file_profile(
        path,
        FileProfileFacts {
            file_size,
            parse_time,
            compile_time,
            total_time,
            template_size,
            script_size,
            style_count,
        },
        settings,
        StatsCacheStatus::NotRequested,
    );

    let output = CompileOutput {
        filename,
        code: result.code,
        css: result.css,
        errors: result.errors.into_iter().map(|e| e.message).collect(),
        warnings: result.warnings.into_iter().map(|e| e.message).collect(),
        script_lang,
        macro_artifacts: result.macro_artifacts,
    };

    Ok((output, profile))
}
