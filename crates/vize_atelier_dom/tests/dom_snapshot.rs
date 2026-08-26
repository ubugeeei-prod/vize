//! DOM compiler snapshot tests.
//!
//! These tests compare the DOM compiler output against expected snapshots.
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_dom::compile_template;
use vize_s0::Allocator;

/// Helper to get the compiled code
fn get_compiled(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(&allocator, src);

    if !errors.is_empty() {
        panic!("Compilation errors: {:?}", errors);
    }

    format!("{}\n{}", result.preamble, result.code)
}

// =============================================================================
// Static Element Tests
// =============================================================================

mod static_element {
    use super::get_compiled;

    #[test]
    fn simple_div() {
        insta::assert_snapshot!(get_compiled("<div></div>"));
    }

    #[test]
    fn div_with_text() {
        insta::assert_snapshot!(get_compiled("<div>hello</div>"));
    }

    #[test]
    fn nested_elements() {
        insta::assert_snapshot!(get_compiled("<div><span>hello</span></div>"));
    }

    #[test]
    fn nested_elements_with_static_attrs() {
        insta::assert_snapshot!(get_compiled(
            r#"<div class="wrapper"><span>hello</span></div>"#
        ));
    }

    #[test]
    fn nested_dynamic_element_does_not_hoist_parent_attrs() {
        insta::assert_snapshot!(get_compiled(
            r#"<div class="wrapper"><span :class="active">hello</span></div>"#
        ));
    }

    #[test]
    fn nested_component_does_not_hoist_parent_attrs() {
        insta::assert_snapshot!(get_compiled(
            r#"<div class="wrapper"><MyComponent class="child" /></div>"#
        ));
    }
}

// =============================================================================
// Interpolation Tests
// =============================================================================

mod interpolation {
    use super::get_compiled;

    #[test]
    fn simple_interpolation() {
        insta::assert_snapshot!(get_compiled("{{ msg }}"));
    }

    #[test]
    fn interpolation_in_element() {
        insta::assert_snapshot!(get_compiled("<div>{{ msg }}</div>"));
    }
}

// =============================================================================
// v-if Tests
// =============================================================================

mod v_if {
    use super::get_compiled;

