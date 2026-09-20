use vize_carton::{Allocator, Box as VizeBox};
use vize_croquis::{Analyzer, AnalyzerOptions};
use vize_relief::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, SimpleExpressionNode, SourceLocation,
    TemplateChildNode,
};

use super::{FallthroughComponentScope, fallthrough_props_type_ref, possible_raw_if_chain_tags};
use crate::virtual_ts::types::VirtualTsOptions;

fn fallthrough_type(script: &str, template: &str) -> Option<vize_carton::String> {
    fallthrough_type_with(script, template, false)
}

fn fallthrough_type_with(
    script: &str,
    template: &str,
    check_required: bool,
) -> Option<vize_carton::String> {
    let allocator = Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let scope = FallthroughComponentScope {
        summary: &summary,
        options: &VirtualTsOptions::default(),
        syntactic_type_only_imported_names: &Default::default(),
        resolve_component_roots: true,
        check_required,
    };
    fallthrough_props_type_ref(&scope, Some(&root), false)
}

// Without `fallthroughAttributes` a component root stays the open surface it
// always was: no project gets new listener or prop types it never asked for.
#[test]
fn component_roots_stay_open_without_fallthrough_attributes() {
    let allocator = Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, "<Basic>{{ title }}</Basic>");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer
        .analyze_script_setup("import Basic from './basic.vue'\ndefineProps<{ title: string }>()");
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let scope = FallthroughComponentScope {
        summary: &summary,
        options: &VirtualTsOptions::default(),
        syntactic_type_only_imported_names: &Default::default(),
        resolve_component_roots: false,
        check_required: false,
    };

    assert_eq!(
        fallthrough_props_type_ref(&scope, Some(&root), false).as_deref(),
        Some("Record<string, unknown>")
    );
}

fn raw_branch<'a>(
    allocator: &'a Allocator,
    tag: &'a str,
    directive_name: &'a str,
    condition: Option<&'a str>,
) -> TemplateChildNode<'a> {
    let mut element = ElementNode::new(allocator, tag, SourceLocation::STUB);
    let mut directive = DirectiveNode::new(allocator, directive_name, SourceLocation::STUB);
    directive.exp = condition.map(|condition| {
        ExpressionNode::Simple(VizeBox::new_in(
            SimpleExpressionNode::new(condition, false, SourceLocation::STUB),
            &allocator,
        ))
    });
    element
        .props
        .push(PropNode::Directive(VizeBox::new_in(directive, &allocator)));
    TemplateChildNode::Element(VizeBox::new_in(element, &allocator))
}

#[test]
fn emits_native_root_fallthrough_props_for_single_root() {
    let ty = fallthrough_type(
        "defineProps<{ title: string }>()",
        "<button>{{ title }}</button>",
    )
    .expect("single native root should accept fallthrough props");

    assert_eq!(ty, "Partial<__VizeNativeElement<\"button\">>");
}

#[test]
fn emits_open_fallthrough_props_for_single_component_root() {
    let ty = fallthrough_type(
        "defineProps<{ title: string }>()",
        "<BaseInput>{{ title }}</BaseInput>",
    )
    .expect("single component root should keep fallthrough props open");

    assert_eq!(ty, "Record<string, unknown>");
}

// `<child>` over an imported `<basic />` root forwards what `basic` accepts:
// its props and the listener props of its emits, not an open record.
#[test]
fn resolves_fallthrough_props_of_an_imported_single_component_root() {
    let ty = fallthrough_type(
        "import Basic from './basic.vue'\ndefineProps<{ title: string }>()",
        "<Basic>{{ title }}</Basic>",
    )
    .expect("single imported component root should forward that component's props");

    assert_eq!(ty, "Partial<__VizeComponentFallthroughProps<typeof Basic>>");

    let kebab = fallthrough_type(
        "import BaseInput from './base-input.vue'\ndefineProps<{ title: string }>()",
        "<base-input>{{ title }}</base-input>",
    )
    .expect("kebab-case usage resolves the PascalCase import");

    assert_eq!(
        kebab,
        "Partial<__VizeComponentFallthroughProps<typeof BaseInput>>"
    );
}

