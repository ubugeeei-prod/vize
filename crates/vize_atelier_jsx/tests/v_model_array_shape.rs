//! `v-model` array elements outside `[value, argument?, modifiers?]` and
//! non-string modifiers are rejected instead of being dropped (#13167).
//!
//! `@vue/babel-plugin-jsx` reads only the first three elements and filters out
//! every modifier that is not a string literal, so `[a, "x", "y"]` binds model
//! `x` and `[a, ["trim", 1, m]]` keeps `{ trim }` with no diagnostic. Neither
//! dropped part takes effect anywhere, so every mode reports it.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::string_slice,
    reason = "tests assert by panicking; fixtures use std strings"
)]

use vize_atelier_jsx::{
    JsxCompatMode, JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx, lower_source,
};
use vize_l0::Allocator;

/// Each rejected source, the authored bytes the error points at, and its message.
const REJECTED: [(&str, &str, &str); 6] = [
    (
        "const A = () => <B v-model={[a, \"x\", \"y\"]}/>;",
        "\"y\"",
        "v-model array element `\"y\"` is not used; the array form is `[value, argument?, modifiers?]`.",
    ),
    (
        "const A = () => <B v-model={[a, \"x\", [\"trim\"], \"y\"]}/>;",
        "[\"trim\"]",
        "v-model array element `[\"trim\"]` is not used; the array form is `[value, argument?, modifiers?]`.",
    ),
    (
        "const A = () => <B v-model={[a, [\"trim\", 1, m]]}/>;",
        "1",
        "v-model modifier `1` must be a string literal.",
    ),
    (
        "const A = () => <input v-model={[a, [\"trim\", m]]}/>;",
        "m",
        "v-model modifier `m` must be a string literal.",
    ),
    (
        "const A = () => <B v-model={[a, [\"trim\", ...mods]]}/>;",
        "...mods",
        "v-model modifier `...mods` must be a string literal.",
    ),
    (
        "const A = () => <B v-models={[[a, \"x\", \"y\"]]}/>;",
        "\"y\"",
        "v-model array element `\"y\"` is not used; the array form is `[value, argument?, modifiers?]`.",
    ),
];

/// Shapes that use every element keep compiling without diagnostics.
const ACCEPTED: [&str; 6] = [
    "const A = () => <B v-model={[a]}/>;",
    "const A = () => <B v-model={[a, \"x\"]}/>;",
    "const A = () => <B v-model={[a, []]}/>;",
    "const A = () => <B v-model={[a, [\"trim\", \"lazy\"]]}/>;",
    "const A = () => <B v-model={[a, \"x\", [\"trim\"]]}/>;",
    "const A = () => <B v-models={[[a, \"x\", [\"trim\"]], [b]]}/>;",
];

fn configs() -> [JsxCompileConfig; 3] {
    [
        JsxCompileConfig::default(),
        JsxCompileConfig {
            compat: JsxCompatMode::Babel,
            ..Default::default()
        },
        JsxCompileConfig {
            default_mode: JsxOutputMode::Vapor,
            ..Default::default()
        },
    ]
}

#[test]
fn lowering_reports_the_unused_element_or_modifier_at_its_bytes() {
    for (source, at, message) in REJECTED {
        let bump = Allocator::new();
        let lowered = lower_source(&bump, bump.as_oxc(), source, JsxLang::Jsx);
        let errors: Vec<_> = lowered
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.is_error())
            .collect();
        assert_eq!(errors.len(), 1, "{source}: {errors:?}");
        assert_eq!(errors[0].message.as_str(), message, "{source}");
        assert_eq!(
            &source[errors[0].start as usize..errors[0].end as usize],
            at,
            "{source}"
        );
    }
}

#[test]
fn every_mode_rejects_the_model_instead_of_binding_part_of_it() {
    for (source, _, message) in REJECTED {
        for config in configs() {
            let bump = Allocator::new();
            let output = compile_jsx(&bump, source, JsxLang::Jsx, &config);
            assert!(
                output
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.message.as_str() == message),
                "{source} ({:?}, {:?}): {:?}",
                config.compat,
                config.default_mode,
                output.diagnostics
            );
            let code = output.module_code();
            assert!(!code.contains("onUpdate:"), "{source}: {code}");
            assert!(!code.contains("Modifiers"), "{source}: {code}");
        }
    }
}

#[test]
fn layouts_that_use_every_element_still_compile() {
    for source in ACCEPTED {
        for config in configs() {
            let bump = Allocator::new();
            let output = compile_jsx(&bump, source, JsxLang::Jsx, &config);
            assert!(
                output.diagnostics.is_empty(),
                "{source} ({:?}, {:?}): {:?}",
                config.compat,
                config.default_mode,
                output.diagnostics
            );
            assert!(output.module_code().contains("onUpdate:"), "{source}");
        }
    }
}
