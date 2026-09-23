//! Real Chromium executes Rust-owned scenarios against both backends and the
//! pinned Vue compilers/runtimes. The dedicated Davinci Actions lane runs this
//! explicitly; ordinary Rust suites do not require a browser installation.

#![expect(
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    reason = "tests assert by panicking"
)]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

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

mod cases;
mod scope;

#[test]
#[ignore = "requires installed Chromium; enforced by the Davinci browser Actions lane"]
fn mounted_event_handlers_match_vue_reference_and_independent_traces() {
    let mut fixtures = Vec::new();
    for &(name, handler, setup, calls, increment) in cases::CASES {
        let source = format!(r#"<button id="target" @click="{handler}">{{{{ count }}}}</button>"#);
        fixtures.push(fixture(
            name, &source, handler, setup, calls, increment, false,
        ));
    }
    // The implicit handler binding must be popped before the sibling read.
    fixtures.push(fixture(
        "sibling-scope",
        r#"<div><button id="target" @click="save($event.type, $event.target.id)">{{ count }}</button><span>{{ $event.type }}</span></div>"#,
        "save($event.type, $event.target.id)",
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
    handler: &str,
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
        let code = compile(backend, mode, source);
        compiled.push(json!({"backend": backend, "mode": mode, "code": code}));
    }
    // Keep the observed upstream lexical-scope differences explicit. A
    // future upstream fix must fail these observations until they are retired.
    let reference_calls = match name {
        "named-function" | "class-self" => Some("none"),
        "var-function-scope" => Some("var-binding"),
        "inline-program-var" => Some("error:TypeError"),
        "switch-scope" => Some("error:ReferenceError"),
        _ => None,
    };
    let reference_expected = reference_calls.map(|calls| {
        json!({
            "vdom": expected(setup, calls, false, sibling, false),
            "vapor": expected(setup, calls, false, sibling, true),
        })
    });
    let native_handler = if name == "inline-program-var" {
        format!("$event => {{ {handler} }}")
    } else {
        handler.to_owned()
    };
    let mutant_code = cases::mutant_handler(name).map(|handler| {
        let source = format!(r#"<button id="target" @click="{handler}">{{{{ count }}}}</button>"#);
        compile("vapor", "prefix", &source)
    });
    json!({"name": name, "source": source, "setup": setup, "compiled": compiled,
        "nativeHandler": native_handler,
        "mutantCode": mutant_code,
        "referenceExpected": reference_expected,
        "expected": {"vdom": expected(setup, calls, increment, sibling, false),
                     "vapor": expected(setup, calls, increment, sibling, true)}})
}

fn compile(backend: &str, mode: &str, source: &str) -> String {
    let allocator = Allocator::new();
    if backend == "vdom" {
        let mut bindings = BindingMetadata {
            is_script_setup: true,
            ..Default::default()
        };
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
        assert!(errors.is_empty(), "{source}: {errors:?}");
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
            "{source}: {:?}",
            output.error_messages
        );
        output.code.to_string()
    }
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
            if let Some(error) = calls.strip_prefix("error:") {
                diagnostics.push(json!([phase, error]));
            } else if invalid && vapor {
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
            "sibling": (mounted && sibling).then_some(if second {"setup-b"} else {"setup-a"}),
            "present": mounted, "connected": mounted, "sameNode": mounted,
            "events": events, "diagnostics": diagnostics}));
    }
    Value::Array(snapshots)
}
