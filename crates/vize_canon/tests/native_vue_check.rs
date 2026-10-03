//! Mandatory actual Corsa diagnostics from the genuine original SFC owners.
#![expect(clippy::unwrap_used, reason = "fixture assertions panic")]

use corsa::runtime::block_on;
use lsp_types::{DocumentDiagnosticReport, DocumentDiagnosticReportResult};
use std::path::{Path, PathBuf};
use vize_canon::{CorsaBridge, CorsaBridgeConfig};
use vize_l0::{
    Allocator, Span, String,
    config::{VueDialect, VueVersion},
    cstr,
    line_index::LineBreaks,
};
use vize_l1::{SurfaceParseOptions, container::vue::DescriptorOptions};
use vize_l1_to_l2::native_file::lower_sfc_native;

#[path = "native_vue_check/configuration.rs"]
mod configuration;
#[path = "native_vue_check/const_semantics.rs"]
mod const_semantics;

#[path = "native_vue_check/jsdoc_semantics.rs"]
mod jsdoc_semantics;
#[cfg(unix)]
#[path = "native_vue_check/options_custody.rs"]
mod options_custody;
#[cfg(unix)]
#[path = "native_vue_check/ref_semantics.rs"]
mod ref_semantics;

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}
fn backend() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
            project_root: Some(root),
        },
    )
    .unwrap()
}
fn project(root: &Path, config: serde_json::Value) -> CorsaBridge {
    std::fs::write(
        root.join("tsconfig.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    // The ordinary bridge may initialize before a SFC projection is requested.
    std::fs::write(root.join("seed.ts"), "export {};").unwrap();
    CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(backend()),
        working_dir: Some(root.to_path_buf()),
        ..Default::default()
    })
}
fn configured(root: &Path) -> CorsaBridge {
    project(
        root,
        serde_json::json!({"compilerOptions":{"strict":true,"target":"ESNext","module":"ESNext","moduleResolution":"Bundler","types":[],"noEmit":true,"skipLibCheck":true,"allowJs":true,"checkJs":true},"include":["**/*"]}),
    )
}

#[test]
fn real_native_vue_checker_retains_complete_js_ts_diagnostics_and_original_owners() {
    for ts in [false, true] {
        let arena = Allocator::default();
        let root = tempfile::TempDir::new().unwrap();
        let bridge = configured(root.path());
        let script = "/*😀*/ let 日本語 = 1; let other=2;";
        let source = cstr!(
            "\r\n<template><div>{{{{日本語.missing}}}}<span>{{{{other.missing}}}}</span></div></template>\r\n<script setup{}>{script}</script>",
            if ts { " lang=ts" } else { "" }
        );
        let path = root.path().join("Original.vue");
        std::fs::write(&path, source.as_bytes()).unwrap();
        let observed = lower_sfc_native(&arena, &source, options());
        assert!(observed.admitted().is_some(), "{:?}", observed.issues());
        block_on(bridge.spawn()).unwrap();
        let result =
            block_on(bridge.check_native_vue(observed.admitted().unwrap(), &path)).unwrap();
        let custody = result.diagnosing_configuration();
        assert!(!custody.session().session_id.is_empty());
        for state in [custody.before(), custody.after()] {
            assert_eq!(
                state.project().compiler_options,
                result.configuration().options
            );
            assert_eq!(
                Path::new(&state.project().config_file_name),
                result.diagnostic_configuration_path()
            );
            assert!(
                state
                    .response()
                    .projects
                    .iter()
                    .any(|project| project.id == state.project().id)
            );
        }
        assert!(std::ptr::eq(
            result.projection().original().observation(),
            &observed
        ));
        assert!(std::ptr::eq(
            result.projection().file(),
            observed.file().unwrap().file()
        ));
        assert_eq!(result.source_path(), path.canonicalize().unwrap());
        assert_eq!(
            result.source_uri(),
            cstr!("file://{}", path.canonicalize().unwrap().display())
        );
        assert_eq!(
            result.projection_uri(),
            cstr!(
                "file://{}.{}",
                path.canonicalize().unwrap().display(),
                if ts { "ts" } else { "mjs" }
            )
        );
        assert_eq!(result.source_digest().len(), 64);
        assert_eq!(result.configuration().options["strict"], true);
        assert_eq!(
            result.diagnostic_configuration_path(),
            root.path().join("tsconfig.json").canonicalize().unwrap()
        );
        assert_eq!(
            result.project().config_file_name,
            root.path()
                .join("tsconfig.json")
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
        );
        let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(full)) =
            result.report()
        else {
            panic!("complete raw report required")
        };
        let actual = &full.full_document_diagnostic_report.items;
        let text = result.projection().document().as_str();
        let diagnostic = |needle: &str, length: usize, code: u32, message: String| {
            let byte = text.find(needle).unwrap();
            let (line, character) = LineBreaks::Lsp.offset_to_position(text, byte);
            let (end_line, end_character) = LineBreaks::Lsp.offset_to_position(text, byte + length);
            serde_json::json!({"range":{"start":{"line":line,"character":character},"end":{"line":end_line,"character":end_character}},"severity":1,"code":code,"source":"ts","message":message})
        };
        let mut expected = Vec::new();
        expected.push(diagnostic(
            "missing",
            7,
            2339,
            String::from("Property 'missing' does not exist on type 'number'."),
        ));
        let second = text.rfind("missing").unwrap();
        let (line, character) = LineBreaks::Lsp.offset_to_position(text, second);
        expected.push(serde_json::json!({"range":{"start":{"line":line,"character":character},"end":{"line":line,"character":character+7}},"severity":1,"code":2339,"source":"ts","message":"Property 'missing' does not exist on type 'number'."}));
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(&expected).unwrap()
        );
        let mut authored = Vec::new();
        for (start, _) in source.match_indices("missing") {
            authored.push(Ok(Span::new(start as u32, start as u32 + 7)));
        }
        assert_eq!(result.authored_spans(), authored);
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
        assert!(
            !path
                .with_file_name(if ts {
                    "Original.vue.ts"
                } else {
                    "Original.vue.mjs"
                })
                .exists()
        );
        block_on(bridge.shutdown()).unwrap();
    }
}
