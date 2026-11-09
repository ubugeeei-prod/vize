#![expect(clippy::expect_used, reason = "tests assert by panicking")]
use std::borrow::Cow;

use vize_glyph::{FormatOptions, format_sfc};
use vize_l0::{FxHashMap, String};
use vize_l1::container::vue::compat::split_blocks;

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

/// Root block roles and attributes in source order, from the L1 container walk.
fn root_block_attrs(source: &str) -> Vec<(String, BlockAttrs<'_>)> {
    let mut blocks = Vec::new();
    split_blocks(source, |block| {
        let name = core::str::from_utf8(block.tag_name).expect("tag names are ASCII");
        let role = if block.attrs.contains_key("setup") {
            String::from("script setup")
        } else {
            String::from(name)
        };
        blocks.push((role, block.attrs));
        true
    })
    .expect("SFC must split");
    blocks
}

fn snapshot_bytes(name: &str, output: &str) {
    insta::with_settings!({prepend_module_to_snapshot => false}, {
        insta::assert_binary_snapshot!(name, output.as_bytes().to_vec());
    });
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
    snapshot_bytes("history_sfc_block_attribute_quotes.txt", &first.code);
    let second = format_sfc(&first.code, &options).expect("formatted SFC must reformat");

    assert_eq!(first.code, second.code, "formatting must be idempotent");

    let serialized_attrs = r#"data-both="a'b &quot;c&quot;" data-double='a"b' data-single="a'b""#;
    assert_eq!(
        first.code.matches(serialized_attrs).count(),
        5,
        "script, script setup, template, style, and custom roots must share safe serialization"
    );

    let before = root_block_attrs(source);
    let after = root_block_attrs(&first.code);
    let roles = ["script", "script setup", "template", "style", "i18n"];
    assert_eq!(
        before
            .iter()
            .map(|(role, _)| role.as_str())
            .collect::<Vec<_>>(),
        roles
    );
    assert_eq!(
        after
            .iter()
            .map(|(role, _)| role.as_str())
            .collect::<Vec<_>>(),
        roles,
        "formatted SFC must keep every root block"
    );

    for ((_, before), (_, after)) in before.iter().zip(&after) {
        assert_quote_values_preserved(before, after);
    }
}
