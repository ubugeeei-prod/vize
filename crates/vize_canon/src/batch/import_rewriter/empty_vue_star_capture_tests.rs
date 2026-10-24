#![expect(
    clippy::disallowed_macros,
    reason = "fixtures assert complete debug and source contracts"
)]

use std::cell::RefCell;

use oxc_allocator::Allocator;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{String, ToCompactString};

use super::{ImportRewriter, apply_captured_empty_vue_stars};
use crate::PackageResolutionMode::{self, Contextual, Require};

mod literal_custody;

fn mirror_vue(path: &str, _: PackageResolutionMode) -> Option<String> {
    path.ends_with(".vue")
        .then(|| format!("{path}.ts").to_compact_string())
}

#[test]
fn initial_same_parse_retains_only_authentic_top_level_bare_edges_and_original_modes() {
    let source = "import './Before.vue';\nexport * from './\\u0045mpty.vue';\nexport type * from '@alias/Typed.vue' with { 'resolution-mode': 'require' };\nexport * as NS from './Other.vue';\ndeclare module 'ambient' { export * from './Nested.vue'; }\n";
    let expected_base = "import './Before.vue.ts';\nexport * from './Empty.vue.ts';\nexport type * from '@alias/Typed.vue.ts' with { 'resolution-mode': 'require' };\nexport * as NS from './Other.vue.ts';\ndeclare module 'ambient' { export * from './Nested.vue.ts'; }\n";
    let expected_empty = "import './Before.vue.ts';\nexport {} from './Empty.vue.ts';\nexport type {} from '@alias/Typed.vue.ts' with { 'resolution-mode': 'require' };\nexport * as NS from './Other.vue.ts';\ndeclare module 'ambient' { export * from './Nested.vue.ts'; }\n";
    let literal_calls = RefCell::new(Vec::new());
    let (baseline, stars) = ImportRewriter::new().rewrite_with_captured_stars(
        source,
        SourceType::ts(),
        |path, mode| {
            literal_calls.borrow_mut().push((path.to_owned(), mode));
            mirror_vue(path, mode)
        },
    );
    assert_eq!(baseline.code.as_str(), expected_base);
    assert_eq!(
        stars
            .iter()
            .map(|star| (star.specifier(), star.mode()))
            .collect::<Vec<_>>(),
        [("./Empty.vue", Contextual), ("@alias/Typed.vue", Require),]
    );
    let original_calls = literal_calls.borrow().clone();
    assert_eq!(
        original_calls,
        [
            ("./Before.vue".to_owned(), Contextual),
            ("./Empty.vue".to_owned(), Contextual),
            ("@alias/Typed.vue".to_owned(), Require),
            ("./Other.vue".to_owned(), Contextual),
            ("ambient".to_owned(), Contextual),
            ("./Nested.vue".to_owned(), Contextual),
        ]
    );
    let ordinary = ImportRewriter::new().rewrite_with(source, SourceType::ts(), mirror_vue);
    assert_eq!(baseline.code, ordinary.code);
    assert_eq!(baseline.source_map, ordinary.source_map);
    assert_eq!(
        format!("{:?}", baseline.source_map),
        format!("{:?}", ordinary.source_map)
    );
    let result =
        apply_captured_empty_vue_stars(&baseline.code, &baseline.source_map, &stars, |_, _, _| {
            true
        });
    assert_eq!(result.code.as_str(), expected_empty);
    let original_stars = [
        source.find("* from").unwrap(),
        source.find("type *").unwrap() + 5,
    ];
    let base_stars = [
        expected_base.find("* from").unwrap(),
        expected_base.find("type *").unwrap() + 5,
    ];
    for offset in 0..=source.len() {
        let added = original_stars.iter().filter(|star| **star < offset).count() as u32;
        assert_eq!(
            result.source_map.get_virtual_offset(offset as u32),
            baseline.source_map.get_virtual_offset(offset as u32) + added
        );
    }
    for offset in 0..=expected_base.len() {
        let added = base_stars.iter().filter(|star| **star < offset).count();
        assert_eq!(
            result
                .source_map
                .get_original_offset((offset + added) as u32),
            baseline.source_map.get_original_offset(offset as u32)
        );
    }
    for (original, base_offset) in original_stars.into_iter().zip(base_stars) {
        let start = result.source_map.get_virtual_offset(original as u32);
        assert_eq!(
            baseline.source_map.get_virtual_offset(original as u32),
            base_offset as u32
        );
        assert_eq!(
            result.source_map.get_original_offset(start),
            original as u32
        );
        assert_eq!(
            result.source_map.get_original_offset(start + 1),
            original as u32
        );
    }
    assert_eq!(*literal_calls.borrow(), original_calls);
}

