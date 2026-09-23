//! FP-1 (`docs/davinci/plan/ledger-fp.md`): `type/require-typed-emits` and
//! `type/require-typed-props` report the `defineEmits` / `defineProps` call
//! they are about — not template text.
//!
//! The rules read macro calls from croquis' script analysis, whose offsets
//! are relative to the script block. They used to report through the
//! template frame, so the range landed wherever "script-relative offset +
//! template offset" happened to fall. The oracle here is independent of the
//! linter: the expected range is the call's own text, found in the source.

use vize_patina::{Linter, OutputFormat, format_results};
use vize_s0::String;

const TYPED_RULES: [&str; 2] = ["type/require-typed-emits", "type/require-typed-props"];

/// The byte range of `call` (the whole macro call text) in `source`.
fn call_range(source: &str, call: &str) -> (u32, u32) {
    let start = source.find(call).expect("the fixture contains the call");
    let start = u32::try_from(start).expect("fits");
    (start, start + u32::try_from(call.len()).expect("fits"))
}

/// Every typed-rule diagnostic as `(rule, start, end, covered text)`.
fn typed_diagnostics(source: &str) -> Vec<(&'static str, u32, u32, &str)> {
    let result = Linter::new().lint_sfc(source, "Fixture.vue");
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| TYPED_RULES.contains(&diagnostic.rule_name))
        .map(|diagnostic| {
            let text = &source[diagnostic.start as usize..diagnostic.end as usize];
            (diagnostic.rule_name, diagnostic.start, diagnostic.end, text)
        })
        .collect()
}

fn expected(source: &str) -> Vec<(&'static str, u32, u32, &'static str)> {
    let emits = "defineEmits(['edit'])";
    let props = "defineProps(['label'])";
    let mut rows = vec![
        (TYPED_RULES[0], call_range(source, emits), emits),
        (TYPED_RULES[1], call_range(source, props), props),
    ];
    rows.sort_by_key(|(_, (start, _), _)| *start);
    rows.into_iter()
        .map(|(rule, (start, end), text)| (rule, start, end, text))
        .collect()
}

const SCRIPT: &str = "const props = defineProps(['label'])\nconst emit = defineEmits(['edit'])\n";

#[test]
fn template_first_components_report_the_macro_call_itself() {
    // The layoutit-grid `AreaBox.vue` shape FP-1 was measured on.
    let source = String::from(
        "<template>\n  <button @click=\"emit('edit')\">{{ props.label }}</button>\n</template>\n\n<script setup>\n",
    ) + SCRIPT
        + "</script>\n";
    assert_eq!(typed_diagnostics(&source), expected(&source));
}

#[test]
fn script_first_components_report_the_macro_call_itself() {
    let source = String::from("<script setup>\n")
        + SCRIPT
        + "</script>\n\n<template>\n  <button @click=\"emit('edit')\">{{ props.label }}</button>\n</template>\n";
    assert_eq!(typed_diagnostics(&source), expected(&source));
}

#[test]
fn a_classic_script_before_the_setup_block_does_not_shift_the_frame() {
    let source = String::from(
        "<script>\nexport default { inheritAttrs: false }\n</script>\n\n<template>\n  <p>{{ props.label }}</p>\n</template>\n\n<script setup>\n",
    ) + SCRIPT
        + "</script>\n";
    assert_eq!(typed_diagnostics(&source), expected(&source));
}

#[test]
fn multibyte_template_text_and_crlf_line_ends_keep_byte_exact_ranges() {
    let source = String::from(
        "<template>\r\n  <p>ラベル：{{ props.label }}</p>\r\n</template>\r\n\r\n<script setup>\r\n",
    ) + &SCRIPT.replace('\n', "\r\n")
        + "</script>\r\n";
    assert_eq!(typed_diagnostics(&source), expected(&source));
}

#[test]
fn the_report_renders_on_the_macro_line() {
    let source =
        String::from("<template>\n  <p>{{ props.label }}</p>\n</template>\n\n<script setup>\n")
            + SCRIPT
            + "</script>\n";
    let result = Linter::new().lint_sfc(&source, "Fixture.vue");
    let sources = [(String::from("Fixture.vue"), source.clone())];
    let json = format_results(&[result], &sources, OutputFormat::Json);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("json output");
    let locations: Vec<(String, u64, u64, u64, u64)> = parsed[0]["messages"]
        .as_array()
        .expect("messages")
        .iter()
        .filter(|message| TYPED_RULES.contains(&message["ruleId"].as_str().expect("rule id")))
        .map(|message| {
            let at = |key: &str| message[key].as_u64().expect("a position");
            let rule = String::from(message["ruleId"].as_str().expect("rule id"));
            (
                rule,
                at("line"),
                at("column"),
                at("endLine"),
                at("endColumn"),
            )
        })
        .collect();
    // Line 6 is `const props = defineProps(['label'])`, line 7 the emits call;
    // columns are 1-based and the end column is exclusive of the last char + 1.
    assert_eq!(
        locations,
        [
            (String::from(TYPED_RULES[1]), 6, 15, 6, 37),
            (String::from(TYPED_RULES[0]), 7, 14, 7, 35),
        ]
    );
}
