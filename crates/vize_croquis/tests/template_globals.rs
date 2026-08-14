use vize_armature::parse;
use vize_carton::Allocator;
use vize_croquis::croquis::BindingMetadata;
use vize_croquis::script_parser::parse_script_setup;
use vize_croquis::virtual_ts::{VirtualTsConfig, VirtualTsGenerator};

const ATTRS_DECLARATION: &str = "const $attrs: Readonly<Record<string, unknown>> = {};";

#[test]
fn full_sfc_declares_attrs_only_inside_template_scope() {
    let script = "const visible = true";
    let template = r#"<div v-if="visible" v-bind="$attrs">{{ $attrs.id }}</div>"#;
    let parse_result = parse_script_setup(script);
    let allocator = Allocator::new();
    let (template_ast, errors) = parse(&allocator, template);
    assert!(errors.is_empty(), "template should parse without errors");

    let mut generator = VirtualTsGenerator::new();
    let output = generator.generate_from_croquis(
        script,
        &parse_result,
        Some(&template_ast),
        &VirtualTsConfig::default(),
        None,
    );
    let (setup_scope, template_scope) = output
        .content
        .split_once(";(function __template() {")
        .expect("generated output should contain a template scope");

    assert!(!setup_scope.contains(ATTRS_DECLARATION));
    assert!(template_scope.contains(ATTRS_DECLARATION));
    assert!(
        template_scope.contains(" = $attrs;"),
        "generated output should check the attribute expression:\n{}",
        output.content
    );
}

#[test]
fn standalone_template_declares_attrs_before_expressions() {
    let allocator = Allocator::new();
    let (template_ast, errors) = parse(&allocator, r#"<main v-bind="$attrs"></main>"#);
    assert!(errors.is_empty(), "template should parse without errors");

    let mut generator = VirtualTsGenerator::new();
    let output = generator.generate_template(&template_ast, &BindingMetadata::default(), 0, false);
    let declaration = output
        .content
        .find(ATTRS_DECLARATION)
        .expect("generated output should declare template attributes");
    let expression = output
        .content
        .find(" = $attrs;")
        .expect("generated output should check the attribute expression");

    assert!(declaration < expression);
}
