//! The original #7887 SFC must register its CSS getter with its real renderer.

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
use vize_l0::String;

const ORIGINAL: &str =
    include_str!("../../../tests/_fixtures/differential/compiler/vapor-css-vars/App.vue.txt");
const FLAG: &str =
    include_str!("../../../tests/_fixtures/differential/compiler/vapor-css-vars/flag.vue.txt");
const TYPED: &str =
    include_str!("../../../tests/_fixtures/differential/compiler/vapor-css-vars/typed.vue.txt");

fn compile(source: &str, requested: bool, typed: bool, production: bool, ssr: bool) -> Value {
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "App.vue".into(),
            ..Default::default()
        },
    )
    .expect("parse original whole SFC");
    let mut options = SfcCompileOptions {
        vapor: requested,
        scope_id: Some("abc12345".into()),
        ..Default::default()
    };
    options.parse.filename = "App.vue".into();
    options.script.id = Some("App.vue".into());
    options.script.is_ts = typed;
    options.template.is_prod = production;
    options.template.ssr = ssr;
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
            SfcScriptOutputMode::InlineTemplate,
        )
        .expect("compile original component")
    };
    let plain = build(false);
    let mapped = build(true);
    assert!(plain.errors.is_empty(), "{:?}", plain.errors);
    assert_eq!(plain.map, None);
    assert_eq!(
        mapped.map.as_ref().expect("source map")["sources"],
        json!(["App.vue"])
    );
    let mut complete_mapped = serde_json::to_value(&mapped).expect("complete mapped result");
    complete_mapped["map"] = Value::Null;
    assert_eq!(
        serde_json::to_value(&plain).expect("complete plain result"),
        complete_mapped,
        "source mapping changes no code, CSS, diagnostics, bindings or artifacts"
    );
    let vapor = requested || source == ORIGINAL || source == TYPED;
    assert_eq!(plain.warnings.len(), usize::from(vapor && ssr));
    if vapor && ssr {
        assert_eq!(
            plain
                .warnings
                .first()
                .expect("fallback diagnostic")
                .code
                .as_deref(),
            Some("VAPOR_SSR_FALLBACK")
        );
    }
    json!({ "result": plain, "mapped": mapped })
}

#[test]
fn vapor_css_variables_mount_update_and_preserve_vdom_ssr() {
    let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
        "full"
    } else {
        "pr"
    };
    let destination = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/nextest")
        .join(profile);
    std::fs::create_dir_all(&destination).expect("runtime proof directory");
    let mut receipts = Vec::new();
    for production in [false, true] {
        let mut cases = Vec::new();
        for (name, source, requested, typed, vapor) in [
            ("reported", ORIGINAL, false, false, true),
            ("flag", FLAG, true, false, true),
            ("typed", TYPED, false, true, true),
            ("vdom", FLAG, false, false, false),
        ] {
            for ssr in [false, true] {
                cases.push(json!({
                    "name": name, "source": source, "vapor": vapor, "ssr": ssr,
                    "compiled": compile(source, requested, typed, production, ssr)
                }));
            }
        }
        let input = json!({ "production": production, "cases": cases });
        let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/tooling/support/vapor-css-vars-runtime.mjs");
        let mut child = Command::new("node")
            .arg(runner)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run real Vue component assertions");
        child
            .stdin
            .take()
            .expect("stdin")
            .write_all(input.to_string().as_bytes())
            .expect("write full component outputs");
        let output = child.wait_with_output().expect("runtime results");
        std::fs::write(
            destination.join(if production {
                "vapor-css-vars-process-production.json"
            } else {
                "vapor-css-vars-process-development.json"
            }),
            serde_json::to_vec(&json!({
                "input": input, "exitCode": output.status.code(),
                "success": output.status.success(),
                "stdoutBytes": output.stdout, "stderrBytes": output.stderr
            }))
            .expect("full original process result"),
        )
        .expect("retain failure and success custody before assertions");
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        let observations: Value =
            serde_json::from_slice(&output.stdout).expect("whole runtime rows");
        assert_eq!(observations.as_array().expect("rows").len(), 8);
        receipts.push(json!({ "input": input, "observations": observations }));
    }
    std::fs::write(
        destination.join("vapor-css-vars-runtime.json"),
        serde_json::to_vec(&receipts).expect("complete original sources and observations"),
    )
    .expect("preserve full runtime proof beside the JUnit artifact");
}
