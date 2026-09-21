//! The structured SFC module map end to end: maps are additive (TS-11) for
//! every module shape, and representative modules pin every segment.

#![allow(clippy::disallowed_types, clippy::disallowed_macros)]

use oxc_sourcemap::SourceMap;
use vize_atelier_core::CodegenOptions;

use crate::types::{ScriptCompileOptions, SfcCompileOptions, SfcParseOptions};

const FILENAME: &str = "/src/Fixture.vue";

fn compile(source: &str, inline_template: bool, map: bool) -> crate::types::SfcCompileResult {
    compile_with(source, inline_template, false, map)
}

fn compile_with(
    source: &str,
    inline_template: bool,
    is_ts: bool,
    map: bool,
) -> crate::types::SfcCompileResult {
    let parse = || SfcParseOptions {
        filename: FILENAME.into(),
        ..Default::default()
    };
    let descriptor = crate::parse_sfc(source, parse()).expect("fixture parses");
    crate::compile_sfc_with_template_syntax_and_codegen_options(
        &descriptor,
        SfcCompileOptions {
            parse: parse(),
            script: ScriptCompileOptions {
                id: Some(FILENAME.into()),
                inline_template,
                is_ts,
                ..Default::default()
            },
            ..Default::default()
        },
        vize_atelier_core::TemplateSyntaxMode::Standard,
        CodegenOptions {
            source_map: map,
            ..Default::default()
        },
    )
    .expect("fixture compiles")
}

/// Every segment as `generated text -> authored text`, 12 chars each.
fn render(code: &str, source: &str, map: &serde_json::Value) -> Vec<String> {
    let json = serde_json::to_string(map).unwrap();
    let map = SourceMap::from_json_string(&json).unwrap();
    let at = |text: &str, line: u32, column: u32| -> String {
        let start: usize = text
            .split_inclusive('\n')
            .take(line as usize)
            .map(str::len)
            .sum();
        let window: String = text[start + column as usize..].chars().take(12).collect();
        window.replace('\n', "⏎")
    };
    map.get_tokens()
        .map(|token| {
            let generated = at(code, token.get_dst_line(), token.get_dst_col());
            let authored = at(source, token.get_src_line(), token.get_src_col());
            format!("{generated:?} -> {authored:?}")
        })
        .collect()
}

const SHAPES: [&str; 8] = [
    "<script setup>\nimport { ref } from 'vue'\nconst n = ref(1)\n</script>\n<template><p>{{ n }}</p></template>\n",
    "<script setup lang=\"ts\">\nconst props = defineProps<{ a: string }>()\nconst b: number = 1\n</script>\n<template><p>{{ props.a }}{{ b }}</p></template>\n",
    "<script lang=\"ts\">\nexport default { data: () => ({ n: 1 as number }) }\n</script>\n<template><p>{{ n }}</p></template>\n",
    "<script>\nexport const v = 1\n</script>\n<script setup>\nconst w = v + 1\n</script>\n<template><p>{{ w }}</p></template>\n",
    "<script setup>\nconst Lazy = defineLazyHydrationComponent('visible', () => import('./A.vue'))\nconst keep = 1\n</script>\n<template><Lazy :n=\"keep\" /></template>\n",
    "<script setup>\ndefinePageMeta({ layout: 'x' })\nconst after = 2\n</script>\n<template><p>{{ after }}</p></template>\n",
    "<script setup>\nimport { reactive } from 'vue'\nconst form = reactive({ a: '' })\nconst stay = await Promise.resolve(1)\n</script>\n<template><input v-model=\"form\">{{ stay }}</template>\n",
    "<script setup>\r\nconst crlf = 1\r\nconst next = crlf + 1\r\n</script>\r\n<template><p>{{ next }}</p></template>\r\n",
];

#[test]
fn maps_are_additive_and_verified_for_every_module_shape() {
    for inline_template in [false, true] {
        for source in SHAPES {
            let off = compile(source, inline_template, false);
            let on = compile(source, inline_template, true);
            assert_eq!(off.code, on.code, "maps changed the module: {source}");
            assert!(off.map.is_none());
            let map = on.map.unwrap_or_else(|| panic!("no map for {source}"));
            assert_eq!(map["sources"], serde_json::json!([FILENAME]));
            assert!(!render(&on.code, source, &map).is_empty(), "{source}");
        }
    }
}

