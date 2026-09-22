//! The `fix(ssr)!` Vue 3.5 alignment, pinned two ways per changed shape:
//!
//! 1. the compiled SSR module is byte-identical to the pinned aligned output;
//! 2. rendered through the real Vue 3.5 server renderer
//!    (`tests/tooling/support/ssr-vue-render-diff.mjs`), it produces exactly
//!    the HTML `@vue/compiler-ssr` 3.5's output produces for the same state
//!    (attribute order inside a tag normalized), while the pinned pre-fix
//!    output either threw (`ReferenceError: _directives is not defined`,
//!    `k is not defined`) or rendered different HTML.
//!
//! The harness loads Vue from the `vue-stable` catalog (3.5) and asserts the
//! version, so a catalog bump fails loudly instead of silently re-pinning.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

#[path = "vue_ssr_render/fixtures.rs"]
mod fixtures;

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use vize_atelier_core::options::{BindingMetadata, BindingType};
use vize_atelier_ssr::{SsrCompilerOptions, compile_ssr_with_options};
use vize_s0::Allocator;

fn binding_type(name: &str) -> BindingType {
    match name {
        "setup-const" => BindingType::SetupConst,
        "setup-ref" => BindingType::SetupRef,
        other => panic!("unknown binding type {other}"),
    }
}

fn compile(fixture: &fixtures::Fixture) -> String {
    let mut options = SsrCompilerOptions::default();
    if !fixture.bindings.is_empty() {
        let mut metadata = BindingMetadata::default();
        for (name, binding) in fixture.bindings {
            metadata
                .bindings
                .insert((*name).into(), binding_type(binding));
        }
        options.binding_metadata = Some(metadata);
    }
    let allocator = Allocator::new();
    let (_, errors, result) = compile_ssr_with_options(&allocator, fixture.template, options);
    assert!(errors.is_empty(), "{}: {errors:?}", fixture.name);
    format!("{}{}", result.preamble, result.code)
}

fn json(text: &str) -> String {
    let mut out = String::from("\"");
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn case(fixture: &fixtures::Fixture, vize: &str) -> String {
    let setup = fixture
        .setup
        .iter()
        .map(|name| json(name))
        .collect::<Vec<_>>()
        .join(",");
    let bindings = if fixture.bindings.is_empty() {
        "null".to_owned()
    } else {
        let entries = fixture
            .bindings
            .iter()
            .map(|(name, kind)| format!("{}:{}", json(name), json(kind)))
            .collect::<Vec<_>>()
            .join(",");
        format!("{{{entries}}}")
    };
    format!(
        "{{\"name\":{},\"template\":{},\"vize\":{},\"legacy\":{},\"data\":{},\"setup\":[{setup}],\"bindings\":{bindings},\"attrs\":{}}}",
        json(fixture.name),
        json(fixture.template),
        json(vize),
        json(fixture.legacy),
        fixture.data,
        fixture.attrs,
    )
}

#[test]
fn aligned_ssr_shapes_compile_to_the_pinned_output() {
    for fixture in fixtures::FIXTURES {
        assert_eq!(compile(fixture), fixture.expected, "{}", fixture.name);
    }
}

#[test]
fn aligned_ssr_shapes_render_like_vue_while_the_old_output_did_not() {
    let cases = fixtures::FIXTURES
        .iter()
        .map(|fixture| case(fixture, &compile(fixture)))
        .collect::<Vec<_>>()
        .join(",");
    let input = format!("{{\"check\":true,\"cases\":[{cases}]}}");
    let mut child = Command::new("node")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/tooling/support/ssr-vue-render-diff.mjs"),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("node runs the SSR render differential");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(input.as_bytes())
        .expect("write cases");
    let output = child
        .wait_with_output()
        .expect("render differential output");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout),
    );
    let rendered = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        rendered.lines().count(),
        fixtures::FIXTURES.len(),
        "one rendered result per fixture:\n{rendered}"
    );
}
