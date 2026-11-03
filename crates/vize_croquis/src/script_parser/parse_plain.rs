//! Parse entry points for plain (Options API) scripts.

use oxc_allocator::Allocator;
use oxc_span::SourceType;

use super::globals::setup_global_scopes;
use super::module_exports::{ModuleExportCapture, parse_script_plain};
use super::process;
use super::recovery::parse_program_for_analysis;
use super::result::{ScriptParseResult, ScriptParserOptions};
use crate::croquis::BindingMetadata;
use crate::scope::NonScriptSetupScopeData;
use vize_carton::profile;

/// Parse non-script-setup (Options API) source code using OXC parser.
pub fn parse_script(source: &str) -> ScriptParseResult {
    parse_script_with_options(source, ScriptParserOptions::default())
}

/// Parse non-script-setup source code using OXC parser with explicit options.
pub fn parse_script_with_options(source: &str, options: ScriptParserOptions) -> ScriptParseResult {
    parse_script_with_options_source_type(
        source,
        options,
        SourceType::from_path("script.ts").unwrap_or_default(),
        false,
        false,
        false,
        None,
    )
}

/// Parse non-script-setup source code using OXC parser with JSX enabled.
pub fn parse_script_with_options_and_jsx(
    source: &str,
    options: ScriptParserOptions,
    jsx: bool,
) -> ScriptParseResult {
    parse_script_plain(source, options, jsx, false, false)
}

pub(crate) fn parse_script_plain_with_exports(
    source: &str,
    options: ScriptParserOptions,
    jsx: bool,
    skip_diagnostics: bool,
    occurrences: bool,
    unused: bool,
    exports: Option<&mut ModuleExportCapture>,
) -> ScriptParseResult {
    let path = if jsx { "script.tsx" } else { "script.ts" };
    parse_script_with_options_source_type(
        source,
        options,
        SourceType::from_path(path).unwrap_or_default(),
        skip_diagnostics,
        occurrences,
        unused,
        exports,
    )
}

pub(crate) fn parse_script_with_options_source_type(
    source: &str,
    options: ScriptParserOptions,
    source_type: SourceType,
    skip_diagnostics: bool,
    occurrences: bool,
    unused: bool,
    exports: Option<&mut ModuleExportCapture>,
) -> ScriptParseResult {
    with_parsed_script_plain(source, source_type, |parsed| {
        analyze_repaired_script_plain(
            parsed,
            source,
            options,
            skip_diagnostics,
            occurrences,
            unused,
            exports,
        )
    })
}

/// Parse and repair a plain script block once, in the dialect its analysis reads.
pub(crate) fn with_parsed_script_plain<R>(
    source: &str,
    source_type: SourceType,
    analyze: impl FnOnce(&oxc_parser::ParserReturn<'_>) -> R,
) -> R {
    vize_carton::micro_count!("croquis.plain.parse_requests.bytes", source.len());
    let allocator = Allocator::default();
    let parsed = profile!(
        "croquis.script_plain.oxc_parse",
        parse_program_for_analysis(&allocator, source, source_type)
    );
    analyze(&parsed)
}

/// Analyze `ret`, a plain script parse already repaired for analysis.
pub(crate) fn analyze_repaired_script_plain(
    ret: &oxc_parser::ParserReturn<'_>,
    source: &str,
    options: ScriptParserOptions,
    skip_diagnostics: bool,
    occurrences: bool,
    unused: bool,
    mut exports: Option<&mut ModuleExportCapture>,
) -> ScriptParseResult {
    ModuleExportCapture::observe_if(exports.as_deref_mut(), ret);
    if ret.panicked {
        let mut result = ScriptParseResult {
            source_program_count: 1,
            ..Default::default()
        };
        if unused {
            result
                .setup_context
                .set_module_read_facts(super::ModuleReadFacts::Unknown);
        }
        return result;
    }

    let source_len = source.len() as u32;

    let mut result = ScriptParseResult {
        source_program_count: 1,
        occurrence_capture: occurrences.then(Default::default),
        bindings: BindingMetadata::new(), // Not script setup
        nuxt_static_render_script: Some(!ret.panicked && ret.diagnostics.is_empty()),
        is_non_setup_script: true, // Mark as non-setup script for violation detection
        skip_diagnostics,
        ..Default::default()
    };
    result.scopes.reserve_scopes(16);

    // Setup global scope hierarchy (universal → mod)
    profile!(
        "croquis.script_plain.global_scopes",
        setup_global_scopes(&mut result.scopes, source_len)
    );

    // Enter non-script-setup scope (parent: ~mod)
    result.scopes.enter_non_script_setup_scope(
        NonScriptSetupScopeData {
            is_ts: true,
            has_define_component: false,
        },
        0,
        source_len,
    );

    if !skip_diagnostics {
        super::extract::prepare_race_analysis(&mut result, &ret.program);
    }

    process::collect_options_api_component_metadata(
        &mut result,
        &ret.program,
        source,
        options.options_api,
        options.legacy_vue2,
    );

    if !skip_diagnostics {
        result.value_binding_origins = Some(Box::new(
            super::value_binding_origins::ValueBindingOrigins::new(
                result.scopes.current_id(),
                false,
            ),
        ));
    }

    // Process all statements
    profile!("croquis.script_plain.walk_statements", {
        for stmt in ret.program.body.iter() {
            ModuleExportCapture::observe_statement_if(exports.as_deref_mut(), stmt, false);
            process::process_statement(&mut result, stmt, source);
        }
    });

    if let Some(origins) = result.value_binding_origins.as_mut() {
        if !result.invalid_exports.is_empty() {
            origins.refuse();
        }
        origins.finish(&result.scopes);
    }

    // Mirror the setup path so non-setup scripts also keep typeof-anchored
    // types adjacent to their value bindings in any downstream emitters.
    profile!(
        "croquis.script_plain.resolve_type_hoisting",
        result.resolve_type_export_hoisting()
    );

    if !ret.diagnostics.is_empty() {
        if let Some(origins) = result.value_binding_origins.as_mut() {
            origins.refuse();
        }
        result.setup_context.clear_ssr_facts();
        result.occurrence_capture = None;
    }
    if unused {
        result.setup_context.set_module_read_facts(profile!(
            "croquis.script_plain.module_reads",
            if ret.diagnostics.is_empty() {
                super::module_reads::observe(&ret.program)
            } else {
                super::ModuleReadFacts::Unknown
            }
        ));
    }
    result.macros.invalidate_default_objects();
    result
}
