//! Public parse entry points for script setup and plain (Options API) scripts.

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::globals::setup_global_scopes;
use super::process;
use super::result::{ScriptParseResult, ScriptParserOptions};
use crate::croquis::BindingMetadata;
use crate::scope::{NonScriptSetupScopeData, ScopeChain, ScriptSetupScopeData};
use vize_carton::{CompactString, profile};

/// Parse script setup source code using OXC parser with an optional generic parameter.
///
/// `generic` is the value from `<script setup generic="T">` attribute, if present.
///
/// This is a high-performance alternative to string-based analysis,
/// providing accurate AST-based detection with proper span tracking.
pub fn parse_script_setup_with_generic(source: &str, generic: Option<&str>) -> ScriptParseResult {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();

    let ret = profile!(
        "croquis.script_setup.oxc_parse",
        Parser::new(&allocator, source, source_type).parse()
    );

    if ret.panicked {
        return ScriptParseResult::default();
    }

    analyze_script_setup_program(&ret.program, source, generic)
}

/// Analyze an already-parsed script setup program.
///
/// This is the parse-free core of [`parse_script_setup_with_generic`]: callers
/// that already hold an oxc `Program` for the same source (e.g. the SFC
/// compiler's parse-once pipeline) can run the binding/scope analysis without
/// paying for another parse. `source` must be the exact text the program was
/// parsed from.
pub fn analyze_script_setup_program(
    program: &Program<'_>,
    source: &str,
    generic: Option<&str>,
) -> ScriptParseResult {
    let source_len = source.len() as u32;

    let mut result = ScriptParseResult {
        bindings: BindingMetadata::script_setup(),
        scopes: ScopeChain::with_capacity(16),
        ..Default::default()
    };

    // Setup global scope hierarchy (universal → mod)
    profile!(
        "croquis.script_setup.global_scopes",
        setup_global_scopes(&mut result.scopes, source_len)
    );

    // Enter script setup scope (parent: ~mod)
    result.scopes.enter_script_setup_scope(
        ScriptSetupScopeData {
            is_ts: true,
            is_async: false,
            generic: generic.map(CompactString::new),
        },
        0,
        source_len,
    );

    // Process all statements
    profile!("croquis.script_setup.walk_statements", {
        for stmt in program.body.iter() {
            process::process_statement(&mut result, stmt, source);
        }
    });

    // After every binding is known, demote any `type` / `interface` that
    // references a setup-scope value via `typeof` so the virtual TS keeps
    // it inside `__setup` instead of hoisting it to module scope.
    profile!(
        "croquis.script_setup.resolve_type_hoisting",
        result.resolve_type_export_hoisting()
    );

    result
}

/// Parse script setup source code using OXC parser.
///
/// This is a high-performance alternative to string-based analysis,
/// providing accurate AST-based detection with proper span tracking.
pub fn parse_script_setup(source: &str) -> ScriptParseResult {
    parse_script_setup_with_generic(source, None)
}

/// Parse non-script-setup (Options API) source code using OXC parser.
pub fn parse_script(source: &str) -> ScriptParseResult {
    parse_script_with_options(source, ScriptParserOptions::default())
}

/// Parse non-script-setup source code using OXC parser with explicit options.
pub fn parse_script_with_options(source: &str, options: ScriptParserOptions) -> ScriptParseResult {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();

    let ret = profile!(
        "croquis.script_plain.oxc_parse",
        Parser::new(&allocator, source, source_type).parse()
    );

    if ret.panicked {
        return ScriptParseResult::default();
    }

    let source_len = source.len() as u32;

    let mut result = ScriptParseResult {
        bindings: BindingMetadata::new(), // Not script setup
        scopes: ScopeChain::with_capacity(16),
        is_non_setup_script: true, // Mark as non-setup script for violation detection
        ..Default::default()
    };

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

    process::collect_options_api_component_metadata(
        &mut result,
        &ret.program,
        options.options_api,
        options.legacy_vue2,
    );

    // Process all statements
    profile!("croquis.script_plain.walk_statements", {
        for stmt in ret.program.body.iter() {
            process::process_statement(&mut result, stmt, source);
        }
    });

    // Mirror the setup path so non-setup scripts also keep typeof-anchored
    // types adjacent to their value bindings in any downstream emitters.
    profile!(
        "croquis.script_plain.resolve_type_hoisting",
        result.resolve_type_export_hoisting()
    );

    result
}
