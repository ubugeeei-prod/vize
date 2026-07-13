//! Canon virtual TypeScript emission from shared Atlas SFC artifacts.

use std::path::Path;

use vize_atelier_sfc::{
    SfcDescriptor,
    croquis::{SfcCroquisOptions, script_content_for_descriptor},
};
use vize_carton::{Bump, cstr, profile};
use vize_croquis::CroquisDocument;
use vize_relief::ReliefArtifact;

use crate::batch::error::{CorsaError, CorsaResult};
use crate::batch::{Diagnostic, SfcBlockType};
use crate::script_parse::collect_script_parse_diagnostics;
use crate::virtual_ts::{
    VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts_with_offsets_and_checks,
};

use super::diagnostics::{
    collect_sfc_compile_diagnostic, diagnostic_for_offset, invalid_sfc_fallback_virtual_ts,
};
use super::vue_codegen::{GeneratedVueFile, VueCodegenOptions};

pub(super) struct VueArtifactInputs<'a, 'source> {
    pub(super) descriptor: &'a SfcDescriptor<'source>,
    pub(super) syntax: Option<&'a ReliefArtifact>,
    pub(super) semantics: &'a CroquisDocument,
    pub(super) extra_template_referenced_names:
        Option<&'a vize_carton::FxHashSet<vize_carton::String>>,
}

/// Generate the exact batch virtual module while borrowing the descriptor,
/// parse-only Relief syntax, and complete Croquis document cached by Atlas.
pub(super) fn generate_vue_virtual_ts_from_artifacts(
    path: &Path,
    source: &str,
    artifacts: VueArtifactInputs<'_, '_>,
    options: &VirtualTsOptions,
    codegen_options: VueCodegenOptions,
) -> CorsaResult<GeneratedVueFile> {
    let VueArtifactInputs {
        descriptor,
        syntax,
        semantics,
        extra_template_referenced_names,
    } = artifacts;
    let mut diagnostics = script_parse_diagnostics(path, source, descriptor);
    let allocator = Bump::new();
    let template_offset = descriptor
        .template
        .as_ref()
        .map(|template| template.loc.start as u32)
        .unwrap_or(0);
    let template_ast = match (descriptor.template.as_ref(), syntax) {
        (Some(_), Some(syntax)) => {
            for error in syntax.parse_diagnostics() {
                if error.code.is_recovery() {
                    continue;
                }
                let start = error
                    .loc
                    .as_ref()
                    .map(|loc| template_offset + loc.start.offset)
                    .unwrap_or(template_offset);
                diagnostics.push(diagnostic_for_offset(
                    path,
                    source,
                    start,
                    cstr!("Template parse error: {}", error.message),
                    SfcBlockType::Template,
                ));
            }
            Some(syntax.snapshot().materialize(&allocator))
        }
        (None, None) => None,
        _ => {
            return Err(CorsaError::ArtifactGraph(
                "SFC descriptor and Relief syntax disagree about template presence".into(),
            ));
        }
    };

    if !diagnostics.is_empty() {
        return Ok(GeneratedVueFile {
            code: invalid_sfc_fallback_virtual_ts(),
            mappings: Vec::new(),
            diagnostics,
        });
    }

    let (script_content, script_offset) =
        script_content_for_descriptor(descriptor, SfcCroquisOptions::full());
    let vue2_compat = codegen_options.legacy_vue2
        || matches!(
            codegen_options.dialect,
            vize_carton::config::VueVersion::V2 | vize_carton::config::VueVersion::V2_7
        );
    let output = profile!(
        "canon.virtual_ts.generate.shared",
        generate_virtual_ts_with_offsets_and_checks(
            semantics.analysis(),
            script_content.as_deref(),
            template_ast.as_ref(),
            script_offset,
            template_offset,
            options,
            VirtualTsGenerationOptions {
                check_options: codegen_options.check_options,
                dialect: codegen_options.dialect,
                preserve_unused_diagnostics: codegen_options.preserve_unused_diagnostics,
                extra_template_referenced_names,
                options_api: codegen_options.options_api || vue2_compat,
                legacy_vue2: vue2_compat,
                template_syntax_quirks: matches!(
                    codegen_options.template_syntax,
                    vize_relief::TemplateSyntaxMode::Quirks
                ),
                hoist_shared_preamble: codegen_options.hoist_shared_preamble && !vue2_compat,
                lib_references: None,
            },
        )
    );

    if let Some(diagnostic) = profile!(
        "canon.sfc.compile_validate",
        collect_sfc_compile_diagnostic(path, source, descriptor)
    ) {
        diagnostics.push(diagnostic);
    }
    Ok(GeneratedVueFile {
        code: output.code,
        mappings: output.mappings,
        diagnostics,
    })
}

fn script_parse_diagnostics(
    path: &Path,
    source: &str,
    descriptor: &SfcDescriptor<'_>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for (block, block_type) in [
        (descriptor.script.as_ref(), SfcBlockType::Script),
        (descriptor.script_setup.as_ref(), SfcBlockType::ScriptSetup),
    ] {
        let Some(block) = block else { continue };
        diagnostics.extend(
            collect_script_parse_diagnostics(
                &block.content,
                block.loc.start as u32,
                block.lang.as_deref(),
            )
            .into_iter()
            .map(|diagnostic| {
                diagnostic_for_offset(
                    path,
                    source,
                    diagnostic.start,
                    cstr!("Script parse error: {}", diagnostic.message),
                    block_type,
                )
            }),
        );
    }
    diagnostics
}
