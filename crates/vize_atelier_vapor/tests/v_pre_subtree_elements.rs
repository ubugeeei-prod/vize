//! #13314: a v-pre subtree compiles to one static Vapor template. Like
//! `@vue/compiler-vapor` 3.6.0-beta.10, nothing in it creates a component; a
//! literal `<slot>` stays markup where that compiler also emits `createSlot`.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "whole Vapor modules are compared as std strings"
)]

use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_l0::Allocator;

fn compile(source: &str) -> String {
    let allocator = Allocator::new();
    let result = compile_vapor(&allocator, source, VaporCompilerOptions::default());
    assert_eq!(result.error_messages, Vec::<String>::new(), "{source}");
    result.code.to_string()
}

fn static_module(markup: &str) -> String {
    let markup = markup.replace('"', "\\\"");
    format!(
        "import {{ template as _template }} from 'vue';\n\
const t0 = _template(\"{markup}\", true)\n\n\
export function render(_ctx) {{\n  const n0 = t0()\n  return n0\n}}\n"
    )
}

#[test]
fn a_v_pre_subtree_is_one_static_template() {
    for (source, markup) in [
        (
            "<div v-pre><div><Child>{{ x }}</Child></div></div>",
            "<div><div><Child>{{ x }}</Child></div></div>",
        ),
        ("<Child v-pre>{{ x }}</Child>", "<Child>{{ x }}</Child>"),
        (
            r#"<div v-pre><component is="x"/></div>"#,
            r#"<div><component is="x"></component></div>"#,
        ),
        (
            "<div v-pre><my-comp/></div>",
            "<div><my-comp></my-comp></div>",
        ),
        ("<div v-pre><slot/></div>", "<div><slot></slot></div>"),
        (
            r#"<div v-pre><template v-if="a">x</template></div>"#,
            r#"<div><template v-if="a">x</template></div>"#,
        ),
        (
            r#"<component v-pre is="x"/>"#,
            r#"<component is="x"></component>"#,
        ),
        ("<slot v-pre/>", "<slot></slot>"),
        (
            r#"<MyComp v-pre :x="1">c</MyComp>"#,
            r#"<MyComp :x="1">c</MyComp>"#,
        ),
    ] {
        assert_eq!(compile(source), static_module(markup), "{source}");
    }
}

#[test]
fn a_component_outside_v_pre_is_still_created() {
    assert_eq!(
        compile("<div><Child/></div>"),
        "import { resolveComponent as _resolveComponent, createComponentWithFallback as _createComponentWithFallback, setInsertionState as _setInsertionState, template as _template } from 'vue';\n\
const t0 = _template(\"<div></div>\", true)\n\n\
export function render(_ctx) {\n  const n1 = t0()\n  const _component_Child = _resolveComponent(\"Child\")\n  _setInsertionState(n1)\n  const n0 = _createComponentWithFallback(_component_Child, null, null, false)\n  return n1\n}\n"
    );
}
