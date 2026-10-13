//! #8652: hydrate complete source-built development SFCs without photo/text drift.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::expect_used,
    reason = "complete source-built compiler/runtime corpus observations"
)]

use serde_json::json;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_dom::DomCompilerOptions;
use vize_atelier_sfc::{
    SfcCompileOptions, SfcCompileResult, SfcParseOptions, TemplateCompileOptions, compile_sfc,
    parse_sfc,
};

const SOURCE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/dev-comment-hydration/groups.vue.txt"
);
const SLOTS: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/dev-comment-hydration/slots.vue.txt"
);
const ORIGINAL: &str = "eslint-disable-next-line @intlify/vue-i18n/no-raw-text";

#[test]
fn inline_slot_annotation_keeps_hydration_text_grouping() {
    let original = include_str!(
        "../../../tests/_fixtures/differential/compiler/dev-comment-hydration/inline-slot.vue.txt"
    );
    let non_ascii = include_str!(
        "../../../tests/_fixtures/differential/compiler/dev-comment-hydration/inline-slot-non-ascii.vue.txt"
    );
    let mut inputs = Vec::new();
    let mut observations = Vec::new();
    for (fixture, source) in [
        ("inline-slot.vue.txt", original),
        ("inline-slot-non-ascii.vue.txt", non_ascii),
    ] {
        let input = json!({"fixture": fixture, "source": source, "dom": compile(source, false), "ssr": compile(source, true)});
        let mut child = Command::new("node")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/tooling/support/dev-comment-inline-slot.mjs"),
            )
            .env("NODE_ENV", "development")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("real slot hydration");
        child
            .stdin
            .take()
            .expect("stdin")
            .write_all(&serde_json::to_vec(&input).expect("complete modules"))
            .expect("send modules");
        let output = child.wait_with_output().expect("slot hydration completed");
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        let observed: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("complete inline slot observation");
        inputs.push(input);
        observations.push(observed);
    }
    retain(
        "dev-comment-inline-slot-runtime.json",
        &json!(inputs),
        &json!(observations),
    );
}

fn compile(source: &str, ssr: bool) -> SfcCompileResult {
    let parse = SfcParseOptions {
        filename: "Groups.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("whole source SFC");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse,
            template: TemplateCompileOptions {
                ssr,
                is_prod: false,
                compiler_options: Some(DomCompilerOptions {
                    comments: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("whole development native SFC");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    result
}

#[test]
fn source_built_development_ssr_hydration_keeps_each_groups_images() {
    let annotations = [
        "eslint-disable rule",
        "eslint-disable-line rule",
        "eslint-disable-next-line rule",
        "eslint-enable rule",
        "oxlint-disable rule",
        "oxlint-disable-line rule",
        "oxlint-disable-next-line rule",
        "oxlint-enable rule",
        "ordinary",
        "eslint-disableish",
        "note eslint-disable",
    ];
    let cases: Vec<_> = [SOURCE, SLOTS]
        .into_iter()
        .flat_map(|original| {
            annotations.into_iter().map(move |comment| {
                let source = original.replace(ORIGINAL, comment);
                json!({ "comment": comment, "source": source,
            "dom": compile(&source, false), "ssr": compile(&source, true) })
            })
        })
        .collect();
    let mut child = Command::new("node")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/tooling/support/dev-comment-hydration.mjs"),
        )
        .env("NODE_ENV", "development")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("real development Vue hydration runtime");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(&serde_json::to_vec(&cases).expect("complete modules"))
        .expect("send complete compiler outputs");
    let output = child.wait_with_output().expect("hydration completed");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let observed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("all observations");
    assert_eq!(observed.as_array().expect("case list").len(), 22);
    retain(
        "dev-comment-hydration-runtime.json",
        &json!(cases),
        &observed,
    );
}

fn retain(name: &str, input: &serde_json::Value, observed: &serde_json::Value) {
    let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
        "full"
    } else {
        "pr"
    };
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../target/nextest/{profile}"));
    std::fs::create_dir_all(&directory).expect("test artifact directory");
    std::fs::write(
        directory.join(name),
        serde_json::to_vec(&json!({"input": input, "observations": observed}))
            .expect("complete compiler/runtime packet"),
    )
    .expect("retain all source, compiler and hydration observations");
}
