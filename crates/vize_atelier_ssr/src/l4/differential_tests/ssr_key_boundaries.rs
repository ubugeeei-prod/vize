//! Separate provider neighbors; the original differential battery is unchanged.

use super::{
    Allocator, BindingMetadata, BindingType, CustomElementMatcher, SsrCompilerExperimentalOptions,
    SsrCompilerOptions, SsrL4Selection, SsrLane, TemplateSyntaxMode, compile_ssr_on_lane,
    selection,
};

fn paired_code(source: &str, options: SsrCompilerOptions, admitted: bool) -> vize_l0::String {
    paired_result(source, options, admitted).code
}

fn paired_result(
    source: &str,
    options: SsrCompilerOptions,
    admitted: bool,
) -> crate::SsrCodegenResult {
    let experimental = SsrCompilerExperimentalOptions {
        source_map: true,
        source_map_filename: Some("SsrKeyBoundary.vue".into()),
        ..Default::default()
    };
    let observed_selection = selection(source, &options, &experimental);
    let compile = |lane| {
        let arena = Allocator::new();
        let (_, errors, result) = compile_ssr_on_lane(
            &arena,
            source,
            options.clone(),
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            experimental.clone(),
            true,
            lane,
        );
        (errors, result)
    };
    let (current_errors, current) = compile(SsrLane::Selected);
    let (legacy_errors, legacy) = compile(SsrLane::LegacyOnly);
    println!(
        "SSR_KEY_BOUNDARY_PACKET source={source:?} options={options:?} experimental={experimental:?} selection={observed_selection:?} current_errors={current_errors:?} current={current:?} legacy_errors={legacy_errors:?} legacy={legacy:?}"
    );
    if admitted {
        assert!(matches!(observed_selection, SsrL4Selection::Emitted(_)));
    } else {
        assert!(matches!(
            observed_selection,
            SsrL4Selection::Legacy(super::super::LegacyReason::Options)
        ));
    }
    assert!(current_errors.is_empty(), "{current_errors:?}");
    assert!(legacy_errors.is_empty(), "{legacy_errors:?}");
    assert_eq!(current.code, legacy.code, "{source}");
    assert_eq!(current.preamble, legacy.preamble, "{source}");
    assert_eq!(current.map, legacy.map, "{source}");
    let map: serde_json::Value =
        serde_json::from_str(current.map.as_deref().expect("requested low-level map"))
            .expect("complete original raw map");
    assert_eq!(map["sourcesContent"], serde_json::json!([source]));
    assert!(!map["mappings"].as_str().expect("map mappings").is_empty());
    current
}

#[test]
fn dynamic_camel_helpers_keep_scope_aware_keys_and_full_import_map_parity() {
    for (kind, inline_key) in [
        (BindingType::SetupRef, "keys.value[_ctx.index]"),
        (BindingType::SetupMaybeRef, "_unref(keys)[_ctx.index]"),
        (BindingType::SetupConst, "keys[_ctx.index]"),
    ] {
        for inline in [false, true] {
            let mut metadata = BindingMetadata {
                is_script_setup: true,
                ..Default::default()
            };
            metadata.bindings.insert("keys".into(), kind);
            let result = paired_result(
                "<Child :[keys[index]].camel=\"value\"></Child>",
                SsrCompilerOptions {
                    inline,
                    binding_metadata: Some(metadata),
                    ..Default::default()
                },
                !inline,
            );
            let key = if inline {
                inline_key
            } else {
                "$setup.keys[_ctx.index]"
            };
            assert!(
                result
                    .code
                    .contains(vize_l0::cstr!("[_camelize(({key}) || \"\") || \"\"]").as_str()),
                "{result:?}"
            );
            assert_eq!(
                result.preamble.matches("camelize as _camelize").count(),
                1,
                "{result:?}"
            );
        }
    }
    for source in [
        "<Child :[key]=\"value\"></Child>",
        "<Child :data-key.camel=\"value\" @[event]=\"handler\"></Child>",
    ] {
        let result = paired_result(source, SsrCompilerOptions::default(), true);
        assert!(
            !result.preamble.contains("camelize as _camelize"),
            "{result:?}"
        );
    }
}

#[test]
fn nonidentifier_setup_keys_use_the_same_provider_without_widening_inline_admission() {
    for (kind, inline_key) in [
        (BindingType::SetupRef, "keys.value[_ctx.index]"),
        (BindingType::SetupMaybeRef, "_unref(keys)[_ctx.index]"),
        (BindingType::SetupConst, "keys[_ctx.index]"),
    ] {
        for inline in [false, true] {
            let mut metadata = BindingMetadata {
                is_script_setup: true,
                ..Default::default()
            };
            metadata.bindings.insert("keys".into(), kind);
            let code = paired_code(
                "<Child :[keys[index]]=\"value\"></Child>",
                SsrCompilerOptions {
                    inline,
                    binding_metadata: Some(metadata),
                    ..Default::default()
                },
                !inline,
            );
            assert!(
                code.contains(if inline {
                    inline_key
                } else {
                    "$setup.keys[_ctx.index]"
                }),
                "{code}"
            );
        }
    }
}

#[test]
fn whole_ssr_key_controls_keep_context_globals_literals_and_real_local_scopes() {
    for (source, required, forbidden) in [
        (
            "<Child :[keys['name]']].camel=\"value\"></Child>",
            "_ctx.keys['name]']",
            "[_ctx._ctx",
        ),
        (
            "<Child :[Math.max(0,index)]=\"value\"></Child>",
            "Math.max(0,_ctx.index)",
            "_ctx.Math",
        ),
        (
            "<Child :['literal']=\"value\"></Child>",
            "['literal' || \"\"]",
            "_ctx.literal",
        ),
        (
            "<Child :[key]=\"value\"></Child>",
            "[_ctx.key || \"\"]",
            "[_ctx._ctx.key]",
        ),
        (
            "<Child v-for=\"(row,index) in rows\" :[row.keys[index]+suffix]=\"value\"></Child>",
            "row.keys[index]+_ctx.suffix",
            "_ctx.row.",
        ),
        (
            "<Host v-slot=\"{row,index}\"><Child :[row.keys[index]+suffix]=\"value\"></Child></Host>",
            "row.keys[index]+_ctx.suffix",
            "_ctx.row.",
        ),
    ] {
        let code = paired_code(source, SsrCompilerOptions::default(), true);
        assert!(
            code.contains(required),
            "required {required:?} in {source}: {code}"
        );
        assert!(
            !code.contains(forbidden),
            "forbidden {forbidden:?} in {source}: {code}"
        );
    }
}
