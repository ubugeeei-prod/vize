//! Generating virtual TypeScript for `.vue` SFCs: parsing the template, running
//! Croquis analysis, augmenting type-based props, and emitting the `.vue.ts`
//! source consumed by Corsa. Parse/compile errors are surfaced as diagnostics
//! and replaced with a typed fallback module.

use std::path::Path;
use vize_carton::config::VueVersion;
use vize_carton::{Bump, String as CompactString, cstr, profile};

use vize_atelier_core::{
    ParserOptions, TemplateSyntaxMode, parser::parse_with_options_and_template_syntax,
};
use vize_atelier_sfc::{
    SfcDescriptor,
    croquis::{
        SfcCroquisOptions, analyze_sfc_descriptor_with_context,
        analyze_sfc_descriptor_with_context_legacy_vue2,
        analyze_sfc_descriptor_with_context_options_api,
    },
};

use crate::batch::error::CorsaResult;
use crate::batch::{Diagnostic, SfcBlockType};
use crate::script_parse::collect_script_parse_diagnostics;
use crate::virtual_ts::{
    VirtualTsCheckOptions, VirtualTsGenerationOptions, VirtualTsOptions,
    generate_virtual_ts_with_offsets_and_checks,
};

use super::diagnostics::{
    collect_sfc_compile_diagnostic, diagnostic_for_offset, invalid_sfc_fallback_virtual_ts,
};
use super::{
    art_usage::collect_art_template_referenced_names,
    setup_props::augment_type_based_props_from_script_context,
};

pub(super) struct GeneratedVueFile {
    pub(super) code: CompactString,
    pub(super) mappings: Vec<crate::virtual_ts::VizeMapping>,
    pub(super) diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy)]
pub(super) struct VueCodegenOptions {
    pub(super) check_options: VirtualTsCheckOptions,
    pub(super) preserve_unused_diagnostics: bool,
    pub(super) options_api: bool,
    pub(super) legacy_vue2: bool,
    pub(super) dialect: VueVersion,
    pub(super) template_syntax: TemplateSyntaxMode,
    /// Hoist shared helpers to the batch ambient `.d.ts`; socket sessions keep
    /// them inline because they do not materialize that file.
    pub(super) hoist_shared_preamble: bool,
}

pub(super) fn generate_vue_virtual_ts(
    path: &Path,
    source: &str,
    descriptor: &SfcDescriptor,
    options: &VirtualTsOptions,
    codegen_options: VueCodegenOptions,
) -> CorsaResult<GeneratedVueFile> {
    let allocator = Bump::new();
    let mut diagnostics = Vec::new();

    if let Some(ref script) = descriptor.script {
        let script_diagnostics = collect_script_parse_diagnostics(
            &script.content,
            script.loc.start as u32,
            script.lang.as_deref(),
        );
        if !script_diagnostics.is_empty() {
            diagnostics.extend(script_diagnostics.into_iter().map(|diagnostic| {
                diagnostic_for_offset(
                    path,
                    source,
                    diagnostic.start,
                    cstr!("Script parse error: {}", diagnostic.message),
                    SfcBlockType::Script,
                )
            }));
        }
    }

    if let Some(ref script_setup) = descriptor.script_setup {
        let script_diagnostics = collect_script_parse_diagnostics(
            &script_setup.content,
            script_setup.loc.start as u32,
            script_setup.lang.as_deref(),
        );
        if !script_diagnostics.is_empty() {
            diagnostics.extend(script_diagnostics.into_iter().map(|diagnostic| {
                diagnostic_for_offset(
                    path,
                    source,
                    diagnostic.start,
                    cstr!("Script parse error: {}", diagnostic.message),
                    SfcBlockType::ScriptSetup,
                )
            }));
        }
    }

    let template_offset = descriptor
        .template
        .as_ref()
        .map(|template| template.loc.start as u32)
        .unwrap_or(0);
    // Track whether the template produced any *hard* parse error. Only hard
    // errors abort codegen and collapse the file to the fallback stub;
    // recovery-level diagnostics (`ErrorCode::ExtendPoint`, pushed by the HTML
    // tree-construction recovery path for self-closing rewrites, fostered
    // elements, auto-closed `<p>`, etc.) describe repairs the parser already
    // applied and must keep the real virtual TS (#1065/#1090 regression).
    let mut template_hard_error = false;
    let template_ast = descriptor.template.as_ref().and_then(|template| {
        profile!("canon.template.parse", {
            let (root, errors) = parse_with_options_and_template_syntax(
                &allocator,
                &template.content,
                ParserOptions::default(),
                codegen_options.template_syntax,
            );
            for error in errors {
                if error.code.is_recovery() {
                    continue;
                }
                template_hard_error = true;
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
            // Drop the AST only when a hard error occurred; recovery-level
            // diagnostics leave a fully usable tree.
            (!template_hard_error).then_some(root)
        })
    });

    // Abort to the fallback stub only on hard errors — from any block. Pure
    // recovery-level template diagnostics must not suppress real codegen.
    if !diagnostics.is_empty() {
        return Ok(GeneratedVueFile {
            code: invalid_sfc_fallback_virtual_ts(),
            mappings: Vec::new(),
            diagnostics,
        });
    }

    let croquis_options = SfcCroquisOptions::full();

    let analysis = profile!(
        "canon.croquis.analyze_sfc",
        if codegen_options.legacy_vue2 {
            analyze_sfc_descriptor_with_context_legacy_vue2(
                descriptor,
                template_ast.as_ref(),
                croquis_options,
            )
        } else if codegen_options.options_api {
            analyze_sfc_descriptor_with_context_options_api(
                descriptor,
                template_ast.as_ref(),
                croquis_options,
            )
        } else {
            analyze_sfc_descriptor_with_context(descriptor, template_ast.as_ref(), croquis_options)
        }
    );
    let vize_atelier_sfc::croquis::SfcCroquisAnalysis {
        mut croquis,
        script_content,
        script_offset,
    } = analysis;
    profile!(
        "canon.croquis.augment_type_props",
        augment_type_based_props_from_script_context(&mut croquis, descriptor, path)
    );
    let extra_template_referenced_names = codegen_options.preserve_unused_diagnostics.then(|| {
        collect_art_template_referenced_names(descriptor, codegen_options.template_syntax)
    });

    let hoist_shared_preamble = codegen_options.hoist_shared_preamble
        && !codegen_options.legacy_vue2
        && !matches!(codegen_options.dialect, VueVersion::V2 | VueVersion::V2_7);
    let output = profile!(
        "canon.virtual_ts.generate",
        generate_virtual_ts_with_offsets_and_checks(
            &croquis,
            script_content.as_deref(),
            template_ast.as_ref(),
            script_offset,
            template_offset,
            options,
            VirtualTsGenerationOptions {
                check_options: codegen_options.check_options,
                dialect: codegen_options.dialect,
                preserve_unused_diagnostics: codegen_options.preserve_unused_diagnostics,
                extra_template_referenced_names: extra_template_referenced_names.as_ref(),
                options_api: codegen_options.options_api,
                legacy_vue2: codegen_options.legacy_vue2,
                template_syntax_quirks: matches!(
                    codegen_options.template_syntax,
                    TemplateSyntaxMode::Quirks
                ),
                hoist_shared_preamble,
            },
        )
    );

    // Surface Vue-specific semantic errors (e.g. DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE)
    // that the SFC compiler catches but TypeScript itself does not. Without this,
    // `vize check` would silently accept SFCs that `vize build` rejects.
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