#[test]
fn replay_always_uses_original_target_identity_and_can_reset_from_the_retained_baseline() {
    let source = "export * from './A.vue';\nexport * from './B.vue';\n";
    let baseline_code = "export * from 'a';\nexport * from 'long/mirrored/path/B.vue.ts';\n";
    let expected = "export {} from 'a';\nexport * from 'long/mirrored/path/B.vue.ts';\n";
    let calls = RefCell::new(0);
    let (baseline, stars) =
        ImportRewriter::new().rewrite_with_captured_stars(source, SourceType::ts(), |path, _| {
            *calls.borrow_mut() += 1;
            match path {
                "./A.vue" => Some("a".into()),
                "./B.vue" => Some("long/mirrored/path/B.vue.ts".into()),
                _ => None,
            }
        });
    assert_eq!(baseline.code.as_str(), baseline_code);
    assert_eq!(*calls.borrow(), 2);
    let captured_identity = RefCell::new(Vec::new());
    let empty = apply_captured_empty_vue_stars(
        &baseline.code,
        &baseline.source_map,
        &stars,
        |path, projected, mode| {
            captured_identity
                .borrow_mut()
                .push((path.to_owned(), projected.to_owned(), mode));
            path == "./A.vue"
        },
    );
    assert_eq!(empty.code.as_str(), expected);
    assert_eq!(
        *captured_identity.borrow(),
        [
            ("./A.vue".to_owned(), "a".to_owned(), Contextual),
            (
                "./B.vue".to_owned(),
                "long/mirrored/path/B.vue.ts".to_owned(),
                Contextual
            )
        ]
    );
    let unknown =
        apply_captured_empty_vue_stars(&baseline.code, &baseline.source_map, &stars, |_, _, _| {
            false
        });
    assert_eq!(unknown.code, baseline.code);
    assert_eq!(unknown.source_map, baseline.source_map);
    let empty_again = apply_captured_empty_vue_stars(
        &baseline.code,
        &baseline.source_map,
        &stars,
        |path, _, _| path == "./A.vue",
    );
    assert_eq!(empty_again.code, empty.code);
    assert_eq!(empty_again.source_map, empty.source_map);
    assert_eq!(
        *calls.borrow(),
        2,
        "replay must not rerun unrelated literal policies"
    );
}

#[test]
fn replay_requires_the_current_mapped_exact_star_byte_and_duplicate_edges_cannot_overlap() {
    let (baseline, stars) = ImportRewriter::new().rewrite_with_captured_stars(
        "export * from './A.vue';",
        SourceType::ts(),
        mirror_vue,
    );
    assert_eq!(stars.len(), 1);
    let changed = "export + from './A.vue.ts';";
    let refused =
        apply_captured_empty_vue_stars(changed, &baseline.source_map, &stars, |_, _, _| true);
    assert_eq!(refused.code.as_str(), changed);
    assert_eq!(refused.source_map, baseline.source_map);
    let duplicate = [stars[0].clone(), stars[0].clone()];
    let result = apply_captured_empty_vue_stars(
        &baseline.code,
        &baseline.source_map,
        &duplicate,
        |_, _, _| true,
    );
    assert_eq!(result.code.as_str(), "export {} from './A.vue.ts';");
    let again =
        apply_captured_empty_vue_stars(&result.code, &result.source_map, &stars, |_, _, _| true);
    assert_eq!(again.code, result.code);
    assert_eq!(again.source_map, result.source_map);
}

#[test]
fn replay_preserves_existing_private_ranges_when_a_new_original_star_is_admitted() {
    let source = "export * from './A.vue';\nexport type * from './B.vue';\nconst tail = 1;";
    let (baseline, stars) =
        ImportRewriter::new().rewrite_with_captured_stars(source, SourceType::ts(), mirror_vue);
    let together =
        apply_captured_empty_vue_stars(&baseline.code, &baseline.source_map, &stars, |_, _, _| {
            true
        });
    assert_eq!(
        together.code.as_str(),
        "export {} from './A.vue.ts';\nexport type {} from './B.vue.ts';\nconst tail = 1;"
    );
    for first in ["./A.vue", "./B.vue"] {
        let partial = apply_captured_empty_vue_stars(
            &baseline.code,
            &baseline.source_map,
            &stars,
            |path, _, _| path == first,
        );
        let second = apply_captured_empty_vue_stars(
            &partial.code,
            &partial.source_map,
            &stars,
            |_, _, _| true,
        );
        assert_eq!(second.code, together.code);
        assert_eq!(second.source_map, together.source_map);
        for offset in 0..=together.code.len() as u32 {
            assert_eq!(
                second.source_map.get_original_offset(offset),
                together.source_map.get_original_offset(offset)
            );
        }
    }
}

