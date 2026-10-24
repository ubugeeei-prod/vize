#![expect(
    clippy::disallowed_macros,
    reason = "fixtures assert complete debug and source contracts"
)]

use std::cell::RefCell;

use oxc_span::SourceType;
use vize_carton::ToCompactString;

use super::{Contextual, ImportRewriter, apply_captured_empty_vue_stars};
use crate::batch::import_rewriter::{ImportSourceMap, OffsetAdjustment};

#[test]
fn complete_projected_literals_survive_each_prior_and_current_expand_shrink_pair() {
    let source = "import './Before.vue';\r\nexport * from './A.vue';\r\nexport type * from './B.vue';\r\nconst 雪 = 1;";
    let original_stars = [
        source.find("* from").unwrap(),
        source.find("type *").unwrap() + 5,
    ];
    for before in ["p", "/mirror/long/path/Before.vue.ts"] {
        for current in ["a", "/mirror/long/雪/A.vue.ts"] {
            let (baseline, stars) = ImportRewriter::new().rewrite_with_captured_stars(
                source,
                SourceType::ts(),
                |path, _| {
                    Some(
                        match path {
                            "./Before.vue" => before,
                            "./A.vue" => current,
                            "./B.vue" => "/mirror/B.vue.ts",
                            _ => panic!("unexpected original literal"),
                        }
                        .to_compact_string(),
                    )
                },
            );
            let expected_base = format!(
                "import '{before}';\r\nexport * from '{current}';\r\nexport type * from '/mirror/B.vue.ts';\r\nconst 雪 = 1;"
            );
            let expected = format!(
                "import '{before}';\r\nexport {{}} from '{current}';\r\nexport type {{}} from '/mirror/B.vue.ts';\r\nconst 雪 = 1;"
            );
            assert_eq!(baseline.code.as_str(), expected_base);
            let receipts = RefCell::new(Vec::new());
            let rewritten = apply_captured_empty_vue_stars(
                &baseline.code,
                &baseline.source_map,
                &stars,
                |authored, projected, mode| {
                    receipts
                        .borrow_mut()
                        .push((authored.to_owned(), projected.to_owned(), mode));
                    match authored {
                        "./A.vue" => projected == current,
                        "./B.vue" => projected == "/mirror/B.vue.ts",
                        _ => false,
                    }
                },
            );
            assert_eq!(rewritten.code.as_str(), expected);
            assert_eq!(
                *receipts.borrow(),
                [
                    ("./A.vue".to_owned(), current.to_owned(), Contextual),
                    (
                        "./B.vue".to_owned(),
                        "/mirror/B.vue.ts".to_owned(),
                        Contextual
                    ),
                ]
            );
            for offset in 0..=source.len() {
                let added = original_stars.iter().filter(|star| **star < offset).count() as u32;
                assert_eq!(
                    rewritten.source_map.get_virtual_offset(offset as u32),
                    baseline.source_map.get_virtual_offset(offset as u32) + added
                );
            }
            let base_stars = [
                expected_base.find("* from").unwrap(),
                expected_base.find("type *").unwrap() + 5,
            ];
            for offset in 0..=expected_base.len() {
                let added = base_stars.iter().filter(|star| **star < offset).count();
                assert_eq!(
                    rewritten
                        .source_map
                        .get_original_offset((offset + added) as u32),
                    baseline.source_map.get_original_offset(offset as u32)
                );
            }
        }
    }
}

#[test]
fn a_differing_projected_target_preserves_complete_baseline_source_and_map() {
    let source = "import './Before.vue';\nexport * from './A.vue';\nconst tail = 1;";
    let expected_base =
        "import '/mirror/Before.vue.ts';\nexport * from '/foreign/A.vue.ts';\nconst tail = 1;";
    let (baseline, stars) =
        ImportRewriter::new().rewrite_with_captured_stars(source, SourceType::ts(), |path, _| {
            Some(
                match path {
                    "./Before.vue" => "/mirror/Before.vue.ts",
                    "./A.vue" => "/foreign/A.vue.ts",
                    _ => panic!("unexpected original literal"),
                }
                .to_compact_string(),
            )
        });
    assert_eq!(baseline.code.as_str(), expected_base);
    let receipts = RefCell::new(Vec::new());
    let refused = apply_captured_empty_vue_stars(
        &baseline.code,
        &baseline.source_map,
        &stars,
        |authored, projected, mode| {
            receipts
                .borrow_mut()
                .push((authored.to_owned(), projected.to_owned(), mode));
            authored == "./A.vue" && projected == "/mirror/A.vue.ts" && mode == Contextual
        },
    );
    assert_eq!(
        *receipts.borrow(),
        [(
            "./A.vue".to_owned(),
            "/foreign/A.vue.ts".to_owned(),
            Contextual
        )]
    );
    assert_eq!(refused.code.as_str(), expected_base);
    assert_eq!(refused.source_map, baseline.source_map);
    assert_eq!(
        format!("{:?}", refused.source_map),
        format!("{:?}", baseline.source_map)
    );
    for offset in 0..=source.len() as u32 {
        assert_eq!(
            refused.source_map.get_virtual_offset(offset),
            baseline.source_map.get_virtual_offset(offset)
        );
    }
    for offset in 0..=expected_base.len() as u32 {
        assert_eq!(
            refused.source_map.get_original_offset(offset),
            baseline.source_map.get_original_offset(offset)
        );
    }
}

