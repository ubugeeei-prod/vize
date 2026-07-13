//! Owned compiler projections built while authored script Programs are live.

use crate::script::{ScriptCompileContext, type_import_specifiers_from_program};
use crate::types::{SfcError, SfcMacroArtifact};
use vize_carton::String;

use super::ScriptCompileResult;
use super::artifacts::{
    contains_artifact_macro_candidate, erase_artifact_macro_statements_from_program,
    extract_macro_artifacts_from_program,
};
use super::function_mode::compile_script_setup_with_context;
use super::lazy_hydration::transform_lazy_hydration_macros_from_program;
use super::statement_sections::{
    ScriptSections, extract_script_sections_from_program,
    extract_script_sections_from_program_with_options,
};

#[derive(Debug, Clone)]
pub(crate) struct NormalScriptCompilerFacts {
    context: ScriptCompileContext,
    type_import_specifiers: Vec<String>,
    source_is_ts: bool,
    macro_artifacts: Vec<SfcMacroArtifact>,
    dual_preserved: String,
    rewritten_default: String,
}

#[derive(Debug, Clone)]
pub(crate) struct PreanalyzedScriptSetup {
    source: String,
    context: ScriptCompileContext,
    validation_context: ScriptCompileContext,
    block_start: usize,
    sections: ScriptSections,
    runtime_preserved_sections: ScriptSections,
    type_import_specifiers: Vec<String>,
    source_is_ts: bool,
    preamble: String,
    macro_artifacts: Vec<SfcMacroArtifact>,
}

pub(crate) fn analyze_normal_script_program(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
    source_is_ts: bool,
    absolute_offset: usize,
) -> NormalScriptCompilerFacts {
    let mut context = ScriptCompileContext::new("");
    context.collect_types_from_program(program, source);
    let (derived, preamble) = derive_script_source(program, source);
    let (mut dual_preserved, mut rewritten_default) = derived
        .as_ref()
        .and_then(|derived| normal_outputs_from_derived(derived.as_str(), source_is_ts))
        .unwrap_or_else(|| normal_outputs_from_program(program, source, source_is_ts));
    if !preamble.is_empty() {
        dual_preserved.insert_str(0, preamble.as_str());
        rewritten_default.insert_str(0, preamble.as_str());
    }
    NormalScriptCompilerFacts {
        context,
        type_import_specifiers: type_import_specifiers_from_program(program, source, source_is_ts),
        source_is_ts,
        macro_artifacts: extract_macro_artifacts_from_program(program, source, absolute_offset),
        dual_preserved,
        rewritten_default,
    }
}

pub(crate) fn preanalyze_script_setup_program(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
    source_is_ts: bool,
    absolute_offset: usize,
    normal: Option<&NormalScriptCompilerFacts>,
) -> PreanalyzedScriptSetup {
    let mut validation_context = ScriptCompileContext::new(source);
    if let Some(normal) = normal {
        validation_context.merge_types_from(&normal.context);
    }
    validation_context.analyze_program(program, source);
    let artifacts = extract_macro_artifacts_from_program(program, source, absolute_offset);
    let (derived, preamble) = derive_script_source(program, source);
    let source_is_ts = source_is_ts || normal.is_some_and(|normal| normal.source_is_ts);
    let mut type_import_specifiers =
        type_import_specifiers_from_program(program, source, source_is_ts);
    if let Some(normal) = normal {
        type_import_specifiers.extend(normal.type_import_specifiers.iter().cloned());
    }
    let (source, mut context, sections, runtime_preserved_sections) = match derived {
        Some(source) => analyze_derived_script(source, source_is_ts, normal),
        None => {
            let mut context = ScriptCompileContext::new(source);
            if let Some(normal) = normal {
                context.merge_types_from(&normal.context);
            }
            context.analyze_program(program, source);
            let sections = extract_script_sections_from_program(program, source, source_is_ts)
                .unwrap_or_else(|| fallback_sections(source));
            let runtime_preserved_sections = extract_script_sections_from_program_with_options(
                program,
                source,
                source_is_ts,
                true,
            )
            .unwrap_or_else(|| fallback_sections(source));
            (source.into(), context, sections, runtime_preserved_sections)
        }
    };
    if let Some(normal) = normal {
        context.merge_types_from(&normal.context);
    }
    PreanalyzedScriptSetup {
        source,
        context,
        validation_context,
        block_start: absolute_offset,
        sections,
        runtime_preserved_sections,
        type_import_specifiers,
        source_is_ts,
        preamble,
        macro_artifacts: artifacts,
    }
}

fn derive_script_source(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
) -> (Option<String>, String) {
    let lazy = transform_lazy_hydration_macros_from_program(program, source);
    let preamble = lazy
        .as_ref()
        .map(|transform| transform.preamble.clone())
        .unwrap_or_default();
    let derived = if let Some(transform) = lazy {
        let source = transform.code;
        erase_artifacts_from_derived(source.as_str()).or(Some(source))
    } else {
        erase_artifact_macro_statements_from_program(program, source)
    };
    (derived, preamble)
}

