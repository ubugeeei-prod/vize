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
//! Native release artifacts unwind so this boundary also protects CLI, LSP
//! and Node consumers from engine defects outside the known value guard.
//! The pre-parse guard avoids known expensive/panicking inputs; it is not a
//! complete model of upstream calc type algebra. Consumers overriding the
//! panic strategy to `abort` lose the recovery guarantee. Browser WASM builds
//! do not include the native LightningCSS engine and keep their abort profile.

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

#[inline]
fn recover_engine<T>(body: impl FnOnce() -> T, fallback: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(body)).unwrap_or_else(|_| fallback())
}

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
    recover_engine(
        || parse_css_ast_internal(css, filename, custom_media, css_modules),
        || CssAstInternalResult {
            ast: None,
            errors: vec![engine_panic_message("CSS parse error: ")],
            warnings: vec![],
        },
    )
}

/// Run `print_css_ast_internal` behind the panic boundary.
pub(crate) fn print_css_ast_guarded(
    ast: Value,
    minify: bool,
    targets: Targets,
) -> CssInternalResult {
    recover_engine(
        || print_css_ast_internal(ast, minify, targets),
        || CssInternalResult {
            code: String::from(""),
            errors: vec![engine_panic_message("CSS print error: ")],
            exports: None,
        },
    )
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
    recover_engine(
        || compile_css_internal(css, filename, minify, targets, custom_media, css_modules),
        || CssInternalResult {
            code: css.to_compact_string(),
            errors: vec![engine_panic_message("CSS compile error: ")],
            exports: None,
        },
    )
}

/// Run `bundle_css_internal` behind the panic boundary.
pub(crate) fn bundle_css_guarded(
    entry_path: &str,
    minify: bool,
    targets: Targets,
    css_modules: bool,
    custom_media: bool,
) -> CssInternalResult {
    recover_engine(
        || bundle_css_internal(entry_path, minify, targets, css_modules, custom_media),
        || CssInternalResult {
            code: String::from(""),
            errors: vec![engine_panic_message("CSS bundle error: ")],
            exports: None,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injected_engine_panic_recovers_without_hiding_an_escaping_panic() {
        let recovered = catch_unwind(AssertUnwindSafe(|| {
            recover_engine(
                || -> CssAstInternalResult { panic!("injected engine failure") },
                || CssAstInternalResult {
                    ast: None,
                    errors: vec![engine_panic_message("CSS parse error: ")],
                    warnings: vec![],
                },
            )
        }))
        .expect("the actual production recovery helper catches its engine panic");
        assert!(recovered.ast.is_none());
        assert_eq!(
            recovered.errors,
            [
                "CSS parse error: the CSS engine hit an internal defect (upstream lightningcss panic; see vize issue #3295)"
            ]
        );
        assert!(recovered.warnings.is_empty());
        for (operation, source) in [
            ("CSS compile error: ", "complete original CSS"),
            ("CSS print error: ", ""),
            ("CSS bundle error: ", ""),
        ] {
            let result = recover_engine(
                || -> CssInternalResult { panic!("injected output failure") },
                || CssInternalResult {
                    code: source.to_compact_string(),
                    errors: vec![engine_panic_message(operation)],
                    exports: None,
                },
            );
            assert_eq!(result.code, source);
            assert_eq!(result.errors, [engine_panic_message(operation)]);
            assert!(result.exports.is_none());
        }
        assert_eq!(
            recover_engine(|| 17, || panic!("successful engine must not run fallback")),
            17
        );
        let escaped = catch_unwind(|| {
            recover_engine(
                || -> () { panic!("injected engine failure") },
                || panic!("uncaught integration fallback defect"),
            )
        });
        assert!(
            escaped.is_err(),
            "an actual escaping panic remains a failure"
        );
    }
}
