//! Regression tests for the pre-parse CSS bracket nesting guard (#3105).
//!
//! The css_parse fuzz target found a 3.7KB stylesheet nesting brackets 192
//! deep through unclosed color-function chains. LightningCSS backtracks
//! exponentially on that shape (each nesting level roughly doubles the work),
//! so the reproducer burned >25s under fuzz instrumentation only to report a
//! parse error. The guard rejects such sources up front.
#![cfg(feature = "native")]

use vize_atelier_sfc::{CssCompileOptions, compile_css, parse_css_ast};
use vize_carton::String;

/// One repetition of the dominant pattern in the fuzz reproducer; each
/// repetition opens six brackets that are never closed.
const REPRODUCER_CHUNK: &str =
    "height: hwb(-het: hwb(max(60*\\ %\n (light-dark(fieldtext: hwb(-let: hwb(ma\n  ";

const NESTING_DEPTH_ERROR: &str =
    "CSS parse error: bracket nesting exceeds the supported depth of 32";

fn reproducer(repetitions: usize) -> String {
    let mut css = String::from(".a {");
    for _ in 0..repetitions {
        css.push_str(REPRODUCER_CHUNK);
    }
    css
}

#[test]
fn parse_css_ast_rejects_the_fuzz_timeout_shape_without_backtracking() {
    // 32 repetitions nest 193 brackets deep — the depth of the actual fuzz
    // artifact. Without the guard this exact call takes seconds of CPU.
    let result = parse_css_ast(&reproducer(32), &CssCompileOptions::default());
    assert!(result.ast.is_none());
    assert_eq!(result.errors, [NESTING_DEPTH_ERROR]);
    assert_eq!(result.warnings, Vec::<vize_carton::String>::new());

    // The smallest reproducer shape past the boundary is rejected the same way.
    let result = parse_css_ast(&reproducer(6), &CssCompileOptions::default());
    assert!(result.ast.is_none());
    assert_eq!(result.errors, [NESTING_DEPTH_ERROR]);
}

#[test]
fn parse_css_ast_keeps_accepting_the_documented_depth_boundary() {
    let nested = |depth: usize| {
        [
            ".a { --x: ",
            &"f(".repeat(depth),
            "1",
            &")".repeat(depth),
            "; }",
        ]
        .concat()
    };

    // `.a {` plus 31 function tokens sits exactly on the depth-32 boundary.
    let allowed = parse_css_ast(&nested(31), &CssCompileOptions::default());
    assert!(allowed.ast.is_some());
    assert_eq!(allowed.errors, Vec::<vize_carton::String>::new());

    let rejected = parse_css_ast(&nested(32), &CssCompileOptions::default());
    assert!(rejected.ast.is_none());
    assert_eq!(rejected.errors, [NESTING_DEPTH_ERROR]);
}

#[test]
fn compile_css_rejects_over_deep_sources_and_passes_them_through() {
    let css = reproducer(32);
    let result = compile_css(&css, &CssCompileOptions::default());
    assert_eq!(result.code, css);
    assert!(result.map.is_none());
    assert_eq!(result.css_vars, Vec::<vize_carton::String>::new());
    assert_eq!(result.errors, [NESTING_DEPTH_ERROR]);
    assert_eq!(result.warnings, Vec::<vize_carton::String>::new());
    assert!(result.exports.is_none());
}

#[test]
fn compile_css_still_compiles_realistic_nesting() {
    let css = ".a { .b { .c { width: calc(1px + min(2px, max(3px, 4px))); } } }";
    let result = compile_css(css, &CssCompileOptions::default());
    assert_eq!(result.errors, Vec::<vize_carton::String>::new());
    assert!(!result.code.is_empty());
}
