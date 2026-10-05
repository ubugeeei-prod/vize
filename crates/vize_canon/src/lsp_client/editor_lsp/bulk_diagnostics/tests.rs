use super::{conversion, diagnostic_methods, positions::Positions};
use lsp_types::{Position, Range};
use serde_json::json;
use vize_l0::{FxHashMap, String};

fn rows(value: serde_json::Value) -> Vec<conversion::NativeDiagnostic> {
    serde_json::from_value(value).unwrap()
}

#[test]
fn absolute_utf16_matches_lsp_crlf_cr_lf_and_unicode_separator_coordinates() {
    let positions = Positions::new("😀x\r\né\u{2028}Y\rZ\n").unwrap();
    assert_eq!(
        positions.range(2, 3),
        Some(Range::new(Position::new(0, 2), Position::new(0, 3)))
    );
    // Pinned ComputeLSPLineStarts explicitly excludes Unicode separators.
    assert_eq!(
        positions.range(6, 8),
        Some(Range::new(Position::new(1, 1), Position::new(1, 3)))
    );
    assert_eq!(
        positions.range(9, 10),
        Some(Range::new(Position::new(2, 0), Position::new(2, 1)))
    );
    assert_eq!(
        positions.range(11, 11),
        Some(Range::new(Position::new(3, 0), Position::new(3, 0)))
    );
    for (start, end) in [(1, 2), (3, 2), (0, 12)] {
        assert!(positions.range(start, end).is_none());
    }
}

#[test]
fn category_order_duplicates_and_empty_results_preserve_complete_lsp_rows() {
    let mut documents = FxHashMap::default();
    documents.insert("file:///a.ts".into(), "😀x\r\nabc".into());
    documents.insert("file:///empty.ts".into(), "export {};".into());
    let categories = vec![
        rows(
            json!([{"fileName":"/a.ts","pos":2,"end":3,"code":1005,"category":1,"text":"Syntax"}]),
        ),
        rows(
            json!([{"fileName":"/a.ts","pos":5,"end":8,"code":6133,"category":1,"text":"Style","reportsUnnecessary":true}]),
        ),
        rows(
            json!([{"fileName":"/a.ts","pos":5,"end":8,"code":80001,"category":2,"text":"Suggestion"}]),
        ),
        rows(
            json!([{"fileName":"/a.ts","pos":5,"end":8,"code":9007,"category":1,"text":"Declaration"}]),
        ),
    ];
    let uris = [
        "file:///a.ts".into(),
        "file:///empty.ts".into(),
        "file:///a.ts".into(),
    ];
    let actual = project(&categories, &uris, &documents);
    let expected = json!([
        {"range":{"start":{"line":0,"character":2},"end":{"line":0,"character":3}},"severity":1,"code":1005,"source":"ts","message":"Syntax"},
        {"range":{"start":{"line":1,"character":0},"end":{"line":1,"character":3}},"severity":2,"code":6133,"source":"ts","message":"Style"},
        {"range":{"start":{"line":1,"character":0},"end":{"line":1,"character":3}},"severity":4,"code":80001,"source":"ts","message":"Suggestion"},
        {"range":{"start":{"line":1,"character":0},"end":{"line":1,"character":3}},"severity":1,"code":9007,"source":"ts","message":"Declaration"},
    ]);
    assert_eq!(actual, json!([expected, [], expected]));
}

fn project(
    categories: &[Vec<conversion::NativeDiagnostic>],
    uris: &[String],
    documents: &FxHashMap<String, String>,
) -> serde_json::Value {
    serde_json::to_value(conversion::project_diagnostics(categories, uris, documents).unwrap())
        .unwrap()
}

