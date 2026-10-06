//! #7885: real original parent/child event delivery, with whole result custody.

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::{
    CodegenOptions, TemplateSyntaxMode,
    steps::expression::{is_event_handler_reference_expression, is_function_expression},
};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode, compile_sfc_for_adapter, parse_sfc,
};
use vize_l0::{String, cstr};

const ROOT: &str = "../../../tests/_fixtures/differential/compiler/vapor-component-listeners/";
const APP: &str = include_str!(concat!(
    "../../../tests/_fixtures/differential/compiler/vapor-component-listeners/",
    "App.vue.txt"
));
const CHILD: &str = include_str!(concat!(
    "../../../tests/_fixtures/differential/compiler/vapor-component-listeners/",
    "Child.vue.txt"
));
const VDOM: &str = include_str!(concat!(
    "../../../tests/_fixtures/differential/compiler/vapor-component-listeners/",
    "Vdom.vue.txt"
));
const VDOM_CHILD: &str = include_str!(concat!(
    "../../../tests/_fixtures/differential/compiler/vapor-component-listeners/",
    "VdomChild.vue.txt"
));
const CALLBACKS: &str = include_str!(concat!(
    "../../../tests/_fixtures/differential/compiler/vapor-component-listeners/",
    "Callbacks.vue.txt"
));
const SPREAD: &str = include_str!(concat!(
    "../../../tests/_fixtures/differential/compiler/vapor-component-listeners/",
    "Spread.vue.txt"
));

#[test]
fn whole_function_shapes_keep_typed_callbacks_and_reject_statement_prefixes() {
    let sources = [
        "(a) => log.push(a)",
        "(a: number) => log.push(a)",
        "(a: number): void => log.push(a)",
        "async (a: number): Promise<void> => { await log(a) }",
        "function (a: number): void { log(a) }",
        "<T>(a: T) => a",
        "(a: number) => log(a) // retained comment",
        "log",
        "handlers['a;b']",
        "log(a)",
        "log(a); next()",
        "(a: number) => log(a); next()",
        "(a: number => log(a)",
    ];
    assert_eq!(
        sources.map(is_function_expression),
        [
            true, true, true, true, true, true, true, false, false, false, false, false, false
        ]
    );
    assert_eq!(
        sources.map(is_event_handler_reference_expression),
        [
            false, false, false, false, false, false, false, true, true, false, false, false, false
        ]
    );
}

fn compile(
    filename: &str,
    source: &str,
    vapor: bool,
    production: bool,
    output: SfcScriptOutputMode,
) -> Value {
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.into(),
            ..Default::default()
        },
    )
    .expect("parse whole pinned SFC");
    let mut options = SfcCompileOptions {
        vapor,
        ..Default::default()
    };
    options.parse.filename = filename.into();
    options.script.id = Some(filename.into());
    options.script.is_ts = true;
    options.template.is_prod = production;
    let build = |map| {
        compile_sfc_for_adapter(
            &descriptor,
            options.clone(),
            TemplateSyntaxMode::Standard,
            Default::default(),
            CodegenOptions {
                source_map: map,
                ..Default::default()
            },
            output,
        )
        .expect("compile complete public component result")
    };
    json!({ "source": source, "filename": filename, "plain": build(false), "mapped": build(true) })
}

fn git_identity(argument: &str) -> Value {
    let output = Command::new("git")
        .args(["rev-parse", argument])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("capture actual compiled source identity");
    assert_eq!(output.status.code(), Some(0));
    json!(String::from_utf8_lossy(&output.stdout).trim())
}

#[test]
fn original_component_listeners_mount_in_both_output_modes() {
    let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
        "full"
    } else {
        "pr"
    };
    let destination = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/nextest")
        .join(profile);
    std::fs::create_dir_all(&destination).expect("whole runtime evidence directory");
    let custody = json!({ "head": git_identity("HEAD"), "tree": git_identity("HEAD^{tree}"),
        "parent": git_identity("HEAD^"), "fixtureRoot": ROOT });
    let mut receipts = Vec::new();
    for production in [false, true] {
        let mut cases = Vec::new();
        for (mode, output) in [
            ("inline", SfcScriptOutputMode::InlineTemplate),
            ("separate", SfcScriptOutputMode::SeparateTemplate),
        ] {
            for (name, app, child, vapor) in [
                ("reported", APP, CHILD, true),
                ("vdom", VDOM, VDOM_CHILD, false),
                ("callbacks", CALLBACKS, CHILD, true),
                ("spread", SPREAD, CHILD, true),
            ] {
                cases.push(json!({ "name": name, "mode": mode, "vapor": vapor,
                    "app": compile("App.vue", app, vapor, production, output),
                    "child": compile("Child.vue", child, vapor, production, output) }));
            }
        }
        let input = json!({ "production": production, "cases": cases, "sourceCustody": custody });
        let label = if production {
            "production"
        } else {
            "development"
        };
        let input_path =
            destination.join(cstr!("vapor-component-listeners-input-{label}.json").as_str());
        std::fs::write(
            &input_path,
            serde_json::to_vec(&input).expect("full input JSON"),
        )
        .expect("retain complete sources and both full results before runtime assertions");
        let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/tooling/support/vapor-component-listeners.mjs");
        let mut child = Command::new("node")
            .arg(runner)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("execute actual official and source-built components");
        let write = child
            .stdin
            .take()
            .expect("child input")
            .write_all(input.to_string().as_bytes());
        let output = child
            .wait_with_output()
            .expect("retain runtime exit and streams");
        std::fs::write(
            destination.join(std::format!(
                "vapor-component-listeners-process-{label}.json"
            )),
            serde_json::to_vec(&json!({ "input": input, "exitCode": output.status.code(),
                "exitStatus": output.status.to_string(), "success": output.status.success(),
                "stdinWriteError": write.as_ref().err().map(ToString::to_string),
                "stdoutBytes": output.stdout, "stderrBytes": output.stderr }))
            .expect("complete original process custody"),
        )
        .expect("save genuine failure or success before assertion");
        assert!(write.is_ok(), "{write:?}");
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        let evidence: Value =
            serde_json::from_slice(&output.stdout).expect("whole runtime evidence");
        assert_eq!(evidence["stage"], "complete");
        assert_eq!(
            evidence["rows"]
                .as_array()
                .expect("complete row vector")
                .len(),
            8
        );
        receipts.push(evidence);
    }
    std::fs::write(
        destination.join("vapor-component-listeners-runtime.json"),
        serde_json::to_vec(&receipts).expect("both complete runtime processes"),
    )
    .expect("preserve full runtime witness alongside JUnit");
}
