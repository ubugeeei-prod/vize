//! Block-tag attribute order and formatter presets (#6098).

use vize_glyph::{FormatOptions, FormatPreset, format_sfc};

#[test]
fn block_attributes_follow_sort_attributes() {
    let source = r#"<script lang="ts" setup>
const a = 1;
</script>

<template lang="html" functional>
  <div>{{ a }}</div>
</template>

<style module="css" lang="css" scoped>
.a {
  color: red;
}
</style>

<i18n locale="en" lang="json">
{}
</i18n>
"#;
    let authored = FormatOptions {
        sort_attributes: false,
        sort_blocks: false,
        ..FormatOptions::default()
    };
    let formatted = format_sfc(source, &authored).expect("format").code;
    assert_eq!(formatted.as_str(), source);

    let canonical = format_sfc(source, &FormatOptions::default())
        .expect("format")
        .code;
    assert_eq!(
        canonical.as_str(),
        r#"<script setup lang="ts">
const a = 1;
</script>

<template lang="html" functional>
  <div>{{ a }}</div>
</template>

<style scoped lang="css" module="css">
.a {
  color: red;
}
</style>

<i18n lang="json" locale="en">
{}
</i18n>
"#
    );
}

#[test]
fn prettier_preset_keeps_authored_order() {
    let source = r#"<template>
  <div v-bind:id="id" class="a">{{ id }}</div>
</template>

<script lang="ts" setup>
const id = "a";
</script>
"#;
    let mut options = FormatOptions::default();
    options.apply_preset(FormatPreset::Prettier);
    assert_eq!(options.print_width, 80);
    assert_eq!(
        format_sfc(source, &options).expect("format").code.as_str(),
        source
    );

    let compatibility = FormatOptions::prettier_compat();
    let formatted = format_sfc(source, &compatibility).expect("format").code;
    assert_eq!(formatted.as_str(), source);
    assert_eq!(
        format_sfc(&formatted, &compatibility).expect("format").code,
        formatted
    );
    assert_eq!(
        format_sfc(source, &FormatOptions::default())
            .expect("format")
            .code
            .as_str(),
        r#"<script setup lang="ts">
const id = "a";
</script>

<template>
  <div :id="id" class="a">{{ id }}</div>
</template>
"#
    );

    options.apply_preset(FormatPreset::Vize);
    let defaults = FormatOptions::default();
    assert_eq!(options.print_width, defaults.print_width);
    assert_eq!(options.sort_attributes, defaults.sort_attributes);
    assert_eq!(options.sort_blocks, defaults.sort_blocks);
    assert_eq!(
        options.normalize_directive_shorthands,
        defaults.normalize_directive_shorthands
    );
    assert_eq!(options.normalize_css_quotes, defaults.normalize_css_quotes);
    assert_eq!(
        options.normalize_inline_styles,
        defaults.normalize_inline_styles
    );
    assert_eq!(
        options.normalize_v_for_parens,
        defaults.normalize_v_for_parens
    );
    assert_eq!(
        options.self_close_void_elements,
        defaults.self_close_void_elements
    );
    assert_eq!(
        options.reindent_preprocessor_styles,
        defaults.reindent_preprocessor_styles
    );
}

#[test]
fn compact_preset_keeps_dense_authored_layout_at_a_fixed_point() {
    let source = include_str!("fixtures/profiles/compact.input.vue");
    let expected = include_str!("fixtures/profiles/compact.expected.vue");
    let mut options = FormatOptions::prettier_compat();
    options.apply_preset(FormatPreset::Compact);

    assert_eq!(options.print_width, 120);
    assert!(options.bracket_same_line);
    assert!(!options.sort_attributes);
    assert!(!options.sort_blocks);
    assert!(options.normalize_directive_shorthands);
    assert!(!options.normalize_css_quotes);
    assert!(!options.normalize_inline_styles);
    assert!(!options.normalize_v_for_parens);
    assert!(!options.self_close_void_elements);
    assert!(!options.reindent_preprocessor_styles);

    let formatted = format_sfc(source, &options).expect("compact format").code;
    assert_eq!(formatted.as_str(), expected);
    assert_eq!(
        format_sfc(&formatted, &options)
            .expect("compact second pass")
            .code,
        formatted
    );

    let vize = format_sfc(source, &FormatOptions::default())
        .expect("default format")
        .code;
    assert_ne!(vize, formatted);
}

#[test]
fn prettier_preset_requotes_css_strings() {
    let source = "<style>\n@import 'a.css';\n.a {\n  content: 'it\\'s';\n  font-family: 'A \"B\"';\n}\n</style>\n";
    let kept = format_sfc(source, &FormatOptions::default()).expect("format");
    assert_eq!(kept.code.as_str(), source);

    let mut options = FormatOptions::default();
    options.apply_preset(FormatPreset::Prettier);
    let requoted = format_sfc(source, &options).expect("format").code;
    assert_eq!(
        requoted.as_str(),
        "<style>\n@import \"a.css\";\n.a {\n  content: \"it's\";\n  font-family: 'A \"B\"';\n}\n</style>\n"
    );
    assert_eq!(
        format_sfc(&requoted, &options).expect("format").code,
        requoted
    );
}

