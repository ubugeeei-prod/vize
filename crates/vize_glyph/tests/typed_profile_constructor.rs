//! Direct typed construction of every named formatter profile (#6098).

use vize_glyph::{FormatOptions, FormatPreset, format_sfc};

const SOURCE: &str = r#"<template>
  <section data-kind="example" aria-live="polite" class="card">
    <img src="/logo.svg">
  </section>
</template>

<script setup lang="ts">
const value = (input) => ({ message: 'ready', input })
</script>
"#;

#[test]
fn every_named_profile_constructs_its_complete_assignment() {
    for preset in [
        FormatPreset::Vize,
        FormatPreset::Prettier,
        FormatPreset::Compact,
        FormatPreset::Documentation,
    ] {
        let constructed = FormatOptions::from_preset(preset);
        let mut overlaid = FormatOptions::default();
        overlaid.apply_preset(preset);

        assert_eq!(
            serde_json::to_value(constructed).expect("serialize constructed profile"),
            serde_json::to_value(overlaid).expect("serialize overlaid profile"),
            "{preset:?}",
        );
    }
}

#[test]
fn every_typed_profile_reaches_a_second_pass_fixed_point() {
    for preset in [
        FormatPreset::Vize,
        FormatPreset::Prettier,
        FormatPreset::Compact,
        FormatPreset::Documentation,
    ] {
        let options = FormatOptions::from_preset(preset);
        let first = format_sfc(SOURCE, &options).expect("first pass").code;
        let second = format_sfc(&first, &options).expect("second pass").code;

        assert_eq!(second, first, "{preset:?}");
    }
}
