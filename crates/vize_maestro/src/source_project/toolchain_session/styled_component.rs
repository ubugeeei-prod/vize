use super::*;
use std::sync::Arc;
use vize_atelier_sfc::native::{NativeScssSession, compile_native_sfc_with_scss};

pub(super) fn restore_formatted_style(
    docs: &DocumentStore,
    snapshots: &SourceSnapshotCache,
    repaired: &SourceSnapshot,
    formatted: &SourceSnapshot,
    reference: &Value,
    options: NativeSfcCompileOptions<'_>,
) -> Arc<SourceSnapshot> {
    let uri = repaired.uri();
    let end = offset_to_position_str(repaired.source(), repaired.source().len());
    let (prefix, _) = formatted.source().split_once("<script").unwrap();
    apply(docs, uri, 5, Some(Range::new(end, end)), prefix);
    let live = snapshots.capture(docs, uri).unwrap();
    let arena = Allocator::new();
    let session = NativeScssSession::new(&arena, live.source(), options);
    let owner = session.observation().descriptor();
    check_source(owner, &live);
    let types = live
        .prepare_literal_products(docs, owner, ROLE)
        .unwrap()
        .publish(docs, Some(at(live.source(), "other:")))
        .unwrap();
    assert!(types.type_errors.is_empty() && types.binding_hover.is_some());
    let rules = CssLinter::with_all_rules();
    let (prepared, lint) = live
        .prepare_scss_with(docs, owner, 0, |s| {
            rules.lint_file_style(StyleRuleInput::Observed(s), true)
        })
        .unwrap();
    assert!(!lint.diagnostics.is_empty());
    let fmt = FormatOptions::default();
    assert!(
        prepared
            .verified_format(docs, &GlyphFormatter::new(&fmt, &arena))
            .unwrap()
            .publish(docs)
            .unwrap()
            .is_none()
    );
    let cursor = at(live.source(), "$ink;");
    let Some(GotoDefinitionResponse::Scalar(target)) = live
        .scss_definition(docs, prepared.products(), cursor)
        .unwrap()
    else {
        panic!("authored variable definition")
    };
    assert_eq!(target.uri, *uri);
    assert_eq!(target.range.start, at(live.source(), "$ink:"));
    let classes = prepared.products().classes().unwrap();
    assert!(classes.names().iter().map(|n| n.as_str()).eq(["card"]));
    let compiled = session.emit([prepared.products()]).unwrap();
    let independent = compile_native_sfc_with_scss(&arena, live.source(), options);
    let cold = independent.result().unwrap();
    assert_eq!(compiled.document(), cold.document());
    assert_eq!(compiled.css_document(), cold.css_document());
    assert_eq!(compiled.source_map(), cold.source_map());
    assert_eq!(compiled.css_source_map(), cold.css_source_map());
    assert_eq!(
        compiled.css(),
        Some(prepared.products().css().unwrap().as_str())
    );
    assert_eq!(compiled.code(), reference["code"].as_str().unwrap());
    let mut expected = reference["nativeMap"].clone();
    expected["sourcesContent"] = serde_json::json!([live.source()]);
    let map: Value = serde_json::from_str(compiled.source_map().unwrap()).unwrap();
    assert_eq!(map, expected);
    if let Some(path) = std::env::var_os("VIZE_STYLED_SESSION_OUTPUT") {
        let css_map: Value = serde_json::from_str(compiled.css_source_map().unwrap()).unwrap();
        let rows = serde_json::json!([{
            "name": options.filename, "filename": options.filename, "source": live.source(), "trim": options.style_trim,
            "code": compiled.code(), "map": map, "css": compiled.css(), "cssMap": css_map
        }]);
        std::fs::write(path, serde_json::to_vec(&rows).unwrap()).unwrap();
    }
    live
}
