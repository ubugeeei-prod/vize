//! #7884: filename recursion and local setup directives in complete mounted SFCs.
#![expect(
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "whole source and raw runtime regression evidence"
)]
use serde_json::json;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode, compile_sfc_for_adapter, parse_sfc,
};

const TREE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-setup-resolution/Tree.vue.txt"
);
const IMPORTED: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-setup-resolution/Imported.vue.txt"
);
const CHILD: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-setup-resolution/Child.vue.txt"
);
const MIXED: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-setup-resolution/Mixed.vue.txt"
);

fn compile(
    source: &str,
    filename: &str,
    production: bool,
    inline: bool,
) -> vize_atelier_sfc::SfcCompileResult {
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.into(),
            ..Default::default()
        },
    )
    .expect("complete fixture");
    let mut options = SfcCompileOptions::default();
    options.parse.filename = filename.into();
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
        .expect("whole SFC compile")
    };
    let plain = build(false);
    let mapped = build(true);
    assert!(plain.errors.is_empty(), "{:?}", plain.errors);
    assert!(plain.warnings.is_empty(), "{:?}", plain.warnings);
    assert!(mapped.map.is_some(), "module map");
    let mut no_map = serde_json::to_value(mapped).expect("mapped result");
    *no_map.get_mut("map").expect("map field") = serde_json::Value::Null;
    assert_eq!(serde_json::to_value(&plain).expect("plain result"), no_map);
    plain
}

#[test]
fn recursive_original_and_binding_precedence_mount_like_locked_vue() {
    for production in [false, true] {
        for inline in [false, true] {
            for (fixture, filename, source) in [
                ("Tree", "Tree.vue", TREE),
                ("Imported", "Tree.vue", IMPORTED),
                ("Mixed", "Mixed.vue", MIXED),
            ] {
                let output = compile(source, filename, production, inline);
                let child_output = compile(CHILD, "Child.vue", production, inline);
                let input = json!({"fixture":fixture,"filename":filename,"source":source,"code":output.code,"child":{"filename":"Child.vue","source":CHILD,"code":child_output.code},"production":production,"inline":inline});
                let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/tooling/support/vapor-setup-resolution.mjs");
                let mut child = Command::new("node")
                    .arg(runner)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("locked Vue process");
                let write = child
                    .stdin
                    .take()
                    .expect("stdin")
                    .write_all(input.to_string().as_bytes());
                let result = child.wait_with_output().expect("reap runtime process");
                let destination = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../target/nextest")
                    .join(
                        if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
                            "full"
                        } else {
                            "pr"
                        },
                    );
                std::fs::create_dir_all(&destination).expect("custody directory");
                std::fs::write(destination.join(vize_l0::cstr!("vapor-setup-resolution-{fixture}-{production}-{inline}.json")), serde_json::to_vec(&json!({"input":input,"success":result.status.success(),"stdout":result.stdout,"stderr":result.stderr,"stdinError":write.as_ref().err().map(ToString::to_string)})).expect("whole raw packet")).expect("retain before assertion");
                write.expect("complete packet");
                assert!(
                    result.status.success(),
                    "{}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
        }
    }
}
