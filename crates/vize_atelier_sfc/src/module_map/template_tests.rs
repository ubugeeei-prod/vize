//! Template code in the module map of a script-bearing SFC.
//!
//! The render code of a component with a `<script>` was emitted without any
//! mapping, so a breakpoint, a stack frame or a coverage range inside it
//! could not be followed back to the template. Every test here decodes the
//! map and pins each segment that lands in the template, rendered as
//! `line:column "generated text" -> line:column "authored text"` with
//! 1-based lines and 0-based columns, the way a stack frame names a line and
//! a source map counts a column.
#![expect(clippy::string_slice, reason = "tests assert by panicking")]
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "tests compare std strings and build them with format!"
)]

use oxc_sourcemap::SourceMap;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};

use crate::types::{
    ScriptCompileOptions, SfcCompileOptions, SfcCompileResult, SfcError, SfcParseOptions,
    TemplateCompileOptions,
};
use crate::{SfcCompileExperimentalOptions, SfcScriptOutputMode};

mod constructs;
mod invariants;

const FILENAME: &str = "/src/Fixture.vue";

/// One way the compiler lays a component out.
#[derive(Clone, Copy, Debug, Default)]
struct Shape {
    /// The render is inlined into `setup` instead of emitted beside it.
    inline: bool,
    ssr: bool,
    /// The module keeps TypeScript instead of being emitted as JavaScript.
    typescript: bool,
    /// `mode: "function"`: the module is rewritten to a function body.
    standalone: bool,
}

const INLINE: Shape = Shape {
    inline: true,
    ssr: false,
    typescript: false,
    standalone: false,
};
const SEPARATE: Shape = Shape {
    inline: false,
    ..INLINE
};
const SSR: Shape = Shape {
    ssr: true,
    ..SEPARATE
};

