//! Authored `.ts` syntax stays valid, and exactly mapped, in TSX output.
#![expect(clippy::unwrap_used, reason = "projection laws fail by panicking")]
#![expect(clippy::string_slice, reason = "projection laws fail by panicking")]

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::batch::{ContentMapperTransform, generate_vue_content_mapper_transform};

/// Protocol v1 span kinds.
const VERBATIM: usize = 0;
const ATOM: usize = 1;

fn transform(source: &str) -> ContentMapperTransform {
    let result = generate_vue_content_mapper_transform(Path::new("App.vue"), source).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &result.text, SourceType::tsx()).parse();
    assert!(!parsed.panicked, "{}", result.text);
    assert_eq!(
        parsed.diagnostics.len(),
        0,
        "{:?}\n{}",
        parsed.diagnostics,
        result.text
    );
    for span in &result.mappings {
        let [generated, generated_len, original, original_len, kind, _] = span.0;
        if kind == VERBATIM {
            assert_eq!(
                &result.text[generated..generated + generated_len],
                &source[original..original + original_len],
                "{span:?}\n{}",
                result.text
            );
        }
    }
    result
}

/// The shortest generated text from the first to the last byte of the first
/// authored `snippet`, both through verbatim spans.
fn projected<'a>(source: &str, result: &'a ContentMapperTransform, snippet: &str) -> &'a str {
    let start = source.find(snippet).unwrap();
    let last = start + snippet.len() - 1;
    let generated = |at: usize| {
        result
            .mappings
            .iter()
            .filter(move |span| {
                span.0[4] == VERBATIM && span.0[2] <= at && at < span.0[2] + span.0[3]
            })
            .map(move |span| span.0[0] + (at - span.0[2]))
    };
    let (from, to) = generated(start)
        .flat_map(|from| {
            generated(last)
                .filter(move |to| *to >= from)
                .map(move |to| (from, to + 1))
        })
        .min_by_key(|(from, to)| to - from)
        .unwrap_or_else(|| panic!("{snippet:?} is unmapped: {:?}", result.mappings));
    &result.text[from..to]
}

/// The generated offset of `needle` inside the first authored `context`.
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
            span.0[4] == VERBATIM
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

