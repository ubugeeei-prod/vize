//! Vue 2 pipe-filter codegen parity + zero-cost dialect gating.
//!
//! Compiled only under `--features legacy`; the whole file is gated so the
//! default Vue 3 build never sees it. Asserts that:
//!
//! - under dialect V2 (and V2.7), `{{ msg | capitalize }}` and
//!   `:id="raw | formatId"` lower to `@vue/compiler-core`'s Vue-2-compat filter
//!   output (`const _filter_x = _resolveFilter("x")` + `_filter_x(...)`), and
//! - under the default dialect V3, the *same* input stays bitwise-or with no
//!   `_resolveFilter` anywhere — proving the feature is inert for Vue 3 even in
//!   a `legacy`-enabled build.
#![cfg(feature = "legacy")]
// Integration test: plain `std::string::String` / `{out}` formatting is fine
// here (the crate's internal `vize_carton::String` rule does not apply to an
// out-of-crate test harness).
#![allow(clippy::disallowed_types, clippy::disallowed_macros)]

use vize_atelier_core::{CodegenOptions, TransformOptions, codegen, parser, transform};
use vize_carton::config::VueVersion;

fn compile(input: &str, dialect: VueVersion) -> String {
    let allocator = bumpalo::Bump::new();
    let (mut root, errors) = parser::parse(&allocator, input);
    assert!(errors.is_empty(), "parse errors: {errors:?}");

    let transform_opts = TransformOptions {
        prefix_identifiers: true,
        dialect,
        ..Default::default()
    };
    transform::transform(&allocator, &mut root, transform_opts, None);

    let codegen_opts = CodegenOptions {
        prefix_identifiers: true,
        ..Default::default()
    };
    let result = codegen::generate(&root, codegen_opts);
    let mut out = String::with_capacity(result.preamble.len() + result.code.len() + 1);
    out.push_str(result.preamble.as_str());
    out.push('\n');
    out.push_str(result.code.as_str());
    out
}

#[test]
fn v2_interpolation_single_filter() {
    let out = compile("<div>{{ message | capitalize }}</div>", VueVersion::V2);
    assert!(
        out.contains(r#"const _filter_capitalize = _resolveFilter("capitalize")"#),
        "{out}"
    );
    assert!(out.contains("_filter_capitalize(_ctx.message)"), "{out}");
}

#[test]
fn v2_interpolation_filter_with_args() {
    let out = compile("<div>{{ a | f(b) }}</div>", VueVersion::V2);
    assert!(
        out.contains(r#"const _filter_f = _resolveFilter("f")"#),
        "{out}"
    );
    // base and the in-paren arg are both prefixed; the base/arg comma has no
    // space (mirrors @vue/compiler-core's wrapFilter).
    assert!(out.contains("_filter_f(_ctx.a,_ctx.b)"), "{out}");
}

#[test]
fn v2_interpolation_filter_chain() {
    let out = compile("<div>{{ a | f | g(c) }}</div>", VueVersion::V2);
    assert!(
        out.contains(r#"const _filter_f = _resolveFilter("f")"#),
        "{out}"
    );
    assert!(
        out.contains(r#"const _filter_g = _resolveFilter("g")"#),
        "{out}"
    );
    assert!(out.contains("_filter_g(_filter_f(_ctx.a),_ctx.c)"), "{out}");
}

#[test]
fn v2_bind_filter() {
    let out = compile(r#"<div :id="raw | formatId"></div>"#, VueVersion::V2);
    assert!(
        out.contains(r#"const _filter_formatId = _resolveFilter("formatId")"#),
        "{out}"
    );
    assert!(out.contains("_filter_formatId(_ctx.raw)"), "{out}");
}

#[test]
fn v2_logical_or_is_not_a_filter() {
    // `||` must never be treated as a filter, even under V2.
    let out = compile("<div>{{ a || b }}</div>", VueVersion::V2);
    assert!(!out.contains("_resolveFilter"), "{out}");
    assert!(out.contains("_ctx.a || _ctx.b"), "{out}");
}

#[test]
fn v2_string_pipe_is_not_a_filter() {
    let out = compile(r#"<div>{{ a | f('a|b') }}</div>"#, VueVersion::V2);
    assert!(
        out.contains(r#"const _filter_f = _resolveFilter("f")"#),
        "{out}"
    );
    // The pipe inside the string literal is preserved verbatim.
    assert!(out.contains(r#"_filter_f(_ctx.a,'a|b')"#), "{out}");
}

#[test]
fn v2_dash_filter_name_is_valid_asset_id() {
    let out = compile("<div>{{ x | foo-bar }}</div>", VueVersion::V2);
    // `-` maps to `_` in the asset id but the resolved name keeps the dash.
    assert!(
        out.contains(r#"const _filter_foo_bar = _resolveFilter("foo-bar")"#),
        "{out}"
    );
    assert!(out.contains("_filter_foo_bar(_ctx.x)"), "{out}");
}

#[test]
fn v2_7_shares_the_v2_filter_dialect() {
    let out = compile("<div>{{ message | capitalize }}</div>", VueVersion::V2_7);
    assert!(out.contains("_filter_capitalize(_ctx.message)"), "{out}");
}

// --- Zero-cost: dialect V3 keeps `|` as bitwise-or, no filters ---------------

#[test]
fn v3_default_dialect_keeps_pipe_as_bitwise_or() {
    // Same input the V2 tests treat as a filter must stay bitwise-or under the
    // default Vue 3 dialect — even in this `legacy`-feature build.
    let out = compile("<div>{{ message | capitalize }}</div>", VueVersion::V3);
    assert!(!out.contains("_resolveFilter"), "{out}");
    assert!(!out.contains("_filter_"), "{out}");
    assert!(out.contains("_ctx.message | _ctx.capitalize"), "{out}");
}

#[test]
fn v3_bind_pipe_is_bitwise_or() {
    let out = compile(r#"<div :id="raw | formatId"></div>"#, VueVersion::V3);
    assert!(!out.contains("_resolveFilter"), "{out}");
    assert!(out.contains("_ctx.raw | _ctx.formatId"), "{out}");
}

#[test]
fn v2_and_v3_outputs_diverge_only_for_filters() {
    // A `|`-free template must produce identical output under V2 and V3,
    // demonstrating the dialect only changes filter handling.
    let v2 = compile("<div>{{ a + b }}</div>", VueVersion::V2);
    let v3 = compile("<div>{{ a + b }}</div>", VueVersion::V3);
    assert_eq!(v2, v3);
}