#[test]
fn malformed_ambiguous_or_unmapped_literal_spans_cannot_request_target_authority() {
    let source = "export * from './A.vue';";
    let (baseline, stars) =
        ImportRewriter::new().rewrite_with_captured_stars(source, SourceType::ts(), |_, _| None);
    assert_eq!(baseline.code.as_str(), source);
    for current in [
        "",
        "export * from \"./A.vue\";",
        "export * from './A.vue;",
        "export * from 'x';",
    ] {
        let refused =
            apply_captured_empty_vue_stars(current, &baseline.source_map, &stars, |_, _, _| {
                panic!("unproven literal span cannot request authority")
            });
        assert_eq!(refused.code.as_str(), current);
        assert_eq!(refused.source_map, baseline.source_map);
    }
    let bad_map = ImportSourceMap::new(vec![OffsetAdjustment {
        original_offset: 15,
        adjustment: -100,
    }]);
    let refused = apply_captured_empty_vue_stars(&baseline.code, &bad_map, &stars, |_, _, _| {
        panic!("out-of-bounds mapped literal cannot request authority")
    });
    assert_eq!(refused.code, baseline.code);
    assert_eq!(refused.source_map, bad_map);
    for current_literal in ["bad'path", "bad\\path", "bad\r\npath"] {
        let (baseline, stars) =
            ImportRewriter::new().rewrite_with_captured_stars(source, SourceType::ts(), |_, _| {
                Some(current_literal.to_compact_string())
            });
        let expected = format!("export * from '{current_literal}';");
        assert_eq!(baseline.code.as_str(), expected);
        let refused = apply_captured_empty_vue_stars(
            &baseline.code,
            &baseline.source_map,
            &stars,
            |_, _, _| panic!("current decoded identity is unproven"),
        );
        assert_eq!(refused.code.as_str(), expected);
        assert_eq!(refused.source_map, baseline.source_map);
    }
}

#[test]
fn double_quotes_unicode_and_decoded_authored_identity_keep_the_complete_current_literal() {
    let source = "export * from \"./\\u0041.vue\";";
    let projected = "/mirror/雪/A.vue.ts";
    let (baseline, stars) = ImportRewriter::new().rewrite_with_captured_stars(
        source,
        SourceType::ts(),
        |path, mode| {
            assert_eq!(path, "./A.vue");
            assert_eq!(mode, Contextual);
            Some(projected.to_compact_string())
        },
    );
    assert_eq!(
        baseline.code.as_str(),
        "export * from \"/mirror/雪/A.vue.ts\";"
    );
    let receipts = RefCell::new(Vec::new());
    let rewritten = apply_captured_empty_vue_stars(
        &baseline.code,
        &baseline.source_map,
        &stars,
        |authored, current, mode| {
            receipts
                .borrow_mut()
                .push((authored.to_owned(), current.to_owned(), mode));
            authored == "./A.vue" && current == projected && mode == Contextual
        },
    );
    assert_eq!(
        *receipts.borrow(),
        [("./A.vue".to_owned(), projected.to_owned(), Contextual)]
    );
    assert_eq!(
        rewritten.code.as_str(),
        "export {} from \"/mirror/雪/A.vue.ts\";"
    );
    let star = source.find('*').unwrap() as u32;
    let start = rewritten.source_map.get_virtual_offset(star);
    assert_eq!(rewritten.source_map.get_original_offset(start), star);
    assert_eq!(rewritten.source_map.get_original_offset(start + 1), star);
}
