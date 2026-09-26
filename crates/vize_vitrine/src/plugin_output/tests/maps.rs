use super::*;
use oxc_sourcemap::SourceMap;

#[test]
fn unicode_banner_rebases_every_native_anchor_without_inventing_provenance() {
    let compiled = native(true);
    let response = r#"[{"placement":"prepend","comment":"License 日本語🎨\nsecond line"},{"placement":"append","comment":"tail"}]"#;
    let (result, count) = rewrite::apply(&compiled, "output", response).unwrap();
    assert_eq!(count, 2);
    let before_json = compiled.map.as_ref().unwrap().to_string();
    let after_json = result.map.as_ref().unwrap().to_string();
    let before = SourceMap::from_json_string(&before_json).unwrap();
    let after = SourceMap::from_json_string(&after_json).unwrap();
    let before = before.get_source_view_tokens().collect::<Vec<_>>();
    let after = after.get_source_view_tokens().collect::<Vec<_>>();
    assert!(!before.is_empty());
    assert_eq!(before.len(), after.len());
    for (old, new) in before.iter().zip(after.iter()) {
        assert_eq!(new.get_dst_line(), old.get_dst_line() + 2);
        assert_eq!(new.get_dst_col(), old.get_dst_col());
        assert_eq!(new.get_src_line(), old.get_src_line());
        assert_eq!(new.get_src_col(), old.get_src_col());
        assert_eq!(new.get_source(), old.get_source());
        assert_eq!(new.get_source_content(), old.get_source_content());
        assert_eq!(new.get_name(), old.get_name());
    }
}

#[test]
fn whitespace_deletion_rebases_following_anchors_exactly() {
    let source = "<div>{{ label }}</div>";
    let mut document = vize_atelier_core::codegen::document::EmitDocument::default();
    document.push_str("const a =   ");
    document.push_named("label", vize_l0::Span::new(8, 13), "label");
    document.push_str(";\n");
    let mut compiled = simple(document.as_str());
    compiled.map = Some(serde_json::from_str(&document.source_map("Example.vue", source)).unwrap());
    let (result, _) = rewrite::apply(
        &compiled,
        "formatter",
        r#"[{"start":9,"end":12,"text":" "}]"#,
    )
    .unwrap();
    let json = result.map.unwrap().to_string();
    let map = SourceMap::from_json_string(&json).unwrap();
    let token = map.get_source_view_tokens().next().unwrap();
    assert_eq!((token.get_dst_line(), token.get_dst_col()), (0, 10));
    assert_eq!((token.get_src_line(), token.get_src_col()), (0, 8));
    assert_eq!(token.get_name(), Some("label"));
}

#[test]
fn invalid_map_anchors_are_rejected_instead_of_clamped() {
    let compiled = native(true);
    for mappings in ["AAAAA", "//////", "AACA"] {
        let mut changed = compiled.clone();
        let map = changed.map.as_mut().unwrap().as_object_mut().unwrap();
        map.insert("mappings".to_owned(), serde_json::json!(mappings));
        if mappings == "AAAAA" {
            map.insert("names".to_owned(), serde_json::json!([]));
        }
        assert!(
            rewrite::apply(
                &changed,
                "output",
                r#"[{"placement":"prepend","comment":"license"}]"#
            )
            .is_err(),
            "{mappings}"
        );
    }
}
