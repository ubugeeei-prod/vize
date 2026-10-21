//! Executable output contracts for the typed formatter profiles (#6098).

use vize_glyph::{FormatOptions, FormatPreset, format_sfc};

const AUTHORED_ORDER: &str = r#"<template>
  <div v-bind:id="id" class="a">{{ id }}</div>
</template>

<script lang="ts" setup>
const id = "a";
</script>
"#;

const VIZE_OUTPUT: &str = r#"<script setup lang="ts">
const id = "a";
</script>

<template>
  <div :id="id" class="a">{{ id }}</div>
</template>
"#;

fn assert_profile_output(preset: FormatPreset, source: &str, expected: &str) {
    let options = FormatOptions::from_preset(preset);
    let first = format_sfc(source, &options).expect("first pass").code;
    assert_eq!(first.as_str(), expected, "{preset:?} output");
    assert_eq!(
        format_sfc(&first, &options).expect("second pass").code,
        first,
        "{preset:?} fixed point",
    );
}

#[test]
fn vize_typed_profile_pins_canonical_output() {
    assert_profile_output(FormatPreset::Vize, AUTHORED_ORDER, VIZE_OUTPUT);
}

#[test]
fn prettier_typed_profile_pins_source_order_output() {
    assert_profile_output(FormatPreset::Prettier, AUTHORED_ORDER, AUTHORED_ORDER);
}

#[test]
fn compact_typed_profile_pins_dense_output() {
    assert_profile_output(
        FormatPreset::Compact,
        include_str!("fixtures/profiles/compact.input.vue"),
        include_str!("fixtures/profiles/compact.expected.vue"),
    );
}

#[test]
fn documentation_typed_profile_pins_example_output() {
    assert_profile_output(
        FormatPreset::Documentation,
        include_str!("fixtures/profiles/documentation.input.vue"),
        include_str!("fixtures/profiles/documentation.expected.vue"),
    );
}
