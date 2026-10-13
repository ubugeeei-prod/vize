#![expect(
    clippy::disallowed_macros,
    reason = "fixtures assert complete debug and source contracts"
)]

use std::cell::RefCell;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use vize_carton::{String, ToCompactString};

use super::{ImportRewriter, ImportSourceMap, OffsetAdjustment, collect::BareStarExport};
use crate::PackageResolutionMode;

fn mirror_vue(path: &str, _: PackageResolutionMode) -> Option<String> {
    path.ends_with(".vue")
        .then(|| format!("{path}.ts").to_compact_string())
}

#[test]
fn preserves_complete_unicode_crlf_comments_and_type_star_source() {
    let source = "// λ barrel\r\nexport /* fake * from */ type /* × */ * /* keep * */ from /* μ */ './A.vue';\r\nexport *from\"./B.vue\";\r\nconst 後 = 1;\r\n";
    let expected = "// λ barrel\r\nexport /* fake * from */ type /* × */ {} /* keep * */ from /* μ */ './A.vue.ts';\r\nexport {}from\"./B.vue.ts\";\r\nconst 後 = 1;\r\n";
    let result = ImportRewriter::new().rewrite_with_empty_vue_stars(
        source,
        SourceType::ts(),
        mirror_vue,
        |path, _| matches!(path, "./A.vue" | "./B.vue"),
    );
    assert_eq!(result.code.as_str(), expected);
    for (original, virtual_token) in [
        (
            source.find("* /* keep").unwrap(),
            expected.find("{} /* keep").unwrap(),
        ),
        (
            source.find("*from").unwrap(),
            expected.find("{}from").unwrap(),
        ),
    ] {
        assert_eq!(
            result.source_map.get_virtual_offset(original as u32),
            virtual_token as u32
        );
        assert_eq!(
            result.source_map.get_original_offset(virtual_token as u32),
            original as u32
        );
        assert_eq!(
            result
                .source_map
                .get_original_offset(virtual_token as u32 + 1),
            original as u32
        );
    }
    let original_tail = source.find("const 後").unwrap();
    let virtual_tail = expected.find("const 後").unwrap();
    for offset in 0..=source.len() - original_tail {
        assert_eq!(
            result
                .source_map
                .get_virtual_offset((original_tail + offset) as u32),
            (virtual_tail + offset) as u32
        );
        assert_eq!(
            result
                .source_map
                .get_original_offset((virtual_tail + offset) as u32),
            (original_tail + offset) as u32
        );
    }
}

#[test]
fn star_only_map_preserves_every_original_byte_and_clamps_both_inserted_braces() {
    let source =
        "// λ *\r\nexport * from './A.vue';\r\nexport type * from './B.vue';\r\nconst 後 = 1;";
    let expected =
        "// λ *\r\nexport {} from './A.vue';\r\nexport type {} from './B.vue';\r\nconst 後 = 1;";
    let stars = [
        source.find("* from").unwrap(),
        source.find("type *").unwrap() + 5,
    ];
    let result = ImportRewriter::new().rewrite_with_empty_vue_stars(
        source,
        SourceType::ts(),
        |_, _| None,
        |_, _| true,
    );
    assert_eq!(result.code.as_str(), expected);
    let mut original_for_virtual = Vec::new();
    for original in 0..source.len() {
        let virtual_offset = original_for_virtual.len();
        original_for_virtual.push(original as u32);
        if stars.contains(&original) {
            original_for_virtual.push(original as u32);
        }
        assert_eq!(
            result.source_map.get_virtual_offset(original as u32),
            virtual_offset as u32
        );
    }
    original_for_virtual.push(source.len() as u32);
    assert_eq!(original_for_virtual.len(), expected.len() + 1);
    for (virtual_offset, original) in original_for_virtual.into_iter().enumerate() {
        assert_eq!(
            result.source_map.get_original_offset(virtual_offset as u32),
            original
        );
    }
    assert_eq!(
        result.source_map.get_virtual_offset(source.len() as u32),
        expected.len() as u32
    );
}