#[test]
fn setup_statements_map_token_by_token() {
    let source = SHAPES[0];
    let result = compile(source, false, true);
    let segments = render(&result.code, source, result.map.as_ref().unwrap());
    // The rebuilt import is anchored at its statement; the copied statement
    // maps every token.
    assert_eq!(
        segments,
        [
            r#""import { ref" -> "import { ref""#,
            r#""const n = re" -> "const n = re""#,
            r#""n = ref(1)⏎⏎" -> "n = ref(1)⏎<""#,
            r#""= ref(1)⏎⏎re" -> "= ref(1)⏎</s""#,
            r#""ref(1)⏎⏎retu" -> "ref(1)⏎</scr""#,
            r#""(1)⏎⏎return " -> "(1)⏎</script""#,
            r#""1)⏎⏎return (" -> "1)⏎</script>""#,
            r#"")⏎⏎return (_" -> ")⏎</script>⏎""#,
        ],
        "{}",
        result.code
    );
}

#[test]
fn rewritten_statements_anchor_at_their_authored_statement() {
    let source = SHAPES[1];
    let result = compile(source, false, true);
    let segments = render(&result.code, source, result.map.as_ref().unwrap());
    for expected in [
        r#""const props " -> "const props ""#,
        r#""const b = 1;" -> "const b: num""#,
    ] {
        assert!(
            segments.iter().any(|segment| segment == expected),
            "{expected} not in {segments:#?}"
        );
    }
    let source = SHAPES[2];
    let result = compile(source, false, true);
    let segments = render(&result.code, source, result.map.as_ref().unwrap());
    assert!(
        segments
            .iter()
            .any(|segment| segment == r#""const _sfc_m" -> "export defau""#),
        "{segments:#?}"
    );
}

#[test]
fn stripping_typescript_at_the_boundary_carries_the_map() {
    let source = SHAPES[1];
    let typed = compile_with(source, false, true, true);
    assert!(typed.code.contains("const b: number = 1"), "{}", typed.code);
    let (code, _, map) =
        crate::module_shape::finalize_module_output_with_map(typed.code.clone(), false, typed.map);
    assert!(!code.contains(": number"), "{code}");
    let (plain, _) = crate::module_shape::finalize_module_output(typed.code, false);
    assert_eq!(code, plain, "carrying the map changed the stripped module");
    let segments = render(&code, source, &map.expect("the map survives the strip"));
    assert!(
        segments
            .iter()
            .any(|segment| segment == r#""const b = 1;" -> "const b: num""#),
        "{segments:#?}"
    );
}

/// Carried over from the deleted text-matching recovery's tests (#3399): the
/// codegen flag gates the map, which embeds the authored `.vue`, and a copied
/// statement resolves to its authored line and column.
#[test]
fn compile_sfc_attaches_a_map_only_when_the_flag_is_on() {
    const COUNTER_VUE: &str = "<template>\n  <button @click=\"bump\">{{ count }}</button>\n</template>\n\n<script setup>\nimport { ref } from 'vue'\n\nconst count = ref(0)\nfunction bump() {\n  count.value += 1\n}\n</script>\n";
    assert_eq!(compile(COUNTER_VUE, false, false).map, None);

    let result = compile(COUNTER_VUE, false, true);
    let map = result.map.expect("source_map: true attaches a map");
    assert_eq!(map["sources"], serde_json::json!([FILENAME]));
    assert_eq!(map["sourcesContent"], serde_json::json!([COUNTER_VUE]));
    let line = result
        .code
        .lines()
        .position(|line| line == "const count = ref(0)")
        .expect("the emitter copies the authored statement verbatim") as u32;
    let json = serde_json::to_string(&map).unwrap();
    let parsed = SourceMap::from_json_string(&json).unwrap();
    let first = parsed
        .get_tokens()
        .find(|token| token.get_dst_line() == line)
        .map(|token| {
            (
                token.get_dst_col(),
                token.get_src_line(),
                token.get_src_col(),
            )
        });
    assert_eq!(first, Some((0, 7, 0)));
}
