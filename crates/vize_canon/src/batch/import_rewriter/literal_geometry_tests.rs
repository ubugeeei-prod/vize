use super::{ImportRewriter, ImportSourceMap, OffsetAdjustment};
use oxc_span::SourceType;

fn assert_literal_geometry(source: &str, target: &str, replacement: &str) {
    let rewritten = ImportRewriter::new().rewrite_with_alias_resolver(
        source,
        SourceType::ts(),
        None,
        &|specifier, _| (specifier == target).then(|| replacement.to_owned()),
    );
    let expected = source.replace(target, replacement);
    assert_eq!(rewritten.code.as_str(), expected);
    let start = source.find(target).unwrap() as u32;
    let end = start + target.len() as u32;
    let generated_start = expected.find(replacement).unwrap() as u32;
    let generated_end = generated_start + replacement.len() as u32;
    let map = &rewritten.source_map;
    for (original, generated) in [
        (start - 1, generated_start - 1),
        (end, generated_end),
        (end + 1, generated_end + 1),
        (source.len() as u32, expected.len() as u32),
    ] {
        assert_eq!(map.get_original_offset(generated), original);
        assert_eq!(map.get_virtual_offset(original), generated);
        assert_eq!(
            source
                .get(..map.get_original_offset(generated) as usize)
                .unwrap()
                .encode_utf16()
                .count(),
            source
                .get(..original as usize)
                .unwrap()
                .encode_utf16()
                .count()
        );
    }
    for offset in start..end {
        assert_eq!(map.get_virtual_offset(offset), generated_start);
    }
    for offset in generated_start..generated_end {
        assert_eq!(map.get_original_offset(offset), start);
    }
}

#[test]
fn changed_literal_boundaries_and_interiors_are_atomic_in_both_directions() {
    for newline in ["\n", "\r\n"] {
        let source = format!(
            "// 日本語🍣{newline}import Widget from '#ui/日本語🍣';{newline}void Widget;{newline}"
        );
        assert_literal_geometry(&source, "#ui/日本語🍣", "/mirror/a/long/日本語🍣/Widget");
        assert_literal_geometry(&source, "#ui/日本語🍣", "x");
    }
}

#[test]
fn repeated_imports_keep_each_literal_and_later_symbol_coordinates() {
    let source = "import A from './Widget.vue';\r\nimport B from './Widget.vue';\r\nvoid [A, B];";
    let rewritten = ImportRewriter::new().rewrite(source, SourceType::ts(), None);
    let expected =
        "import A from './Widget.vue.ts';\r\nimport B from './Widget.vue.ts';\r\nvoid [A, B];";
    assert_eq!(rewritten.code.as_str(), expected);
    for ((original, _), (generated, _)) in source
        .match_indices("./Widget.vue")
        .zip(expected.match_indices("./Widget.vue.ts"))
    {
        assert_eq!(
            rewritten
                .source_map
                .get_original_offset(generated as u32 + 4),
            original as u32
        );
        assert_eq!(
            rewritten.source_map.get_virtual_offset(original as u32 + 4),
            generated as u32
        );
    }
    let original = source.find("void").unwrap() as u32;
    let generated = expected.find("void").unwrap() as u32;
    assert_eq!(
        rewritten.source_map.get_original_offset(generated),
        original
    );
    assert_eq!(rewritten.source_map.get_virtual_offset(original), generated);
}

#[test]
fn unchanged_literal_contents_keep_every_byte_coordinate_and_callback_occurrence() {
    let source = "import A from 'same'; import B from 'same'; void [A, B];";
    let calls = std::cell::Cell::new(0);
    let rewritten = ImportRewriter::new().rewrite_with_alias_resolver(
        source,
        SourceType::ts(),
        None,
        &|specifier, _| {
            calls.set(calls.get() + 1);
            Some(specifier.to_owned())
        },
    );
    assert_eq!(calls.get(), 2);
    assert_eq!(rewritten.code.as_str(), source);
    for offset in 0..=source.len() as u32 {
        assert_eq!(rewritten.source_map.get_original_offset(offset), offset);
        assert_eq!(rewritten.source_map.get_virtual_offset(offset), offset);
    }
}