#[test]
fn original_line_comment_spans_and_ecmascript_unicode_trivia_keep_all_bytes() {
    let source = "export // decoy *\r\n type\u{a0}* // keep *\r\n from\u{feff}'./A.vue';\r\nexport\u{2028}*\u{3000}from './B.vue';";
    let expected = "export // decoy *\r\n type\u{a0}{} // keep *\r\n from\u{feff}'./A.vue';\r\nexport\u{2028}{}\u{3000}from './B.vue';";
    let result = ImportRewriter::new().rewrite_with_empty_vue_stars(
        source,
        SourceType::ts(),
        |_, _| None,
        |_, _| true,
    );
    assert_eq!(result.code.as_str(), expected);
}

#[test]
fn merges_star_and_literal_edits_across_imports_and_multiple_export_statements() {
    let source = "import './Lead.vue';\nexport * from './A.vue';\nexport type * from './B.vue';\nconst tail = 1;";
    let expected = "import './Lead.vue.ts';\nexport {} from './A.vue.ts';\nexport type {} from './B.vue.ts';\nconst tail = 1;";
    let result = ImportRewriter::new().rewrite_with_empty_vue_stars(
        source,
        SourceType::ts(),
        mirror_vue,
        |_, _| true,
    );
    assert_eq!(result.code.as_str(), expected);
    for marker in [";\nexport", ";\nconst", "const tail"] {
        let original = source.find(marker).unwrap() as u32;
        let virtual_offset = expected.find(marker).unwrap() as u32;
        assert_eq!(
            result.source_map.get_virtual_offset(original),
            virtual_offset
        );
        assert_eq!(
            result.source_map.get_original_offset(virtual_offset),
            original
        );
    }
}

#[test]
fn namespace_named_import_and_dynamic_forms_never_request_empty_star_authority() {
    let source = "export * as ns from './A.vue';\nexport type * as types from './B.vue';\nexport { default as A, type Props } from './C.vue';\nimport D from './D.vue';\nconst star = '*';\nconst lazy = import('./E.vue');\n";
    let expected = "export * as ns from './A.vue.ts';\nexport type * as types from './B.vue.ts';\nexport { default as A, type Props } from './C.vue.ts';\nimport D from './D.vue.ts';\nconst star = '*';\nconst lazy = import('./E.vue.ts');\n";
    let result = ImportRewriter::new().rewrite_with_empty_vue_stars(
        source,
        SourceType::ts(),
        mirror_vue,
        |_, _| panic!("only top-level bare star exports may ask for authority"),
    );
    assert_eq!(result.code.as_str(), expected);
}

#[test]
fn nested_ambient_stars_keep_their_tokens_and_top_level_star_uses_its_own_target() {
    let source =
        "declare module 'virtual' { export * from './Nested.vue'; }\nexport * from './Top.vue';\n";
    let expected = "declare module 'virtual' { export * from './Nested.vue.ts'; }\nexport {} from './Top.vue.ts';\n";
    let certified = RefCell::new(Vec::new());
    let result = ImportRewriter::new().rewrite_with_empty_vue_stars(
        source,
        SourceType::ts(),
        mirror_vue,
        |path, mode| {
            certified.borrow_mut().push((path.to_owned(), mode));
            path == "./Top.vue"
        },
    );
    assert_eq!(result.code.as_str(), expected);
    assert_eq!(
        *certified.borrow(),
        [("./Top.vue".to_owned(), PackageResolutionMode::Contextual)]
    );
}

#[test]
fn unknown_named_and_default_only_target_authority_remains_with_the_callback() {
    let source = "export * from './Empty.vue';\nexport * from './Unknown.vue';\nexport * from './Named.vue';\nexport * from './DefaultOnly.vue';\n";
    let expected = "export {} from './Empty.vue.ts';\nexport * from './Unknown.vue.ts';\nexport * from './Named.vue.ts';\nexport {} from './DefaultOnly.vue.ts';\n";
    let result = ImportRewriter::new().rewrite_with_empty_vue_stars(
        source,
        SourceType::ts(),
        mirror_vue,
        |path, _| matches!(path, "./Empty.vue" | "./DefaultOnly.vue"),
    );
    assert_eq!(result.code.as_str(), expected);
}