#[test]
fn prettier_preset_requotes_less_and_scss_strings() {
    let mut options = FormatOptions::default();
    options.apply_preset(FormatPreset::Prettier);
    for lang in ["less", "scss"] {
        let source = format!(
            "<style lang=\"{lang}\">\n// it's kept\n@import 'a';\n.a {{ background: url(http://x/'y'); }}\n</style>\n"
        );
        let expected = format!(
            "<style lang=\"{lang}\">\n// it's kept\n@import \"a\";\n.a {{ background: url(http://x/'y'); }}\n</style>\n"
        );
        assert_eq!(
            format_sfc(&source, &options).expect("format").code.as_str(),
            expected
        );
    }
    let sass = "<style lang=\"sass\">\n@import 'a'\n</style>\n";
    assert_eq!(
        format_sfc(sass, &options).expect("format").code.as_str(),
        sass
    );
}

#[test]
fn prettier_preset_lays_out_inline_styles() {
    let source =
        "<template>\n  <div style=\"color:red;top:0;\" :style=\"{ a: 1 }\">a</div>\n</template>\n";
    let kept = format_sfc(source, &FormatOptions::default()).expect("format");
    assert_eq!(kept.code.as_str(), source);

    let mut options = FormatOptions::default();
    options.apply_preset(FormatPreset::Prettier);
    let formatted = format_sfc(source, &options).expect("format").code;
    assert_eq!(
        formatted.as_str(),
        "<template>\n  <div style=\"color: red; top: 0\" :style=\"{ a: 1 }\">a</div>\n</template>\n"
    );
    assert_eq!(
        format_sfc(&formatted, &options).expect("format").code,
        formatted
    );
}

#[test]
fn prettier_preset_writes_v_for_parens_and_closes_void_elements() {
    let source = "<template>\n  <ul>\n    <li v-for=\"item, i in items\" :key=\"i\"><img :src=\"item\"><br></li>\n    <li v-for=\"{ a, b } of pairs\" :key=\"a\">{{ b }}</li>\n    <Input v-for=\"x in xs\" :key=\"x\"></Input>\n  </ul>\n</template>\n";
    let kept = format_sfc(source, &FormatOptions::default()).expect("format");
    assert_eq!(kept.code.as_str(), source);

    let mut options = FormatOptions::default();
    options.apply_preset(FormatPreset::Prettier);
    let formatted = format_sfc(source, &options).expect("format").code;
    assert_eq!(
        formatted.as_str(),
        "<template>\n  <ul>\n    <li v-for=\"(item, i) in items\" :key=\"i\"><img :src=\"item\" /><br /></li>\n    <li v-for=\"{ a, b } of pairs\" :key=\"a\">{{ b }}</li>\n    <Input v-for=\"x in xs\" :key=\"x\"></Input>\n  </ul>\n</template>\n"
    );
    assert_eq!(
        format_sfc(&formatted, &options).expect("format").code,
        formatted
    );
}

#[test]
fn prettier_preset_reindents_scss_and_less() {
    let mut options = FormatOptions::default();
    options.apply_preset(FormatPreset::Prettier);
    for lang in ["scss", "less"] {
        let source = format!(
            "<style lang=\"{lang}\">\n.a {{\n\tcolor: red;\n\n\n\t&:hover {{\n\t\tcolor: blue;\n\t}}\n}}\n</style>\n"
        );
        let kept = format_sfc(&source, &FormatOptions::default()).expect("format");
        assert_eq!(kept.code.as_str(), source);
        let expected = format!(
            "<style lang=\"{lang}\">\n.a {{\n  color: red;\n\n  &:hover {{\n    color: blue;\n  }}\n}}\n</style>\n"
        );
        assert_eq!(
            format_sfc(&source, &options).expect("format").code.as_str(),
            expected
        );
    }
}

#[test]
fn default_options_serialize_as_the_published_reference() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter/format-options-reference.json"
    ))
    .expect("reference");
    let defaults = serde_json::to_value(FormatOptions::default()).expect("serialize");
    assert_eq!(defaults, reference["formatOptions"]);

    let mut compatibility = reference["formatOptions"].clone();
    compatibility["printWidth"] = serde_json::json!(80);
    for key in [
        "sortAttributes",
        "sortBlocks",
        "normalizeDirectiveShorthands",
    ] {
        compatibility[key] = serde_json::json!(false);
    }
    for key in [
        "normalizeCssQuotes",
        "normalizeInlineStyles",
        "normalizeVForParens",
        "selfCloseVoidElements",
        "reindentPreprocessorStyles",
    ] {
        compatibility[key] = serde_json::json!(true);
    }
    assert_eq!(
        serde_json::to_value(FormatOptions::prettier_compat()).expect("serialize"),
        compatibility
    );

    let mut options = FormatOptions::default();
    options.apply_preset(FormatPreset::Prettier);
    let round_trip: FormatOptions =
        serde_json::from_value(serde_json::to_value(&options).expect("serialize"))
            .expect("deserialize");
    assert_eq!(
        serde_json::to_value(round_trip).expect("serialize"),
        serde_json::to_value(options).expect("serialize")
    );
}

#[test]
fn config_profiles_resolve_presets_under_authored_options() {
    let document: vize_l0::config::ConfigDocument = serde_json::from_str(
        r#"{"formatter":{"preset":"prettier","printWidth":120,"selfCloseVoidElements":false}}"#,
    )
    .expect("config");
    let options = FormatOptions::from_profile(document.formatter_profile()).expect("options");
    let mut expected = FormatOptions::prettier_compat();
    expected.print_width = 120;
    expected.self_close_void_elements = false;
    assert_eq!(
        serde_json::to_value(options).expect("serialize"),
        serde_json::to_value(expected).expect("serialize")
    );

    let unset = vize_l0::config::FormatterProfile::default();
    assert_eq!(
        serde_json::to_value(FormatOptions::from_profile(&unset).expect("options"))
            .expect("serialize"),
        serde_json::to_value(FormatOptions::default()).expect("serialize")
    );
}
