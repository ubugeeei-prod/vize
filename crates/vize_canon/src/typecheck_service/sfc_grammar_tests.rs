use super::{
    CorsaBridge, SfcDiagnosticSeverity, SfcTypeCheckResult, TypeCheckService,
    TypeCheckServiceOptions,
};
use crate::corsa_bridge::CorsaBridgeConfig;
use davinci_test_support::original_diagnosing_process as native;
use serde_json::{Value, json};

fn packet(result: &SfcTypeCheckResult) -> Value {
    json!({
        "errorCount": result.error_count,
        "warningCount": result.warning_count,
        "virtualTs": result.virtual_ts,
        "diagnostics": result.diagnostics.iter().map(|diagnostic| json!({
            "message": diagnostic.message,
            "severity": format!("{:?}", diagnostic.severity),
            "start": diagnostic.start,
            "end": diagnostic.end,
            "code": diagnostic.code,
            "related": diagnostic.related.iter().map(|related| json!({
                "message": related.message,
                "filename": related.filename,
                "start": related.start,
                "end": related.end,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

#[test]
fn adjacent_quoted_attributes_reach_the_native_service_with_identical_output() {
    if !native::enabled() {
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let service = TypeCheckService {
        bridge: std::sync::Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(native::runtime()),
            working_dir: Some(root.path().to_path_buf()),
            ..Default::default()
        })),
    };
    let options = TypeCheckServiceOptions::default();
    corsa::runtime::block_on(async {
        service.bridge.spawn().await.unwrap();
        let reference = service
            .check_sfc(
                r#"<template><div id="a" class="b">ok</div></template>"#,
                "SfcGrammar.vue",
                &options,
            )
            .await
            .unwrap();
        assert!(reference.virtual_ts.is_some());
        let expected = packet(&reference);
        for attributes in [r#"id="a"class="b""#, "id='a'class='b'", "id=\"a\"class='b'"] {
            let source = format!("<template><div {attributes}>ok</div></template>");
            let actual = service
                .check_sfc(&source, "SfcGrammar.vue", &options)
                .await
                .unwrap();
            assert_eq!(packet(&actual), expected, "{source}");
        }
        service.shutdown().await.unwrap();
    });
}

#[test]
fn adjacent_quoted_attributes_keep_the_exact_malformed_service_diagnostic() {
    let service = TypeCheckService {
        bridge: std::sync::Arc::new(CorsaBridge::new()),
    };
    let result = corsa::runtime::block_on(service.check_sfc(
        r#"<template><div id="a"class="b"></template>"#,
        "SfcGrammar.vue",
        &TypeCheckServiceOptions::default(),
    ))
    .unwrap();
    assert_eq!(
        packet(&result),
        json!({
            "errorCount": 1,
            "warningCount": 0,
            "virtualTs": null,
            "diagnostics": [{
                "message": "Template parse error: Element is missing end tag.",
                "severity": format!("{:?}", SfcDiagnosticSeverity::Error),
                "start": 10,
                "end": 31,
                "code": "template-parse-error",
                "related": [],
            }],
        })
    );
}
