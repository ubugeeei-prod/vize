use super::static_type::{has_only_native_element_descendants, has_only_static_nested_children};
use super::{StaticType, get_static_type, is_static_node};
use crate::TemplateChildNode;
use crate::parser::parse;
use vize_s0::Allocator;

#[test]
fn test_static_text() {
    let allocator = Allocator::new();
    let (root, _) = parse(&allocator, "hello");
    assert!(is_static_node(&root.children[0]));
}

#[test]
fn test_static_element() {
    let allocator = Allocator::new();
    let (root, _) = parse(&allocator, "<div>static</div>");
    assert!(is_static_node(&root.children[0]));
}

#[test]
fn test_dynamic_element() {
    let allocator = Allocator::new();
    let (root, _) = parse(&allocator, "<div :class=\"cls\">dynamic</div>");
    assert!(!is_static_node(&root.children[0]));
}

#[test]
fn test_interpolation_not_static() {
    let allocator = Allocator::new();
    let (root, _) = parse(&allocator, "{{ msg }}");
    assert!(!is_static_node(&root.children[0]));
}

#[test]
fn test_nested_dynamic_class_not_static() {
    let allocator = Allocator::new();
    let (root, _) = parse(
        &allocator,
        r#"<div class="checkbox"><span class="icon" :class="{ active: checked }" /></div>"#,
    );
    assert!(!is_static_node(&root.children[0]));
}

#[test]
fn test_sibling_with_v_if() {
    let allocator = Allocator::new();
    let (root, _) = parse(
        &allocator,
        r#"<div class="wrapper"><div class="checkbox"><span :class="{ active: checked }" /></div><label v-if="label">{{ label }}</label></div>"#,
    );

    assert!(!is_static_node(&root.children[0]));
}

#[test]
fn test_nested_static_element_is_static() {
    let allocator = Allocator::new();
    let (root, _) = parse(
        &allocator,
        r#"<div class="outer"><span class="a">x</span></div>"#,
    );
    assert!(is_static_node(&root.children[0]));
    assert_eq!(get_static_type(&root.children[0]), StaticType::FullyStatic);
}

#[test]
fn test_deeply_nested_static_element_is_static() {
    let allocator = Allocator::new();
    let (root, _) = parse(
        &allocator,
        r#"<div class="outer"><div class="inner"><span>deep</span></div></div>"#,
    );
    assert!(is_static_node(&root.children[0]));
}

#[test]
fn test_nested_with_dynamic_text_not_fully_static() {
    let allocator = Allocator::new();
    let (root, _) = parse(
        &allocator,
        r#"<div class="outer"><span>{{ msg }}</span></div>"#,
    );
    assert_eq!(get_static_type(&root.children[0]), StaticType::NotStatic);
}

fn compile_hoisted(src: &str) -> (vize_s0::String, vize_s0::String) {
    let allocator = Allocator::new();
    let (mut root, _errors) = parse(&allocator, src);
    let opts = crate::options::TransformOptions {
        hoist_static: true,
        ..crate::options::TransformOptions::default()
    };
    crate::lane::transform(&allocator, &mut root, opts, None);
    let result = crate::codegen::generate(&root, crate::options::CodegenOptions::default());
    (result.preamble, result.code)
}

#[test]
fn test_codegen_nested_static_subtree_caches_recursively() {
    let (_preamble, code) =
        compile_hoisted(r#"<div class="outer"><div class="inner"><span>deep</span></div></div>"#);
    assert!(
        code.contains(
            "_createElementVNode(\"div\", { class: \"inner\" }, [\n      _createElementVNode(\"span\", null, \"deep\")\n    ], -1 /* CACHED */)"
        ),
        "unexpected codegen:\n{code}"
    );
}

#[test]
fn test_codegen_hoisted_nested_vnode_keeps_descendant() {
    let (preamble, _code) =
        compile_hoisted(r#"<div><p v-if="ok"><span class="a"><b>x</b></span></p></div>"#);
    assert!(
        preamble.contains(
            "_createElementVNode(\"span\", { class: \"a\" }, [_createElementVNode(\"b\", null, \"x\")])"
        ),
        "nested <b> was dropped from hoisted subtree:\n{preamble}"
    );
}

#[test]
fn static_nested_predicates_accept_native_dynamic_text_subtrees() {
    let allocator = Allocator::new();
    let (root, errors) = parse(
        &allocator,
        r#"<div><span title="label">text</span>{{ value }}</div>"#,
    );
    assert!(errors.is_empty(), "unexpected parse errors: {errors:?}");
    let TemplateChildNode::Element(root_element) = &root.children[0] else {
        panic!("expected root element");
    };

    assert!(has_only_static_nested_children(root_element));
    assert!(has_only_native_element_descendants(root_element));

    let (empty_root, errors) = parse(&allocator, "<div></div>");
    assert!(errors.is_empty(), "unexpected parse errors: {errors:?}");
    let TemplateChildNode::Element(empty_element) = &empty_root.children[0] else {
        panic!("expected empty root element");
    };
    assert!(!has_only_static_nested_children(empty_element));
}

#[test]
fn constant_bind_modifiers_are_preserved_in_hoisted_props() {
    let (preamble, code) = compile_hoisted(
        r#"<div :data-id.camel="'x'" :value.prop="'y'" :role.attr="'z'">{{ message }}</div>"#,
    );

    assert!(
        preamble.contains(r#"dataId: 'x'"#),
        "camel modifier was not hoisted:\n{preamble}\n{code}"
    );
    assert!(
        preamble.contains(r#"".value": 'y'"#),
        "prop modifier was not hoisted:\n{preamble}\n{code}"
    );
    assert!(
        preamble.contains(r#""^role": 'z'"#),
        "attr modifier was not hoisted:\n{preamble}\n{code}"
    );
    assert!(
        code.contains("_hoisted_1"),
        "hoisted props were not used:\n{code}"
    );
}

#[test]
fn dynamic_component_hoisted_props_omit_is() {
    let (preamble, code) = compile_hoisted(r#"<component is="Foo" id="a">hello</component>"#);
    assert!(
        preamble.contains("const _hoisted_1 = { id: \"a\" }"),
        "hoisted props should keep id and drop is:\n{preamble}\n{code}"
    );
    assert!(
        !preamble.contains("is:"),
        "hoisted props must not retain is:\n{preamble}"
    );
    assert!(
        code.contains("_resolveDynamicComponent(\"Foo\"), _hoisted_1"),
        "dynamic component should consume the filtered hoist:\n{code}"
    );
}