    #[test]
    fn simple_v_if() {
        insta::assert_snapshot!(get_compiled(r#"<div v-if="ok">hello</div>"#));
    }

    #[test]
    fn v_if_v_else() {
        insta::assert_snapshot!(get_compiled(
            r#"<div v-if="ok">yes</div><div v-else>no</div>"#
        ));
    }

    #[test]
    fn v_if_component_with_slot() {
        insta::assert_snapshot!(get_compiled(
            r#"<MyComponent v-if="ok"><span>slot content</span></MyComponent>"#
        ));
    }

    #[test]
    fn v_if_component_with_named_slot() {
        insta::assert_snapshot!(get_compiled(
            r#"<MyComponent v-if="ok"><template #header><h1>title</h1></template></MyComponent>"#
        ));
    }

    #[test]
    fn duplicate_event_keys_are_merged_without_dropping_handlers() {
        let code = get_compiled(r#"<button v-if="ok" @click="a" @click.ctrl="b"></button>"#);

        assert_eq!(code.matches("onClick:").count(), 1, "{code}");
        assert!(
            code.contains(r#"onClick: [a, _withModifiers(b, ["ctrl"])]"#),
            "{code}",
        );
    }
}

// =============================================================================
// v-for Tests
// =============================================================================

mod v_for {
    use super::get_compiled;

    #[test]
    fn simple_v_for() {
        insta::assert_snapshot!(get_compiled(
            r#"<div v-for="item in items">{{ item }}</div>"#
        ));
    }

    #[test]
    fn duplicate_event_keys_after_a_spread_are_merged() {
        let code = get_compiled(
            r#"<li v-for="item in items" :key="item.id" v-bind="item.props" @keydown="a" @keydown.enter.prevent="b"></li>"#,
        );

        assert_eq!(code.matches("onKeydown:").count(), 1, "{code}");
        assert!(
            code.contains(
                r#"onKeydown: [a, _withKeys(_withModifiers(b, ["prevent"]), ["enter"])]"#,
            ),
            "{code}",
        );
    }
}

// =============================================================================
// v-bind Tests
// =============================================================================

mod v_bind {
    use super::get_compiled;

    #[test]
    fn dynamic_id() {
        insta::assert_snapshot!(get_compiled(r#"<div :id="foo"></div>"#));
    }

    #[test]
    fn dynamic_class() {
        insta::assert_snapshot!(get_compiled(r#"<div :class="cls"></div>"#));
    }

    #[test]
    fn merge_static_and_dynamic_class_with_vbind_object() {
        insta::assert_snapshot!(get_compiled(
            r#"<input v-bind="attrs" class="base" :class="stateClass" />"#
        ));
    }

    #[test]
    fn merge_static_and_dynamic_style_with_vbind_object() {
        insta::assert_snapshot!(get_compiled(
            r#"<input v-bind="attrs" style="color: red" :style="dynamicStyle" />"#
        ));
    }
}

// =============================================================================
// v-on Tests
// =============================================================================

mod v_on {
    use super::get_compiled;

    #[test]
    fn click_handler() {
        insta::assert_snapshot!(get_compiled(r#"<div @click="handler"></div>"#));
    }
}

// =============================================================================
// v-model Tests
// =============================================================================

mod v_model {
    use super::get_compiled;

    #[test]
    fn input_text() {
        insta::assert_snapshot!(get_compiled(r#"<input v-model="msg" />"#));
    }
}

// =============================================================================
// v-show Tests
// =============================================================================

mod v_show {
    use super::get_compiled;

    #[test]
    fn simple_v_show() {
        insta::assert_snapshot!(get_compiled(r#"<div v-show="visible">content</div>"#));
    }

    #[test]
    fn v_show_on_child_component() {
        insta::assert_snapshot!(get_compiled(
            r#"<div><MyComponent v-show="visible" /></div>"#
        ));
    }

    #[test]
    fn v_show_on_root_component() {
        insta::assert_snapshot!(get_compiled(r#"<MyComponent v-show="visible" />"#));
    }
}

// =============================================================================
// Component Tests
// =============================================================================

mod component {
    use super::get_compiled;

    #[test]
    fn simple_component() {
        insta::assert_snapshot!(get_compiled("<MyComponent></MyComponent>"));
    }

    #[test]
    fn pascal_html_element_name_compiles_as_component() {
        let code = get_compiled(r#"<Table><span class="x">hello</span></Table>"#);

        assert!(code.contains(r#"_resolveComponent("Table")"#), "{code}");
        assert!(code.contains(r#"_createElementVNode("span""#), "{code}");
    }

    #[test]
    fn model_update_handlers_merge_in_template_source_order() {
        let model_first = get_compiled(r#"<Foo v-model="value" @update:modelValue="onUpdate" />"#);
        assert_eq!(
            model_first.matches(r#""onUpdate:modelValue":"#).count(),
            1,
            "{model_first}",
        );
        assert!(
            model_first.contains(
                r#"modelValue: value,
    "onUpdate:modelValue": [$event => ((value) = $event), onUpdate]"#,
            ),
            "{model_first}",
        );
        assert!(
            model_first.contains(r#"["modelValue", "onUpdate:modelValue"]"#),
            "{model_first}",
        );

        let listener_first =
            get_compiled(r#"<Foo @update:modelValue="onUpdate" v-model="value" />"#);
        assert_eq!(
            listener_first.matches(r#""onUpdate:modelValue":"#).count(),
            1,
            "{listener_first}",
        );
        assert!(
            listener_first.contains(
                r#""onUpdate:modelValue": [onUpdate, $event => ((value) = $event)],
    modelValue: value"#,
            ),
            "{listener_first}",
        );
        assert!(
            listener_first.contains(r#"["onUpdate:modelValue", "modelValue"]"#),
            "{listener_first}",
        );
    }
}