#[test]
fn full_message_chains_and_related_information_use_distinct_original_semantics() {
    let mut documents = FxHashMap::default();
    documents.insert("file:///a.ts".into(), "abc".into());
    documents.insert("file:///%E6%97%A5%20%23.ts".into(), "😀z".into());
    let categories = vec![rows(json!([{
        "fileName":"/a.ts","pos":0,"end":3,"code":2322,"category":1,"text":"",
        "messageChain":[{"pos":0,"end":0,"code":0,"category":1,"text":"Child","messageChain":[{"pos":0,"end":0,"code":0,"category":1,"text":"Grandchild"}]},{"pos":0,"end":0,"code":0,"category":1,"text":"Sibling"}],
        "relatedInformation":[{"fileName":"/日 #.ts","pos":2,"end":3,"code":0,"category":1,"text":"Related","messageChain":[{"pos":0,"end":0,"code":0,"category":1,"text":"Not flattened here"}]}]
    }]))];
    assert_eq!(
        project(&categories, &["file:///a.ts".into()], &documents),
        json!([[{
            "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":3}},"severity":1,"code":2322,"source":"ts","message":"\n  Child\n    Grandchild\n  Sibling",
            "relatedInformation":[{"location":{"uri":"file:///%E6%97%A5%20%23.ts","range":{"start":{"line":0,"character":2},"end":{"line":0,"character":3}}},"message":"Related"}]
        }]])
    );
}

#[test]
fn incomplete_source_or_related_custody_refuses_the_whole_batch() {
    let mut documents = FxHashMap::default();
    documents.insert("file:///a.ts".into(), "abc".into());
    let uris = ["file:///a.ts".into()];
    for row in [
        json!({"pos":0,"end":0,"code":1,"category":1,"text":"Fileless"}),
        json!({"fileName":"/a.ts","pos":0,"end":4,"code":1,"category":1,"text":"Outside"}),
        json!({"fileName":"/a.ts","pos":0,"end":1,"code":1,"category":1,"text":"Related","relatedInformation":[{"fileName":"/unknown.ts","pos":0,"end":1,"code":1,"category":1,"text":"Unknown"}]}),
    ] {
        assert!(
            conversion::project_diagnostics(&[rows(json!([row]))], &uris, &documents).is_none()
        );
    }
    assert!(!conversion::requested_members_are_present(
        &uris,
        &["/other.ts".into()]
    ));
    assert!(conversion::requested_members_are_present(
        &uris,
        &["/a.ts".into(), "/other.ts".into()]
    ));
    assert!(
        conversion::project_diagnostics(&[], &["file:///missing.ts".into()], &documents).is_none()
    );
}

#[test]
fn native_nullable_empties_and_effective_declaration_options_preserve_category_contract() {
    let empty: Option<Vec<conversion::NativeDiagnostic>> =
        serde_json::from_value(json!(null)).unwrap();
    assert!(empty.is_none());
    let base = vec![
        "getSyntacticDiagnostics",
        "getSemanticDiagnostics",
        "getSuggestionDiagnostics",
    ];
    for options in [json!({}), json!({"declaration":false,"composite":null})] {
        assert_eq!(diagnostic_methods(&options), base);
    }
    for options in [
        json!({"declaration":true}),
        json!({"declaration":false,"composite":true}),
    ] {
        let mut expected = base.clone();
        expected.push("getDeclarationDiagnostics");
        assert_eq!(diagnostic_methods(&options), expected);
    }
}

#[test]
fn all_native_style_warnings_and_categories_keep_complete_capability_fields() {
    let mut documents = FxHashMap::default();
    documents.insert("file:///a.ts".into(), "x".into());
    let styles = [6133, 6138, 6192, 6196, 7027, 7028, 7029, 7030];
    let native = styles.iter().map(|code| json!({
        "fileName":"/a.ts","pos":0,"end":1,"code":code,"category":1,"text":"Style",
        "reportsUnnecessary":true,"reportsDeprecated":true,
    })).chain([0, 1, 2, 3].map(|category| json!({
        "fileName":"/a.ts","pos":0,"end":1,"code":2322,"category":category,"text":"Category",
    }))).collect::<Vec<_>>();
    let expected = styles
        .iter()
        .map(|code| {
            json!({
                "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},
                "severity":2,"code":code,"source":"ts","message":"Style",
            })
        })
        .chain([2, 1, 4, 3].map(|severity| {
            json!({
                "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},
                "severity":severity,"code":2322,"source":"ts","message":"Category",
            })
        }))
        .collect::<Vec<_>>();
    assert_eq!(
        project(&[rows(json!(native))], &["file:///a.ts".into()], &documents),
        json!([expected])
    );
}

#[cfg(unix)]
mod native;
