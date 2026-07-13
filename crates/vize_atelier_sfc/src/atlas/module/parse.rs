//! Authored script parsing and live-Program projections.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atlas::ProviderError;
use vize_carton::{cstr, source_anchor::SourceAnchor};
use vize_croquis::script_parser::ScriptParseResult;
use vize_module::{ModuleLanguage, ModuleSyntax, snapshot_program};

use crate::compile_script::{
    NormalScriptCompilerFacts, PreanalyzedScriptSetup, analyze_normal_script_program,
    preanalyze_script_setup_program,
};

use super::super::script_generator::SfcScriptGeneratorFacts;
use super::{AUTHORED_SCRIPT_PARSE_INVOCATIONS, PlainScriptAnalysis};

pub(super) fn parse_plain(
    filename: &str,
    source: &str,
    lang: Option<&str>,
    start: usize,
    end: usize,
    root_anchor: SourceAnchor,
) -> Result<(ModuleSyntax, PlainScriptAnalysis, SfcScriptGeneratorFacts), ProviderError> {
    let (language, source_type) = language_and_source_type(lang);
    let allocator = Allocator::default();
    AUTHORED_SCRIPT_PARSE_INVOCATIONS
        .set(AUTHORED_SCRIPT_PARSE_INVOCATIONS.get().saturating_add(1));
    let parsed = Parser::new(&allocator, source, source_type).parse();
    let module = module_snapshot(
        filename,
        "script",
        source,
        language,
        start,
        end,
        root_anchor,
        &parsed.program,
        &parsed.errors,
    )?;
    let analyze = |options| {
        if parsed.panicked {
            ScriptParseResult::default()
        } else {
            vize_croquis::script_parser::analyze_script_program(&parsed.program, source, options)
        }
    };
    let compiler = analyze_normal_script_program(
        &parsed.program,
        source,
        matches!(language, ModuleLanguage::TypeScript | ModuleLanguage::Tsx),
        start,
    );
    let generator = SfcScriptGeneratorFacts::from_program(&parsed.program, source);
    Ok((
        module,
        PlainScriptAnalysis {
            standard: analyze(Default::default()),
            options_api: analyze(vize_croquis::script_parser::ScriptParserOptions {
                options_api: true,
                legacy_vue2: false,
            }),
            legacy_vue2: analyze(vize_croquis::script_parser::ScriptParserOptions {
                options_api: false,
                legacy_vue2: true,
            }),
            compiler,
        },
        generator,
    ))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn parse_setup(
    filename: &str,
    source: &str,
    lang: Option<&str>,
    generic: Option<&str>,
    start: usize,
    end: usize,
    root_anchor: SourceAnchor,
    normal: Option<&NormalScriptCompilerFacts>,
) -> Result<
    (
        ModuleSyntax,
        ScriptParseResult,
        PreanalyzedScriptSetup,
        SfcScriptGeneratorFacts,
    ),
    ProviderError,
> {
    let (language, source_type) = language_and_source_type(lang);
    let allocator = Allocator::default();
    AUTHORED_SCRIPT_PARSE_INVOCATIONS
        .set(AUTHORED_SCRIPT_PARSE_INVOCATIONS.get().saturating_add(1));
    let parsed = Parser::new(&allocator, source, source_type).parse();
    let module = module_snapshot(
        filename,
        "script-setup",
        source,
        language,
        start,
        end,
        root_anchor,
        &parsed.program,
        &parsed.errors,
    )?;
    let analysis = if parsed.panicked {
        ScriptParseResult::default()
    } else {
        vize_croquis::script_parser::analyze_script_setup_program(&parsed.program, source, generic)
    };
    let compiler = preanalyze_script_setup_program(
        &parsed.program,
        source,
        matches!(language, ModuleLanguage::TypeScript | ModuleLanguage::Tsx),
        start,
        normal,
    );
    let generator = SfcScriptGeneratorFacts::from_program(&parsed.program, source);
    Ok((module, analysis, compiler, generator))
}

#[allow(clippy::too_many_arguments)]
fn module_snapshot(
    filename: &str,
    role: &str,
    source: &str,
    language: ModuleLanguage,
    start: usize,
    end: usize,
    root_anchor: SourceAnchor,
    program: &oxc_ast::ast::Program<'_>,
    errors: &[oxc_diagnostics::OxcDiagnostic],
) -> Result<ModuleSyntax, ProviderError> {
    let start = u32::try_from(start)
        .map_err(|_| ProviderError::message("SFC script offset exceeds u32"))?;
    let end =
        u32::try_from(end).map_err(|_| ProviderError::message("SFC script offset exceeds u32"))?;
    let anchor =
        root_anchor.with_parent_range(vize_carton::source_range::SourceRange::new(start, end));
    Ok(snapshot_program(
        &cstr!("{filename}#{role}"),
        source,
        language,
        start,
        Some(anchor),
        program,
        errors,
    ))
}

fn language_and_source_type(lang: Option<&str>) -> (ModuleLanguage, SourceType) {
    match lang.map(str::trim) {
        Some("ts" | "typescript") => (ModuleLanguage::TypeScript, SourceType::ts()),
        Some("tsx") => (ModuleLanguage::Tsx, SourceType::tsx()),
        Some("jsx") => (ModuleLanguage::Jsx, SourceType::jsx()),
        _ => (ModuleLanguage::JavaScript, SourceType::mjs()),
    }
}
