//! #8003: a dynamic component alias is a local value, not a registry key.
#![expect(clippy::expect_used, reason = "tests assert by panicking")]
use crate::virtual_ts::{
    VirtualTsCheckOptions, VirtualTsGenerationOptions, VirtualTsOptions, VirtualTsOutput,
    generate_virtual_ts_with_offsets_and_checks,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

#[path = "global_components_tests/dynamic_props_tests.rs"]
mod dynamic_props;

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

#[test]
fn named_dynamic_values_require_their_fresh_ast_and_matching_usage() {
    for template in [
        "<component :is=\"selected\" />",
        "<component :is=\"registry.button\" />",
    ] {
        let allocator = vize_carton::Allocator::new();
        let (mut root, errors) = vize_armature::parse_with_options(
            &allocator,
            template,
            vize_relief::options::ParserOptions {
                is_native_tag: Some(vize_carton::is_native_tag),
                ..Default::default()
            },
        );
        assert!(errors.is_empty(), "{errors:?}");
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(
            "const selected = 'button'; const registry = { button: selected };",
        );
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();
        let usage = vize_croquis::facts::component_usage_list(&summary)
            .first()
            .expect("actual named dynamic value")
            .clone();
        let owned = crate::virtual_ts::scope::is_owned_named_dynamic_component;
        assert!(owned(Some(&root), &usage));
        assert!(!owned(None, &usage));
        let mut mismatched = usage.clone();
        mismatched.start += 1;
        assert!(!owned(Some(&root), &mismatched));
        mismatched = usage.clone();
        mismatched.name = "other.button".into();
        assert!(!owned(Some(&root), &mismatched));
        {
            let Some(vize_relief::TemplateChildNode::Element(element)) = root.children.first_mut()
            else {
                panic!("actual component element");
            };
            let expression = element
                .props
                .iter_mut()
                .find_map(|prop| match prop {
                    vize_relief::PropNode::Directive(directive) => directive.exp.as_mut(),
                    _ => None,
                })
                .expect("actual bound is");
            let vize_relief::ExpressionNode::Simple(expression) = expression else {
                panic!("actual parsed simple expression");
            };
            assert!(expression.js_ast.is_some());
            expression.content = "other.button";
        }
        assert!(!owned(Some(&root), &mismatched));
        {
            let Some(vize_relief::TemplateChildNode::Element(element)) = root.children.first_mut()
            else {
                panic!("actual component element");
            };
            let expression = element
                .props
                .iter_mut()
                .find_map(|prop| match prop {
                    vize_relief::PropNode::Directive(directive) => directive.exp.as_mut(),
                    _ => None,
                })
                .expect("actual bound is");
            let vize_relief::ExpressionNode::Simple(expression) = expression else {
                panic!("actual parsed simple expression");
            };
            expression.js_ast = None;
        }
        assert!(!owned(Some(&root), &mismatched));
    }

    let ordinary = generate(
        "const registry = { button: 'button' };",
        "<Unknown :is=\"registry.button\" />",
        true,
    );
    assert!(ordinary.code.contains("__vize_global_component_"));
}

#[test]
fn named_member_exclusion_removes_only_the_exact_registry_check_and_mapping() {
    let allocator = vize_carton::Allocator::new();
    let template = "<component :is=\"registry.button\" />";
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
    analyzer.analyze_script_setup("const registry = { button: 'button' };");
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let emit = |ast| {
        let mut code = vize_carton::String::default();
        let mut mappings = Vec::new();
        let options = VirtualTsOptions::default();
        let plan = super::GlobalComponentPlan::new((&summary, ast), false, true, None);
        plan.emit(
            &mut code,
            &summary,
            &options,
            &vize_carton::FxHashSet::default(),
            &vize_carton::FxHashSet::default(),
            Some(super::GlobalComponentDiagnostics {
                mappings: &mut mappings,
                template_offset: 100,
            }),
        );
        (code, mappings)
    };
    let (owned, owned_maps) = emit(Some(&root));
    let (unowned, unowned_maps) = emit(None);
    const REGISTRY_CHECK: &str = concat!(
        "const { \"registry.button\": __vize_global_component_0 } = {} as (",
        "\"Registry.button\" extends keyof import('vue').GlobalComponents ? { \"registry.button\": unknown } : ",
        "\"registry.button\" extends keyof import('vue').GlobalComponents ? { \"registry.button\": unknown } : ",
        "\"registry.button\" extends keyof import('vue').GlobalComponents ? { \"registry.button\": unknown } : {});\n",
        "void __vize_global_component_0;\n",
    );
    assert_eq!(unowned, vize_carton::cstr!("{REGISTRY_CHECK}{owned}"));
    assert!(owned.contains("declare const registry_button:"));
    assert!(owned_maps.is_empty());
    assert_eq!(
        unowned_maps,
        vec![crate::virtual_ts::VizeMapping {
            gen_range: 8..25,
            src_range: 101..116,
            sub_spans: Vec::new(),
        }]
    );
}
