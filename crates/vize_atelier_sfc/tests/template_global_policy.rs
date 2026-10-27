//! Whole-module free-reference controls for Vue's template globals (#10927).
//! Native runtime/oracle tests retain the complete modules and maps as well.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    TemplateCompileOptions, compile_sfc_for_adapter, parse_sfc,
};

const DENIED: &[&str] = &[
    "document",
    "window",
    "navigator",
    "globalThis",
    "Promise",
    "Proxy",
    "Function",
    "Reflect",
    "WeakMap",
    "WeakSet",
    "module",
    "exports",
    "TypeError",
    "Uint8Array",
];
const SOURCE: &str = r#"<script setup lang="ts">
const emit = defineEmits<{ press: [] }>();
</script>
<template><button :data-values="[document, window, navigator, globalThis, Promise, Proxy, Function, Reflect, WeakMap, WeakSet, module, exports, TypeError, Uint8Array]" @click="(event: MouseEvent) => { if (document.activeElement !== event.currentTarget) return; emit('press'); }">{{ Math.max(1, 2) }}{{ Intl.NumberFormat().format(1) }}</button></template>"#;
const LOCAL: &str = r#"<script setup lang="ts">
const document = { activeElement: 'local' };
</script>
<template><button :data-value="document.activeElement" @click="(document: MouseEvent) => document.currentTarget">{{ Math.max(1, 2) }}{{ Intl.NumberFormat().format(1) }}</button></template>"#;
const LOOP: &str = r#"<script setup lang="ts">
const docs = [{ activeElement: 'loop' }];
</script>
<template><p v-for="document in docs" :data-value="document.activeElement">{{ Math.max(1, 2) }}{{ Intl.NumberFormat().format(1) }}</p></template>"#;
const IMPORT: &str = r#"<script setup lang="ts">
import { document } from './host';
</script>
<template><p :data-value="document.activeElement">{{ Math.max(1, 2) }}{{ Intl.NumberFormat().format(1) }}</p></template>"#;

#[test]
fn every_sfc_backend_keeps_host_globals_on_the_component_proxy() {
    for (id, source) in [
        ("globals", SOURCE),
        ("local", LOCAL),
        ("loop", LOOP),
        ("import", IMPORT),
    ] {
        for (target, vapor, ssr) in [
            ("dom", false, false),
            ("vapor", true, false),
            ("ssr", false, true),
        ] {
            for inline in [false, true] {
                let descriptor =
                    parse_sfc(source, SfcParseOptions::default()).expect("authored SFC");
                let result = compile_sfc_for_adapter(
                    &descriptor,
                    SfcCompileOptions {
                        script: ScriptCompileOptions {
                            inline_template: inline,
                            is_ts: true,
                            ..Default::default()
                        },
                        template: TemplateCompileOptions {
                            ssr,
                            ..Default::default()
                        },
                        vapor,
                        ..Default::default()
                    },
                    TemplateSyntaxMode::Standard,
                    CustomElementMatcher::default(),
                    CodegenOptions::default(),
                    if inline {
                        SfcScriptOutputMode::InlineTemplate
                    } else {
                        SfcScriptOutputMode::SeparateTemplate
                    },
                )
                .expect("complete SFC module");
                let allocator = Allocator::default();
                let parsed = Parser::new(&allocator, result.code.as_str(), SourceType::ts())
                    .with_options(oxc_parser::ParseOptions {
                        allow_return_outside_function: inline,
                        ..Default::default()
                    })
                    .parse();
                assert!(
                    parsed.diagnostics.is_empty(),
                    "{id}/{target}/{inline}: {:?}\n{}",
                    parsed.diagnostics,
                    result.code
                );
                let built = SemanticBuilder::new().build(&parsed.program);
                let mut roots: Vec<_> = built
                    .semantic
                    .scoping()
                    .root_unresolved_references()
                    .keys()
                    .map(|name| name.as_str())
                    .collect();
                roots.sort_unstable();
                let denied: Vec<_> = roots
                    .iter()
                    .copied()
                    .filter(|name| DENIED.contains(name))
                    .collect();
                if let Some(directory) = std::env::var_os("VIZE_TEMPLATE_GLOBAL_CAPTURE_DIR") {
                    let directory = std::path::PathBuf::from(directory);
                    std::fs::create_dir_all(&directory).expect("capture directory");
                    let packet = serde_json::json!({"id":id,"target":target,"inline":inline,"wholeSource":source,"wholeResult":result,"wholeRootReferences":roots});
                    std::fs::write(
                        directory.join(vize_l0::cstr!("{id}-{target}-{inline}.json").as_str()),
                        serde_json::to_vec_pretty(&packet).expect("complete packet"),
                    )
                    .expect("retain complete packet");
                }
                if std::env::var_os("VIZE_TEMPLATE_GLOBAL_BEFORE").is_some() {
                    if id == "globals" {
                        assert_eq!(
                            denied.iter().filter(|name| **name == "document").count(),
                            1,
                            "the immutable before producer must reproduce the real global: {roots:?}"
                        );
                    }
                } else {
                    assert_eq!(
                        denied,
                        Vec::<&str>::new(),
                        "{id}/{target}/{inline}: {roots:?}\n{}",
                        result.code
                    );
                }
                assert_eq!(
                    roots.iter().filter(|name| **name == "Math").count(),
                    1,
                    "allowed Math must remain a real global"
                );
                // The immutable producer also incorrectly prefixes Intl. Preserve
                // that observed baseline while requiring the Vue policy after repair.
                if std::env::var_os("VIZE_TEMPLATE_GLOBAL_BEFORE").is_none() {
                    assert_eq!(
                        roots.iter().filter(|name| **name == "Intl").count(),
                        1,
                        "allowed Intl must remain a real global"
                    );
                }
            }
        }
    }
    capture_legacy_fixture();
}

fn capture_legacy_fixture() {
    let Some(directory) = std::env::var_os("VIZE_TEMPLATE_GLOBAL_CAPTURE_DIR") else {
        return;
    };
    let descriptor = parse_sfc(
        SOURCE,
        SfcParseOptions {
            filename: "Layout.vue".into(),
            ..Default::default()
        },
    )
    .expect("exact legacy CLI input");
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions {
            parse: SfcParseOptions {
                filename: "Layout.vue".into(),
                ..Default::default()
            },
            script: ScriptCompileOptions {
                id: Some("Layout.vue".into()),
                is_ts: false,
                ..Default::default()
            },
            template: TemplateCompileOptions {
                id: Some("Layout.vue".into()),
                is_ts: false,
                ssr: true,
                ..Default::default()
            },
            style: vize_atelier_sfc::StyleCompileOptions {
                id: "Layout.vue".into(),
                ..Default::default()
            },
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::InlineTemplate,
    )
    .expect("exact CLI adapter mode");
    assert!(result.errors.is_empty(), "legacy input must compile");
    let packet = serde_json::json!({
        "wholeSource": SOURCE,
        "wholeResult": result,
        "options": {
            "filename": "Layout.vue",
            "scriptIsTs": false,
            "templateIsTs": false,
            "ssr": true,
            "scriptOutput": "InlineTemplate"
        }
    });
    std::fs::write(
        std::path::PathBuf::from(directory).join("legacy-fixture.json"),
        serde_json::to_vec_pretty(&packet).expect("whole legacy packet"),
    )
    .expect("retain exact legacy CLI packet");
}
