//! Authored `.ts` syntax stays valid, and exactly mapped, in TSX output.

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::{ContentMapperSpanKind, generate_vue_content_mapper_transform};
use crate::batch::ContentMapperTransform;

fn transform(source: &str) -> ContentMapperTransform {
    generate_vue_content_mapper_transform(Path::new("App.vue"), source).unwrap()
}

fn assert_tsx(result: &ContentMapperTransform) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &result.text, SourceType::tsx()).parse();
    assert!(
        !parsed.panicked && parsed.diagnostics.is_empty(),
        "{:?}\n{}",
        parsed.diagnostics,
        result.text
    );
}

/// Every verbatim span names identical bytes on both sides.
fn assert_verbatim(source: &str, result: &ContentMapperTransform) {
    for span in &result.mappings {
        let [generated, generated_len, original, original_len, kind, _] = span.0;
        if kind == ContentMapperSpanKind::Verbatim as usize {
            assert_eq!(
                &result.text[generated..generated + generated_len],
                &source[original..original + original_len],
                "{span:?}\n{}",
                result.text
            );
        }
    }
}

/// The generated offset of `needle` inside the first authored `context`,
/// through one verbatim span.
fn generated_at(
    source: &str,
    result: &ContentMapperTransform,
    context: &str,
    needle: &str,
) -> usize {
    let authored = source.find(context).unwrap() + context.find(needle).unwrap();
    let span = result
        .mappings
        .iter()
        .find(|span| {
            span.0[4] == ContentMapperSpanKind::Verbatim as usize
                && span.0[2] <= authored
                && authored + needle.len() <= span.0[2] + span.0[3]
        })
        .unwrap_or_else(|| panic!("{needle:?} is unmapped: {:?}", result.mappings));
    span.0[0] + (authored - span.0[2])
}

fn unmapped(result: &ContentMapperTransform, generated: usize) -> bool {
    result
        .mappings
        .iter()
        .all(|span| !(span.0[0]..span.0[0] + span.0[1]).contains(&generated))
}

#[test]
fn lone_arrow_type_parameters_gain_an_unmapped_comma() {
    let source = r#"<script setup lang="ts">
const identity = <T>(value: T): T => value;
const run = async <T>(value: T) => value;
const constant = <const T>(value: T) => value;
const commented = <T /* only */>(value: T) => value;
const box = { make: <T>(value: T) => [value] };
class Holder { map = <T>(value: T) => value; }
function withPick(pick = <T>(value: T) => value) { return pick; }
const mapped = [1].map(<T>(value: T) => value);
const count = identity(1);
</script>
<template>{{ count }} {{ run }} {{ constant }} {{ commented }} {{ box }} {{ Holder }} {{ withPick }} {{ mapped }}</template>
"#;
    let result = transform(source);
    assert_tsx(&result);
    assert_verbatim(source, &result);
    let text = result.text.as_str();
    for expected in [
        "const identity = <T,>(value: T): T => value;",
        "const run = async <T,>(value: T) => value;",
        "const constant = <const T,>(value: T) => value;",
        "const commented = <T, /* only */>(value: T) => value;",
        "const box = { make: <T,>(value: T) => [value] };",
        "class Holder { map = <T,>(value: T) => value; }",
        "function withPick(pick = <T,>(value: T) => value) { return pick; }",
        "const mapped = [1].map(<T,>(value: T) => value);",
    ] {
        assert!(text.contains(expected), "{expected}\n{text}");
    }
    for line in source
        .lines()
        .filter(|line| line.contains("<T") || line.contains("<const T"))
    {
        let parameter = generated_at(source, &result, line, "T");
        assert_eq!(&text[parameter..parameter + 2], "T,", "{line}");
        assert!(unmapped(&result, parameter + 1), "{line}: {text}");
        assert!(!unmapped(&result, parameter), "{line}: {text}");
    }
    let value = generated_at(source, &result, "value: T): T => value", "value");
    assert_eq!(&text[value..value + 5], "value");
}

#[test]
fn unambiguous_type_parameters_stay_authored() {
    let source = r#"<script setup lang="ts">
const trailing = <T,>(value: T) => value;
const bounded = <T extends unknown>(value: T) => value;
const defaulted = <T = string>(value: T) => value;
const pair = <T, U>(left: T, right: U) => [left, right];
function declared<T>(value: T) { return value; }
const typed: <T>(value: T) => T = (value) => value;
</script>
<template>{{ trailing }} {{ bounded }} {{ defaulted }} {{ pair }} {{ declared }} {{ typed }}</template>
"#;
    let result = transform(source);
    assert_tsx(&result);
    assert_verbatim(source, &result);
    let script = &source[source.find("const trailing").unwrap()..source.find("</script>").unwrap()];
    for line in script.lines() {
        assert!(result.text.contains(line), "{line}\n{}", result.text);
    }
}

