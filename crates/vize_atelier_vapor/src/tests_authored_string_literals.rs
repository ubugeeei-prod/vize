//! Authored attribute values, text and names reach the generated module only
//! as JavaScript string literals, in both lanes.

#![expect(clippy::disallowed_types, reason = "test fixtures compare std strings")]

use std::string::String;

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

fn compiled(source: &str, retained: bool) -> String {
    let allocator = Allocator::new();
    let result = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            davinci_retained_lane: retained,
            ..Default::default()
        },
    );
    assert!(
        result.error_messages.is_empty(),
        "{source}: {:?}",
        result.error_messages
    );
    String::from(result.code.as_str())
}

/// Both lanes compile every source to the module beside it.
fn assert_modules(cases: &[(&str, String)]) {
    let lane = |retained| -> Vec<String> {
        cases
            .iter()
            .map(|(source, _)| compiled(source, retained))
            .collect()
    };
    let expected: Vec<String> = cases.iter().map(|(_, module)| module.clone()).collect();
    assert_eq!([lane(false), lane(true)], [expected.clone(), expected]);
}

/// The module rendering one `<UserCard>` whose props argument is `props`.
fn user_card(props: &str) -> String {
    [
        "import { resolveComponent as _resolveComponent, createComponentWithFallback as _createComponentWithFallback } from 'vue';\n\nexport function render(_ctx) {\n  const _component_UserCard = _resolveComponent(\"UserCard\")\n  const n0 = _createComponentWithFallback(_component_UserCard, ",
        props,
        ", null, true)\n  return n0\n}\n",
    ]
    .concat()
}

