//! Real Chromium executes Rust-owned scenarios against both backends and the
//! pinned Vue compilers/runtimes. The dedicated Davinci Actions lane runs this
//! explicitly; ordinary Rust suites do not require a browser installation.

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::options::{BindingMetadata, BindingType};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[test]
#[ignore = "requires installed Chromium; enforced by the Davinci browser Actions lane"]
fn mounted_event_handlers_match_vue_reference_and_independent_traces() {
    let mut fixtures = Vec::new();
    for (name, handler, setup, calls, increment) in [
        ("method", "save", "value", "event", false),
        (
            "var-function-scope",
            "event => { const deliver = () => save($event.type, $event.target.id ?? 'missing'); { var $event = event } deliver() }",
            "value",
            "event",
            false,
        ),
        (
            "nested-function-var",
            "event => { (() => { var $event = event })(); save($event.type, $event.target) }",
            "value",
            "binding",
            false,
        ),
        (
            "rest-arrow",
            "(...$event) => save($event[0].type, $event[0].target.id)",
            "value",
            "event",
            false,
        ),
        (
            "rest-function",
            "function (...$event) { save($event[0].type, $event[0].target.id) }",
            "value",
            "event",
            false,
        ),
        (
            "destructure",
            "({ type: $event, target }) => save($event, target.id)",
            "value",
            "event",
            false,
        ),
        (
            "named-function",
            "function $event(event) { if (typeof $event === 'function') save(event.type, event.target.id) }",
            "value",
            "event",
            false,
        ),
        (
            "block-capture",
            "() => { { const $event = 42; } save($event.type, $event.target) }",
            "value",
            "binding",
            false,
        ),
        (
            "catch-capture",
            "() => { try { throw 42 } catch ($event) {} save($event.type, $event.target) }",
            "value",
            "binding",
            false,
        ),
        ("member", "actions.save", "value", "event", false),
        ("parenthesized", "(actions.save)", "value", "event", false),
        (
            "reference-statement",
            "save; count++",
            "value",
            "none",
            true,
        ),
        ("event-reference", "$event", "function", "event", false),
        ("event-member", "$event.target", "member", "event", false),
        (
            "event-computed",
            "$event['target']",
            "member",
            "event",
            false,
        ),
        ("event-optional", "$event?.target", "member", "event", false),
        (
            "inline",
            "save($event.type, $event.target.id)",
            "value",
            "event",
            false,
        ),
        (
            "statements",
            "count++; save($event.type, $event.target.id)",
            "value",
            "event",
            true,
        ),
        (
            "arrow",
            "event => save(event.type, event.target.id)",
            "value",
            "event",
            false,
        ),
        (
            "shadow",
            "$event => save($event.type, $event.target.id)",
            "value",
            "event",
            false,
        ),
        (
            "function",
            "function ($event) { save($event.type, $event.target.id) }",
            "value",
            "event",
            false,
        ),
        (
            "capture",
            "() => save($event.type, $event.target)",
            "value",
            "binding",
            false,
        ),
        (
            "shorthand",
            "() => save(({ $event }).$event.type, $event.target)",
            "value",
            "binding",
            false,
        ),
        (
            "nested-inline",
            "count++; (() => save($event.type, $event.target.id))()",
            "value",
            "event",
            true,
        ),
        ("missing-reference", "$event", "missing", "none", false),
        ("null-reference", "$event", "null", "none", false),
        ("value-reference", "$event", "number", "none", false),
    ] {
        let source = format!(r#"<button id="target" @click="{handler}">{{{{ count }}}}</button>"#);
        fixtures.push(fixture(name, &source, setup, calls, increment, false));
    }
    // The implicit handler binding must be popped before the sibling read.
    fixtures.push(fixture(
        "sibling-scope",
        r#"<div><button id="target" @click="save($event.type, $event.target.id)">{{ count }}</button><span>{{ $event.type }}</span></div>"#,
        "value", "event", false, true,
    ));
    let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support/davinci-event-handlers.mjs");
    let mut child = Command::new("node")
        .arg(runner)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start Chromium event contract runner");
    let written = child
        .stdin
        .take()
        .unwrap()
        .write_all(json!({"fixtures": fixtures}).to_string().as_bytes());
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    written.expect("deliver Rust-owned fixtures to browser runner");
    println!("{}", String::from_utf8_lossy(&output.stdout));
}

fn fixture(
    name: &str,
    source: &str,
    setup: &str,
    calls: &str,
    increment: bool,
    sibling: bool,
) -> Value {
    let mut compiled = Vec::new();
    for (backend, mode) in [
        ("vdom", "context"),
        ("vdom", "setup"),
        ("vdom", "cached"),
        ("vapor", "prefix"),
        ("vapor", "plain"),
    ] {
        if mode == "cached" && matches!(setup, "missing" | "null" | "number") {
            continue;
        }
        let allocator = Allocator::new();
        let code = if backend == "vdom" {
            let mut bindings = BindingMetadata::default();
            bindings.is_script_setup = true;
            for name in ["$event", "save", "actions", "count"] {
                bindings.bindings.insert(name.into(), BindingType::SetupLet);
            }
            let (_, errors, output) = compile_template_with_options(
                &allocator,
                source,
                DomCompilerOptions {
                    prefix_identifiers: true,
                    cache_handlers: mode == "cached",
                    binding_metadata: (mode == "setup").then_some(bindings),
                    ..Default::default()
                },
            );
            assert!(errors.is_empty(), "{name}: {errors:?}");
            format!("{}\n{}", output.preamble, output.code)
        } else {
            let output = compile_vapor(
                &allocator,
                source,
                VaporCompilerOptions {
                    prefix_identifiers: mode == "prefix",
                    ..Default::default()
                },
            );
            assert!(
                output.error_messages.is_empty(),
                "{name}: {:?}",
                output.error_messages
            );
            output.code.to_string()
        };
        compiled.push(json!({"backend": backend, "mode": mode, "code": code}));
    }
    // Keep incorrect upstream self-name / hoisted-var rewrites explicit. A
    // future upstream fix must fail these observations until they are retired.
    let reference_calls = match name {
        "named-function" => Some("none"),
        "var-function-scope" => Some("var-binding"),
        _ => None,
    };
    let reference_expected = reference_calls.map(|calls| {
        json!({
            "vdom": expected(setup, calls, false, sibling, false),
            "vapor": expected(setup, calls, false, sibling, true),
        })
    });
    json!({"name": name, "source": source, "setup": setup, "compiled": compiled,
        "referenceExpected": reference_expected,
        "expected": {"vdom": expected(setup, calls, increment, sibling, false),
                     "vapor": expected(setup, calls, increment, sibling, true)}})
}

fn expected(setup: &str, calls: &str, increment: bool, sibling: bool, vapor: bool) -> Value {
    let invalid = matches!(setup, "missing" | "null" | "number");
    let mut events = Vec::new();
    let mut diagnostics = Vec::new();
    let mut snapshots = Vec::new();
    let mut count = 0;
    if invalid && !vapor && setup == "number" {
        diagnostics.push(json!(["mount", "invalid-handler"]));
    }
    for phase in ["mount", "click-a", "replace", "click-b", "unmount"] {
        let second = matches!(phase, "replace" | "click-b" | "unmount");
        if phase.starts_with("click-") {
            if invalid && vapor {
                diagnostics.push(json!([phase, "TypeError"]));
            } else if !invalid {
                if increment {
                    count += 1;
                }
                let generation = if second { "b" } else { "a" };
                let args = if calls == "var-binding" {
                    json!([format!("setup-{generation}"), "missing"])
                } else if calls == "binding" {
                    json!([
                        format!("setup-{generation}"),
                        format!("binding-{generation}")
                    ])
                } else {
                    json!(["click", "target"])
                };
                if calls != "none" {
                    events.push(json!([generation, args]));
                }
            }
        }
        let mounted = phase != "unmount";
        snapshots.push(json!({"phase": phase, "count": count,
            "text": mounted.then(|| count.to_string()),
            "sibling": (mounted && sibling).then(|| if second {"setup-b"} else {"setup-a"}),
            "present": mounted, "connected": mounted, "sameNode": mounted,
            "events": events, "diagnostics": diagnostics}));
    }
    Value::Array(snapshots)
}