#[test]
fn angle_bracket_assertions_become_mapped_as_expressions() {
    let source = r#"<script setup lang="ts">
const element = <HTMLElement>document.body;
const loose = <any>element;
const nested = <string><unknown>1;
const name = (<HTMLElement>element).tagName;
const tuple = <const>['a', 'b'];
const boxed = () => <{ value: number }>{ value: 1 };
const cast = <T,>(value: unknown) => <T>value;
const wrapped = <(value: number) => number>(<T>(value: T) => value);
const spaced = < HTMLElement >
  element;
</script>
<template>{{ loose }} {{ nested }} {{ name }} {{ tuple }} {{ boxed }} {{ cast }} {{ wrapped }} {{ spaced }}</template>
"#;
    let result = transform(source);
    assert_tsx(&result);
    assert_verbatim(source, &result);
    let text = result.text.as_str();
    for expected in [
        "const element = ((document.body) as HTMLElement);",
        "const loose = ((element) as any);",
        "const nested = ((((1) as unknown)) as string);",
        "const name = (((element) as HTMLElement)).tagName;",
        "const tuple = ((['a', 'b']) as const);",
        "const boxed = () => (({ value: 1 }) as { value: number });",
        "const cast = <T,>(value: unknown) => ((value) as T);",
        "const wrapped = (((<T,>(value: T) => value)) as (value: number) => number);",
        "const spaced = ((element) as HTMLElement);",
    ] {
        assert!(text.contains(expected), "{expected}\n{text}");
    }
    // The moved expression and type each map to their authored bytes.
    let body = generated_at(source, &result, "document.body", "document.body");
    let element = generated_at(source, &result, "HTMLElement>document", "HTMLElement");
    assert_eq!(&text[body..body + 13], "document.body");
    assert_eq!(&text[element..element + 11], "HTMLElement");
    assert!(body < element);
    let open = text.find("((document.body) as HTMLElement)").unwrap();
    for synthetic in [open, open + 15, open + 16, open + 31] {
        assert!(unmapped(&result, synthetic), "{synthetic}: {text}");
    }
    // The inner `(` starts the assertion node: diagnostics only, from `<`.
    let less_than = source.find("<HTMLElement>document").unwrap();
    assert!(
        result.mappings.iter().any(|span| span.0
            == [
                open + 1,
                1,
                less_than,
                1,
                ContentMapperSpanKind::Atom as usize,
                0
            ]),
        "{:?}",
        result.mappings
    );
}

#[test]
fn template_expressions_follow_the_same_projection() {
    let source = r#"<script setup lang="ts">
const count: unknown = 1;
const target = null as unknown;
</script>
<template>
  <button @click="(<HTMLElement>target).focus()">{{ (<number>count).toFixed() }}</button>
</template>
"#;
    let result = transform(source);
    assert_tsx(&result);
    assert_verbatim(source, &result);
    let text = result.text.as_str();
    assert!(text.contains("(((count) as number)).toFixed()"), "{text}");
    assert!(
        text.contains("(((target) as HTMLElement)).focus()"),
        "{text}"
    );
    let count = generated_at(source, &result, "count).toFixed", "count");
    assert_eq!(&text[count..count + 5], "count");
}

#[test]
fn plain_and_generic_scripts_follow_the_same_projection() {
    let plain = r#"<script lang="ts">
export const helper = <T>(value: T) => value;
const root = <HTMLElement>document.body;
export default { name: 'Plain', data: () => ({ root }) };
</script>
<template><div /></template>
"#;
    let result = transform(plain);
    assert_tsx(&result);
    assert_verbatim(plain, &result);
    assert!(
        result
            .text
            .contains("export const helper = <T,>(value: T) => value;")
            || result
                .text
                .contains("const helper = <T,>(value: T) => value;"),
        "{}",
        result.text
    );
    assert!(
        result.text.contains("((document.body) as HTMLElement)"),
        "{}",
        result.text
    );

    let generic = r#"<script setup lang="ts" generic="T extends string">
const props = defineProps<{ value: T }>();
const echo = <U>(input: U) => input;
const typed = <T>props.value;
</script>
<template>{{ echo(typed) }}</template>
"#;
    let result = transform(generic);
    assert_tsx(&result);
    assert_verbatim(generic, &result);
    assert!(
        result
            .text
            .contains("const echo = <U,>(input: U) => input;"),
        "{}",
        result.text
    );
    assert!(
        result.text.contains("const typed = ((props.value) as T);"),
        "{}",
        result.text
    );
}

#[test]
fn jsx_scripts_keep_their_authored_syntax() {
    let source = r#"<script setup lang="tsx">
const identity = <T,>(value: T) => value;
const view = <div>{identity(1)}</div>;
</script>
<template><component :is="() => view" /></template>
"#;
    let result = transform(source);
    assert_verbatim(source, &result);
    assert!(
        result
            .text
            .contains("const identity = <T,>(value: T) => value;")
    );
    assert!(
        result
            .text
            .contains("const view = <div>{identity(1)}</div>;")
    );
}