#[test]
fn unadmitted_namespace_nested_or_diagnostic_sources_never_produce_replay_edges() {
    for source in [
        "export * as NS from './A.vue';",
        "export type * as Types from './A.vue';",
        "declare module 'ambient' { export * from './A.vue'; }",
        "declare namespace Ambient { export * from './A.vue'; }",
        "declare namespace Outer.Inner { export type * from './A.vue'; }",
        "export * from './A.vue'; const broken = ;",
        "export * from './A.vue'; const unclosed = '",
    ] {
        let (baseline, stars) =
            ImportRewriter::new().rewrite_with_captured_stars(source, SourceType::ts(), mirror_vue);
        assert!(stars.is_empty(), "{source}");
        let ordinary = ImportRewriter::new().rewrite_with(source, SourceType::ts(), mirror_vue);
        assert_eq!(baseline.code, ordinary.code);
        assert_eq!(baseline.source_map, ordinary.source_map);
        let replayed = apply_captured_empty_vue_stars(
            &baseline.code,
            &baseline.source_map,
            &stars,
            |_, _, _| panic!("unadmitted source cannot request target authority"),
        );
        assert_eq!(replayed.code, baseline.code);
        assert_eq!(replayed.source_map, baseline.source_map);
    }
}

#[test]
fn statement_depth_keeps_only_top_level_stars_around_a_genuine_ambient_module() {
    let source = "export /* first * */ * from './Top.vue';\n\
declare module 'ambient' { export /* nested * */ type * from './Nested.vue'; }\n\
export type /* last * */ * from './After.vue';\n";
    let expected = "export /* first * */ {} from './Top.vue.ts';\n\
declare module 'ambient' { export /* nested * */ type * from './Nested.vue.ts'; }\n\
export type /* last * */ {} from './After.vue.ts';\n";
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(!parsed.panicked && parsed.diagnostics.is_empty());
    let (baseline, stars) =
        ImportRewriter::new().rewrite_with_captured_stars(source, SourceType::ts(), mirror_vue);
    assert_eq!(
        stars
            .iter()
            .map(|star| (star.specifier(), star.mode()))
            .collect::<Vec<_>>(),
        [("./Top.vue", Contextual), ("./After.vue", Contextual)]
    );
    let ordinary = ImportRewriter::new().rewrite_with(source, SourceType::ts(), mirror_vue);
    assert_eq!(baseline.code, ordinary.code);
    assert_eq!(baseline.source_map, ordinary.source_map);
    let rewritten =
        apply_captured_empty_vue_stars(&baseline.code, &baseline.source_map, &stars, |_, _, _| {
            true
        });
    assert_eq!(rewritten.code.as_str(), expected);
    for (marker, token) in [
        ("* from './Top", "{} from './Top"),
        ("* from './After", "{} from './After"),
    ] {
        let original = source.find(marker).unwrap() as u32;
        let current = expected.find(token).unwrap() as u32;
        assert_eq!(rewritten.source_map.get_virtual_offset(original), current);
        assert_eq!(rewritten.source_map.get_original_offset(current), original);
        assert_eq!(
            rewritten.source_map.get_original_offset(current + 1),
            original
        );
    }
}

#[test]
fn genuine_namespace_ast_depth_does_not_waive_its_original_parser_diagnostic() {
    let source = "export * from './Top.vue'; declare namespace Outer.Inner { export type * from './Nested.vue'; } export * from './After.vue';";
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(!parsed.panicked && !parsed.diagnostics.is_empty());
    let mut captured = super::collect::ModuleSpecifierCollector::with_stars();
    captured.visit_program(&parsed.program);
    assert_eq!(
        captured
            .empty_stars
            .iter()
            .map(|star| star.specifier.as_str())
            .collect::<Vec<_>>(),
        ["./Top.vue", "./After.vue"]
    );
    let mut ordinary = super::collect::ModuleSpecifierCollector::new();
    ordinary.visit_program(&parsed.program);
    assert!(ordinary.empty_stars.is_empty());
    assert_eq!(captured.specifiers, ordinary.specifiers);
    let (baseline, stars) =
        ImportRewriter::new().rewrite_with_captured_stars(source, SourceType::ts(), mirror_vue);
    assert!(
        stars.is_empty(),
        "original diagnostics still refuse production admission"
    );
    let ordinary = ImportRewriter::new().rewrite_with(source, SourceType::ts(), mirror_vue);
    assert_eq!(baseline.code, ordinary.code);
    assert_eq!(baseline.source_map, ordinary.source_map);
}