fn normal_outputs_from_derived(source: &str, source_is_ts: bool) -> Option<(String, String)> {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(
        &allocator,
        source,
        oxc_span::SourceType::from_path("script.ts").unwrap_or_default(),
    )
    .parse();
    (!parsed.panicked).then(|| normal_outputs_from_program(&parsed.program, source, source_is_ts))
}

fn normal_outputs_from_program(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
    source_is_ts: bool,
) -> (String, String) {
    let preserved = crate::compile::extract_normal_script_content_from_program(
        program,
        source,
        source_is_ts,
        true,
    );
    let (rewritten, _) =
        crate::rewrite_default::rewrite_default_from_program(program, source, "_sfc_main");
    (preserved, rewritten)
}

fn erase_artifacts_from_derived(source: &str) -> Option<String> {
    if !contains_artifact_macro_candidate(source) {
        return None;
    }
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(
        &allocator,
        source,
        oxc_span::SourceType::from_path("script.ts").unwrap_or_default(),
    )
    .parse();
    (!parsed.panicked)
        .then(|| erase_artifact_macro_statements_from_program(&parsed.program, source))
        .flatten()
}

fn analyze_derived_script(
    source: String,
    source_is_ts: bool,
    normal: Option<&NormalScriptCompilerFacts>,
) -> (String, ScriptCompileContext, ScriptSections, ScriptSections) {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(
        &allocator,
        source.as_str(),
        oxc_span::SourceType::from_path("script.ts").unwrap_or_default(),
    )
    .parse();
    let mut context = ScriptCompileContext::new(source.as_str());
    if let Some(normal) = normal {
        context.merge_types_from(&normal.context);
    }
    if !parsed.panicked {
        context.analyze_program(&parsed.program, source.as_str());
    }
    let sections = (!parsed.panicked)
        .then(|| {
            extract_script_sections_from_program(&parsed.program, source.as_str(), source_is_ts)
        })
        .flatten()
        .unwrap_or_else(|| fallback_sections(source.as_str()));
    let runtime_preserved_sections = (!parsed.panicked)
        .then(|| {
            extract_script_sections_from_program_with_options(
                &parsed.program,
                source.as_str(),
                source_is_ts,
                true,
            )
        })
        .flatten()
        .unwrap_or_else(|| fallback_sections(source.as_str()));
    (source, context, sections, runtime_preserved_sections)
}

fn fallback_sections(source: &str) -> ScriptSections {
    (
        Vec::new(),
        source
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(String::from)
            .collect(),
        Vec::new(),
    )
}

pub(crate) fn compile_preanalyzed_script_setup(
    projection: &PreanalyzedScriptSetup,
    component_name: &str,
    is_vapor: bool,
    preserve_types: bool,
    template_content: Option<&str>,
    filename: Option<&str>,
) -> Result<ScriptCompileResult, SfcError> {
    let mut context = projection.context.clone();
    if let Some(filename) = filename.filter(|filename| !filename.is_empty()) {
        context
            .collect_imported_types_from_specifiers(&projection.type_import_specifiers, filename);
    }
    let mut result = compile_script_setup_with_context(
        projection.source.as_str(),
        component_name,
        is_vapor,
        preserve_types,
        projection.source_is_ts,
        template_content,
        context,
        if filename.is_some_and(is_art_source) {
            projection.runtime_preserved_sections.clone()
        } else {
            projection.sections.clone()
        },
    )?;
    if !projection.preamble.is_empty() {
        let mut code = projection.preamble.clone();
        code.push_str(&result.code);
        result.code = code;
    }
    Ok(result)
}

fn is_art_source(filename: &str) -> bool {
    filename.ends_with(".art.vue")
        || filename.char_indices().any(|(index, character)| {
            matches!(character, '?' | '#') && filename[..index].ends_with(".art.vue")
        })
}

impl NormalScriptCompilerFacts {
    pub(crate) fn macro_artifacts(&self) -> &[SfcMacroArtifact] {
        &self.macro_artifacts
    }

    pub(crate) fn dual_content(&self) -> &str {
        self.dual_preserved.as_str()
    }

    pub(crate) fn source_is_ts(&self) -> bool {
        self.source_is_ts
    }

    pub(crate) fn rewritten_default(&self) -> &str {
        self.rewritten_default.as_str()
    }
}

impl PreanalyzedScriptSetup {
    pub(crate) fn macro_artifacts(&self) -> &[SfcMacroArtifact] {
        &self.macro_artifacts
    }

    pub(crate) fn validate_semantics(&self, sfc_source: &str) -> Result<(), SfcError> {
        super::props::validate_props_destructure_default_types(
            &self.validation_context,
            self.block_start,
            sfc_source,
        )
    }
}
