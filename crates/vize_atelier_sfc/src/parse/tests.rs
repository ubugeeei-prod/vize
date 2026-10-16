#![expect(
    clippy::disallowed_macros,
    reason = "original-source fixtures use format!"
)]

use std::{borrow::Cow, cell::Cell};

use super::{parse_sfc, with_legacy};
use crate::{PadOption, SfcParseOptions};
use serde_json::Value;

fn corpus() -> Value {
    serde_json::from_str(include_str!(
        "../../tests/fixtures/native_template_descriptor_vue_3_5_41.json"
    ))
    .expect("independent original descriptor controls")
}

#[test]
fn original_native_template_descriptors_preserve_complete_legacy_results_and_dispatch() {
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
        let calls = Cell::new(0);
        let actual = with_legacy(source, options.clone(), |source, options| {
            calls.set(calls.get() + 1);
            vize_croquis::sfc::parse_sfc(source, options)
        });
        let legacy = vize_croquis::sfc::parse_sfc(source, options.clone());
        assert_eq!(calls.get(), usize::from(!native), "{}", case["id"]);
        assert_eq!(
            serde_json::to_value(&actual).unwrap(),
            serde_json::to_value(&legacy).unwrap(),
            "{}",
            case["id"]
        );
        assert_eq!(
            serde_json::to_value(parse_sfc(source, options)).unwrap(),
            serde_json::to_value(&legacy).unwrap(),
            "{}",
            case["id"]
        );
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
    assert_eq!(native_count, 11);
}

#[test]
fn nondefault_parse_policies_keep_the_original_fallback_and_complete_diagnostics() {
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
        let calls = Cell::new(0);
        let actual = with_legacy(source, options.clone(), |source, options| {
            calls.set(calls.get() + 1);
            vize_croquis::sfc::parse_sfc(source, options)
        });
        assert_eq!(calls.get(), 1);
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(vize_croquis::sfc::parse_sfc(source, options)).unwrap()
        );
    }
    // This flag changes downstream map requests, not SFC container geometry.
    let calls = Cell::new(0);
    let options = SfcParseOptions {
        source_map: true,
        ..Default::default()
    };
    let actual = with_legacy(source, options.clone(), |source, options| {
        calls.set(calls.get() + 1);
        vize_croquis::sfc::parse_sfc(source, options)
    });
    assert_eq!(calls.get(), 0);
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(vize_croquis::sfc::parse_sfc(source, options)).unwrap()
    );
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
fn common_unsupported_envelopes_do_not_acquire_a_native_arena() {
    for source in [
        "<script setup>const value=1;</script><template>{{value}}</template>",
        "<template><p/></template><style scoped>.p{color:red}</style>",
        "<template lang=html><p/></template>",
    ] {
        vize_l0::pool::clear();
        let calls = Cell::new(0);
        let actual = with_legacy(source, SfcParseOptions::default(), |source, options| {
            // Observe the rejected native attempt before the retained parser:
            // its style CSS-variable extraction legitimately acquires an arena.
            assert_eq!(vize_l0::pool::idle(), 0);
            assert_eq!(vize_l0::pool::checked_out(), 0);
            calls.set(calls.get() + 1);
            vize_croquis::sfc::parse_sfc(source, options)
        });
        assert_eq!(calls.get(), 1);
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(vize_croquis::sfc::parse_sfc(
                source,
                SfcParseOptions::default()
            ))
            .unwrap()
        );
    }
}

mod parity;
