//! #7821: whole original SFCs retain runtime page metadata outside Nuxt.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "complete original compiler and runtime regression observations"
)]

use serde_json::json;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcCompileResult, SfcParseOptions,
    SfcScriptOutputMode, TemplateCompileOptions, compile_sfc_for_adapter_with_experimental_options,
    compile_sfc_for_adapter_with_nuxt_page_meta, parse_sfc,
};

macro_rules! source {
    ($name:literal) => {
        include_str!(concat!(
            "../../../tests/_fixtures/differential/compiler/page-meta-runtime/",
            $name,
            ".vue.txt"
        ))
    };
}
const CASES: &[(&str, &str)] = &[
    ("Page", source!("Page")),
    ("global", source!("global")),
    ("mixed", source!("mixed")),
    ("local", source!("local")),
    ("alias", source!("alias")),
    ("shadowed", source!("shadowed")),
];

fn compile(source: &str, shape: &str, nuxt: bool, map: bool) -> SfcCompileResult {
    let ssr = matches!(shape, "ssr" | "vapor-ssr");
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "Page.vue".into(),
            ..Default::default()
        },
    )
    .expect("parse full original SFC");
    let options = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: "Page.vue".into(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some("Page.vue".into()),
            ..Default::default()
        },
        template: TemplateCompileOptions {
            ssr,
            is_prod: true,
            ..Default::default()
        },
        vapor: shape.starts_with("vapor"),
        ..Default::default()
    };
    let compiler = if nuxt {
        compile_sfc_for_adapter_with_nuxt_page_meta
    } else {
        compile_sfc_for_adapter_with_experimental_options
    };
    let result = compiler(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        Default::default(),
        CodegenOptions {
            source_map: map,
            ..Default::default()
        },
        if matches!(shape, "inline" | "vapor") {
            SfcScriptOutputMode::InlineTemplate
        } else {
            SfcScriptOutputMode::SeparateTemplate
        },
        Default::default(),
    )
    .expect("compile complete module");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert_eq!(
        result.warnings.len(),
        usize::from(shape == "vapor-ssr"),
        "{:?}",
        result.warnings
    );
    let script_erased = nuxt && (source == source!("Page") || source == source!("global"));
    assert_eq!(
        result.map.is_some(),
        map && !script_erased,
        "{shape}/nuxt={nuxt}"
    );
    if let Some(map) = &result.map {
        assert_eq!(map["sources"], json!(["Page.vue"]));
    }
    result
}

#[test]
fn ordinary_and_nuxt_page_meta_keep_distinct_complete_outputs() {
    for shape in ["module", "inline", "ssr", "vapor", "vapor-ssr"] {
        for &(name, source) in CASES {
            let plain = compile(source, shape, false, false);
            let mapped = compile(source, shape, false, true);
            assert_eq!(
                plain.code, mapped.code,
                "{shape}/{name}: map changes no code"
            );
            assert!(
                plain.macro_artifacts.is_empty(),
                "ordinary calls are not Nuxt artifacts"
            );
            let nuxt = compile(source, shape, true, true);
            assert_eq!(nuxt.code, compile(source, shape, true, false).code);
            let extracted = matches!(name, "Page" | "global" | "mixed");
            assert_eq!(
                nuxt.macro_artifacts.len(),
                usize::from(extracted),
                "{shape}/{name}"
            );
            if extracted {
                let artifact = nuxt.macro_artifacts.first().expect("actual Nuxt artifact");
                assert_eq!(artifact.kind, "nuxt.definePageMeta");
                assert_eq!(artifact.name, "definePageMeta");
                assert_eq!(
                    source
                        .get(artifact.start..artifact.end)
                        .expect("original artifact span"),
                    artifact.source.as_str()
                );
            } else {
                assert_eq!(
                    plain.code, nuxt.code,
                    "local and aliased calls remain runtime calls"
                );
            }
        }
    }
}

#[test]
fn original_page_meta_calls_mount_and_ssr_like_the_official_compiler() {
    let mut cases = Vec::new();
    for shape in ["module", "inline", "ssr", "vapor", "vapor-ssr"] {
        for &(name, source) in CASES {
            for nuxt in [false, true] {
                let result = compile(source, shape, nuxt, true);
                cases.push(json!({ "name": name, "source": source, "shape": shape,
                    "nuxt": nuxt, "code": result.code,
                    "artifact": result.macro_artifacts.first().and_then(|artifact| artifact.module_code.as_ref()) }));
            }
        }
    }
    let mut child = Command::new("node")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/tooling/support/page-meta-runtime.mjs"),
        )
        .env("NODE_ENV", "production")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run independent Vue component oracle");
    child
        .stdin
        .take()
        .expect("oracle stdin")
        .write_all(
            serde_json::to_string(&cases)
                .expect("serialize complete modules")
                .as_bytes(),
        )
        .expect("send complete source modules");
    let output = child.wait_with_output().expect("oracle exits");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).lines().count(), 60);
    let observations: Vec<serde_json::Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).expect("whole runtime observation"))
        .collect();
    let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
        "full"
    } else {
        "pr"
    };
    let proof = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/nextest")
        .join(profile);
    std::fs::create_dir_all(&proof).expect("runtime proof directory");
    std::fs::write(
        proof.join("page-meta-runtime.json"),
        serde_json::to_vec(&json!({ "cases": cases, "observations": observations }))
            .expect("serialize complete source and runtime observations"),
    )
    .expect("preserve actual component oracle observations with the JUnit artifact");
}
