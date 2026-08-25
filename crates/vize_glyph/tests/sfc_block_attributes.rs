use std::borrow::Cow;

use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_glyph::{FormatOptions, format_sfc};
use vize_s0::{FxHashMap, String};

type BlockAttrs<'a> = FxHashMap<Cow<'a, str>, Cow<'a, str>>;

fn decoded_quote_entities(value: &str) -> String {
    value.replace("&quot;", "\"").replace("&#39;", "'").into()
}

fn assert_quote_values_preserved(before: &BlockAttrs<'_>, after: &BlockAttrs<'_>) {
    for name in ["data-single", "data-double", "data-both"] {
        let before = before.get(name).expect("source attribute must be parsed");
        let after = after
            .get(name)
            .expect("formatted attribute must still be parsed");
        assert_eq!(
            decoded_quote_entities(after),
            decoded_quote_entities(before),
            "attribute value changed for {name}"
        );
    }
}

#[test]
fn root_block_attributes_escape_quotes_and_preserve_parse_results() {
    let source = r#"<script data-single="a'b" data-double='a"b' data-both="a'b &quot;c&quot;">
const classic = 1
</script>

<script setup data-single="a'b" data-double='a"b' data-both="a'b &quot;c&quot;">
const setup = 1
</script>

<template data-single="a'b" data-double='a"b' data-both="a'b &quot;c&quot;">
<div>content</div>
</template>

<style data-single="a'b" data-double='a"b' data-both="a'b &quot;c&quot;">
.root { color: red; }
</style>

<i18n data-single="a'b" data-double='a"b' data-both="a'b &quot;c&quot;">
{"content":"Content"}
</i18n>
"#;
    let options = FormatOptions::default();

    let first = format_sfc(source, &options).expect("source SFC must format");
    let second = format_sfc(&first.code, &options).expect("formatted SFC must reformat");

    assert_eq!(first.code, second.code, "formatting must be idempotent");

    let serialized_attrs = r#"data-both="a'b &quot;c&quot;" data-double='a"b' data-single="a'b""#;
    assert_eq!(
        first.code.matches(serialized_attrs).count(),
        5,
        "script, script setup, template, style, and custom roots must share safe serialization"
    );

    let before = parse_sfc(source, SfcParseOptions::default()).expect("source SFC must parse");
    let after = parse_sfc(&first.code, SfcParseOptions::default())
        .expect("formatted SFC must remain parseable");

    let before_attrs = [
        &before.script.as_ref().expect("script block").attrs,
        &before
            .script_setup
            .as_ref()
            .expect("script setup block")
            .attrs,
        &before.template.as_ref().expect("template block").attrs,
        &before.styles.first().expect("style block").attrs,
        &before.custom_blocks.first().expect("custom block").attrs,
    ];
    let after_attrs = [
        &after.script.as_ref().expect("script block").attrs,
        &after
            .script_setup
            .as_ref()
            .expect("script setup block")
            .attrs,
        &after.template.as_ref().expect("template block").attrs,
        &after.styles.first().expect("style block").attrs,
        &after.custom_blocks.first().expect("custom block").attrs,
    ];

    for (before, after) in before_attrs.into_iter().zip(after_attrs) {
        assert_quote_values_preserved(before, after);
    }
}