// `checkRequiredFallthroughAttributes`: the parent must supply the root's
// required props, except the ones the root binds itself.
#[test]
fn check_required_forwards_declared_props_minus_the_roots_own_bindings() {
    let ty = fallthrough_type_with(
        "import Basic from './basic.vue'\ndefineProps<{ title: string }>()",
        r#"<Basic foo="..." :bar-baz="title" @save="() => {}" v-model="title" />"#,
        true,
    )
    .expect("single imported component root should forward that component's props");

    assert_eq!(
        ty,
        "Omit<__VizeComponentFallthroughSurface<typeof Basic>, \"foo\" | \"barBaz\" | \"onSave\" | \"modelValue\">"
    );

    let native = fallthrough_type_with("defineProps<{ title: string }>()", "<input />", true)
        .expect("single native root should forward its attributes");

    assert_eq!(native, "__VizeNativeElement<\"input\">");
}

#[test]
fn skips_fallthrough_props_when_inherit_attrs_is_false() {
    let ty = fallthrough_type(
        "defineOptions({ inheritAttrs: false })\ndefineProps<{ title: string }>()",
        "<div>{{ title }}</div>",
    );

    assert_eq!(ty, None);
}

#[test]
fn inherit_attrs_false_uses_explicit_attrs_forwarding_target() {
    let native_ty = fallthrough_type(
        "defineOptions({ inheritAttrs: false })\ndefineProps<{ title: string }>()",
        r#"<div><button v-bind="$attrs">{{ title }}</button></div>"#,
    )
    .expect("explicit native $attrs forwarding should accept fallthrough props");

    assert_eq!(native_ty, "Partial<__VizeNativeElement<\"button\">>");

    let component_ty = fallthrough_type(
        "defineOptions({ inheritAttrs: false })\ndefineProps<{ title: string }>()",
        r#"<Wrapper><Primitive v-bind="{ ...scopeIdAttrs, ...$attrs }" /></Wrapper>"#,
    )
    .expect("explicit component $attrs forwarding should keep fallthrough open");

    assert_eq!(component_ty, "Record<string, unknown>");
}

#[test]
fn explicit_attrs_forwarding_takes_precedence_over_automatic_root_fallthrough() {
    let ty = fallthrough_type(
        "defineProps<{ title: string }>()",
        r#"<li><a v-bind="$attrs">{{ title }}</a></li>"#,
    )
    .expect("explicit $attrs forwarding should define the accepted attr surface");

    assert_eq!(ty, "Partial<__VizeNativeElement<\"a\">>");
}

#[test]
fn skips_fallthrough_props_for_multi_root_or_mixed_v_if_branch() {
    for template in [
        "<div /> <span />",
        "<div v-if=\"on\" /><template v-else><p /><p /></template>",
        "<template v-if=\"true\"><p /><p /></template>",
        "<div v-if=\"true\" /><span />",
        "<div v-if=\"on\" /><span v-else-if=\"maybe\" />",
        "<div v-for=\"item in items\" />",
        "<template v-for=\"item in items\"><div /></template>",
    ] {
        assert_eq!(
            fallthrough_type("const on = true\nconst items = [1]", template),
            None,
            "template should not be an always-single native root: {template}"
        );
    }
}

#[test]
fn combines_native_fallthrough_props_when_all_v_if_branches_are_single_roots() {
    let ty = fallthrough_type("const on = true", "<div v-if=\"on\" /><span v-else />")
        .expect("v-if/v-else single native roots should accept common fallthrough props");

    assert_eq!(
        ty,
        "Partial<__VizeNativeElement<\"div\">> & Partial<__VizeNativeElement<\"span\">>"
    );
}

#[test]
fn literal_true_v_else_if_terminates_single_native_root_chain() {
    let ty = fallthrough_type(
        "const on = true",
        "<div v-if=\"on\" /><span v-else-if=\"true\" />",
    )
    .expect("literal true v-else-if is an exhaustive native-root branch");

    assert_eq!(
        ty,
        "Partial<__VizeNativeElement<\"div\">> & Partial<__VizeNativeElement<\"span\">>"
    );
}

#[test]
fn raw_literal_true_v_else_if_terminates_single_native_root_chain() {
    let allocator = Allocator::new();
    let first = raw_branch(&allocator, "div", "if", Some("on"));
    let second = raw_branch(&allocator, "span", "else-if", Some("true"));
    let refs: std::vec::Vec<_> = [&first, &second].into_iter().collect();

    assert_eq!(
        possible_raw_if_chain_tags(refs.as_slice()),
        Some(vec![
            vize_carton::String::from("div"),
            vize_carton::String::from("span")
        ])
    );
}
