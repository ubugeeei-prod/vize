//! Complete first-pass bytes for historical directive layout regressions.
//! Inputs/options are copied from the named source tests; binary snapshots keep
//! all CR/LF and final newline bytes, alongside three-pass fixed-point checks.
#![expect(clippy::unwrap_used, reason = "fixture helpers fail by panicking")]

use vize_glyph::{FormatOptions, format_sfc, format_template};
use vize_l0::String;

fn snapshot_bytes(name: &str, output: &str) {
    insta::with_settings!({prepend_module_to_snapshot => false}, {
        insta::assert_binary_snapshot!(name, output.as_bytes().to_vec());
    });
}

fn sfc(name: &str, source: &str, options: &FormatOptions) -> String {
    let first = format_sfc(source, options).unwrap().code;
    snapshot_bytes(name, &first);
    let second = format_sfc(&first, options).unwrap();
    let third = format_sfc(&second.code, options).unwrap();
    assert_eq!(first, second.code);
    assert_eq!(second.code, third.code);
    assert!(!second.changed);
    assert!(!third.changed);
    first
}

fn template(name: &str, source: &str, options: &FormatOptions) -> String {
    let first = format_template(source, options).unwrap();
    snapshot_bytes(name, &first);
    let second = format_template(&first, options).unwrap();
    let third = format_template(&second, options).unwrap();
    assert_eq!(first, second);
    assert_eq!(second, third);
    first
}

#[test]
fn sfc_single_multiline_directive_attribute_is_idempotent() {
    // Source: tests/template_directive_attributes.rs::sfc_single_multiline_directive_attribute_is_idempotent.
    let source = r#"<template>
  <label
    :style="props.reverseOrder
      ? 'grid-template-areas: \'toggle . label-text\''
      : 'grid-template-areas: \'label-text . toggle\''"
  >
  </label>
</template>
"#;
    let options = FormatOptions::default();
    let output = sfc(
        "history_sfc_single_multiline_directive.txt",
        source,
        &options,
    );
    assert!(output.contains("\n    :style="));
}

#[test]
fn sfc_verbatim_multiline_directive_attribute_is_idempotent() {
    // Source: tests/template_directive_attributes.rs::sfc_verbatim_multiline_directive_attribute_is_idempotent.
    let source = r#"<template>
  <QBtn
    @click.stop="
      selectWord(key);
      editWord();
    "
  />
</template>
"#;
    let options = FormatOptions::default();
    let output = sfc(
        "history_sfc_verbatim_multiline_directive.txt",
        source,
        &options,
    );
    assert!(output.contains("selectWord(key);"));
    assert!(output.contains("editWord();"));
}

#[test]
fn sfc_multiline_v_for_collection_is_idempotent() {
    // Source: tests/template_directive_attributes.rs::sfc_multiline_v_for_collection_is_idempotent.
    let source = r#"<template>
  <template
    v-for="(engineId, engineIndex) in sortedEngineInfos.map(
      (engineInfo) => engineInfo.uuid,
    )"
    :key="engineIndex"
  >
    <span>{{ engineId }}</span>
  </template>
</template>
"#;
    let options = FormatOptions::default();
    let output = sfc("history_sfc_multiline_v_for.txt", source, &options);
    assert!(output.contains("sortedEngineInfos.map("));
    assert!(output.contains(":key=\"engineIndex\""));
    assert!(output.contains("<span>{{ engineId }}</span>"));
}

#[test]
fn template_blank_lines_and_leading_directive_comments_round_trip() {
    // Source: src/tests.rs::template_blank_lines_and_leading_directive_comments_round_trip.
    let source = "<script setup lang=\"ts\">\nconst loading = true;\nfunction close() {}\n</script>\n\n<template>\n  <main\n    :data-loading=\"\n      // explain why this flag is used here\n      loading\n    \"\n    @close=\"\n      // always attach a close handler\n      () => close()\n    \"\n  >\n    <header>title</header>\n\n    <footer>foot</footer>\n  </main>\n</template>\n";
    let options = FormatOptions {
        sort_attributes: false,
        ..FormatOptions::default()
    };
    let output = sfc(
        "history_sfc_blank_lines_directive_comments.txt",
        source,
        &options,
    );
    assert!(output.contains(
        ":data-loading=\"\n      // explain why this flag is used here\n      loading\n    \""
    ));
    assert!(output.contains(
        "@close=\"\n      // always attach a close handler\n      () => close()\n    \""
    ));
    assert!(output.contains("<header>title</header>\n\n    <footer>foot</footer>"));
}

#[test]
fn test_format_sfc_template_quotes_and_style_numbers_match_standalone_formatting() {
    // Source: src/tests.rs::test_format_sfc_template_quotes_and_style_numbers_match_standalone_formatting.
    let source = r#"<script setup>
const label = "sample";
</script>

<template>
  <div :class="{ 'is-active': active }" @click="() => emit('change', 'x')">sample</div>
</template>

<style scoped>
.sample {
  opacity: 0.5;
  transition: opacity 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
</style>
"#;
    let options = FormatOptions {
        single_quote: false,
        print_width: 120,
        sort_attributes: false,
        ..FormatOptions::default()
    };
    let output = sfc(
        "history_sfc_template_quotes_style_numbers.txt",
        source,
        &options,
    );
    assert!(output.contains("const label = \"sample\";"));
    assert!(output.contains(":class=\"{ 'is-active': active }\""));
    assert!(output.contains("@click=\"() => emit('change', 'x')\""));
    assert!(output.contains("opacity: 0.5;"));
    assert!(output.contains("opacity 0.2s cubic-bezier(0.4, 0, 0.2, 1)"));
}

#[test]
fn multiline_directive_value_on_a_pinned_line_stays_idempotent() {
    // Source: tests/template_suppression_line.rs::multiline_directive_value_on_a_pinned_line_stays_idempotent.
    let source = concat!(
        "<div>\n",
        "  <!-- eslint-disable-next-line -->\n",
        "  label: <span :style=\"{\n",
        "    color: 'red',\n",
        "  }\">x</span>\n",
        "</div>",
    );
    let options = FormatOptions::default();
    let output = template(
        "history_template_pinned_multiline_directive.txt",
        source,
        &options,
    );
    assert!(output.contains("label: <span"));
}
