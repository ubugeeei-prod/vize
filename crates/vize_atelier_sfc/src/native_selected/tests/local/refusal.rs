use super::*;
use crate::NativeSelectedSfcDomError;
use vize_l3::decision::dom::DomUnsupported;
use vize_l4::targets::dom::DomErrorKind;

#[test]
fn pinned_root_and_sibling_prefix_boundaries_return_no_whole_output() {
    let pack: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/native_handler_local_sfc_click_vue_3_5_35.json"
    ))
    .unwrap();
    for fixture in pack.get("divergences").and_then(Value::as_array).unwrap() {
        let source = text(fixture, "source").unwrap();
        let arena = Allocator::default();
        let compilation = compile_native_selected_sfc_dom(
            &arena,
            source,
            NativeSelectedSfcDomOptions {
                source_map: true,
                ..NativeSelectedSfcDomOptions::default()
            },
        );
        assert!(compilation.result().is_err());
        assert!(core::ptr::eq(
            compilation.observation().descriptor().source(),
            source
        ));
        assert!(compilation.observation().template().is_some());
        if compilation.observation().admitted().is_some() {
            let NativeSelectedSfcDomError::Dom(error) = compilation.result().unwrap_err() else {
                panic!("typed target boundary")
            };
            assert_eq!(
                error.kind,
                DomErrorKind::Unsupported(DomUnsupported::HandlerAccess)
            );
            assert_eq!(error.span.slice(source), "x");
        } else {
            assert!(!compilation.observation().issues().is_empty());
        }
    }
}

#[test]
fn active_block_proof_cannot_admit_mixed_outer_roles_or_unsupported_envelopes() {
    for template in [
        "<button @click='let u;{let x=1;$event.c=x+y;}'/>",
        "<button @click='let u;{let x=1;return x;}'/>",
        "<button @click='let u;{let x:number=1;$event.c=x;}'/>",
        "<button @click='let u;{let x=1;$event.c=x;}' id='late'/>",
        "<button @keyup='let u;{let x=1;$event.c=x;}'/>",
        "<button @click.stop='let u;{let x=1;$event.c=x;}'/>",
        "<button v-for='x in [1]' @click='let u;{let x=1;$event.c=x;}'/>",
        "<slot @click='let u;{let x=1;$event.c=x;}'/>",
        "<button @click='let u;{let x=1;$event.c=x;}'/><button @click='var x=1;$event.c=x;'/>",
    ] {
        let source = format!("<!--Local雪🌸-->\r\n<template>{template}</template>");
        let arena = Allocator::default();
        let compilation = compile_native_selected_sfc_dom(
            &arena,
            &source,
            NativeSelectedSfcDomOptions::default(),
        );
        assert!(compilation.result().is_err(), "{template}");
        assert!(core::ptr::eq(
            compilation.observation().descriptor().source(),
            source.as_str()
        ));
        assert!(compilation.observation().template().is_some());
    }
    for sibling in [
        "<script></script>",
        "<script setup>const x=1;</script>",
        "<style>.x{}</style>",
        "<docs>custom</docs>",
    ] {
        let source = format!(
            "{sibling}<template><button @click='let u;{{let x=1;$event.c=x;}}'/></template>"
        );
        let arena = Allocator::default();
        let compilation = compile_native_selected_sfc_dom(
            &arena,
            &source,
            NativeSelectedSfcDomOptions::default(),
        );
        assert_eq!(
            compilation.result().err(),
            Some(NativeSelectedSfcDomError::Observation)
        );
        assert!(compilation.observation().admitted().is_none());
        assert!(core::ptr::eq(
            compilation.observation().descriptor().source(),
            source.as_str()
        ));
    }
}