fn assert_projected(source: &str, result: &ContentMapperTransform, pairs: &[(&str, &str)]) {
    for (authored, expected) in pairs {
        assert_eq!(projected(source, result, authored), *expected, "{authored}");
    }
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
    let lines = source
        .lines()
        .filter(|line| line.contains('<') && line.contains("=>"));
    for line in lines {
        let parameter = generated_at(source, &result, line, "T");
        assert_eq!(&result.text[parameter..parameter + 2], "T,", "{line}");
        assert!(unmapped(&result, parameter + 1), "{line}: {}", result.text);
        assert!(!unmapped(&result, parameter), "{line}: {}", result.text);
    }
    assert_projected(
        source,
        &result,
        &[
            (
                "const identity = <T>(value: T): T => value;",
                "const identity = <T,>(value: T): T => value;",
            ),
            (
                "const run = async <T>(value: T) => value;",
                "const run = async <T,>(value: T) => value;",
            ),
            (
                "const constant = <const T>(value: T) => value;",
                "const constant = <const T,>(value: T) => value;",
            ),
            (
                "const commented = <T /* only */>(value: T) => value;",
                "const commented = <T, /* only */>(value: T) => value;",
            ),
            (
                "const box = { make: <T>(value: T) => [value] };",
                "const box = { make: <T,>(value: T) => [value] };",
            ),
            (
                "class Holder { map = <T>(value: T) => value; }",
                "class Holder { map = <T,>(value: T) => value; }",
            ),
            (
                "function withPick(pick = <T>(value: T) => value) { return pick; }",
                "function withPick(pick = <T,>(value: T) => value) { return pick; }",
            ),
            (
                "const mapped = [1].map(<T>(value: T) => value);",
                "const mapped = [1].map(<T,>(value: T) => value);",
            ),
        ],
    );
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
    for line in source
        .lines()
        .filter(|line| line.starts_with("const") || line.starts_with("function"))
    {
        assert_eq!(projected(source, &result, line), line, "{}", result.text);
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
    assert_projected(
        source,
        &result,
        &[
            (
                "const element = <HTMLElement>document.body;",
                "const element = ((document.body) as HTMLElement);",
            ),
            (
                "const loose = <any>element;",
                "const loose = ((element) as any);",
            ),
            (
                "const nested = <string><unknown>1;",
                "const nested = ((((1) as unknown)) as string);",
            ),
            (
                "const name = (<HTMLElement>element).tagName;",
                "const name = (((element) as HTMLElement)).tagName;",
            ),
            (
                "const tuple = <const>['a', 'b'];",
                "const tuple = ((['a', 'b']) as const);",
            ),
            (
                "const boxed = () => <{ value: number }>{ value: 1 };",
                "const boxed = () => (({ value: 1 }) as { value: number });",
            ),
            (
                "const cast = <T,>(value: unknown) => <T>value;",
                "const cast = <T,>(value: unknown) => ((value) as T);",
            ),
            (
                "const wrapped = <(value: number) => number>(<T>(value: T) => value);",
                "const wrapped = (((<T,>(value: T) => value)) as (value: number) => number);",
            ),
            (
                "const spaced = < HTMLElement >\n  element;",
                "const spaced = ((element) as HTMLElement);",
            ),
        ],
    );
    // The moved expression and type each map to their authored bytes.
    let text = result.text.as_str();
    let body = generated_at(source, &result, "document.body", "document.body");
    let element = generated_at(source, &result, "HTMLElement>document", "HTMLElement");
    assert_eq!(&text[body..body + 13], "document.body");
    assert_eq!(&text[element..element + 11], "HTMLElement");
    let open = body - 2;
    assert_eq!(
        &text[open..element + 12],
        "((document.body) as HTMLElement)"
    );
    for synthetic in [open, open + 15, open + 16, open + 31] {
        assert!(unmapped(&result, synthetic), "{synthetic}: {text}");
    }
    // The inner `(` starts the assertion node: diagnostics only, from `<`.
    let less_than = source.find("<HTMLElement>document").unwrap();
    let anchors = result
        .mappings
        .iter()
        .filter(|span| span.0 == [open + 1, 1, less_than, 1, ATOM, 0])
        .count();
    assert_eq!(anchors, 1, "{:?}", result.mappings);
    // Nested assertions and a line that also carries a generator-inserted
    // arrow comma keep exactly one anchor per assertion.
    for (context, needle) in [
        ("const nested = <string>", "<"),
        ("<string><unknown>1", "<unknown>"),
        ("const wrapped = <(", "<"),
        ("=> number>(<T>", "<T>"),
        ("const spaced = <", "<"),
    ] {
        let less_than = source.find(context).unwrap() + context.find(needle).unwrap();
        let anchors = result
            .mappings
            .iter()
            .filter(|span| span.0[2..] == [less_than, 1, ATOM, 0])
            .count();
        let expected = usize::from(needle != "<T>");
        assert_eq!(anchors, expected, "{context}: {:?}", result.mappings);
    }
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
    assert_projected(
        source,
        &result,
        &[
            (
                "(<number>count).toFixed()",
                "(((count) as number)).toFixed()",
            ),
            (
                "(<HTMLElement>target).focus()",
                "(((target) as HTMLElement)).focus()",
            ),
        ],
    );
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
    assert_projected(
        plain,
        &result,
        &[
            (
                "helper = <T>(value: T) => value;",
                "helper = <T,>(value: T) => value;",
            ),
            (
                "const root = <HTMLElement>document.body;",
                "const root = ((document.body) as HTMLElement);",
            ),
        ],
    );

    let generic = r#"<script setup lang="ts" generic="T extends string">
const props = defineProps<{ value: T }>();
const echo = <U>(input: U) => input;
const typed = <T>props.value;
</script>
<template>{{ echo(typed) }}</template>
"#;
    let result = transform(generic);
    assert_projected(
        generic,
        &result,
        &[
            (
                "const echo = <U>(input: U) => input;",
                "const echo = <U,>(input: U) => input;",
            ),
            (
                "const typed = <T>props.value;",
                "const typed = ((props.value) as T);",
            ),
        ],
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
    let result = generate_vue_content_mapper_transform(Path::new("App.vue"), source).unwrap();
    for line in [
        "const identity = <T,>(value: T) => value;",
        "const view = <div>{identity(1)}</div>;",
    ] {
        assert_eq!(projected(source, &result, line), line, "{}", result.text);
    }
}
