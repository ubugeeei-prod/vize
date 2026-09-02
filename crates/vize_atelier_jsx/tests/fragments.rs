//! Lowering of JSX fragments (`<>...</>`).

mod common;

use common::{as_element, as_text, lower_all, lower_one, root_element, vapor_code, vdom_code};
use vize_atelier_jsx::JsxLang;
use vize_s0::Allocator;

#[test]
fn top_level_fragment_lifts_children_to_root() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <><span/><b/></>;");
    assert_eq!(root.children.len(), 2);
    assert_eq!(as_element(&root.children[0]).tag, "span");
    assert_eq!(as_element(&root.children[1]).tag, "b");
}

#[test]
fn fragment_with_text_and_elements() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <>hi<b/></>;");
    assert_eq!(root.children.len(), 2);
    assert_eq!(as_text(&root.children[0]).content, "hi");
    assert_eq!(as_element(&root.children[1]).tag, "b");
}

/// A nested fragment used to become an element tagged `Fragment`, which the DOM
/// backend emitted as `resolveComponent("Fragment")` — a component nobody
/// registers, so the subtree rendered as nothing. A fragment in child position
/// carries no props and cannot be keyed, so its children are spliced into the
/// parent instead (#3421).
#[test]
fn nested_fragment_children_are_spliced_into_the_parent() {
    let bump = Allocator::new();
    let out = lower_all(&bump, "const a = <div>{lead}<><p/><i/></>{tail}</div>;");
    let div = root_element(&out.roots[0].root);
    assert_eq!(div.children.len(), 4);
    assert_eq!(as_element(&div.children[1]).tag, "p");
    assert_eq!(as_element(&div.children[2]).tag, "i");
    assert_eq!(
        out.diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.is_error(), diagnostic.message.as_str()))
            .collect::<std::vec::Vec<_>>(),
        vec![]
    );
}

/// Deep nesting collapses all the way: no chain of unresolvable wrappers is
/// left behind.
#[test]
fn doubly_nested_fragments_collapse() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <div><><><i/></></></div>;");
    let div = root_element(&root);
    assert_eq!(div.children.len(), 1);
    assert_eq!(as_element(&div.children[0]).tag, "i");
}

/// The whole emitted module. `@vue/babel-plugin-jsx` renders this as
/// `_createVNode("div", null, [_createVNode(_Fragment, null, […])])` — the same
/// DOM, one vnode level deeper.
#[test]
fn nested_fragment_emits_the_children_inline() {
    assert_eq!(
        vdom_code("const A = () => <div><><i/><b/></></div>;", JsxLang::Jsx).as_str(),
        "export function render(_ctx, _cache) {\n  \
         return (_openBlock(), _createElementBlock(\"div\", null, [\n    \
         _createElementVNode(\"i\"),\n    \
         _createElementVNode(\"b\")\n  \
         ]))\n}"
    );
}

/// Vapor took the same `resolveComponent("Fragment")` route, so the whole
/// generated module is pinned there too.
#[test]
fn nested_fragment_emits_the_children_inline_under_vapor() {
    assert_eq!(
        vapor_code(
            "const A = () => <div><><i/><b/></></div>;",
            JsxLang::Jsx,
            false
        )
        .as_str(),
        "import { template as _template } from 'vue';\n\
         const t0 = _template(\"<div><i></i><b></b></div>\", true)\n\n\
         export function render(_ctx) {\n  \
         const n0 = t0()\n  \
         return n0\n\
         }\n"
    );
}

/// A fragment that has to stay one node — a `v-if` branch — keeps a real
/// `Fragment` block instead of being spliced, because the branch's children
/// have no parent child list to splice into.
#[test]
fn fragment_in_a_conditional_branch_becomes_a_fragment_block() {
    assert_eq!(
        vdom_code(
            "const A = () => <div>{cond ? <><i/><b/></> : <c/>}</div>;",
            JsxLang::Jsx
        )
        .as_str(),
        "export function render(_ctx, _cache) {\n  \
         return (_openBlock(), _createElementBlock(\"div\", null, [\n    \
         (cond)\n      \
         ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [\n        \
         _createElementVNode(\"i\"),\n        \
         _createElementVNode(\"b\")\n      \
         ], 64 /* STABLE_FRAGMENT */))\n      \
         : (_openBlock(), _createElementBlock(\"c\", { key: 1 }))\n  \
         ]))\n}"
    );
}

#[test]
fn empty_fragment_has_no_children() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <></>;");
    assert_eq!(root.children.len(), 0);
}
