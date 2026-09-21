use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

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
    run_mounted(
        backend,
        source,
        json!({ "context": context, "steps": steps, "identities": identities }),
        experimental_patterned_template,
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
    run_mounted(
        backend,
        source,
        json!({ "context": context, "steps": steps, "identities": true, "slots": slots }),
        false,
    )
}

fn run_mounted(
    backend: &str,
    source: &str,
    mut input: Value,
    experimental_patterned_template: bool,
) -> Value {
    // `vapor-legacy` runs the Vapor runtime over the explicitly retained lane:
    // an empty binding-metadata map selects it without changing any binding.
    let legacy = backend == "vapor-legacy";
    let backend = if legacy { "vapor" } else { backend };
    let allocator = Allocator::new();
    let code = if backend == "vdom" {
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
        format!("{}\n{}", result.preamble, result.code)
    } else {
        let result = compile_vapor(
            &allocator,
            source,
            VaporCompilerOptions {
                prefix_identifiers: true,
                experimental_patterned_template,
                binding_metadata: legacy.then(Default::default),
                ..Default::default()
            },
        );
        assert!(
            result.error_messages.is_empty(),
            "Vapor errors: {:?}",
            result.error_messages
        );
        result.code.to_string()
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
