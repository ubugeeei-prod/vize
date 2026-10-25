//! Real comment stripping and emission boundaries must keep authored identity.

use crate::virtual_ts::VirtualTsOptions;
use crate::virtual_ts::expressions::prop_sources::{
    generated_prop_span, generated_prop_span_preserving_comments,
};
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use vize_carton::{String, cstr};
use vize_croquis::{Analyzer, AnalyzerOptions, croquis::PassedProp};

fn prop(value: &str) -> PassedProp {
    PassedProp {
        name: "value".into(),
        name_is_dynamic: false,
        value: Some(value.into()),
        start: 0,
        end: value.len() as u32,
        is_dynamic: true,
    }
}

#[test]
fn actual_value_maps_compose_comments_trim_unicode_and_crlf_without_crossing_copies() {
    let summary = Analyzer::with_options(AnalyzerOptions::full()).finish();
    let options = VirtualTsOptions::default();
    let bindings = TemplateBindingAccess::default().with_instance_globals(
        &summary,
        &options,
        &[],
        false,
        true,
    );
    for raw in [
        "  $ghost.value  ",
        " \r\n /*前*/ $ghost /*中*/ .value + '雪'  ",
        " /*before*/ (() => { /*inside*/ return $ghost.value; })() ",
    ] {
        let prop = prop(raw);
        let span = generated_prop_span(&prop, &bindings).unwrap();
        let emitted = cstr!("first={};second={};", span.text, span.text);
        let start = "first=".len();
        let end = start + span.text.len();
        let mut mappings = Vec::new();
        assert!(span.map_instance_binding(&emitted, &mut mappings, start..end, 1000, &bindings));
        let authored = 1000 + raw.find("$ghost").unwrap();
        let generated = emitted.find("__ctx.$ghost").unwrap() + "__ctx.".len();
        assert!(mappings.iter().any(|row| {
            row.gen_range == (generated..generated + "$ghost".len())
                && row.src_range == (authored..authored + "$ghost".len())
        }));
        for row in &mappings {
            assert!(start <= row.gen_range.start && row.gen_range.end <= end);
            assert_eq!(
                emitted.get(row.gen_range.clone()),
                raw.get(row.src_range.start - 1000..row.src_range.end - 1000)
            );
        }
        let mut prefixes = Vec::new();
        bindings.append_instance_diagnostic_prefixes(&mut prefixes);
        assert_eq!(prefixes.len(), 1);
        let prefix = prefixes.first().unwrap();
        assert_eq!(emitted.get(prefix.gen_range.clone()), Some("__ctx."));
        assert_eq!(prefix.src_range, authored..authored + "$ghost".len());
    }
}

#[test]
fn preserved_callback_comments_and_synthetic_placeholders_keep_distinct_identity() {
    let summary = Analyzer::with_options(AnalyzerOptions::full()).finish();
    let options = VirtualTsOptions::default();
    let bindings = TemplateBindingAccess::default().with_instance_globals(
        &summary,
        &options,
        &[],
        false,
        true,
    );
    let raw = " () => { /*雪*/ return $ghost.value; } ";
    let prop = prop(raw);
    let span = generated_prop_span_preserving_comments(&prop, &bindings).unwrap();
    assert!(span.text.contains("/*雪*/"));
    let mut mappings = Vec::new();
    assert!(span.map_instance_binding(
        &span.text,
        &mut mappings,
        0..span.text.len(),
        50,
        &bindings
    ));
    for row in &mappings {
        assert_eq!(
            span.text.get(row.gen_range.clone()),
            raw.get(row.src_range.start - 50..row.src_range.end - 50)
        );
    }
    let mut placeholder_mappings = Vec::new();
    assert!(span.map_instance_binding(
        "undefined as never",
        &mut placeholder_mappings,
        0.."undefined as never".len(),
        50,
        &bindings,
    ));
    assert!(placeholder_mappings.is_empty());
}

#[test]
fn unchanged_and_disabled_values_retain_no_mapping_metadata() {
    let summary = Analyzer::with_options(AnalyzerOptions::full()).finish();
    let options = VirtualTsOptions::default();
    let bindings = TemplateBindingAccess::default().with_instance_globals(
        &summary,
        &options,
        &[],
        false,
        true,
    );
    for raw in [
        " value /* ordinary */ ",
        " '$ghost' /* literal */ ",
        "(($ghost) => $ghost)(1)",
    ] {
        let prop = prop(raw);
        let span = generated_prop_span(&prop, &bindings).unwrap();
        assert!(span.binding_source.is_none());
    }
    let disabled = TemplateBindingAccess::default();
    let prop = prop(" /* disabled */ $ghost.value ");
    let span = generated_prop_span(&prop, &disabled).unwrap();
    assert!(span.binding_source.is_none());
    assert_eq!(span.text, String::from("   $ghost.value "));
}
