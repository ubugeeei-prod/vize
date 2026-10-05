//! #8003's real namespace props must retain their component constructor.

use super::generate;

const CHOICE: &str = "import { defineComponent } from 'vue'; const Choice = defineComponent({ props: { count: { type: Number, required: true } } });";

#[test]
fn dynamic_namespace_props_resolve_the_exact_value_in_both_registry_modes() {
    for (declarations, expression, reference) in [
        (
            "const registry = { Choice };",
            "registry.Choice",
            "registry.Choice",
        ),
        (
            "const Registry = { Choice: 'button' }; const registry = { Choice };",
            "registry.Choice",
            "registry.Choice",
        ),
        (
            "const registry = { nested: { Choice } };",
            "registry.nested.Choice",
            "registry.nested.Choice",
        ),
        (
            "const Registry = { Choice };",
            r"Regis\u0074ry.Choice",
            "Registry.Choice",
        ),
        (
            "const { alt } = defineProps<{ as: { Choice: typeof Choice }; alt?: string }>();",
            "as.Choice",
            "_as.Choice",
        ),
    ] {
        let script = vize_carton::cstr!("{CHOICE} {declarations}");
        let template = vize_carton::cstr!("<component :is=\"{expression}\" count=\"wrong\" />");
        for checked in [true, false] {
            let output = generate(&script, &template, checked);
            assert!(
                output
                    .code
                    .contains(vize_carton::cstr!(" = typeof {reference} extends ").as_str()),
                "{template}\n{}",
                output.code
            );
            assert!(output.code.contains("\"count\": \"wrong\""));
            assert!(!output.code.contains("typeof registry_Choice extends "));
            let value = 100 + template.find("wrong").expect("authored wrong value");
            let value_span = output
                .mapping
                .spans()
                .iter()
                .flat_map(|mapping| &mapping.sub_spans)
                .find(|span| span.src_range == (value..value + 5))
                .expect("exact authored wrong value mapping");
            assert_eq!(
                output.code.get(value_span.gen_range.clone()),
                Some("\"wrong\"")
            );
            let name = 100 + template.find("count=").expect("authored count name");
            let name_span = output
                .mapping
                .spans()
                .iter()
                .flat_map(|mapping| &mapping.sub_spans)
                .find(|span| span.src_range == (name..name + 5))
                .expect("exact authored count diagnostic anchor");
            assert!(
                output
                    .code
                    .get(name_span.gen_range.clone())
                    .is_some_and(|text| text.starts_with("__vize_prop_check_"))
            );
            assert!(!output.code.contains("__vize_global_component_"));
        }
    }
}

#[test]
fn ordinary_namespaced_tags_keep_their_markup_reference() {
    let script = vize_carton::cstr!("{CHOICE} const Registry = {{ Choice }};");
    let template = "<Registry.Choice count=\"wrong\" />";
    let output = generate(&script, template, true);
    assert!(
        output
            .code
            .contains("type __Registry_Choice_Props_0 = typeof Registry_Choice")
    );
    assert!(output.code.contains("__vize_global_component_"));
    assert!(!output.code.contains("typeof Registry.Choice extends "));
}

#[test]
fn expression_roots_keep_import_external_and_type_only_authority() {
    use crate::virtual_ts::component_reference::resolved_exact_component_binding_reference;
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
    analyzer.analyze_script_setup("import * as registry from './choices'; const Registry = {};");
    let summary = analyzer.finish();
    let options = crate::virtual_ts::VirtualTsOptions {
        external_template_bindings: vec!["external".into(), "registry".into()],
        ..Default::default()
    };
    let mut type_only = vize_carton::FxHashSet::default();
    let resolve = |name, type_only| {
        resolved_exact_component_binding_reference(&summary, &options, type_only, name)
    };
    assert_eq!(resolve("registry", &type_only).as_deref(), Some("registry"));
    assert_eq!(resolve("external", &type_only).as_deref(), Some("external"));
    assert_eq!(resolve("External", &type_only), None);
    type_only.insert(vize_carton::CompactString::new("registry"));
    assert_eq!(resolve("registry", &type_only), None);
    type_only.insert(vize_carton::CompactString::new("external"));
    assert_eq!(resolve("external", &type_only), None);

    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
    analyzer.analyze_script_setup("const Registry = {};");
    assert_eq!(
        resolved_exact_component_binding_reference(
            &analyzer.finish(),
            &crate::virtual_ts::VirtualTsOptions::default(),
            &vize_carton::FxHashSet::default(),
            "registry",
        ),
        None,
        "an uppercase sibling cannot supply the authored expression root",
    );
}

#[test]
fn declined_plain_and_unadmitted_roots_and_legacy_keep_complete_output() {
    use crate::virtual_ts::{
        VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts_with_offsets_and_checks,
    };
    for (legacy_vue2, expression) in [
        (false, "selected"),
        (false, r"regis\u0074ry.Choice"),
        (false, "(registry).Choice"),
        (true, "selected"),
        (true, "registry.Choice"),
    ] {
        let script = vize_carton::cstr!(
            "{CHOICE} const selected = Choice; const Selected = 'button'; const registry = {{ Choice }};"
        );
        let template = vize_carton::cstr!("<component :is=\"{expression}\" count=\"wrong\" />");
        let allocator = vize_carton::Allocator::new();
        let (mut root, errors) = vize_armature::parse_with_options(
            &allocator,
            &template,
            vize_relief::options::ParserOptions {
                is_native_tag: Some(vize_carton::is_native_tag),
                ..Default::default()
            },
        );
        assert!(errors.is_empty(), "{errors:?}");
        let mut analyzer =
            vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
        analyzer.analyze_script_setup(&script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();
        if matches!(expression, r"regis\u0074ry.Choice" | "(registry).Choice") {
            assert!(vize_croquis::facts::component_usage_list(&summary).is_empty());
        }
        let emit = |root| {
            generate_virtual_ts_with_offsets_and_checks(
                &summary,
                Some(&script),
                Some(root),
                0,
                100,
                &VirtualTsOptions::default(),
                VirtualTsGenerationOptions {
                    legacy_vue2,
                    ..Default::default()
                },
            )
        };
        let fresh = emit(&root);
        let Some(vize_relief::TemplateChildNode::Element(element)) = root.children.first_mut()
        else {
            panic!("actual component element");
        };
        let expression_node = element
            .props
            .iter_mut()
            .find_map(|prop| match prop {
                vize_relief::PropNode::Directive(directive) => directive.exp.as_mut(),
                _ => None,
            })
            .expect("actual bound is");
        let vize_relief::ExpressionNode::Simple(expression_node) = expression_node else {
            panic!("actual named bound-is expression");
        };
        assert!(expression_node.js_ast.take().is_some());
        let declined = emit(&root);
        assert_eq!(fresh.code, declined.code, "{template}");
        assert_eq!(fresh.mapping, declined.mapping, "{template}");
        assert!(!fresh.code.contains("typeof registry.Choice extends "));
        if expression == "selected" {
            assert!(fresh.code.contains("typeof Selected extends "));
        }
    }
}
