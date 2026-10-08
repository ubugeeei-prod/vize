//! Whole #7882 component handles, lexical loop ownership and cleanup.
use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode, compile_sfc_for_adapter, parse_sfc,
};

const APP: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-for-template-refs-7882/App.vue.txt"
);
const CHILD: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-for-template-refs-7882/Child.vue.txt"
);
const LOOP: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-component-template-refs-7882/loop-and-scalar.vue.txt"
);
const CALLBACK: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-component-template-refs-7882/computed-prop-and-callback.vue.txt"
);

fn compile(source: &str, filename: &str, inline: bool, production: bool) -> Value {
    let descriptor = match parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.into(),
            ..Default::default()
        },
    ) {
        Ok(descriptor) => descriptor,
        Err(error) => return json!({"parseError": error}),
    };
    let mut options = SfcCompileOptions::default();
    options.parse.filename = filename.into();
    options.script.id = Some(filename.into());
    options.vapor = true;
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
            if inline {
                SfcScriptOutputMode::InlineTemplate
            } else {
                SfcScriptOutputMode::SeparateTemplate
            },
        )
    };
    json!({"parseError": null, "plain": build(false), "mapped": build(true)})
}

#[test]
fn original_component_refs_and_loop_scope_match_pinned_vue() {
    let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
        "full"
    } else {
        "pr"
    };
    let destination = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/nextest")
        .join(profile);
    std::fs::create_dir_all(&destination).expect("whole component-ref custody directory");
    for production in [false, true] {
        for inline in [false, true] {
            let cases: Vec<Value> = [
                ("original", "App.vue", APP),
                ("loop-and-scalar", "loop-and-scalar.vue", LOOP),
                (
                    "computed-prop-and-callback",
                    "computed-prop-and-callback.vue",
                    CALLBACK,
                ),
            ]
            .into_iter()
            .map(|(name, filename, source)| {
                json!({
                    "name": name, "filename": filename, "source": source,
                    "compiled": compile(source, filename, inline, production)
                })
            })
            .collect();
            let input = json!({"production": production, "inline": inline, "cases": cases,
                "child": {"source": CHILD, "filename": "Child.vue",
                    "compiled": compile(CHILD, "Child.vue", inline, production)}});
            let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/tooling/support/vapor-component-ref-runtime.mjs");
            let mut child = Command::new("node")
                .arg(runner)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("independent pinned Vue runtime child");
            let write = child
                .stdin
                .take()
                .expect("child stdin")
                .write_all(input.to_string().as_bytes());
            let output = child
                .wait_with_output()
                .expect("reap component-ref process");
            let name = vize_l0::cstr!("vapor-component-refs-{production}-{inline}.json");
            std::fs::write(destination.join(name), serde_json::to_vec(&json!({
                "input": input, "stdinWriteError": write.as_ref().err().map(ToString::to_string),
                "exit": output.status.to_string(), "success": output.status.success(),
                "stdoutBytes": output.stdout, "stderrBytes": output.stderr,
            })).expect("serialize complete process packet")).expect("save before assertions");
            write.expect("write whole original source and modules");
            assert!(
                output.status.success(),
                "stdout={:?}\nstderr={:?}",
                output.stdout,
                output.stderr
            );
            assert!(output.stderr.is_empty(), "{:?}", output.stderr);
            let receipt: Value =
                serde_json::from_slice(&output.stdout).expect("whole paired runtime result");
            assert_eq!(receipt.get("qualified"), Some(&json!(true)));
        }
    }
}
