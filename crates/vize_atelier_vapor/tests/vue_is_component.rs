//! Complete cross-backend regressions for original third-party #8328.
use vize_atelier_core::options::{CustomElementMatcher, TemplateSyntaxMode};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_atelier_ssr::{
    SsrCompilerOptions, compile_ssr, compile_ssr_with_custom_elements_and_template_syntax,
};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_l0::Allocator;

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/original.template.txt"
);
const PROPS: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/props-slot.template.txt"
);
const ENCODED: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/encoded.template.txt"
);

fn identical_backend_outputs(cast: &str, named: &str) {
    let allocator = Allocator::default();
    let options = DomCompilerOptions {
        prefix_identifiers: true,
        ..Default::default()
    };
    let (_, cast_errors, cast_dom) =
        compile_template_with_options(&allocator, cast, options.clone());
    let (_, named_errors, named_dom) = compile_template_with_options(&allocator, named, options);
    assert!(cast_errors.is_empty(), "{cast_errors:?}");
    assert!(named_errors.is_empty(), "{named_errors:?}");
    assert_eq!(cast_dom.code, named_dom.code, "whole DOM code for {cast}");
    assert_eq!(cast_dom.preamble, named_dom.preamble);
    assert_eq!(cast_dom.map, named_dom.map);
    let (_, cast_errors, cast_ssr) = compile_ssr(&allocator, cast);
    let (_, named_errors, named_ssr) = compile_ssr(&allocator, named);
    assert!(cast_errors.is_empty(), "{cast_errors:?}");
    assert!(named_errors.is_empty(), "{named_errors:?}");
    assert_eq!(cast_ssr.code, named_ssr.code, "whole SSR code for {cast}");
    assert_eq!(cast_ssr.preamble, named_ssr.preamble);
    assert_eq!(cast_ssr.map, named_ssr.map);
    let options = VaporCompilerOptions {
        prefix_identifiers: true,
        ..Default::default()
    };
    let cast_vapor = compile_vapor(&allocator, cast, options.clone());
    let named_vapor = compile_vapor(&allocator, named, options);
    assert!(
        cast_vapor.error_messages.is_empty(),
        "{:?}",
        cast_vapor.error_messages
    );
    assert!(
        named_vapor.error_messages.is_empty(),
        "{:?}",
        named_vapor.error_messages
    );
    assert_eq!(
        cast_vapor.code, named_vapor.code,
        "whole Vapor code for {cast}"
    );
    assert_eq!(cast_vapor.templates, named_vapor.templates);
    assert_eq!(cast_vapor.map, named_vapor.map);
}

#[test]
fn native_cast_uses_the_existing_component_props_and_default_slots() {
    for (cast, named) in [
        (ORIGINAL, "<div>\n  <my-thing>x</my-thing>\n</div>\n"),
        (
            PROPS,
            "<div><my-thing title=\"before\" :id=\"id\">x<span>{{ label }}</span></my-thing></div>",
        ),
        (ENCODED, "<my-thing title=\"a&amp;b\">x</my-thing>"),
        (
            include_str!(
                "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/ordinary-template.template.txt"
            ),
            "<my-thing>x</my-thing>",
        ),
        (
            "<div is=\"vue:my&amp;amp;thing\">x</div>",
            "<my&amp;thing>x</my&amp;thing>",
        ),
        (
            "<div><button v-if=\"ok\" is=\"vue:my-thing\">a</button><div v-else is=\"vue:my-thing\">b</div></div>",
            "<div><my-thing v-if=\"ok\">a</my-thing><my-thing v-else>b</my-thing></div>",
        ),
        (
            "<div><div v-for=\"item in items\" is=\"vue:my-thing\" :key=\"item.id\">{{ item.name }}</div></div>",
            "<div><my-thing v-for=\"item in items\" :key=\"item.id\">{{ item.name }}</my-thing></div>",
        ),
        (
            "<table><tr is=\"vue:my-row\"><td>x</td></tr></table>",
            "<table><my-row><td>x</td></my-row></table>",
        ),
    ] {
        identical_backend_outputs(cast, named);
    }
}

#[test]
fn plain_native_is_remains_a_customized_builtin_attribute() {
    let allocator = Allocator::default();
    let (_, errors, result) =
        compile_ssr(&allocator, "<div is=\"my-thing\" title=\"native\">x</div>");
    assert!(errors.is_empty());
    assert_eq!(
        result.code,
        "function ssrRender(_ctx, _push, _parent, _attrs) {\n  _push(`<div${_ssrRenderAttrs(_mergeProps({ is: \"my-thing\", title: \"native\" }, _attrs))}>x</div>`)\n}\n"
    );
}

#[test]
fn bound_native_is_does_not_select_a_dynamic_component() {
    let allocator = Allocator::default();
    let (_, errors, result) = compile_ssr(&allocator, "<div :is=\"view\">x</div>");
    assert!(errors.is_empty());
    assert_eq!(
        result.code,
        "function ssrRender(_ctx, _push, _parent, _attrs) {\n  _push(`<div${_ssrRenderAttrs(_mergeProps({ is: _ctx.view }, _attrs))}>x</div>`)\n}\n"
    );
}

