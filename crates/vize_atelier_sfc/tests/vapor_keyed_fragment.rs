//! Original #7883 inputs must replace the keyed owner and dispose its scope.

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
use vize_l0::StdVec;

const APP: &str =
    include_str!("../../../tests/_fixtures/differential/compiler/vapor-keyed-fragment/App.vue.txt");
const COUNTER: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-keyed-fragment/Counter.vue.txt"
);
const ELEMENT: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-keyed-fragment/element.vue.txt"
);
const STABLE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-keyed-fragment/stable.vue.txt"
);

fn compile(source: &str, filename: &str, inline: bool, production: bool) -> Value {
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.into(),
            ..Default::default()
        },
    )
    .expect("whole original SFC");
    let mut options = SfcCompileOptions::default();
    options.parse.filename = filename.into();
    options.script.id = Some(filename.into());
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
        .expect("compile whole SFC")
    };
    let plain = build(false);
    let mapped = build(true);
    assert!(plain.errors.is_empty(), "{:?}", plain.errors);
    assert!(plain.warnings.is_empty(), "{:?}", plain.warnings);
    assert_eq!(plain.map, None);
    assert_eq!(
        mapped
            .map
            .as_ref()
            .expect("whole script module map")
            .get("sources")
            .expect("map sources"),
        json!([filename])
    );
    let mut without_map = serde_json::to_value(&mapped).expect("all mapped fields");
    *without_map.get_mut("map").expect("complete map field") = Value::Null;
    assert_eq!(
        serde_json::to_value(&plain).expect("all plain fields"),
        without_map
    );
    json!({"plain":plain, "mapped":mapped})
}

#[test]
fn original_keyed_component_and_element_replace_and_dispose_scopes() {
    let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
        "full"
    } else {
        "pr"
    };
    let destination = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/nextest")
        .join(profile);
    std::fs::create_dir_all(&destination).expect("raw runtime custody directory");
    for production in [false, true] {
        for inline in [false, true] {
            let cases: StdVec<Value> = [("component", "App.vue", APP), ("element", "element.vue", ELEMENT), ("stable", "stable.vue", STABLE)]
                .into_iter().map(|(name, filename, source)| json!({"name":name,"filename":filename,"source":source,"compiled":compile(source,filename,inline,production)})).collect();
            let input = json!({"production":production,"inline":inline,"cases":cases,"counter":{"source":COUNTER,"filename":"Counter.vue","compiled":compile(COUNTER,"Counter.vue",inline,production)}});
            let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/tooling/support/vapor-keyed-fragment-runtime.mjs");
            let mut child = Command::new("node")
                .arg(runner)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("real pinned Vue child");
            let write = child
                .stdin
                .take()
                .expect("stdin")
                .write_all(input.to_string().as_bytes());
            let output = child.wait_with_output().expect("reap real process");
            let name = vize_l0::cstr!("vapor-keyed-fragment-{production}-{inline}.json");
            std::fs::write(destination.join(name), serde_json::to_vec(&json!({"input":input,"stdinWriteError":write.as_ref().err().map(ToString::to_string),"exit":output.status.to_string(),"success":output.status.success(),"stdoutBytes":output.stdout,"stderrBytes":output.stderr})).expect("whole raw receipt")).expect("custody before runtime assertions");
            write.expect("write full input");
            assert!(
                output.status.success(),
                "{:?}\n{:?}",
                output.stderr,
                output.stdout
            );
            let rows: Value =
                serde_json::from_slice(&output.stdout).expect("full runtime observations");
            assert_eq!(
                rows.get("observations")
                    .expect("observations")
                    .as_array()
                    .expect("rows")
                    .len(),
                3
            );
        }
    }
}
