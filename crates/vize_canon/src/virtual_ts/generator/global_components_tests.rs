//! #8003: a dynamic component alias is a local value, not a registry key.
#![expect(clippy::expect_used, reason = "tests assert by panicking")]
use crate::virtual_ts::{
    VirtualTsCheckOptions, VirtualTsGenerationOptions, VirtualTsOptions, VirtualTsOutput,
    generate_virtual_ts_with_offsets_and_checks,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

fn generate(script: &str, template: &str, check_unknown_components: bool) -> VirtualTsOutput {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse_with_options(
        &allocator,
        template,
        vize_relief::options::ParserOptions {
            is_native_tag: Some(vize_carton::is_native_tag),
            ..Default::default()
        },
    );
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    generate_virtual_ts_with_offsets_and_checks(
        &analyzer.finish(),
        Some(script),
        Some(&root),
        0,
        100,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            check_options: VirtualTsCheckOptions {
                check_unknown_components,
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

#[test]
fn unknown_component_flag_preserves_complete_dynamic_code_and_projection() {
    for (script, template) in [
        ("", "<component :is=\"'button'\">x</component>"),
        (
            "const open = true;",
            "<component :is=\"open ? 'div' : 'button'\" ref=\"target\" v-bind=\"{}\" />",
        ),
        (
            "const registry = { button: 'button' };",
            "<component :is=\"registry['button']\" />",
        ),
        (
            "function choose() { return 'button'; }",
            "<component :is=\"choose()\" />",
        ),
        (
            "const open = true; const parts = [1];",
            "<component v-for=\"part in parts\" :is=\"open ? 'div' : 'button'\">{{ part }}</component>",
        ),
    ] {
        let checked = generate(script, template, true);
        let default = generate(script, template, false);
        assert!(checked.code.contains("const __vize_dynamic_is_"));
        assert!(!checked.code.contains("__vize_global_component_"));
        assert_eq!(checked.code, default.code, "{template}");
        assert_eq!(checked.mapping, default.mapping, "{template}");
    }
}

#[test]
fn static_unknown_components_keep_their_authored_registry_mappings() {
    let template = "😀\r\n<component :is=\"'button'\" />\r\n<MissingWidget /><missing-widget />";
    let output = generate("", template, true);
    assert!(output.code.contains("const __vize_dynamic_is_"));
    for name in ["MissingWidget", "missing-widget"] {
        let literal = vize_carton::cstr!("\"{name}\"");
        let start = output
            .code
            .find(literal.as_str())
            .expect("global registry key");
        let row = output
            .mapping
            .spans()
            .iter()
            .find(|row| row.gen_range == (start..start + literal.len()))
            .expect("authored component mapping");
        let authored = 100 + template.find(name).expect("authored tag");
        assert_eq!(row.src_range, authored..authored + name.len());
    }
    assert!(!output.code.contains("\"__vize_dynamic_is_"));
}

#[test]
fn invalid_dynamic_expression_keeps_its_authored_mapping_and_inference() {
    let template = "<component :is=\"missing ? 'div' : 'button'\" />";
    let checked = generate("", template, true);
    let default = generate("", template, false);
    assert_eq!(checked.code, default.code);
    assert_eq!(checked.mapping, default.mapping);
    assert!(checked.code.contains("missing ? 'div' : 'button'"));
    let offset = 100 + template.find("missing").expect("authored reference");
    assert!(checked.mapping.to_generated(offset).is_some());
}

#[test]
fn dynamic_component_constructor_union_keeps_props_checks() {
    let script = "declare const One: new () => { $props: { count: number } }; declare const Two: new () => { $props: { label: string } }; const open = true;";
    let template = "<component :is=\"open ? One : Two\" :count=\"'wrong'\" />";
    let checked = generate(script, template, true);
    let default = generate(script, template, false);
    assert_eq!(checked.code, default.code);
    assert_eq!(checked.mapping, default.mapping);
    assert!(checked.code.contains("typeof __vize_dynamic_is_"));
    assert!(checked.code.contains("'wrong'"));
}

#[test]
fn authored_alias_spelling_never_proves_dynamic_component_ownership() {
    // Low-level mismatched facts must not turn a real ordinary element into
    // a generated alias, even at the exact alias(0) source position.
    for template in ["<Unknown />", "<Unknown :is=\"'button'\" />"] {
        let allocator = vize_carton::Allocator::new();
        let (root, errors) = vize_armature::parse(&allocator, template);
        assert!(errors.is_empty(), "{errors:?}");
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();
        let mut usage = vize_croquis::facts::component_usage_list(&summary)
            .first()
            .expect("real parsed unknown component")
            .clone();
        assert_eq!(usage.name, "Unknown");
        assert_eq!(usage.start, 0);
        usage.name = vize_croquis::drawer::dynamic_component_alias(0);
        assert!(!crate::virtual_ts::scope::is_owned_dynamic_component_alias(
            Some(&root),
            &usage,
        ));
    }

    // The tokenizer does not accept underscore as a tag-start character.
    // Retain these original bytes as opaque text, not a positive unknown tag.
    for template in [
        "<__vize_dynamic_is_0 />",
        "<__vize_dynamic_is_0 :is=\"'button'\" />",
        "<__vize_dynamic_is_777 />",
    ] {
        let allocator = vize_carton::Allocator::new();
        let (root, errors) = vize_armature::parse(&allocator, template);
        assert!(errors.is_empty(), "{errors:?}");
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_template(&root);
        assert!(vize_croquis::facts::component_usage_list(&analyzer.finish()).is_empty());
        let checked = generate("", template, true);
        let default = generate("", template, false);
        assert_eq!(checked.code, default.code);
        assert_eq!(checked.mapping, default.mapping);
    }
}
