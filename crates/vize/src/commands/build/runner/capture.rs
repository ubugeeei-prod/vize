//! Host-side dump of the stages from the exact SFC compile that built a file.

mod page_files;

use std::path::Path;

use vize_atelier_sfc::SfcDescriptor;
use vize_curator::inspector::{ProductCaptureSource, product_capture_value};
use vize_l0::cstr;
use vize_l0::dump::capture::{CaptureOutcome, CaptureSink, StageCapture};
use vize_l0::{Span, String};

use super::super::config::{CompileError, CompileOutput, CompileStats, ErrorPhase, FileProfile};
use super::compile::compile_file_with_profile;
use super::output::PlannedInput;
use super::settings::CompileFileSettings;

/// Metadata from the same parsed descriptor as the emitted build output.
pub(super) struct BuildCapture {
    pub(super) stages: StageCapture,
    pub(super) authored_syntax: String,
    pub(super) compiled_syntax: String,
    pub(super) template_span: Option<Span>,
}

impl BuildCapture {
    pub(super) fn from_descriptor(
        mut stages: StageCapture,
        descriptor: &SfcDescriptor<'_>,
    ) -> Self {
        let template = descriptor.template.as_ref();
        // Only Pug is preprocessed by the product adapter. Preserve other
        // declared syntaxes without claiming a transformation.
        let authored_lower = template
            .and_then(|template| template.lang.as_deref())
            .unwrap_or("html")
            .to_ascii_lowercase();
        let authored_syntax = if authored_lower
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
            && authored_lower
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            authored_lower.as_str()
        } else {
            "vue-template"
        };
        let compiled_syntax = if authored_syntax == "pug" {
            "vue-template"
        } else {
            authored_syntax
        };
        if template.is_some_and(|block| block.src.is_some())
            || descriptor
                .script
                .as_ref()
                .is_some_and(|block| block.src.is_some())
            || descriptor
                .script_setup
                .as_ref()
                .is_some_and(|block| block.src.is_some())
            || descriptor.styles.iter().any(|block| block.src.is_some())
        {
            stages.finish(|| CaptureOutcome::Unavailable(cstr!("external SFC block source")));
        }
        let template_span = template.and_then(|template| {
            Some(Span::new(
                template.loc.start.try_into().ok()?,
                template.loc.end.try_into().ok()?,
            ))
        });
        Self {
            stages,
            authored_syntax: String::from(authored_syntax),
            compiled_syntax: String::from(compiled_syntax),
            template_span,
        }
    }
}

pub(super) fn compile_planned_file(
    input: &PlannedInput,
    settings: &CompileFileSettings,
    stats: &CompileStats,
) -> Result<(CompileOutput, FileProfile), CompileError> {
    let dir = settings
        .davinci
        .dump_dir
        .as_deref()
        .map(|root| page_files::prepare(root, &input.relative_source))
        .transpose()
        .map_err(|error| CompileError {
            path: input.source.clone(),
            error,
            phase: ErrorPhase::Dump,
        })?;
    let (output, profile, capture) = compile_file_with_profile(&input.source, settings, stats)?;
    if let Some(dir) = dir.as_deref() {
        let capture = capture.ok_or_else(|| CompileError {
            path: input.source.clone(),
            error: cstr!("--dump-dir: observed compile did not return a stage capture"),
            phase: ErrorPhase::Dump,
        })?;
        write(
            dir,
            &input.relative_source,
            capture,
            settings.davinci.dump_after_change,
        )
        .map_err(|error| CompileError {
            path: input.source.clone(),
            error,
            phase: ErrorPhase::Dump,
        })?;
    }
    Ok((output, profile))
}

pub(super) fn write(
    dir: &Path,
    relative_source: &Path,
    capture: BuildCapture,
    after_change: bool,
) -> Result<(), String> {
    page_files::write_pages(dir, &capture.stages, after_change)?;

    let source_path = relative_source.to_string_lossy();
    let feed = product_capture_value(
        "vize-build",
        ProductCaptureSource {
            path: Some(&source_path),
            container: "vue-sfc",
            authored_syntax: &capture.authored_syntax,
            compiled_syntax: &capture.compiled_syntax,
            template_span: capture.template_span,
        },
        &capture.stages,
    );
    let path = dir.join("stages.json");
    let bytes = serde_json::to_vec_pretty(&feed)
        .map_err(|error| cstr!("--dump-dir: cannot serialize {}: {error}", path.display()))?;
    page_files::write_feed(&path, &bytes)
}
