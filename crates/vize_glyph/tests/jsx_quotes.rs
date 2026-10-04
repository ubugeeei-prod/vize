//! #7763: JSX attribute quote preference is independent of JS strings and HTML.
#![expect(
    clippy::disallowed_macros,
    reason = "regressions assert complete independent output and immutable options"
)]
use vize_glyph::{
    Allocator, FormatOptions, VueVersion, format_script_with_source_type,
    format_sfc_with_vue_version,
};

const SCRIPT: &str = r#"const label="hello"
const plain=<div title="world"/>
const apostrophe=<div title="it's"/>
const entities=<div title="' &quot;"/>
"#;

// These four complete references are authored from the separate quote contracts.
// Apostrophes choose fewer entities; the configured delimiter resolves ties.
const REFERENCES: [(bool, bool, &str, &str); 4] = [
    (
        false,
        false,
        r#"const label = "hello";
const plain = <div title="world" />;
const apostrophe = <div title="it's" />;
const entities = <div title="' &quot;" />;
"#,
        r#"<script lang="tsx">
const label = "hello";
const plain = <div title="world" />;
const apostrophe = <div title="it's" />;
const entities = <div title="' &quot;" />;
</script>

<template>
  <p title="world">hello</p>
</template>
"#,
    ),
    (
        false,
        true,
        r#"const label = "hello";
const plain = <div title='world' />;
const apostrophe = <div title="it's" />;
const entities = <div title='&apos; "' />;
"#,
        r#"<script lang="tsx">
const label = "hello";
const plain = <div title='world' />;
const apostrophe = <div title="it's" />;
const entities = <div title='&apos; "' />;
</script>

<template>
  <p title="world">hello</p>
</template>
"#,
    ),
    (
        true,
        false,
        r#"const label = 'hello';
const plain = <div title="world" />;
const apostrophe = <div title="it's" />;
const entities = <div title="' &quot;" />;
"#,
        r#"<script lang="tsx">
const label = 'hello';
const plain = <div title="world" />;
const apostrophe = <div title="it's" />;
const entities = <div title="' &quot;" />;
</script>

<template>
  <p title="world">hello</p>
</template>
"#,
    ),
    (
        true,
        true,
        r#"const label = 'hello';
const plain = <div title='world' />;
const apostrophe = <div title="it's" />;
const entities = <div title='&apos; "' />;
"#,
        r#"<script lang="tsx">
const label = 'hello';
const plain = <div title='world' />;
const apostrophe = <div title="it's" />;
const entities = <div title='&apos; "' />;
</script>

<template>
  <p title="world">hello</p>
</template>
"#,
    ),
];

fn assert_sfc_contract(source: &str, expected: &str, options: &FormatOptions, version: VueVersion) {
    let before_source = source.as_bytes().to_vec();
    let before_options = serde_json::to_value(options).unwrap();
    let first = format_sfc_with_vue_version(source, options, version).unwrap();
    assert_eq!(first.code.as_bytes(), expected.as_bytes());
    assert!(first.changed);
    let second = format_sfc_with_vue_version(&first.code, options, version).unwrap();
    let third = format_sfc_with_vue_version(&second.code, options, version).unwrap();
    assert_eq!(second.code.as_bytes(), expected.as_bytes());
    assert_eq!(third.code.as_bytes(), expected.as_bytes());
    assert!(!second.changed && !third.changed);
    assert_eq!(source.as_bytes(), before_source);
    assert_eq!(serde_json::to_value(options).unwrap(), before_options);
}

#[test]
fn jsx_quotes_keep_script_string_policy_and_entity_escaping_independent() {
    let allocator = Allocator::default();
    for source_type in [oxc_span::SourceType::tsx(), oxc_span::SourceType::jsx()] {
        assert_eq!(
            format_script_with_source_type(
                SCRIPT,
                &FormatOptions::default(),
                &allocator,
                source_type
            )
            .unwrap()
            .as_bytes(),
            REFERENCES[0].2.as_bytes(),
        );
        for (single_quote, jsx_single_quote, expected, _) in REFERENCES {
            let options = FormatOptions {
                single_quote,
                jsx_single_quote,
                ..FormatOptions::default()
            };
            let before = serde_json::to_value(&options).unwrap();
            let first =
                format_script_with_source_type(SCRIPT, &options, &allocator, source_type).unwrap();
            let second =
                format_script_with_source_type(&first, &options, &allocator, source_type).unwrap();
            let third =
                format_script_with_source_type(&second, &options, &allocator, source_type).unwrap();
            assert_eq!(first.as_bytes(), expected.as_bytes());
            assert_eq!(second.as_bytes(), expected.as_bytes());
            assert_eq!(third.as_bytes(), expected.as_bytes());
            assert_eq!(serde_json::to_value(&options).unwrap(), before);
        }
    }
}

#[test]
fn sfc_quote_combinations_leave_html_attributes_unchanged() {
    let input = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-single-jsx-quotes/App.vue.txt"
    );
    for version in [VueVersion::V2, VueVersion::V2_7, VueVersion::V3] {
        assert_sfc_contract(input, REFERENCES[0].3, &FormatOptions::default(), version);
        for lang in ["tsx", "jsx"] {
            let source = input.replace("lang=\"tsx\"", &format!("lang=\"{lang}\""));
            for (single_quote, jsx_single_quote, _, expected) in REFERENCES {
                let expected = expected.replace("lang=\"tsx\"", &format!("lang=\"{lang}\""));
                let options = FormatOptions {
                    single_quote,
                    jsx_single_quote,
                    ..FormatOptions::default()
                };
                assert_sfc_contract(&source, &expected, &options, version);
            }
        }
    }
}

#[test]
fn configured_jsx_quotes_preserve_complete_sfc_corpus_and_fixed_points() {
    let input = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-single-jsx-quotes/App.vue.txt"
    );
    let expected = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-single-jsx-quotes/reference.expected.txt"
    );
    let config: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-single-jsx-quotes/vize.config.json"
    ))
    .unwrap();
    let options: FormatOptions = serde_json::from_value(config["formatter"].clone()).unwrap();
    assert_eq!(expected.as_bytes(), REFERENCES[1].3.as_bytes());
    assert!(!options.single_quote && options.jsx_single_quote);
    for version in [VueVersion::V2, VueVersion::V2_7, VueVersion::V3] {
        assert_sfc_contract(input, expected, &options, version);
    }
}