fn try_compile(source: &str, shape: Shape, map: bool) -> Result<SfcCompileResult, SfcError> {
    let parse = || SfcParseOptions {
        filename: FILENAME.into(),
        ..Default::default()
    };
    let descriptor = crate::parse_sfc(source, parse()).expect("fixture parses");
    crate::compile_sfc_for_adapter_with_project(
        &descriptor,
        SfcCompileOptions {
            parse: parse(),
            script: ScriptCompileOptions {
                id: Some(FILENAME.into()),
                is_ts: shape.typescript,
                inline_template: shape.standalone,
                ..Default::default()
            },
            template: TemplateCompileOptions {
                id: Some(FILENAME.into()),
                ssr: shape.ssr,
                ..Default::default()
            },
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions {
            source_map: map,
            ..Default::default()
        },
        if shape.inline {
            SfcScriptOutputMode::InlineTemplate
        } else {
            SfcScriptOutputMode::SeparateTemplate
        },
        SfcCompileExperimentalOptions::default(),
        false,
        None,
    )
}

fn compile(source: &str, shape: Shape) -> SfcCompileResult {
    try_compile(source, shape, true).expect("fixture compiles")
}

/// Decoded `(generated line, generated column, authored line, authored
/// column)` of every segment, 0-based.
fn decode(map: &serde_json::Value) -> Vec<(u32, u32, u32, u32)> {
    let json = serde_json::to_string(map).unwrap();
    SourceMap::from_json_string(&json)
        .unwrap()
        .get_tokens()
        .map(|token| {
            (
                token.get_dst_line(),
                token.get_dst_col(),
                token.get_src_line(),
                token.get_src_col(),
            )
        })
        .collect()
}

fn offset(text: &str, line: u32, column: u32) -> usize {
    let start: usize = text
        .split_inclusive('\n')
        .take(line as usize)
        .map(str::len)
        .sum();
    let content = text.split('\n').nth(line as usize).expect("mapped line");
    start + vize_l0::line_index::utf16_offset(content, column).expect("mapped character boundary")
}

/// Every segment authored inside `<template>`, in generated order.
fn template_segments(result: &SfcCompileResult, source: &str) -> Vec<String> {
    let (start, end) = (
        source.find("<template").expect("fixture has a template"),
        source.rfind("</template>").expect("template closes"),
    );
    let window = |text: &str, at: usize| -> String {
        let text: String = text[at..].chars().take(12).collect();
        text.replace('\n', "⏎")
    };
    decode(result.map.as_ref().expect("the compile returns a map"))
        .into_iter()
        .filter_map(|(line, column, authored_line, authored_column)| {
            let generated = offset(&result.code, line, column);
            let authored = offset(source, authored_line, authored_column);
            (start <= authored && authored < end).then(|| {
                format!(
                    "{}:{column} {:?} -> {}:{authored_column} {:?}",
                    line + 1,
                    window(&result.code, generated),
                    authored_line + 1,
                    window(source, authored),
                )
            })
        })
        .collect()
}

/// The authored `line:column` (1-based line) the map gives the generated
/// position where `needle` starts, as a stack frame would resolve it: the
/// last segment at or before it on its line.
fn resolve(result: &SfcCompileResult, needle: &str) -> Option<(u32, u32)> {
    let at = result
        .code
        .find(needle)
        .expect("generated code has the needle");
    let line = result.code[..at].matches('\n').count() as u32;
    let start = result.code[..at].rfind('\n').map_or(0, |index| index + 1);
    let column = result.code[start..at].encode_utf16().count() as u32;
    decode(result.map.as_ref()?)
        .into_iter()
        .filter(|&(generated_line, generated_column, _, _)| {
            generated_line == line && generated_column <= column
        })
        .last()
        .map(|(_, _, authored_line, authored_column)| (authored_line + 1, authored_column))
}

/// The component of the report: a script that only declares, and a template
/// expression that throws when rendered.
const BROKEN: &str = "<script setup lang=\"ts\">
import Child from '../Child.vue';

const trip = undefined as unknown as { title: string };
</script>

<template>
  <Child label=\"before\" />
  <p>{{ trip.title }}</p>
</template>
";

/// The report's component whose script only imports.
const IMPORT_ONLY: &str = "<script setup lang=\"ts\">
import Child from './Child.vue';
</script>

<template>
  <Child label=\"hello\" />
</template>
";

/// One of each construct a template expression can sit in.
const CONSTRUCTS: &str = "<script setup lang=\"ts\">
import { ref } from 'vue';
import Child from './Child.vue';

const open = ref(true);
const items = ref([{ id: 1, name: 'a' }]);
const title = ref('t');
const text = ref('');
function pick(id: number) {}
</script>

<template>
  <section :title=\"title\" :class=\"{ on: open }\">
    <p v-if=\"open\">{{ title }}</p>
    <span v-else>closed</span>
    <ul>
      <li v-for=\"(item, index) in items\" :key=\"item.id\" @click=\"pick(item.id)\">
        {{ index }}: {{ item.name }}
      </li>
    </ul>
    <input v-model=\"text\" />
    <Child :label=\"title\" @done=\"open = false\">
      <template #default=\"{ row }\">{{ row.name }}</template>
    </Child>
  </section>
</template>
";

#[test]
fn expressions_after_accented_and_astral_text_keep_utf16_columns() {
    let source = BROKEN.replace("<p>{{ trip", "<p>é🌸{{ trip");
    let result = compile(&source, SEPARATE);
    assert_eq!(resolve(&result, "$setup.trip.title"), Some((9, 11)));
    let generated = result.code.find("$setup.trip.title").unwrap();
    let authored = source.find("trip.title").unwrap();
    let origins: Vec<_> = decode(result.map.as_ref().unwrap())
        .into_iter()
        .filter(|&(line, column, _, _)| offset(&result.code, line, column) == generated)
        .map(|(_, _, line, column)| (line + 1, column, offset(&source, line, column)))
        .collect();
    assert_eq!(origins, [(9, 11, authored)]);
}

#[test]
fn the_reported_component_maps_its_throwing_expression() {
    // The shape the Vite plugin serves in development and under Vitest:
    // JavaScript from a TypeScript script, with the render beside `setup`.
    let client = compile(BROKEN, SEPARATE);
    assert_eq!(
        template_segments(&client, BROKEN),
        [
            r#"4:0 "function _sf" -> 7:10 "⏎  <Child la""#,
            r#"6:17 "$setup[\"Chil" -> 8:3 "Child label=""#,
            r#"6:36 "label: \"befo" -> 8:9 "label=\"befor""#,
            r#"6:44 "before\" }),⏎" -> 8:16 "before\" />⏎ ""#,
            r#"7:25 "p\", null, _t" -> 9:3 "p>{{ trip.ti""#,
            r#"7:52 "$setup.trip." -> 9:8 "trip.title }""#,
            r#"7:64 "title), 1 /*" -> 9:13 "title }}</p>""#,
        ],
        "{}",
        client.code
    );
    // `  <p>{{ trip.title }}</p>` is line 9 and `trip` is at column 8: the
    // frame of the read that throws lands on the expression.
    assert_eq!(resolve(&client, "$setup.trip.title"), Some((9, 8)));

    let server = compile(BROKEN, SSR);
    assert_eq!(
        template_segments(&server, BROKEN),
        [
            r#"4:0 "function ssr" -> 7:10 "⏎  <Child la""#,
            r#"6:28 "$setup.Child" -> 8:3 "Child label=""#,
            r#"7:10 "p>${_ssrInte" -> 9:3 "p>{{ trip.ti""#,
            r#"7:30 "$setup.trip." -> 9:8 "trip.title }""#,
            r#"7:42 "title)}</p><" -> 9:13 "title }}</p>""#,
        ],
        "{}",
        server.code
    );
    assert_eq!(resolve(&server, "$setup.trip.title"), Some((9, 8)));

    // Production inlines the render into `setup`. A hoisted prop object is
    // module-level static data with no segment of its own.
    let inline = compile(BROKEN, INLINE);
    assert_eq!(
        template_segments(&inline, BROKEN),
        [
            r#"16:17 "Child, _hois" -> 8:3 "Child label=""#,
            r#"17:25 "p\", null, _t" -> 9:3 "p>{{ trip.ti""#,
            r#"17:52 "_unref(trip)" -> 9:8 "trip.title }""#,
            r#"17:65 "title), 1 /*" -> 9:13 "title }}</p>""#,
        ],
        "{}",
        inline.code
    );
    assert_eq!(resolve(&inline, "_unref(trip).title"), Some((9, 8)));
}

#[test]
fn a_script_that_only_imports_still_maps_its_render() {
    // Nothing in this script is a statement, so the render is the only code
    // a coverage tool can attribute to the file.
    let client = compile(IMPORT_ONLY, SEPARATE);
    assert_eq!(
        template_segments(&client, IMPORT_ONLY),
        [
            r#"4:0 "function _sf" -> 5:10 "⏎  <Child la""#,
            r#"5:9 "(_openBlock(" -> 6:2 "<Child label""#,
            r#"5:37 "$setup[\"Chil" -> 6:3 "Child label=""#,
            r#"5:56 "label: \"hell" -> 6:9 "label=\"hello""#,
            r#"5:64 "hello\" }))⏎}" -> 6:16 "hello\" />⏎</""#,
        ],
        "{}",
        client.code
    );
    let server = compile(IMPORT_ONLY, SSR);
    assert_eq!(
        template_segments(&server, IMPORT_ONLY),
        [
            r#"5:0 "function ssr" -> 5:10 "⏎  <Child la""#,
            r#"6:28 "$setup.Child" -> 6:3 "Child label=""#,
        ],
        "{}",
        server.code
    );
    let inline = compile(IMPORT_ONLY, INLINE);
    assert_eq!(
        template_segments(&inline, IMPORT_ONLY),
        [
            r#"14:9 "(_openBlock(" -> 6:2 "<Child label""#,
            r#"14:37 "Child, _hois" -> 6:3 "Child label=""#,
        ],
        "{}",
        inline.code
    );
}

/// A component with a normal `<script>`: the render is appended to it on the
/// client and precedes it on the server.
const OPTIONS_API: &str = "<script>
export default {
  data: () => ({ count: 0 }),
}
</script>

<template>
  <button @click=\"count++\">{{ count }}</button>
</template>
";

#[test]
fn a_normal_script_component_maps_its_render() {
    let client = compile(OPTIONS_API, SEPARATE);
    assert_eq!(
        template_segments(&client, OPTIONS_API),
        [
            r#"8:0 "function _sf" -> 7:10 "⏎  <button @""#,
            r#"9:9 "(_openBlock(" -> 8:2 "<button @cli""#,
            r#"9:45 "button\", {⏎ " -> 8:3 "button @clic""#,
            r#"10:4 "onClick: $ev" -> 8:11 "click=\"count""#,
            r#"10:13 "$event => ($" -> 8:18 "count++\">{{ ""#,
            r#"11:22 "$data.count)" -> 8:30 "count }}</bu""#,
        ],
        "{}",
        client.code
    );
    let server = compile(OPTIONS_API, SSR);
    assert_eq!(
        template_segments(&server, OPTIONS_API),
        [
            r#"3:0 "function ssr" -> 7:10 "⏎  <button @""#,
            r#"4:10 "button${_ssr" -> 8:3 "button @clic""#,
            r#"4:61 "$data.count)" -> 8:30 "count }}</bu""#,
        ],
        "{}",
        server.code
    );
}

const TEMPLATE_ONLY: &str = "<template>
  <p :title=\"tip\">{{ msg }}</p>
</template>
";

#[test]
fn a_template_without_a_script_is_mapped_like_one_with_a_script() {
    let client = compile(TEMPLATE_ONLY, SEPARATE);
    assert_eq!(
        template_segments(&client, TEMPLATE_ONLY),
        [
            r#"3:0 "export funct" -> 1:10 "⏎  <p :title""#,
            r#"4:9 "(_openBlock(" -> 2:2 "<p :title=\"t""#,
            r#"4:45 "p\", { title:" -> 2:3 "p :title=\"ti""#,
            r#"4:51 "title: _ctx." -> 2:6 "title=\"tip\">""#,
            r#"4:58 "_ctx.tip }, " -> 2:13 "tip\">{{ msg ""#,
            r#"4:87 "_ctx.msg), 9" -> 2:21 "msg }}</p>⏎<""#,
        ],
        "{}",
        client.code
    );
    let server = compile(TEMPLATE_ONLY, SSR);
    assert_eq!(
        template_segments(&server, TEMPLATE_ONLY),
        [
            r#"4:0 "function ssr" -> 1:10 "⏎  <p :title""#,
            r#"5:10 "p${_ssrRende" -> 2:3 "p :title=\"ti""#,
            r#"5:43 "title: _ctx." -> 2:6 "title=\"tip\">""#,
            r#"5:50 "_ctx.tip }, " -> 2:13 "tip\">{{ msg ""#,
            r#"5:90 "_ctx.msg)}</" -> 2:21 "msg }}</p>⏎<""#,
        ],
        "{}",
        server.code
    );
}
