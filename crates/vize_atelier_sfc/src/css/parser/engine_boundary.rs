//! Panic isolation for the LightningCSS engine boundary.
//!
//! The `css_parse` fuzz target found two crash classes that reproduce inside
//! upstream crates on their latest releases (#3276, #3280; tracked for
//! retirement in #3295):
//!
//! 1. `Percentage::parse` hits `unreachable!()` when a math function in a
//!    percentage-typed slot does not constant-fold to a plain percentage
//!    (`lch(sign(-50%))`, `abs(-50%)` as a color component, `opacity:
//!    calc(sign(-50%))`, mixed-type `min(-50%, 2)`, ...). This fires in every
//!    build profile, so without a boundary a stray style block crashes the
//!    CLI and the LSP process.
//! 2. `hsl_to_rgb` debug-asserts hue into `[0, 1]`; a non-finite hue from
//!    `hsl(1e40 0 0)` or `hsl(calc(1e20 * 1e20) 0 0)` trips it in builds with
//!    debug assertions (tests, fuzzing).
//!
//! The defense has two layers because the release profile builds with
//! `panic = "abort"`, where `catch_unwind` cannot help:
//!
//! - `value_guard` rejects the empirically crashing math-function shapes in
//!   color-function and opacity contexts before parsing, in every profile;
//! - the `catch_unwind` boundary here converts any remaining engine panic
//!   into an explicit error result in unwinding profiles (dev, test, ci),
//!   covering percentage-typed slots the guard does not model.
//!
//! An exact pre-parse guard is impossible byte-wise — the panic surface is
//! LightningCSS's calc type algebra — so the guard prefers the realistic
//! authoring surface and the boundary plus the fuzz-target skip-list cover
//! the rest; the caller-facing messages link the tracking issue.

use std::panic::{AssertUnwindSafe, catch_unwind};

mod value_guard;

use lightningcss::targets::Targets;
use serde_json::Value;
use vize_carton::{String, ToCompactString};

use super::{
    CssAstInternalResult, CssInternalResult, bundle_css_internal, compile_css_internal,
    parse_css_ast_internal, print_css_ast_internal,
};

/// Shared detail appended to every boundary error message.
const ENGINE_PANIC_DETAIL: &str =
    "the CSS engine hit an internal defect (upstream lightningcss panic; see vize issue #3295)";

fn engine_panic_message(operation: &str) -> String {
    let mut message = String::from(operation);
    message.push_str(ENGINE_PANIC_DETAIL);
    message
}

/// Run `parse_css_ast_internal`, converting an engine panic into an error AST
/// result instead of crashing the process.
pub(crate) fn parse_css_ast_guarded(
    css: &str,
    filename: &str,
    custom_media: bool,
    css_modules: bool,
) -> CssAstInternalResult {
    if value_guard::css_contains_crashing_math_function(css) {
        return CssAstInternalResult {
            ast: None,
            errors: vec![String::from(value_guard::MATH_FUNCTION_GUARD_ERROR)],
            warnings: vec![],
        };
    }
    catch_unwind(AssertUnwindSafe(|| {
        parse_css_ast_internal(css, filename, custom_media, css_modules)
    }))
    .unwrap_or_else(|_| CssAstInternalResult {
        ast: None,
        errors: vec![engine_panic_message("CSS parse error: ")],
        warnings: vec![],
    })
}

/// Run `print_css_ast_internal` behind the panic boundary.
pub(crate) fn print_css_ast_guarded(
    ast: Value,
    minify: bool,
    targets: Targets,
) -> CssInternalResult {
    catch_unwind(AssertUnwindSafe(|| {
        print_css_ast_internal(ast, minify, targets)
    }))
    .unwrap_or_else(|_| CssInternalResult {
        code: String::from(""),
        errors: vec![engine_panic_message("CSS print error: ")],
        exports: None,
    })
}

/// Run `compile_css_internal` behind the panic boundary. Mirrors the parse
/// error path: the authored source is passed through untouched next to the
/// error, so downstream stages keep operating on the original block.
pub(crate) fn compile_css_guarded(
    css: &str,
    filename: &str,
    minify: bool,
    targets: Targets,
    custom_media: bool,
    css_modules: bool,
) -> CssInternalResult {
    if value_guard::css_contains_crashing_math_function(css) {
        return CssInternalResult {
            code: css.to_compact_string(),
            errors: vec![String::from(value_guard::MATH_FUNCTION_GUARD_ERROR)],
            exports: None,
        };
    }
    catch_unwind(AssertUnwindSafe(|| {
        compile_css_internal(css, filename, minify, targets, custom_media, css_modules)
    }))
    .unwrap_or_else(|_| CssInternalResult {
        code: css.to_compact_string(),
        errors: vec![engine_panic_message("CSS compile error: ")],
        exports: None,
    })
}

/// Run `bundle_css_internal` behind the panic boundary.
pub(crate) fn bundle_css_guarded(
    entry_path: &str,
    minify: bool,
    targets: Targets,
    css_modules: bool,
    custom_media: bool,
) -> CssInternalResult {
    catch_unwind(AssertUnwindSafe(|| {
        bundle_css_internal(entry_path, minify, targets, css_modules, custom_media)
    }))
    .unwrap_or_else(|_| CssInternalResult {
        code: String::from(""),
        errors: vec![engine_panic_message("CSS bundle error: ")],
        exports: None,
    })
}
