//! Independent full-vector law matches the actual official Vue const oracle.

use super::*;

#[test]
fn original_primitive_const_template_comparisons_keep_complete_literal_type_diagnostics() {
    let root = tempfile::TempDir::new().unwrap();
    let bridge = configured(root.path());
    let source = "<script setup lang='ts'>const value=1;</script><template>{{value===2}}{{value.missing}}</template>";
    let path = root.path().join("Source.vue");
    std::fs::write(&path, source).unwrap();
    let arena = Allocator::default();
    let original = lower_sfc_native(&arena, source, options());
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
            {"range":{"start":{"line":4,"character":0},"end":{"line":4,"character":9}},"severity":1,"code":2367,"source":"ts","message":"This comparison appears to be unintentional because the types '1' and '2' have no overlap."},
            {"range":{"start":{"line":7,"character":6},"end":{"line":7,"character":13}},"severity":1,"code":2339,"source":"ts","message":"Property 'missing' does not exist on type '1'."}
        ])
    );
    let comparison = source.find("value===2").unwrap() as u32;
    let member = source.find("missing").unwrap() as u32;
    assert_eq!(
        checked.authored_spans(),
        [
            Ok(Span::new(comparison, comparison + 9)),
            Ok(Span::new(member, member + 7))
        ]
    );
    assert_eq!(
        checked.diagnostic_configuration_path(),
        root.path().join("tsconfig.json").canonicalize().unwrap()
    );
    assert!(std::ptr::eq(
        checked.projection().original().observation(),
        &original
    ));
    assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
    assert!(!root.path().join("Source.vue.ts").exists());
    block_on(bridge.shutdown()).unwrap();
}
