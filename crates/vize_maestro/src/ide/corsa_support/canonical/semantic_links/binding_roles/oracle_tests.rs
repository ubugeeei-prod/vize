use std::ops::Range;

use oxc_ast::AstKind;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use tower_lsp::lsp_types::Url;
use vize_l0::{String, cstr};

use super::setup_binding_ranges;
use crate::{ide::IdeContext, server::ServerState};

// Exact pre-optimization body: the ordered semantic-node vector is the oracle.
fn semantic_ranges(ctx: &IdeContext<'_>, uri: &Url, source: &str) -> Vec<Range<usize>> {
    if !uri.path().ends_with(".vue") {
        return Vec::new();
    }
    let local = IdeContext::for_unopened(ctx.state, uri, 0, source.to_owned());
    let Some(descriptor) = local.descriptor() else {
        return Vec::new();
    };
    let Some(script) = descriptor.script_setup.as_ref() else {
        return Vec::new();
    };
    let allocator = oxc_allocator::Allocator::default();
    let source_type = match script.lang.as_deref() {
        Some("tsx") => SourceType::tsx(),
        Some("jsx") => SourceType::jsx(),
        _ => SourceType::ts(),
    };
    let parsed = Parser::new(&allocator, &script.content, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    semantic
        .nodes()
        .iter()
        .filter_map(|node| match node.kind() {
            AstKind::BindingIdentifier(binding) => Some(
                script.loc.start + binding.span.start as usize
                    ..script.loc.start + binding.span.end as usize,
            ),
            _ => None,
        })
        .collect()
}

const SCRIPTS: &[&str] = &[
    "",
    "const value = 1;",
    "let mutable = 1; var legacy = 2;",
    "const left = 1, right = 2;",
    "const [first, , third, ...rest] = input;",
    "const { key: renamed, shorthand, ...remaining } = input;",
    "const { key: { nested = 1 }, list: [first, ...rest] } = input;",
    "const { [computed]: value } = input;",
    "let first, second; [first, second] = input;",
    "const object = { key: value, shorthand };",
    "const object = { method(argument) { const local = argument; return local; } };",
    "function outer(first, { key: second }, ...rest) { const local = first; }",
    "const arrow = (first, ...rest) => { let local = first; return local; };",
    "const named = function inner(argument) { return argument; };",
    "async function task(input) { const result = await input; return result; }",
    "function* generate(input) { const result = yield input; return result; }",
    "function duplicate(input) {} function duplicate(input) {}",
    "const value = 1; { const value = 2; }",
    "let duplicate = 1; let duplicate = 2;",
    "try { throw 1; } catch (caught) { const recovered = caught; }",
    "try { throw 1; } catch { const recovered = 1; } finally { let complete = true; }",
    "for (const item of items) { const copy = item; }",
    "for (let key in object) { const copy = key; }",
    "for (let index = 0; index < 1; index++) { const copy = index; }",
    "import Default from 'dep';",
    "import { original as local, named } from 'dep';",
    "import * as Namespace from 'dep';",
    "import type Default from 'dep'; import type { Type as Local } from 'dep';",
    "import Alias = Namespace.Member;",
    "import Alias = require('dep');",
    "export const named = 1; export function exported(argument) { return argument; }",
    "export default function Named(argument) { return argument; }",
    "export { value as renamed }; export type { Type as Exported } from 'dep';",
    "type Shape = { key: string; method(argument: number): void };",
    "type Identity<T> = T; type Conditional<T> = T extends infer Inferred ? Inferred : never;",
    "type Mapped<T> = { [Key in keyof T as `get${string & Key}`]?: T[Key] };",
    "interface Shape<T> extends Base<T> { field: T; method(argument: T): void }",
    "enum Choice { First, Second = 2 } const enum Small { One = 1 }",
    "namespace Space { export const inside = 1; export function method(argument: number) {} }",
    "declare namespace Ambient { const value: string; function call(argument: string): void; }",
    "declare const declared: string; declare function overload(argument: string): void;",
    "declare class Ambient { method(argument: number): void; }",
    "class Example { field = 1; #private = 2; method(argument) { const local = argument; } }",
    "const Expression = class Named { constructor(argument) { const local = argument; } };",
    "class Example<T> { constructor(public value: T, private other: T) {} }",
    "class Example { static { const local = 1; } get value() { return 1; } set value(input) {} }",
    "@decorate class Example { @decorate method(@decorate argument: number) {} }",
    "const element = <Component key={value} render={(argument) => argument} />;",
    "const element = <><Component>{items.map((item) => <span>{item}</span>)}</Component></>;",
    "const generic = <T,>(argument: T): T => argument;",
    "function generic<T>(argument: T) { return argument; }",
    "const asserted = (value as Shape)!; const checked = value satisfies Shape;",
    "using resource = acquire(); const next = resource;",
    "await using resource = acquire();",
    r"const \u0061 = 1; const 雪 = '🌸'; function 花(空) { return 空; }",
    "const $value = 1; const _value = $value; const café = _value;",
    "const { label = 'Name', checked: active } = defineProps<{ label: string; checked: boolean }>();",
    "const model = defineModel<string>(); const emit = defineEmits<{ change: [value: string] }>();",
    "const props = withDefaults(defineProps<{ value?: string }>(), { value: 'default' });",
    "const literal = 'const falseBinding = 1'; // const commentBinding = 2\nconst real = 1;",
    "function incomplete( {",
    "const invalid = ; const later = 1;",
    "const optional = object?.value; const later = optional;",
    "const { invalid: } = object;",
    "const element = <Component;",
];

fn paired(state: &ServerState, uri: &Url, source: &str, label: &str) -> Vec<Range<usize>> {
    let ctx = IdeContext::testing(state, uri, 0, source.to_owned());
    let expected = semantic_ranges(&ctx, uri, source);
    let actual = setup_binding_ranges(&ctx, uri, source);
    assert_eq!(actual, expected, "whole ordered vector: {label}");
    println!(
        "binding-range-oracle {}",
        serde_json::json!({ "case": label, "original": expected, "candidate": actual })
    );
    actual
}

#[test]
fn binding_identifier_visitation_preserves_semantic_range_vectors() {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/BindingRanges.vue").expect("URI");
    let mut compared = 0;
    let mut nonempty = 0;
    for (case, script) in SCRIPTS.iter().enumerate() {
        for lang in ["", "ts", "tsx", "jsx", "js"] {
            let attribute = if lang.is_empty() {
                String::new()
            } else {
                cstr!(" lang=\"{lang}\"")
            };
            for (prefix, preamble) in ["", "<template><p>雪🌸</p></template>\n"]
                .iter()
                .enumerate()
            {
                for (newline, separator) in ["\n", "\r\n"].iter().enumerate() {
                    let source = cstr!("{preamble}<script setup{attribute}>\n{script}\n</script>")
                        .replace('\n', separator);
                    let label = cstr!("case={case}/lang={lang}/prefix={prefix}/newline={newline}");
                    let ranges = paired(&state, &uri, &source, &label);
                    compared += 1;
                    nonempty += usize::from(!ranges.is_empty());
                }
            }
        }
    }
    assert_eq!(compared, SCRIPTS.len() * 20);
    assert!(
        nonempty >= 500,
        "the matrix must exercise real positive vectors"
    );

    let anchors: &[(&str, &[&str])] = &[
        ("const first = 1; let second = first;", &["first", "second"]),
        ("const [first, ...rest] = input;", &["first", "rest"]),
        (
            "const { key: renamed, ...remaining } = input;",
            &["renamed", "remaining"],
        ),
        (
            "function outer(argument) { const local = argument; }",
            &["outer", "argument", "local"],
        ),
        (
            "const arrow = (argument) => argument;",
            &["arrow", "argument"],
        ),
        (
            "class Example { method(argument) { const local = argument; } }",
            &["Example", "argument", "local"],
        ),
        (
            "try { throw 1; } catch (caught) { const recovered = caught; }",
            &["caught", "recovered"],
        ),
        (
            "for (const item of items) { const copy = item; }",
            &["item", "copy"],
        ),
        (
            "import Default, { original as local, named } from 'dep';",
            &["Default", "local", "named"],
        ),
        ("const value = 1; { const value = 2; }", &["value", "value"]),
        (r"const \u0061 = 1; const 雪 = '🌸';", &[r"\u0061", "雪"]),
        ("const object = { key: value, shorthand };", &["object"]),
        (
            "export default function Named(argument) { return argument; }",
            &["Named", "argument"],
        ),
    ];
    for (index, (script, names)) in anchors.iter().enumerate() {
        let source = cstr!("<script setup lang=\"ts\">{script}</script>");
        let ranges = paired(&state, &uri, &source, &cstr!("anchor={index}"));
        let actual = ranges
            .iter()
            .map(|range| source.get(range.clone()).expect("authored byte span"))
            .collect::<Vec<_>>();
        assert_eq!(
            actual, *names,
            "exact positive binding names/order: {index}"
        );
    }

    for (index, source) in [
        "<template><p>雪🌸</p></template>",
        "<script>const plain = 1;</script>",
        "<script setup lang=\"ts\">const invalid = ;</script>",
    ]
    .iter()
    .enumerate()
    {
        assert!(paired(&state, &uri, source, &cstr!("negative={index}")).is_empty());
    }
    let other = Url::parse("file:///workspace/BindingRanges.ts").expect("URI");
    assert!(
        paired(
            &state,
            &other,
            "<script setup>const value = 1;</script>",
            "non-vue"
        )
        .is_empty()
    );
    println!(
        "binding-range-oracle summary matrix={compared} nonempty={nonempty} anchors={} negatives=4",
        anchors.len()
    );
}
