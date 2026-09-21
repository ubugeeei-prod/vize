use serde_json::{Map, Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

/// A child component: registered name, template, declared props and emits.
pub(crate) type Child<'s> = (&'s str, &'s str, &'s [&'s str], &'s [&'s str]);

pub(crate) fn mounted_trace(backend: &str, source: &str, context: Value, steps: Value) -> Value {
    mounted_trace_with_patterned_template(backend, source, context, steps, false)
}

pub(crate) fn mounted_trace_with_patterned_template(
    backend: &str,
    source: &str,
    context: Value,
    steps: Value,
    experimental_patterned_template: bool,
) -> Value {
    mounted_trace_with_identity(
        backend,
        source,
        context,
        steps,
        experimental_patterned_template,
        false,
    )
}

pub(crate) fn mounted_trace_with_identity(
    backend: &str,
    source: &str,
    context: Value,
    steps: Value,
    experimental_patterned_template: bool,
    identities: bool,
) -> Value {
    let code = compile(backend, source, experimental_patterned_template);
    run(
        backend,
        code,
        json!({ "context": context, "steps": steps, "identities": identities }),
    )
}

/// Mounts the template under a parent that supplies `slots`
/// (`{name: {"text": ...} | {"prop": ...}}`), with identity observations.
pub(crate) fn mounted_trace_with_slots(
    backend: &str,
    source: &str,
    context: Value,
    steps: Value,
    slots: Value,
) -> Value {
    let code = compile(backend, source, false);
    run(
        backend,
        code,
        json!({ "context": context, "steps": steps, "identities": true, "slots": slots }),
    )
}

/// Parent and children compiled by the same backend and lane.
pub(crate) fn mounted_trace_with_components(
    backend: &str,
    source: &str,
    children: &[Child<'_>],
    context: Value,
    steps: Value,
    identities: bool,
) -> Value {
    let mut components = Map::new();
    for (name, template, props, emits) in children {
        components.insert(
            (*name).to_owned(),
            json!({"code": compile(backend, template, false), "props": props, "emits": emits}),
        );
    }
    let code = compile(backend, source, false);
    run(
        backend,
        code,
        json!({
            "context": context, "steps": steps, "identities": identities,
            "components": components
        }),
    )
}

fn compile(backend: &str, source: &str, experimental_patterned_template: bool) -> String {
    // `vapor-legacy` runs the Vapor runtime over the explicitly retained lane:
    // the explicit selector changes no binding or expression.
    let allocator = Allocator::new();
    if backend == "vdom" {
        let (_, errors, result) = compile_template_with_options(
            &allocator,
            source,
            DomCompilerOptions {
                prefix_identifiers: true,
                experimental_patterned_template,
                ..Default::default()
            },
        );
        assert!(errors.is_empty(), "DOM errors: {errors:?}");
        return format!("{}\n{}", result.preamble, result.code);
    }
    let result = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            experimental_patterned_template,
            davinci_retained_lane: backend == "vapor-legacy",
            ..Default::default()
        },
    );
    assert!(
        result.error_messages.is_empty(),
        "Vapor errors: {:?}",
        result.error_messages
    );
    result.code.to_string()
}

fn run(backend: &str, code: String, mut input: Value) -> Value {
    let backend = if backend == "vapor-legacy" {
        "vapor"
    } else {
        backend
    };
    let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support/davinci-mounted-trace.mjs");
    let mut child = Command::new("node")
        .arg(runner)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start mounted runtime runner (install workspace JS dependencies first)");
    input["backend"] = json!(backend);
    input["code"] = json!(code);
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{backend} mounted runner failed:\n{}\ncode:\n{code}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stdout)))
}