#[test]
fn explicit_custom_element_admission_keeps_the_prefix_as_a_native_attribute() {
    let allocator = Allocator::default();
    let matcher = CustomElementMatcher::from_static_predicate(|tag| tag == "div");
    let (root, errors, result) = compile_ssr_with_custom_elements_and_template_syntax(
        &allocator,
        "<div is=\"vue:my-thing\" title=\"native\">x</div>",
        SsrCompilerOptions::default(),
        TemplateSyntaxMode::Standard,
        matcher,
    );
    assert!(errors.is_empty());
    let [vize_atelier_core::TemplateChildNode::Element(element)] = root.children.as_slice() else {
        panic!("native custom-element owner")
    };
    assert_eq!(element.tag_type, vize_atelier_core::ElementType::Element);
    assert_eq!(element.tag, "div");
    assert_eq!(
        result.code,
        "function ssrRender(_ctx, _push, _parent, _attrs) {\n  _push(`<div${_ssrRenderAttrs(_mergeProps({ is: \"vue:my-thing\", title: \"native\" }, _attrs))}>x</div>`)\n}\n"
    );
}

#[test]
fn verbatim_native_is_is_not_a_component_selector() {
    let allocator = Allocator::default();
    let (_, errors, result) = compile_ssr(&allocator, "<div v-pre is=\"vue:my-thing\">x</div>");
    assert!(errors.is_empty());
    assert_eq!(
        result.code,
        "function ssrRender(_ctx, _push, _parent, _attrs) {\n  _push(`<div${_ssrRenderAttrs(_mergeProps({ is: \"vue:my-thing\" }, _attrs))}>x</div>`)\n}\n"
    );
}

#[test]
fn explicit_dynamic_component_preserves_its_entire_selector_value() {
    let allocator = Allocator::default();
    let (_, errors, result) = compile_ssr(&allocator, "<component is=\"vue:my-thing\" />");
    assert!(errors.is_empty());
    assert_eq!(
        result.code,
        "function ssrRender(_ctx, _push, _parent, _attrs) {\n  _ssrRenderVNode(_push, _createVNode(_resolveDynamicComponent(\"vue:my-thing\"), _attrs, null), _parent)\n}\n"
    );
}

#[test]
fn custom_renderer_classifies_static_casts_without_promoting_unknown_native_tags() {
    for (source, kind) in [
        ("<text></text>", vize_atelier_core::ElementType::Element),
        (
            "<div is=\"vue:my-thing\"></div>",
            vize_atelier_core::ElementType::Component,
        ),
        ("<Thing></Thing>", vize_atelier_core::ElementType::Component),
    ] {
        let allocator = Allocator::default();
        let (root, errors) = vize_atelier_core::parser::parse_with_options(
            &allocator,
            source,
            vize_atelier_core::options::ParserOptions {
                custom_renderer: true,
                is_native_tag: Some(vize_l0::is_native_tag),
                ..Default::default()
            },
        );
        assert!(errors.is_empty(), "{errors:?}");
        let [vize_atelier_core::TemplateChildNode::Element(element)] = root.children.as_slice()
        else {
            panic!("complete custom-renderer owner")
        };
        assert_eq!(element.tag_type, kind);
    }
}

#[test]
fn reserved_component_names_resolve_as_static_components() {
    // Primary Vue 3.5.41 semantics, with Vize's established function formatting.
    // The 3.5.43 resolver/parser sources retain the same cast semantics.
    for (source, ssr, dom) in [
        (
            include_str!(
                "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/reserved-lowercase-component.template.txt"
            ),
            include_str!(
                "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/reserved-lowercase-component.ssr.txt"
            ),
            include_str!(
                "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/reserved-lowercase-component.dom.txt"
            ),
        ),
        (
            include_str!(
                "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/reserved-uppercase-component.template.txt"
            ),
            include_str!(
                "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/reserved-uppercase-component.ssr.txt"
            ),
            include_str!(
                "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/reserved-uppercase-component.dom.txt"
            ),
        ),
        (
            include_str!(
                "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/reserved-slot.template.txt"
            ),
            include_str!(
                "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/reserved-slot.ssr.txt"
            ),
            include_str!(
                "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/reserved-slot.dom.txt"
            ),
        ),
        (
            include_str!(
                "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/reserved-template.template.txt"
            ),
            include_str!(
                "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/reserved-template.ssr.txt"
            ),
            include_str!(
                "../../../tests/_fixtures/differential/compiler/vue-is-component-8328/reserved-template.dom.txt"
            ),
        ),
    ] {
        let allocator = Allocator::default();
        let (_, errors, result) = compile_ssr(&allocator, source);
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(result.code, ssr, "primary SSR function for {source}");
        let (_, errors, result) = compile_template_with_options(
            &allocator,
            source,
            DomCompilerOptions {
                prefix_identifiers: true,
                hoist_static: false,
                ..Default::default()
            },
        );
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(result.code, dom, "primary DOM function for {source}");
        let options = VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        };
        let native = compile_vapor(&allocator, source, options.clone());
        let retained = compile_vapor(
            &allocator,
            source,
            VaporCompilerOptions {
                davinci_retained_lane: true,
                ..options
            },
        );
        assert!(
            native.error_messages.is_empty(),
            "{:?}",
            native.error_messages
        );
        assert!(
            retained.error_messages.is_empty(),
            "{:?}",
            retained.error_messages
        );
        assert_eq!(native.code, retained.code);
        assert_eq!(native.templates, retained.templates);
        assert_eq!(native.map, retained.map);
    }
}
