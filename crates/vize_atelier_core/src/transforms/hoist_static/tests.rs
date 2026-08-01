use super::{StaticType, get_static_type, is_static_node};
use crate::parser::parse;
use bumpalo::Bump;

#[test]
fn test_static_text() {
    let allocator = Bump::new();
    let (root, _) = parse(&allocator, "hello");
    assert!(is_static_node(&root.children[0]));
}

#[test]
fn test_static_element() {
    let allocator = Bump::new();
    let (root, _) = parse(&allocator, "<div>static</div>");
    assert!(is_static_node(&root.children[0]));
}

#[test]
fn test_dynamic_element() {
    let allocator = Bump::new();
    let (root, _) = parse(&allocator, "<div :class=\"cls\">dynamic</div>");
    assert!(!is_static_node(&root.children[0]));
}

#[test]
fn test_interpolation_not_static() {
    let allocator = Bump::new();
    let (root, _) = parse(&allocator, "{{ msg }}");
    assert!(!is_static_node(&root.children[0]));
}

#[test]
fn test_nested_dynamic_class_not_static() {
    let allocator = Bump::new();
    let (root, _) = parse(
        &allocator,
        r#"<div class="checkbox"><span class="icon" :class="{ active: checked }" /></div>"#,
    );
    assert!(!is_static_node(&root.children[0]));
}

#[test]
fn test_sibling_with_v_if() {
    let allocator = Bump::new();
    let (root, _) = parse(
        &allocator,
        r#"<div class="wrapper"><div class="checkbox"><span :class="{ active: checked }" /></div><label v-if="label">{{ label }}</label></div>"#,
    );

    assert!(!is_static_node(&root.children[0]));
}

#[test]
fn test_nested_static_element_is_static() {
    let allocator = Bump::new();
    let (root, _) = parse(
        &allocator,
        r#"<div class="outer"><span class="a">x</span></div>"#,
    );
    assert!(is_static_node(&root.children[0]));
    assert_eq!(get_static_type(&root.children[0]), StaticType::FullyStatic);
}

#[test]
fn test_deeply_nested_static_element_is_static() {
    let allocator = Bump::new();
    let (root, _) = parse(
        &allocator,
        r#"<div class="outer"><div class="inner"><span>deep</span></div></div>"#,
    );
    assert!(is_static_node(&root.children[0]));
}

#[test]
fn test_nested_with_dynamic_text_not_fully_static() {
    let allocator = Bump::new();
    let (root, _) = parse(
        &allocator,
        r#"<div class="outer"><span>{{ msg }}</span></div>"#,
    );
    assert_eq!(get_static_type(&root.children[0]), StaticType::NotStatic);
}

fn compile_hoisted(src: &str) -> (vize_carton::String, vize_carton::String) {
    let allocator = Bump::new();
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
