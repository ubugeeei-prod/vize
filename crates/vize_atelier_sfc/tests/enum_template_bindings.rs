//! Regression corpus for enum template bindings from both script blocks (#7893).

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_sfc::{
    BindingType, SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc, parse_sfc,
};
use vize_l0::cstr;

const SOURCE: &str = include_str!("../../../tests/fixtures/sfc/enum-template-bindings/Badge.vue");

#[test]
fn both_script_blocks_expose_regular_and_const_enums_to_every_renderer() {
    for (renderer, ssr, vapor) in [
        ("dom", false, false),
        ("ssr", true, false),
        ("vapor", false, true),
    ] {
        let descriptor = parse_sfc(SOURCE, SfcParseOptions::default()).expect("parse enum fixture");
        let result = compile_sfc(
            &descriptor,
            SfcCompileOptions {
                vapor,
                template: TemplateCompileOptions {
                    ssr,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .expect("compile enum fixture");
        assert!(result.errors.is_empty(), "{renderer}: {:?}", result.errors);

        let bindings = result.bindings.as_ref().expect("template binding metadata");
        for name in [
            "Tone",
            "LocalTone",
            "SharedMode",
            "LocalMode",
            "SetupMode",
            "SetupTone",
        ] {
            assert_eq!(
                bindings.bindings.get(name),
                Some(&BindingType::LiteralConst),
                "{renderer}: {name} must be a module literal constant"
            );
            assert!(
                !result.code.contains(cstr!("_ctx.{name}").as_str()),
                "{renderer}: enum must not be read from the component proxy:\n{}",
                result.code
            );
        }

        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, result.code.as_str(), SourceType::mjs()).parse();
        assert!(
            !parsed.panicked && parsed.diagnostics.is_empty(),
            "{renderer}: transformed enum output must be valid JavaScript: {:?}\n{}",
            parsed.diagnostics,
            result.code
        );
    }
}

#[test]
fn runtime_dependent_setup_enum_stays_in_setup_scope() {
    let source = r#"<script setup lang="ts">
function makeValue() { return 1; }
enum Runtime { Value = makeValue() }
</script>
<template><span>{{ Runtime.Value }}</span></template>"#;
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse runtime enum");
    let result =
        compile_sfc(&descriptor, SfcCompileOptions::default()).expect("compile runtime enum");
    assert_eq!(
        result
            .bindings
            .as_ref()
            .expect("bindings")
            .bindings
            .get("Runtime"),
        Some(&BindingType::SetupConst)
    );
    assert!(!result.code.contains("_ctx.Runtime"), "{}", result.code);
    let setup = result.code.find("setup(").expect("setup function");
    let enumeration = result.code.find("Runtime =").expect("transformed enum");
    assert!(
        enumeration > setup,
        "runtime enum must stay inside setup:\n{}",
        result.code
    );
}