#[test]
fn a_static_component_prop_value_is_always_a_string_literal() {
    assert_modules(&[
        (
            r#"<UserCard title="__RAW__f()" />"#,
            user_card(r#"{ title: () => ("__RAW__f()") }"#),
        ),
        (
            r#"<UserCard class="__RAW__f()" />"#,
            user_card(r#"{ class: () => ("__RAW__f()") }"#),
        ),
        (
            r#"<UserCard v-bind="rest" title="__RAW__f()" />"#,
            user_card(
                "{ $: [\n    () => (_ctx.rest),\n    { title: () => (\"__RAW__f()\") }\n  ] }",
            ),
        ),
        (
            r#"<UserCard title="plain" />"#,
            user_card(r#"{ title: () => ("plain") }"#),
        ),
    ]);
}

#[test]
fn component_v_model_props_keep_their_generated_callbacks() {
    assert_modules(&[(
        r#"<UserCard v-model.trim="name" />"#,
        user_card(
            "{\n    modelValue: () => (_ctx.name),\n    \"onUpdate:modelValue\": () => ($event => ((_ctx.name) = $event)),\n    modelModifiers: () => ({ \"trim\": true })\n  }",
        ),
    )]);
}

#[test]
fn a_compiler_written_getter_prop_is_emitted_as_it_is() {
    use crate::ir::{IRProp, OperationNode, PropValueKind};
    use vize_atelier_core::{SimpleExpressionNode, SourceLocation};
    use vize_carton::{Box, Vec};

    let getter = "() => _value => (_ctx.name = _value)";
    let generated = |count: usize| -> String {
        let allocator = Allocator::new();
        let arena = &allocator;
        let source = "<UserCard />";
        let (root, _) = vize_atelier_core::parser::parse(arena, source);
        let mut ir = crate::transform_to_ir(arena, &root, source);
        for operation in ir.block.operation.iter_mut() {
            let OperationNode::CreateComponent(component) = operation else {
                continue;
            };
            for _ in 0..count {
                let key =
                    SimpleExpressionNode::new("onUpdate:modelValue", true, SourceLocation::STUB);
                let value = SimpleExpressionNode::new(getter, true, SourceLocation::STUB);
                let mut values = Vec::new_in(&arena);
                values.push(Box::new_in(value, &arena));
                component.props.push(
                    IRProp::new(Box::new_in(key, &arena), values, true)
                        .with_value_kind(PropValueKind::RawGetter),
                );
            }
        }
        String::from(crate::generate_vapor(&ir, None).code.as_str())
    };

    assert_eq!(
        [generated(1), generated(2)],
        [
            user_card(r#"{ "onUpdate:modelValue": () => _value => (_ctx.name = _value) }"#),
            user_card(
                r#"{ "onUpdate:modelValue": () => ([(() => _value => (_ctx.name = _value))(), (() => _value => (_ctx.name = _value))()]) }"#,
            ),
        ]
    );
}

/// The module rendering `<MyButton>` whose default slot is `text` alone.
fn my_button_with_text_slot(text: &str) -> String {
    [
        "import { resolveComponent as _resolveComponent, createComponentWithFallback as _createComponentWithFallback, setText as _setText, template as _template } from 'vue';\nconst t0 = _template(\" \")\n\nexport function render(_ctx) {\n  const _component_MyButton = _resolveComponent(\"MyButton\")\n  const n1 = _createComponentWithFallback(_component_MyButton, null, {\n    \"default\": () => {\n      const n0 = t0()\n      _setText(n0, ",
        text,
        ")\n      return n0\n    }\n  }, true)\n  return n1\n}\n",
    ]
    .concat()
}

#[test]
fn block_root_text_that_opens_with_an_angle_bracket_is_written_as_text() {
    assert_modules(&[
        (
            "<MyButton>&lt;img src=x&gt;</MyButton>",
            my_button_with_text_slot(r#""<img src=x>""#),
        ),
        (
            "<MyButton>< b</MyButton>",
            my_button_with_text_slot(r#""< b""#),
        ),
        (
            "&lt;b&gt;bold&lt;/b&gt;",
            String::from(
                "import { setText as _setText, template as _template } from 'vue';\nconst t0 = _template(\" \", true)\n\nexport function render(_ctx) {\n  const n0 = t0()\n  _setText(n0, \"<b>bold</b>\")\n  return n0\n}\n",
            ),
        ),
        (
            "<slot>&lt;b&gt;</slot>",
            String::from(
                "import { createSlot as _createSlot, setText as _setText, template as _template } from 'vue';\nconst t0 = _template(\" \")\n\nexport function render(_ctx) {\n  const n0 = _createSlot(\"default\", null, () => {\n    const n1 = t0()\n    _setText(n1, \"<b>\")\n    return n1\n  })\n  return n0\n}\n",
            ),
        ),
        (
            r#"<template v-if="ok">&lt;b&gt;</template>"#,
            String::from(
                "import { setText as _setText, createIf as _createIf, template as _template } from 'vue';\nconst t0 = _template(\" \", true)\n\nexport function render(_ctx) {\n  const n0 = _createIf(() => (_ctx.ok), () => {\n    const n2 = t0()\n    _setText(n2, \"<b>\")\n    return n2\n  }, null, 2 /* TRUE_MULTI_ROOT */)\n  return n0\n}\n",
            ),
        ),
        (
            r#"<template v-for="item in list">&lt;b&gt;</template>"#,
            String::from(
                "import { setText as _setText, createFor as _createFor, template as _template } from 'vue';\nconst t0 = _template(\" \")\n\nexport function render(_ctx) {\n  const n0 = _createFor(() => (_ctx.list), (_for_item0) => {\n    const n2 = t0()\n    _setText(n2, \"<b>\")\n    return n2\n  }, undefined, 72 /* IS_SINGLE_NODE, WRAPPED_ROWS */)\n  return n0\n}\n",
            ),
        ),
        (
            "<span/>&lt;b&gt;<span/>",
            String::from(
                "import { setText as _setText, template as _template } from 'vue';\nconst t0 = _template(\"<span></span>\")\nconst t1 = _template(\" \")\nconst t2 = _template(\"<span></span>\")\n\nexport function render(_ctx) {\n  const n0 = t0()\n  const n1 = t1()\n  const n2 = t2()\n  _setText(n1, \"<b>\")\n  return [n0, n1, n2]\n}\n",
            ),
        ),
    ]);
}

#[test]
fn both_lanes_map_such_text_alike() {
    use super::{VaporCompilerExperimentalOptions, compile_vapor_with_sfc_context_and_capture};
    use vize_atelier_core::{TemplateSyntaxMode, options::CustomElementMatcher};
    use vize_l0::dump::capture::StageCapture;

    let lane = |retained| {
        ["<MyButton>< b</MyButton>", "<MyButton>a b</MyButton>"].map(|source| {
            let arena = Allocator::new();
            let mut capture = StageCapture::new("vapor");
            let (result, errors) = compile_vapor_with_sfc_context_and_capture(
                &arena,
                source,
                VaporCompilerOptions {
                    prefix_identifiers: true,
                    davinci_retained_lane: retained,
                    ..Default::default()
                },
                TemplateSyntaxMode::Standard,
                CustomElementMatcher::default(),
                VaporCompilerExperimentalOptions {
                    source_map: true,
                    source_map_filename: Some("Text.vue".into()),
                    ..Default::default()
                },
                None,
                &mut capture,
            );
            assert!(errors.is_empty(), "{source}: {errors:?}");
            (result.code, result.map)
        })
    };
    assert_eq!(lane(false), lane(true));
}

#[test]
fn a_host_supplied_root_writes_such_text_the_same_way() {
    let allocator = Allocator::new();
    let source = "<MyButton>&lt;img src=x&gt;</MyButton>";
    let (root, _) = vize_atelier_core::parser::parse(&allocator, source);
    let (result, diagnostics, _) = crate::generate_vapor_from_transformed_root(
        &allocator,
        &root,
        source,
        None,
        None,
        crate::VaporGenerateOptions { jsx_closure: true },
        crate::VaporGenerateExperimentalOptions {
            jsx_children: true,
            ..Default::default()
        },
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(
        result.code.as_str(),
        my_button_with_text_slot(r#""<img src=x>""#)
    );
}

#[test]
fn other_block_root_text_stays_a_text_template() {
    assert_modules(&[
        (
            "<MyButton>a &lt; b</MyButton>",
            String::from(
                "import { resolveComponent as _resolveComponent, createComponentWithFallback as _createComponentWithFallback, template as _template } from 'vue';\nconst t0 = _template(\"a < b\")\n\nexport function render(_ctx) {\n  const _component_MyButton = _resolveComponent(\"MyButton\")\n  const n1 = _createComponentWithFallback(_component_MyButton, null, {\n    \"default\": () => {\n      const n0 = t0()\n      return n0\n    }\n  }, true)\n  return n1\n}\n",
            ),
        ),
        (
            "plain",
            String::from(
                "import { template as _template } from 'vue';\nconst t0 = _template(\"plain\", true)\n\nexport function render(_ctx) {\n  const n0 = t0()\n  return n0\n}\n",
            ),
        ),
    ]);
}

/// The module rendering one `<div>` whose template ref is set to `name`.
fn div_with_ref(name: &str) -> String {
    [
        "import { createTemplateRefSetter as _createTemplateRefSetter, template as _template } from 'vue';\nconst t0 = _template(\"<div></div>\", true)\n\nexport function render(_ctx) {\n  const _setRef = _createTemplateRefSetter()\n  const n0 = t0()\n  _setRef(n0, ",
        name,
        ")\n  return n0\n}\n",
    ]
    .concat()
}

#[test]
fn a_static_template_ref_name_is_a_string_literal() {
    assert_modules(&[
        (
            r#"<div ref="x&quot;,f(),&quot;"></div>"#,
            div_with_ref(r#""x\",f(),\"""#),
        ),
        (r#"<div ref="a\b"></div>"#, div_with_ref(r#""a\\b""#)),
        (r#"<div ref="box"></div>"#, div_with_ref(r#""box""#)),
    ]);
}

/// The module rendering one component resolved as `variable` by `name`.
fn resolved_component(variable: &str, name: &str) -> String {
    [
        "import { resolveComponent as _resolveComponent, createComponentWithFallback as _createComponentWithFallback } from 'vue';\n\nexport function render(_ctx) {\n  const ",
        variable,
        " = _resolveComponent(",
        name,
        ")\n  const n0 = _createComponentWithFallback(",
        variable,
        ", null, null, true)\n  return n0\n}\n",
    ]
    .concat()
}

#[test]
fn a_component_resolves_by_a_string_literal_of_its_name() {
    assert_modules(&[
        (
            r#"<div is="vue:x&quot;+f()+&quot;"></div>"#,
            resolved_component("_component_x__f____", r#""x\"+f()+\"""#),
        ),
        (
            r#"<Foo"+f()+" />"#,
            resolved_component("_component_Foo__f____", r#""Foo\"+f()+\"""#),
        ),
        (
            r#"<div is="vue:my-thing"></div>"#,
            resolved_component("_component_my_thing", r#""my-thing""#),
        ),
        (
            r#"<component is='x"+f()+"' />"#,
            String::from(
                "import { createComponentWithFallback as _createComponentWithFallback, resolveDynamicComponent as _resolveDynamicComponent } from 'vue';\n\nexport function render(_ctx) {\n  const n0 = _createComponentWithFallback(_resolveDynamicComponent(\"x\\\"+f()+\\\"\"), null, null, true)\n  return n0\n}\n",
            ),
        ),
    ]);
}

/// The module rendering one `<div>` from its imports and its `statement`.
fn div_module(imports: &str, statement: &str) -> String {
    [
        "import { ",
        imports,
        ", template as _template } from 'vue';\nconst t0 = _template(\"<div></div>\", true)\n\nexport function render(_ctx) {\n  const n0 = t0()\n  ",
        statement,
        "\n  return n0\n}\n",
    ]
    .concat()
}

#[test]
fn a_bound_prop_name_is_a_string_literal() {
    assert_modules(&[(
        r#"<div :a",x:f(),"="v"></div>"#,
        div_module(
            "setProp as _setProp, renderEffect as _renderEffect",
            r#"_renderEffect(() => _setProp(n0, "a\",x:f(),\"", _ctx.v))"#,
        ),
    )]);
}

#[test]
fn a_listener_event_name_is_a_string_literal() {
    assert_modules(&[
        (
            r#"<div @a",f(),"="h"></div>"#,
            div_module("on as _on", r#"_on(n0, "a\",f(),\"", e => _ctx.h(e))"#),
        ),
        (
            r#"<div @a",f(),".once="h"></div>"#,
            div_module(
                "on as _on",
                "_on(n0, \"a\\\",f(),\\\"\", e => _ctx.h(e), {\n    once: true\n  })",
            ),
        ),
        (
            r#"<div v-once :onA",f(),"="h"></div>"#,
            div_module("on as _on", r#"_on(n0, "a\",f(),\"", _ctx.h)"#),
        ),
    ]);
}

#[test]
fn a_listener_modifier_name_is_a_string_literal() {
    assert_modules(&[(
        r#"<div @keyup.a",f(),"="h"></div>"#,
        div_module(
            "withKeys as _withKeys, on as _on",
            r#"_on(n0, "keyup", _withKeys(e => _ctx.h(e), ["a\",f(),\""]))"#,
        ),
    )]);
}

#[test]
fn a_directive_resolves_by_a_string_literal_of_its_name() {
    let module = |name: &str| -> String {
        [
            "import { resolveDirective as _resolveDirective, withVaporDirectives as _withVaporDirectives, template as _template } from 'vue';\nconst t0 = _template(\"<div></div>\", true)\n\nexport function render(_ctx) {\n  const _directive_a_b = _resolveDirective(",
            name,
            ")\n  const n0 = t0()\n  _withVaporDirectives(n0, [[_directive_a_b, () => (_ctx.x)]])\n  return n0\n}\n",
        ]
        .concat()
    };
    assert_modules(&[
        (r#"<div v-a"b="x"></div>"#, module(r#""a\"b""#)),
        (r#"<div v-a\b="x"></div>"#, module(r#""a\\b""#)),
    ]);
}