#[test]
fn legacy_adjustment_constructor_keeps_its_existing_contract() {
    let map = ImportSourceMap::new(vec![OffsetAdjustment {
        original_offset: 10,
        adjustment: 3,
    }]);
    assert_eq!(map.get_virtual_offset(9), 9);
    assert_eq!(map.get_virtual_offset(10), 13);
    assert_eq!(map.get_original_offset(13), 10);
    assert_eq!(map.get_original_offset(20), 17);
}

#[test]
fn overflowing_queries_cannot_wrap_onto_an_unrelated_document_byte() {
    let mut growing = ImportSourceMap::for_rewrites(1);
    growing.record_replacement(10, 12, 20);
    growing.finish_descending_replacements();
    assert_eq!(growing.get_virtual_offset(u32::MAX), u32::MAX);
    assert_eq!(growing.get_original_offset(u32::MAX), u32::MAX - 18);
    let mut shrinking = ImportSourceMap::for_rewrites(1);
    shrinking.record_replacement(10, 30, 2);
    shrinking.finish_descending_replacements();
    assert_eq!(shrinking.get_original_offset(u32::MAX), u32::MAX);
    assert_eq!(shrinking.get_virtual_offset(u32::MAX), u32::MAX - 18);
}

#[test]
fn zero_length_and_multiple_replacement_records_keep_exclusive_end_shifts() {
    let mut map = ImportSourceMap::for_rewrites(2);
    map.record_replacement(20, 22, 4);
    map.record_replacement(10, 14, 0);
    map.finish_descending_replacements();
    assert_eq!(map.get_virtual_offset(10), 10);
    assert_eq!(map.get_virtual_offset(13), 10);
    assert_eq!(map.get_virtual_offset(14), 10);
    assert_eq!(map.get_original_offset(10), 14);
    assert_eq!(map.get_virtual_offset(20), 16);
    assert_eq!(map.get_original_offset(18), 20);
    assert_eq!(map.get_original_offset(20), 22);
    assert_eq!(map.get_virtual_offset(22), 20);
}

#[test]
fn changed_module_map_composes_with_the_existing_star_replacement_carrier() {
    let mut map = ImportSourceMap::for_rewrites(1);
    map.record_replacement(10, 14, 8);
    map.finish_descending_replacements();
    let combined = map.with_added_star_replacements(&[(20, 3)]);
    assert_eq!(combined.get_virtual_offset(10), 10);
    assert_eq!(combined.get_original_offset(15), 10);
    assert_eq!(combined.get_virtual_offset(14), 18);
    assert_eq!(combined.get_virtual_offset(20), 24);
    assert_eq!(combined.get_original_offset(25), 20);
    assert_eq!(combined.get_virtual_offset(21), 27);
    assert_eq!(combined.get_original_offset(27), 21);
}

#[test]
fn source_kind_variants_use_the_same_geometry_without_an_extra_parse() {
    for source in [
        "export { default as W } from './Widget.vue';",
        "const load = () => import('./Widget.vue');",
        "type W = import('./Widget.vue').default;",
        "import W = require('./Widget.vue');",
    ] {
        let rewritten = ImportRewriter::new().rewrite(source, SourceType::ts(), None);
        let expected = source.replace("./Widget.vue", "./Widget.vue.ts");
        assert_eq!(rewritten.code.as_str(), expected);
        let start = source.find("./Widget.vue").unwrap() as u32;
        let generated = expected.find("./Widget.vue.ts").unwrap() as u32;
        assert_eq!(
            rewritten.source_map.get_original_offset(generated + 3),
            start
        );
    }
}

