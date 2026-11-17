//! Declaration output regressions exercised through Croquis' public API.
#![expect(clippy::expect_used, reason = "tests assert by panicking")]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_sfc::croquis::{SfcCroquisOptions, analyze_sfc_descriptor};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_croquis::declaration_ts::generate_declaration_ts;

fn generate_sfc(source: &str) -> vize_carton::String {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("fixture must parse");
    let script = descriptor
        .script_setup
        .as_ref()
        .expect("fixture must contain script setup");
    let summary = analyze_sfc_descriptor(&descriptor, None, SfcCroquisOptions::for_declaration());

    generate_declaration_ts(&summary, Some(script.content.as_ref())).content
}

fn assert_parseable(declaration: &str) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, declaration, SourceType::d_ts()).parse();

    assert_eq!(
        parsed.diagnostics.len(),
        0,
        "generated declaration must parse as TypeScript: {:#?}",
        parsed.diagnostics
    );
}

#[test]
fn function_typed_prop_declaration_is_parseable() {
    let declaration = generate_sfc(
        r#"<script setup lang="ts">
defineProps<{ formatter?: (value: number) => string }>();
</script>"#,
    );

    assert_parseable(&declaration);
    insta::assert_snapshot!("function_typed_prop_declaration", declaration.as_str());
}

#[test]
fn benchmark_generic_prop_declaration_is_parseable() {
    let source = r#"<script setup lang="ts" generic="T extends { id: number }">
defineProps<{
  items: T[];
  selected?: T;
  keyOf: (row: T) => string;
}>();

defineEmits<{
  pick: [row: T];
}>();
</script>

<template>
  <ul>
    <li v-for="item in items" :key="item.id">{{ keyOf(item) }}</li>
  </ul>
</template>
"#;
    let declaration = generate_sfc(source);

    assert_parseable(&declaration);
    insta::assert_snapshot!("benchmark_generic_prop_declaration", declaration.as_str());
}

#[test]
fn generic_parameters_named_like_generated_types_stay_bound() {
    // #11485: `Props<Props>` inside the instance would apply the parameter.
    let declaration = generate_sfc(
        r#"<script setup lang="ts" generic="Props extends string, Emits, Slots, __EmitFn">
defineProps<{ v: Props }>();
defineEmits<{ pick: [value: Props] }>();
defineSlots<{ default(p: { v: Props }): any }>();
</script>"#,
    );

    assert_parseable(&declaration);
    insta::assert_snapshot!(
        "generic_parameters_named_like_generated_types",
        declaration.as_str()
    );
}

#[test]
fn constructor_value_props_declare_the_checker_helpers_they_name() {
    // #13112: a class-typed runtime prop names the checker's inference helper.
    let declaration = generate_sfc(
        r#"<script setup lang="ts">
import { Counter } from "./Counter";
defineProps({ counter: { type: Counter, required: true }, label: String });
</script>"#,
    );

    assert_parseable(&declaration);
    insta::assert_snapshot!("constructor_value_prop_declaration", declaration.as_str());
    let checker =
        include_str!("../../vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt");
    // The checker's own value-inference aliases, in their declared order.
    let expected = helper_lines(checker)
        .into_iter()
        .filter(|line| {
            ["Value<", "CtorInner<", "Ctor<"]
                .iter()
                .any(|name| line.starts_with(&*vize_carton::cstr!("type __RuntimeProp{name}")))
        })
        .collect::<Vec<_>>();
    assert_eq!(helper_lines(&declaration), expected, "{declaration}");

    let builtin = generate_sfc(
        r#"<script setup lang="ts">
defineProps({ label: String });
</script>"#,
    );
    assert_eq!(helper_lines(&builtin), Vec::<&str>::new(), "{builtin}");
}

fn helper_lines(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter(|line| line.starts_with("type __RuntimeProp"))
        .collect()
}
