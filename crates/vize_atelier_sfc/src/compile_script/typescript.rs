//! TypeScript transformation utilities.
//!
//! This module handles transforming TypeScript code to JavaScript using OXC.

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer, TypeScriptOptions};
use vize_carton::{String, ToCompactString, profile};

use crate::module_map::{Runs, replace_traced, reprint_points};

/// Transform TypeScript code to JavaScript using OXC
pub fn transform_typescript_to_js(code: &str) -> String {
    strip_typescript(code, false).0
}

/// [`transform_typescript_to_js`] with the provenance of the JavaScript in
/// `code` (Davinci P3-9): the tokens oxc's codegen re-printed, through the
/// tab expansion. The JavaScript is byte-identical to the untraced call's.
pub(crate) fn transform_typescript_to_js_traced(code: &str) -> (String, Runs) {
    let (js, runs) = strip_typescript(code, true);
    (js, runs.unwrap_or_else(|| Runs::identity(code.len())))
}

/// Print `program` as JavaScript with its tabs expanded, plus the printed
/// text's provenance in `source` when `trace` asks for it.
fn print_javascript(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
    trace: bool,
) -> (String, Option<Runs>) {
    if !trace {
        let code = Codegen::new().build(program).code;
        return (code.replace('\t', "  ").into(), None);
    }
    let options = CodegenOptions {
        source_map_path: Some(std::path::PathBuf::from("module.ts")),
        ..CodegenOptions::default()
    };
    let printed = Codegen::new().with_options(options).build(program);
    let (expanded, tab_runs) = replace_traced(&printed.code, "\t", "  ");
    let runs = printed
        .map
        .map(|map| tab_runs.compose(&reprint_points(&map, &printed.code, source)));
    (expanded, Some(runs.unwrap_or_default()))
}

/// The JavaScript for `code` and, when `trace` is set and the code was
/// re-printed, its provenance in `code`. `None` provenance means `code` came
/// back unchanged.
fn strip_typescript(code: &str, trace: bool) -> (String, Option<Runs>) {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let parser = Parser::new(&allocator, code, source_type);
    let parse_result = profile!("atelier.script.ts.parse", parser.parse());

    if !parse_result.diagnostics.is_empty() {
        // If parsing fails, return original code
        return (code.to_compact_string(), None);
    }

    let mut program = parse_result.program;

    // Run semantic analysis to get symbols and scopes
    // `with_enum_eval(true)`: OXC 0.142's TypeScript enum transform panics unless
    // the `Scoping` it is handed carries evaluated enum member values.
    let semantic_ret = profile!(
        "atelier.script.ts.semantic",
        SemanticBuilder::new()
            .with_excess_capacity(2.0)
            .with_enum_eval(true)
            .build(&program)
    );

    if !semantic_ret.diagnostics.is_empty() {
        // If semantic analysis fails, return original code
        return (code.to_compact_string(), None);
    }

    let scoping = semantic_ret.semantic.into_scoping();

    // Transform TypeScript to JavaScript
    // Strip all TypeScript syntax including type parameters (generics)
    let transform_options = TransformOptions {
        typescript: TypeScriptOptions {
            only_remove_type_imports: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let ret = profile!(
        "atelier.script.ts.transform",
        Transformer::new(&allocator, std::path::Path::new(""), &transform_options)
            .build_with_scoping(scoping, &mut program)
    );

    if !ret.diagnostics.is_empty() {
        // If transformation fails, return original code
        return (code.to_compact_string(), None);
    }

    // Generate JavaScript code
    // Replace tabs with 2 spaces for consistent indentation
    profile!(
        "atelier.script.ts.codegen",
        print_javascript(&program, code, trace)
    )
}

/// Report whether `code` already parses as plain ECMAScript.
///
/// This is a parse-only check (no semantic pass, no codegen): the emitter uses
/// it to skip a re-print of output that is already JavaScript. `SourceType::mjs`
/// is the shape the emitter produces — ESM, no JSX, no TypeScript — so any
/// leftover TypeScript syntax surfaces as a parse error and the caller falls
/// back to the full strip.
pub fn is_plain_javascript(code: &str) -> bool {
    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, code, SourceType::mjs());
    profile!("atelier.script.js.probe", parser.parse())
        .diagnostics
        .is_empty()
}

/// Strip TypeScript for the emitter's output guarantee, tolerating semantic
/// diagnostics.
///
/// [`transform_typescript_to_js`] bails whenever `SemanticBuilder` reports
/// anything, which is the right call while compiling a script (a redeclaration
/// there means the pipeline misread the source). It is the wrong call here:
/// this runs on already-generated module code, and the JavaScript-side pass it
/// replaces — Vite's `transformWithOxc` — never ran a semantic check at all.
/// Bailing would hand TypeScript to the bundler over a diagnostic that a
/// syntax-only strip does not care about.
///
/// Returns `None` when the code does not parse as TypeScript, in which case
/// there is nothing this can do and the caller keeps the original.
pub(crate) fn strip_typescript_for_emitter(code: &str) -> Option<String> {
    strip_typescript_for_emitter_traced(code, false).map(|(js, _)| js)
}

/// [`strip_typescript_for_emitter`] with, when `trace` is set, the printed
/// JavaScript's token provenance in `code` (Davinci P3-9 source maps).
pub(crate) fn strip_typescript_for_emitter_traced(
    code: &str,
    trace: bool,
) -> Option<(String, Option<Runs>)> {
    let allocator = Allocator::default();
    let parse_result = Parser::new(&allocator, code, SourceType::ts()).parse();
    if !parse_result.diagnostics.is_empty() {
        return None;
    }

    let mut program = parse_result.program;
    // See `transform_typescript_to_js` for why `with_enum_eval` is required.
    let scoping = SemanticBuilder::new()
        .with_excess_capacity(2.0)
        .with_enum_eval(true)
        .build(&program)
        .semantic
        .into_scoping();

    let transform_options = TransformOptions {
        typescript: TypeScriptOptions {
            only_remove_type_imports: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let ret = Transformer::new(&allocator, std::path::Path::new(""), &transform_options)
        .build_with_scoping(scoping, &mut program);
    if !ret.diagnostics.is_empty() {
        return None;
    }

    Some(print_javascript(&program, code, trace))
}

/// Guarantee that emitted module code contains no TypeScript syntax.
///
/// The overwhelming majority of emitter output is already plain JavaScript
/// (the script pipeline strips TypeScript while compiling), so the owned
/// `code` is handed straight back without a copy. Only the residue — most
/// notably `<script lang="uts">`, a lang `is_ts_lang` does not recognise — pays
/// for the strip.
///
/// Output that parses as neither JavaScript nor TypeScript (`lang="jsx"`,
/// `lang="tsx"`, `lang="coffee"`) is returned unchanged for the bundler to deal
/// with, exactly as the emitter produced it.
pub fn ensure_javascript_output(code: String) -> String {
    if is_plain_javascript(&code) {
        return code;
    }
    strip_typescript_for_emitter(&code).unwrap_or(code)
}