#[test]
fn shared_syntax_still_parses_once_and_resolves_every_emitted_occurrence() {
    let source = "import A from 'target'; import B from 'target'; void [A, B];";
    let mut cache = super::ModuleSpecifierFactsCache::default();
    let calls = std::cell::Cell::new(0);
    let resolver = |_: &str, _| {
        calls.set(calls.get() + 1);
        Some("/mirror/Widget.ts".to_owned())
    };
    for _ in 0..2 {
        let rewritten = ImportRewriter::new()
            .rewrite_with_alias_resolver_and_missing_vue_policy_cached(
                source,
                SourceType::ts(),
                None,
                &resolver,
                true,
                &mut cache,
            );
        let start = rewritten.code.find("/mirror/Widget.ts").unwrap() as u32;
        assert_eq!(
            rewritten.source_map.get_original_offset(start + 3),
            source.find("target").unwrap() as u32
        );
    }
    let stats = cache.stats();
    assert_eq!(stats.parses, 1);
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.retained_entries, 1);
    assert_eq!(calls.get(), 4);
    eprintln!(
        "literal_mapping_syntax {stats:?} resolver_calls={}",
        calls.get()
    );
}

#[test]
fn tsx_module_geometry_does_not_rewrite_attribute_or_text_literals() {
    let source = "import Widget from './Widget.vue';\nconst view = <div title=\"./Widget.vue\">./Widget.vue</div>;";
    let rewritten = ImportRewriter::new().rewrite(source, SourceType::tsx(), None);
    let expected = "import Widget from './Widget.vue.ts';\nconst view = <div title=\"./Widget.vue\">./Widget.vue</div>;";
    assert_eq!(rewritten.code.as_str(), expected);
    let start = source.find("./Widget.vue").unwrap() as u32;
    assert_eq!(rewritten.source_map.get_original_offset(start + 4), start);
    let text = source.find("</div>").unwrap() as u32;
    let generated = expected.find("</div>").unwrap() as u32;
    assert_eq!(rewritten.source_map.get_original_offset(generated), text);
    assert_eq!(rewritten.source_map.get_virtual_offset(text), generated);
}

#[test]
fn lsp_utf16_positions_cross_the_actual_rewritten_byte_map() {
    use crate::batch::source_map::{line_col_to_offset, offset_to_line_col};

    for newline in ["\n", "\r\n"] {
        let prefix = "/* 雪🍣 */ import W from '";
        let source = format!("// 日本語🍣{newline}{prefix}target'; const tail = 1;{newline}");
        let path = "/mirror/日本語🍣/Widget.vue.ts";
        let expected = source.replace("target", path);
        let result = ImportRewriter::new().rewrite_with(&source, SourceType::ts(), |_, _| {
            Some(path.to_owned().into())
        });
        assert_eq!(result.code.as_str(), expected);
        let authored_start = prefix.encode_utf16().count() as u32;
        for (byte, _) in path.char_indices() {
            let generated_col =
                authored_start + path.get(..byte).unwrap().encode_utf16().count() as u32;
            let generated = line_col_to_offset(&result.code, 1, generated_col).unwrap();
            let original = result.source_map.get_original_offset(generated);
            assert_eq!(
                offset_to_line_col(&source, original),
                Some((1, authored_start))
            );
        }
        for (original_col, generated_col) in [
            (authored_start - 1, authored_start - 1),
            (
                authored_start + 6,
                authored_start + path.encode_utf16().count() as u32,
            ),
            (
                authored_start + 7,
                authored_start + path.encode_utf16().count() as u32 + 1,
            ),
        ] {
            let generated = line_col_to_offset(&result.code, 1, generated_col).unwrap();
            let original = result.source_map.get_original_offset(generated);
            assert_eq!(
                offset_to_line_col(&source, original),
                Some((1, original_col))
            );
            assert_eq!(result.source_map.get_virtual_offset(original), generated);
        }
    }
}
