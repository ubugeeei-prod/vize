#![expect(
    clippy::disallowed_macros,
    reason = "original-source fixtures use format!"
)]

use std::borrow::Cow;

use super::parse_sfc;
use crate::{PadOption, SfcDescriptor, SfcError, SfcParseOptions};
use serde_json::Value;

fn corpus() -> Value {
    serde_json::from_str(include_str!(
        "../../tests/fixtures/native_template_descriptor_vue_3_5_41.json"
    ))
    .expect("independent original descriptor controls")
}

fn assert_oracle<'a>(
    source: &'a str,
    options: SfcParseOptions,
) -> Result<SfcDescriptor<'a>, SfcError> {
    let actual = parse_sfc(source, options.clone());
    let expected = vize_croquis::sfc::parse_sfc(source, options);
    assert_eq!(
        serde_json::to_value(&actual).unwrap(),
        serde_json::to_value(&expected).unwrap(),
        "{source}"
    );
    actual
}

#[test]
fn default_entry_cannot_reintroduce_the_croquis_sfc_parser() {
    for source in [include_str!("../parse.rs"), include_str!("native.rs")] {
        assert_eq!(source.matches("vize_croquis::sfc::parse_sfc").count(), 0);
    }
}

#[test]
fn l1_container_descriptors_preserve_complete_shipped_results() {
    let pack = corpus();
    let cases = pack["cases"].as_array().expect("cases");
    assert_eq!(cases.len(), 25);
    let mut native_count = 0;
    for case in cases {
        let source = case["source"].as_str().expect("original source");
        let native = case["native"].as_bool().expect("declared bounded family");
        let options = SfcParseOptions {
            filename: "Native雪🌸.vue".into(),
            ..Default::default()
        };
        let actual = assert_oracle(source, options);
        if native {
            native_count += 1;
            let descriptor = actual.as_ref().expect("native descriptor");
            let template = descriptor.template.as_ref().expect("native template");
            let official = &case["official"]["template"];
            assert_eq!(
                template.content.as_ref(),
                official["content"].as_str().unwrap()
            );
            assert_eq!(
                template.loc.start,
                official["start"].as_u64().unwrap() as usize
            );
            assert_eq!(template.loc.end, official["end"].as_u64().unwrap() as usize);
            assert!(matches!(descriptor.source, Cow::Borrowed(_)));
            assert!(matches!(template.content, Cow::Borrowed(_)));
            assert!(core::ptr::eq(descriptor.source.as_ref(), source));
            assert!(core::ptr::eq(
                template.content.as_ref(),
                source.get(template.loc.start..template.loc.end).unwrap()
            ));
        }
    }
    assert_eq!(native_count, 12);
}

#[test]
fn canonical_html_language_headers_preserve_borrowed_attributes() {
    for source in [
        "<template lang=html><p/></template>",
        "<template lang='html'><p/></template>",
        "\r\n<template lang=\"html\">雪🌸</template>\n",
    ] {
        let options = SfcParseOptions {
            filename: "Language雪🌸.vue".into(),
            ..Default::default()
        };
        let actual = assert_oracle(source, options);
        let descriptor = actual.expect("native descriptor");
        let template = descriptor.template.expect("native template");
        assert!(matches!(template.lang, Some(Cow::Borrowed("html"))));
        assert!(matches!(
            template.attrs.get("lang"),
            Some(Cow::Borrowed("html"))
        ));
    }
}

#[test]
fn every_template_language_retains_the_complete_shipped_descriptor() {
    for source in [
        "<template lang=pug><p/></template>",
        "<template lang=HTML><p/></template>",
        "<template lang='h&#116;ml'><p/></template>",
    ] {
        assert_oracle(source, SfcParseOptions::default()).unwrap();
    }
}

#[test]
fn nondefault_parse_policies_keep_complete_shipped_diagnostics() {
    let source = "<template><p>雪🌸</p></template>";
    for options in [
        SfcParseOptions {
            pad: PadOption::Line,
            ..Default::default()
        },
        SfcParseOptions {
            pad: PadOption::Space,
            ..Default::default()
        },
        SfcParseOptions {
            ignore_empty: true,
            ..Default::default()
        },
        SfcParseOptions {
            template_parse_options: Some(Default::default()),
            ..Default::default()
        },
    ] {
        assert_oracle(source, options).unwrap();
    }
    // This flag changes downstream map requests, not SFC container geometry.
    let options = SfcParseOptions {
        source_map: true,
        ..Default::default()
    };
    assert_oracle(source, options).unwrap();
}

#[test]
fn native_projection_borrows_original_bytes_after_the_reused_arena_is_overwritten() {
    let source = format!("<template><p>{}</p></template>", "雪🌸".repeat(2048));
    let before = vize_l0::pool::checked_out();
    let descriptor = parse_sfc(&source, SfcParseOptions::default()).unwrap();
    assert_eq!(vize_l0::pool::checked_out(), before);
    {
        let arena = vize_l0::pool::acquire();
        let _ = arena.alloc_str(&"x".repeat(source.len()));
    }
    let template = descriptor.template.unwrap();
    assert!(matches!(template.content, Cow::Borrowed(_)));
    let original = source.get(template.loc.start..template.loc.end).unwrap();
    assert!(core::ptr::eq(template.content.as_ref(), original));
    assert_eq!(template.content, format!("<p>{}</p>", "雪🌸".repeat(2048)));
    assert_eq!(vize_l0::pool::checked_out(), before);
}

#[test]
fn markdown_template_payload_is_a_borrowed_authored_slice() {
    let payload = "\r\n---\r\ntitle: \"{{ frontmatter.title }}\"  \r\n---\r\n\r\n# Hello  {{ title }}\r\n\r\n```vue\r\n<template>\r\n  <MyCard :title=\"title\">{{ mustStayInert }}</MyCard>\r\n</template>\r\n```\r\n";
    let source = format!("<template lang=\"md\">{payload}</template>\r\n");
    let descriptor = parse_sfc(&source, SfcParseOptions::default()).unwrap();
    let template = descriptor.template.as_ref().expect("Markdown template");

    assert_eq!(template.lang.as_deref(), Some("md"));
    assert_eq!(template.content.as_ref(), payload);
    let authored = source
        .get(template.loc.start..template.loc.end)
        .expect("authored Markdown payload");
    assert_eq!(authored, payload);
    assert!(matches!(template.content, Cow::Borrowed(_)));
    assert!(core::ptr::eq(template.content.as_ref(), authored));
}

#[test]
fn whole_script_template_and_style_descriptors_do_not_acquire_an_arena() {
    for source in [
        "<script setup>const value=1;</script><template>{{value}}</template>",
        "<template><p/></template><style scoped>.p{color:red}</style>",
    ] {
        vize_l0::pool::clear();
        assert_eq!(vize_l0::pool::idle(), 0);
        assert_eq!(vize_l0::pool::checked_out(), 0);
        assert_oracle(source, SfcParseOptions::default()).unwrap();
        assert_eq!(vize_l0::pool::checked_out(), 0);
    }
}

mod parity;

#[test]
fn non_whitespace_after_the_original_template_boundary_keeps_shipped_geometry() {
    for source in [
        "<template>x</template></template>",
        "<template>x</template>extra</template>",
    ] {
        assert_oracle(source, SfcParseOptions::default()).unwrap();
    }
}
