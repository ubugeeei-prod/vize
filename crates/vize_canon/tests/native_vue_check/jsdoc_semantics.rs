//! Preserve unmasked original fixtures and independently prove admitted JSDoc copies.

use super::*;
use vize_canon::NativeVueError;
use vize_l4::targets::ts::vue::VueProjectionError;

#[test]
fn original_jsdoc_vectors_remain_recorded_but_actual_template_bindings_refuse() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("jsdoc_refusal.json")).unwrap();
    let root = tempfile::TempDir::new().unwrap();
    let bridge = configured(root.path());
    let arena = Allocator::default();
    for name in [
        "historicalOriginalJs",
        "historicalInheritedJs",
        "genuineJsdocRef",
    ] {
        let fixture = &fixtures[name];
        let source = fixture["source"].as_str().unwrap();
        let path = root.path().join("Original.vue");
        std::fs::write(&path, source).unwrap();
        let original = lower_sfc_native(&arena, source, options());
        assert!(original.admitted().is_some(), "{:?}", original.issues());
        let script = original.scripts().first().unwrap();
        assert_eq!(script.block().source(), fixture["script"].as_str().unwrap());
        let program = script.syntax().unwrap().admitted_program().unwrap();
        assert!(program.has_jsdoc_comments());
        assert_eq!(program.source(), script.block().source());
        let checked = block_on(bridge.check_native_vue(original.admitted().unwrap(), &path));
        assert!(matches!(
            checked,
            Err(NativeVueError::Projection(
                VueProjectionError::UnsupportedTemplateBindings
            ))
        ));
        let embed = original.template().unwrap().embeds().first().unwrap();
        let resolution = original
            .file()
            .unwrap()
            .file()
            .expression(embed.node.unwrap())
            .unwrap();
        assert!(std::ptr::eq(
            resolution.table().unwrap().expression().ast,
            embed.syntax.expression().unwrap()
        ));
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
        assert!(!root.path().join("Original.vue.mjs").exists());
    }
}

#[test]
fn original_jsdoc_script_and_literal_template_keep_every_real_diagnostic_and_authored_range() {
    let root = tempfile::TempDir::new().unwrap();
    let bridge = configured(root.path());
    let script =
        "/** @type {number} */ let 日本語 = 'bad'; 日本語.missing; let other=2; other.missing;";
    let source =
        cstr!("\r\n<script setup>{script}</script>\r\n<template>{{{{(1+1).missing}}}}</template>");
    let path = root.path().join("Original.vue");
    std::fs::write(&path, source.as_bytes()).unwrap();
    let arena = Allocator::default();
    let original = lower_sfc_native(&arena, &source, options());
    assert!(original.admitted().is_some(), "{:?}", original.issues());
    block_on(bridge.spawn()).unwrap();
    let checked = block_on(bridge.check_native_vue(original.admitted().unwrap(), &path)).unwrap();
    let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(full)) =
        checked.report()
    else {
        panic!("complete raw report")
    };
    assert_eq!(
        serde_json::to_value(&full.full_document_diagnostic_report.items).unwrap(),
        serde_json::json!([
            {"range":{"start":{"line":0,"character":26},"end":{"line":0,"character":29}},"severity":1,"code":2322,"source":"ts","message":"Type 'string' is not assignable to type 'number'."},
            {"range":{"start":{"line":0,"character":43},"end":{"line":0,"character":50}},"severity":1,"code":2339,"source":"ts","message":"Property 'missing' does not exist on type 'number'."},
            {"range":{"start":{"line":0,"character":71},"end":{"line":0,"character":78}},"severity":1,"code":2339,"source":"ts","message":"Property 'missing' does not exist on type 'number'."},
            {"range":{"start":{"line":4,"character":6},"end":{"line":4,"character":13}},"severity":1,"code":2339,"source":"ts","message":"Property 'missing' does not exist on type 'number'."}
        ])
    );
    let start = source.find("日本語 =").unwrap() as u32;
    let mut authored = vec![Ok(Span::new(start, start + "日本語".len() as u32))];
    for (start, _) in source.match_indices("missing") {
        authored.push(Ok(Span::new(start as u32, start as u32 + 7)));
    }
    assert_eq!(checked.authored_spans(), authored);
    assert_eq!(
        checked.projection().document().as_str(),
        cstr!("{script}\n;\nexport {{}};\nvoid (\n(1+1).missing\n);\n")
    );
    assert!(std::ptr::eq(
        checked.projection().original().observation(),
        &original
    ));
    assert!(std::ptr::eq(
        checked.projection().file(),
        original.file().unwrap().file()
    ));
    assert_eq!(
        checked.diagnostic_configuration_path(),
        root.path().join("tsconfig.json").canonicalize().unwrap()
    );
    assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
    assert!(!root.path().join("Original.vue.mjs").exists());
    block_on(bridge.shutdown()).unwrap();
}

#[cfg(unix)]
#[test]
fn genuine_jsdoc_ref_script_literal_copies_keep_full_error_and_unused_hint() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let dependencies = repo.join("docs/node_modules");
    let package: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dependencies.join("vue/package.json")).unwrap())
            .unwrap();
    assert_eq!(package["version"], "3.5.35");
    let root = tempfile::TempDir::new().unwrap();
    std::os::unix::fs::symlink(&dependencies, root.path().join("node_modules")).unwrap();
    let bridge = configured(root.path());
    let script = "/** @type {import('vue').Ref<number>} */ let value=1;";
    let arena = Allocator::default();
    block_on(bridge.spawn()).unwrap();
    for template in ["", "<template>{{1}}</template>"] {
        let source = cstr!("<script setup>{script}</script>{template}");
        let path = root.path().join("Original.vue");
        std::fs::write(&path, source.as_bytes()).unwrap();
        let original = lower_sfc_native(&arena, &source, options());
        assert!(original.admitted().is_some(), "{:?}", original.issues());
        let checked =
            block_on(bridge.check_native_vue(original.admitted().unwrap(), &path)).unwrap();
        let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(full)) =
            checked.report()
        else {
            panic!("complete raw report")
        };
        assert_eq!(
            serde_json::to_value(&full.full_document_diagnostic_report.items).unwrap(),
            serde_json::json!([
                {"range":{"start":{"line":0,"character":45},"end":{"line":0,"character":50}},"severity":1,"code":2322,"source":"ts","message":"Type 'number' is not assignable to type 'Ref<number, number>'."},
                {"range":{"start":{"line":0,"character":45},"end":{"line":0,"character":50}},"severity":4,"code":6133,"source":"ts","message":"'value' is declared but its value is never read."}
            ])
        );
        let value = source.find("value=1").unwrap() as u32;
        assert_eq!(
            checked.authored_spans(),
            [
                Ok(Span::new(value, value + 5)),
                Ok(Span::new(value, value + 5))
            ]
        );
        assert_eq!(
            checked.projection().document().as_str(),
            cstr!(
                "{script}\n;\nexport {{}};\n{}",
                if template.is_empty() {
                    ""
                } else {
                    "void (\n1\n);\n"
                }
            )
        );
        assert!(std::ptr::eq(
            checked.projection().original().observation(),
            &original
        ));
        assert_eq!(
            checked.diagnostic_configuration_path(),
            root.path().join("tsconfig.json").canonicalize().unwrap()
        );
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
        assert!(!root.path().join("Original.vue.mjs").exists());
    }
    block_on(bridge.shutdown()).unwrap();
}
