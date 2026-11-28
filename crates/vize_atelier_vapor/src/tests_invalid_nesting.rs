//! A child its parent's template string cannot hold gets a template of its
//! own and is inserted at runtime, as `@vue/compiler-vapor` 3.6.0-rc.11 emits
//! it: `innerHTML` would otherwise re-parent it (close the `<p>`, the outer
//! `<a>` or `<form>`, imply a `<tbody>`). The modules below are Vue's, in
//! Vize's template spelling (closed end tags, `true` for the root flag).

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

fn compiled(source: &str) -> vize_carton::String {
    let allocator = Allocator::new();
    let result = compile_vapor(&allocator, source, VaporCompilerOptions::default());
    assert!(
        result.error_messages.is_empty(),
        "{source}: {:?}",
        result.error_messages
    );
    vize_carton::String::from(result.code.as_str())
}

/// `import`, then templates `t0` (the child) and `t1` (the parent), then
/// a render that creates both and inserts the child.
fn split(imports: &str, child: &str, parent: &str, body: &str) -> vize_carton::String {
    vize_carton::cstr!(
        "import {{ {imports} }} from 'vue';\nconst t0 = _template({child:?})\nconst t1 = _template({parent:?}, true)\n\nexport function render(_ctx) {{\n  const n1 = t1()\n  const n0 = t0()\n{body}  return n1\n}}\n"
    )
}

#[test]
fn invalid_children_get_their_own_template() {
    let insert = "  _insert(n0, n1)\n";
    let imports = "insert as _insert, template as _template";
    for (source, expected) in [
        (
            r#"<form @submit="a"><form @submit="b"><button>x</button></form></form>"#,
            split(
                "insert as _insert, on as _on, template as _template",
                "<form><button>x</button></form>",
                "<form></form>",
                "  _on(n1, \"submit\", e => _ctx.a(e))\n  _insert(n0, n1)\n  _on(n0, \"submit\", e => _ctx.b(e))\n",
            ),
        ),
        (
            "<p><ul><li>a</li></ul></p>",
            split(imports, "<ul><li>a</li></ul>", "<p></p>", insert),
        ),
        (
            "<p>one<p>two</p></p>",
            split(imports, "<p>two</p>", "<p>one</p>", insert),
        ),
        (
            r#"<a href="/x"><a href="/y">y</a></a>"#,
            split(
                imports,
                r#"<a href="/y">y</a>"#,
                r#"<a href="/x"></a>"#,
                insert,
            ),
        ),
        (
            "<table><tr><td>a</td></tr></table>",
            split(imports, "<tr><td>a</td></tr>", "<table></table>", insert),
        ),
        (
            "<select><div>x</div></select>",
            split(imports, "<div>x</div>", "<select></select>", insert),
        ),
    ] {
        assert_eq!(compiled(source), expected, "{source}");
    }
}

#[test]
fn an_inserted_child_keeps_its_authored_position() {
    // A template-rendered sibling after it leaves a placeholder to insert
    // before; a trailing one appends.
    assert_eq!(
        compiled("<p>a<div>x</div><span>b</span><div>y</div></p>"),
        "import { insert as _insert, child as _child, next as _next, template as _template } from 'vue';\nconst t0 = _template(\"<div>x</div>\")\nconst t1 = _template(\"<div>y</div>\")\nconst t2 = _template(\"<p>a<!----><span>b</span></p>\", true)\n\nexport function render(_ctx) {\n  const n2 = t2()\n  const n3 = _next(_child(n2))\n  const n0 = t0()\n  const n1 = t1()\n  _insert(n0, n2, n3)\n  _insert(n1, n2)\n  return n2\n}\n"
    );
    // A nested split child: its own bindings run on the inserted node.
    assert_eq!(
        compiled("<div><p><div>{{ x }}</div></p><span>s</span></div>"),
        "import { insert as _insert, child as _child, txt as _txt, toDisplayString as _toDisplayString, setText as _setText, renderEffect as _renderEffect, template as _template } from 'vue';\nconst t0 = _template(\"<div> </div>\")\nconst t1 = _template(\"<div><p></p><span>s</span></div>\", true)\n\nexport function render(_ctx) {\n  const n1 = t1()\n  const n0 = _child(n1)\n  const n2 = t0()\n  const x2 = _txt(n2)\n  _insert(n2, n0)\n  _renderEffect(() => _setText(x2, _toDisplayString(_ctx.x)))\n  return n1\n}\n"
    );
}

#[test]
fn valid_nesting_stays_one_template() {
    for (source, template) in [
        (
            "<table><tbody><tr><td>a</td></tr></tbody></table>",
            "<table><tbody><tr><td>a</td></tr></tbody></table>",
        ),
        (
            "<div><p><span>x</span></p><dl><dd>y</dd></dl></div>",
            "<div><p><span>x</span></p><dl><dd>y</dd></dl></div>",
        ),
    ] {
        assert_eq!(
            compiled(source),
            vize_carton::cstr!(
                "import {{ template as _template }} from 'vue';\nconst t0 = _template({template:?}, true)\n\nexport function render(_ctx) {{\n  const n0 = t0()\n  return n0\n}}\n"
            ),
            "{source}"
        );
    }
}
