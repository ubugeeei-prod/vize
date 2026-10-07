//! Regression corpus for enum template bindings from both script blocks (#7893).

use oxc_allocator::Allocator;
use oxc_ast::ast::{ComputedMemberExpression, Expression, StaticMemberExpression};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_sfc::{
    BindingType, SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc, parse_sfc,
};
use vize_l0::CompactString;

const SOURCE: &str = include_str!("../../../tests/fixtures/sfc/enum-template-bindings/Badge.vue");
const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/enum-template-bindings-7893/Badge.vue.txt"
);

#[derive(Default)]
struct ProxyReads(Vec<CompactString>);

impl<'a> Visit<'a> for ProxyReads {
    fn visit_static_member_expression(&mut self, expression: &StaticMemberExpression<'a>) {
        if let Expression::Identifier(object) = &expression.object
            && object.name == "_ctx"
        {
            self.0.push(expression.property.name.as_str().into());
        }
        walk::walk_static_member_expression(self, expression);
    }

    fn visit_computed_member_expression(&mut self, expression: &ComputedMemberExpression<'a>) {
        if let Expression::Identifier(object) = &expression.object
            && object.name == "_ctx"
        {
            self.0.push("<computed>".into());
        }
        walk::walk_computed_member_expression(self, expression);
    }
}

fn assert_module_has_no_proxy_reads(renderer: &str, code: &str) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, code, SourceType::mjs()).parse();
    assert!(
        !parsed.panicked && parsed.diagnostics.is_empty(),
        "{renderer}: transformed enum output must be valid JavaScript: {:?}\n{code}",
        parsed.diagnostics
    );
    let mut reads = ProxyReads::default();
    reads.visit_program(&parsed.program);
    assert_eq!(reads.0, Vec::<CompactString>::new(), "{renderer}: {code}");
}

#[test]
fn both_script_blocks_expose_regular_and_const_enums_to_every_renderer() {
    for (source, names) in [
        (ORIGINAL, &["Tone"][..]),
        (
            SOURCE,
            &[
                "Tone",
                "LocalTone",
                "SharedMode",
                "LocalMode",
                "SetupMode",
                "SetupTone",
            ][..],
        ),
    ] {
        for (renderer, ssr, vapor) in [
            ("dom", false, false),
            ("ssr", true, false),
            ("vapor", false, true),
        ] {
            let descriptor =
                parse_sfc(source, SfcParseOptions::default()).expect("parse enum fixture");
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
            for &name in names {
                assert_eq!(
                    bindings.bindings.get(name),
                    Some(&BindingType::LiteralConst),
                    "{renderer}: {name} must be a module literal constant"
                );
            }
            assert_module_has_no_proxy_reads(renderer, result.code.as_str());
        }
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
    assert_module_has_no_proxy_reads("runtime enum", result.code.as_str());
    let setup = result.code.find("setup(").expect("setup function");
    let enumeration = result.code.find("Runtime =").expect("transformed enum");
    assert!(
        enumeration > setup,
        "runtime enum must stay inside setup:\n{}",
        result.code
    );
}
