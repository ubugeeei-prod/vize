//! Production routing evidence in its own process: the monotone walk/reparse
//! probes cannot be mixed with concurrently running compiler tests.

use vize_atelier_core::{TemplateSyntaxMode, expr_parse_probe, walk_probe::WalkCounts};
use vize_atelier_vapor::{
    VaporCompilerExperimentalOptions, VaporCompilerOptions, compile_vapor,
    compile_vapor_with_diagnostics, compile_vapor_with_sfc_context,
};
use vize_carton::Allocator;

#[test]
fn accepted_artifacts_bypass_legacy_walks_and_unsupported_inputs_keep_them() {
    let accepted = [
        r#"<main class="shell"><button :disabled="locked" @click="save">{{ label }}</button></main>"#,
        r#"<section><span>fixed</span><div :title="state.title"><b>{{ state.label }}</b></div><button @click="actions.save">{{ label }}</button></section>"#,
        "<div><span>static</span>{{ label }}<br><b>tail</b></div>",
        "<div><section><span>static 雪</span></section><hr></div>",
    ];
    for source in accepted {
        for prefix_identifiers in [false, true] {
            let allocator = Allocator::new();
            let options = VaporCompilerOptions {
                prefix_identifiers,
                ..Default::default()
            };
            let before = WalkCounts::snapshot();
            let parses = expr_parse_probe::expr_parse_count();
            let result = compile_vapor(&allocator, source, options.clone());
            assert!(
                result.error_messages.is_empty(),
                "{source}: {:?}",
                result.error_messages
            );
            assert_eq!(
                WalkCounts::snapshot().since(before).total_walks(),
                0,
                "{source}"
            );
            assert_eq!(expr_parse_probe::expr_parse_count() - parses, 0, "{source}");
            let before = WalkCounts::snapshot();
            let legacy = compile_vapor(
                &allocator,
                source,
                VaporCompilerOptions {
                    binding_metadata: Some(Default::default()),
                    ..options
                },
            );
            assert!(legacy.error_messages.is_empty());
            assert_eq!(
                WalkCounts::snapshot().since(before).total_walks(),
                2,
                "{source}"
            );
            assert_eq!(
                result.templates, legacy.templates,
                "template payload changed: {source}"
            );
        }
    }
    let allocator = Allocator::new();
    let source =
        "<main><button :disabled=\"locked\">{{ label }}</button><span>static</span></main>";
    let before = WalkCounts::snapshot();
    let (scoped, diagnostics) = compile_vapor_with_sfc_context(
        &allocator,
        source,
        VaporCompilerOptions::default(),
        TemplateSyntaxMode::Standard,
        Default::default(),
        VaporCompilerExperimentalOptions {
            source_map: true,
            source_map_filename: Some("Native.vue".into()),
            ..Default::default()
        },
        Some("data-v-native"),
    );
    assert!(diagnostics.is_empty());
    assert_eq!(WalkCounts::snapshot().since(before).total_walks(), 0);
    assert_eq!(
        scoped.templates[0],
        "<main data-v-native><button data-v-native> </button><span data-v-native>static</span></main>"
    );
    let map: serde_json::Value = serde_json::from_str(scoped.map.as_deref().unwrap()).unwrap();
    assert_eq!(map["sources"], serde_json::json!(["Native.vue"]));
    assert_eq!(map["sourcesContent"], serde_json::json!([source]));
    for source in [
        "<div>{{ value + 1 }}</div>",
        "<div v-if=\"ok\">{{ value }}</div>",
        "<button @click=\"save()\">{{ label }}</button>",
        "<div v-pre>{{ raw }}</div>",
    ] {
        let allocator = Allocator::new();
        let before = WalkCounts::snapshot();
        let result = compile_vapor(&allocator, source, VaporCompilerOptions::default());
        assert!(
            result.error_messages.is_empty(),
            "{source}: {:?}",
            result.error_messages
        );
        assert_eq!(
            WalkCounts::snapshot().since(before).total_walks(),
            2,
            "{source}"
        );
    }
    for source in ["<div>", "<div v-if />"] {
        let allocator = Allocator::new();
        let (result, diagnostics) =
            compile_vapor_with_diagnostics(&allocator, source, VaporCompilerOptions::default());
        assert!(
            !result.error_messages.is_empty() || !diagnostics.is_empty(),
            "{source}"
        );
        if diagnostics
            .iter()
            .any(|diagnostic| !diagnostic.is_recoverable())
        {
            assert!(
                result.code.is_empty(),
                "fatal diagnostics must not emit: {source}"
            );
        }
    }
    // With prefixing disabled, the established tolerant API preserves invalid
    // expressions. It must terminate. With prefixing enabled, core expression
    // validation reports them and the compiler must not emit code.
    for expression in ["(", "x +", "[x +", "{value: x +}"] {
        let source = vize_carton::cstr!("<button :disabled=\"{expression}\"></button>");
        let allocator = Allocator::new();
        let tolerant = compile_vapor(&allocator, &source, VaporCompilerOptions::default());
        assert!(
            tolerant.code.contains(expression),
            "{expression}: {}",
            tolerant.code
        );
        let (strict, diagnostics) = compile_vapor_with_diagnostics(
            &allocator,
            &source,
            VaporCompilerOptions {
                prefix_identifiers: true,
                ..Default::default()
            },
        );
        assert!(!diagnostics.is_empty(), "{expression}");
        assert!(!strict.error_messages.is_empty(), "{expression}");
        assert!(strict.code.is_empty(), "{expression}");
    }
}
