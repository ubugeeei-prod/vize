//! #7888's whole original SFC and reactive attribute/property merge controls.
#![expect(
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "whole original compiler result and raw child-process regression evidence"
)]
use serde_json::json;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcScriptOutputMode, compile_sfc_for_adapter, parse_sfc,
};

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-attribute-modifiers/App.vue.txt"
);
const REACTIVE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-attribute-modifiers/Reactive.vue.txt"
);

#[test]
fn original_attribute_modifiers_and_reactive_sources_mount_like_vue() {
    for production in [false, true] {
        for inline in [false, true] {
            for (filename, source) in [("App.vue", ORIGINAL), ("Reactive.vue", REACTIVE)] {
                let descriptor = parse_sfc(source, Default::default()).expect("whole fixture");
                let mut options = SfcCompileOptions::default();
                options.parse.filename = filename.into();
                options.vapor = true;
                options.template.is_prod = production;
                let output = compile_sfc_for_adapter(
                    &descriptor,
                    options,
                    TemplateSyntaxMode::Standard,
                    Default::default(),
                    CodegenOptions::default(),
                    if inline {
                        SfcScriptOutputMode::InlineTemplate
                    } else {
                        SfcScriptOutputMode::SeparateTemplate
                    },
                )
                .expect("whole SFC compilation");
                assert!(output.errors.is_empty(), "{:?}", output.errors);
                let input = json!({"filename":filename, "source":source, "code":output.code, "production":production, "inline":inline});
                let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/tooling/support/vapor-attribute-modifiers.mjs");
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
                let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../target/nextest")
                    .join(
                        if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
                            "full"
                        } else {
                            "pr"
                        },
                    );
                std::fs::create_dir_all(&evidence).expect("evidence directory");
                std::fs::write(evidence.join(vize_l0::cstr!("vapor-attribute-modifiers-{filename}-{production}-{inline}.json")), serde_json::to_vec(&json!({"input":input,"success":result.status.success(),"stdout":result.stdout,"stderr":result.stderr,"stdinError":write.as_ref().err().map(ToString::to_string)})).expect("raw packet")).expect("retain raw result");
                write.expect("complete input");
                assert!(
                    result.status.success(),
                    "{}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
        }
    }
}