#[test]
fn same_parse_target_callback_receives_authored_resolution_modes() {
    let source = "export type * from './Required.vue' with { 'resolution-mode': 'require' };\nexport type * from './Imported.vue' with { 'resolution-mode': 'import' };\n";
    let expected = "export type {} from './Required.vue.ts' with { 'resolution-mode': 'require' };\nexport type {} from './Imported.vue.ts' with { 'resolution-mode': 'import' };\n";
    let certified = RefCell::new(Vec::new());
    let result = ImportRewriter::new().rewrite_with_empty_vue_stars(
        source,
        SourceType::ts(),
        mirror_vue,
        |path, mode| {
            certified.borrow_mut().push((path.to_owned(), mode));
            true
        },
    );
    assert_eq!(result.code.as_str(), expected);
    assert_eq!(
        *certified.borrow(),
        [
            ("./Required.vue".to_owned(), PackageResolutionMode::Require),
            ("./Imported.vue".to_owned(), PackageResolutionMode::Import),
        ]
    );
}

#[test]
fn any_parser_diagnostic_or_panic_refuses_star_authority() {
    let mut saw_panic = false;
    for source in [
        "export * from './A.vue';\nconst value = ;",
        "export * from './A.vue';\nconst unclosed = '",
        "export ** from './A.vue';",
        "export * from ;",
    ] {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
        assert!(!parsed.diagnostics.is_empty());
        saw_panic |= parsed.panicked;
        let result = ImportRewriter::new().rewrite_with_empty_vue_stars(
            source,
            SourceType::ts(),
            |_, _| None,
            |_, _| panic!("malformed source must not request star authority"),
        );
        assert_eq!(result.code.as_str(), source);
        assert_eq!(result.source_map, ImportSourceMap::empty());
    }
    assert!(saw_panic, "include an actual parser-panic control");
}

#[test]
fn header_proof_requires_original_comment_spans_and_exact_bare_star_grammar() {
    let source = "export /* decoy * */ * from './A.vue';";
    let candidate = BareStarExport {
        header: Span::new(0, source.find('\'').unwrap() as u32),
        literal: Span::new(0, 0),
        is_type: false,
        specifier: "./A.vue".into(),
        mode: PackageResolutionMode::Contextual,
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(
        super::empty_vue_stars::star_offset(source, &parsed.program.comments, &candidate),
        Some(source.find("* from").unwrap() as u32)
    );
    assert_eq!(
        super::empty_vue_stars::star_offset(source, &[], &candidate),
        None
    );
    for (header, is_type) in [
        ("export ** from ", false),
        ("export * as Namespace from ", false),
        ("export * extra from ", false),
        ("exporttype * from ", true),
        ("export type * from ", false),
        ("export * from extra ", false),
    ] {
        let candidate = BareStarExport {
            header: Span::new(0, header.len() as u32),
            literal: Span::new(0, 0),
            is_type,
            specifier: "./A.vue".into(),
            mode: PackageResolutionMode::Contextual,
        };
        assert_eq!(
            super::empty_vue_stars::star_offset(header, &[], &candidate),
            None,
            "{header}"
        );
    }
}

#[test]
fn ordinary_api_and_literal_only_debug_and_maps_remain_exact() {
    let source = "export * from './A.vue';";
    let result = ImportRewriter::new().rewrite(source, SourceType::ts(), None);
    assert_eq!(result.code.as_str(), "export * from './A.vue.ts';");
    let expected = ImportSourceMap::new(vec![OffsetAdjustment {
        original_offset: 15,
        adjustment: 3,
    }]);
    assert_eq!(result.source_map, expected);
    assert_eq!(
        format!("{:?}", result.source_map),
        "ImportSourceMap { adjustments: [OffsetAdjustment { original_offset: 15, adjustment: 3 }] }"
    );
    assert_eq!(
        format!("{:#?}", ImportSourceMap::empty()),
        "ImportSourceMap {\n    adjustments: [],\n}"
    );
    let disabled = ImportRewriter::new().rewrite_with_empty_vue_stars(
        source,
        SourceType::ts(),
        mirror_vue,
        |_, _| false,
    );
    assert_eq!(disabled.code, result.code);
    assert_eq!(disabled.source_map, result.source_map);
    for offset in 0..=source.len() as u32 {
        assert_eq!(
            disabled.source_map.get_virtual_offset(offset),
            expected.get_virtual_offset(offset)
        );
    }
    for offset in 0..=result.code.len() as u32 {
        assert_eq!(
            disabled.source_map.get_original_offset(offset),
            expected.get_original_offset(offset)
        );
    }
}
