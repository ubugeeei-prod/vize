//! SVG and MathML output: templates split out of a foreign-namespace parent
//! keep that namespace, and SVG `class`/`style` bindings are normalized.

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::{Allocator, String, cstr};

fn compile(source: &str, davinci_retained_lane: bool) -> String {
    let allocator = Allocator::new();
    let result = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            davinci_retained_lane,
            ..Default::default()
        },
    );
    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    result.code
}

fn template_lines<'c>(code: &'c str, tag: &str) -> Vec<&'c str> {
    let open = cstr!("_template(\"<{tag}");
    code.lines()
        .filter(|line| line.contains(open.as_str()))
        .collect()
}

#[test]
fn svg_branch_templates_keep_the_svg_namespace() {
    let source = r#"<svg viewBox="0 0 64 64"><g v-if="ok" id="a"><circle r="1" /></g><g v-else id="b"><rect width="1" /></g></svg>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        let branches = template_lines(&code, "g");
        assert_eq!(branches.len(), 2, "{code}");
        for line in branches {
            assert!(
                line.ends_with(", true, 1)"),
                "branch without SVG namespace: {line}\n{code}"
            );
        }
        let root = template_lines(&code, "svg");
        assert_eq!(root.len(), 1, "{code}");
        assert!(root[0].ends_with(", true, 1)"), "{code}");
    }
}

#[test]
fn mathml_branch_templates_keep_the_mathml_namespace() {
    let code = compile(
        r#"<math><mi v-if="ok">x</mi><mo v-else>+</mo></math>"#,
        false,
    );
    for tag in ["mi", "mo"] {
        let lines = template_lines(&code, tag);
        assert_eq!(lines.len(), 1, "{code}");
        assert!(lines[0].ends_with(", true, 2)"), "{code}");
    }
}

#[test]
fn html_templates_keep_their_existing_shape() {
    let code = compile(
        r#"<div><span v-if="ok">a</span><p v-else>b</p></div>"#,
        false,
    );
    for tag in ["span", "p"] {
        let lines = template_lines(&code, tag);
        assert_eq!(lines.len(), 1, "{code}");
        assert!(lines[0].ends_with("\", true)"), "{code}");
    }
}

#[test]
fn svg_class_and_style_bindings_are_normalized() {
    let source = r#"<svg class="icon" :class="c" :style="s"><path :class="p" :style="{ color: k }" d="M0 0" /></svg>"#;
    for retained in [false, true] {
        let code = compile(source, retained);
        assert!(
            !code.lines().any(|line| line.contains("_setAttr(")
                && (line.contains("\"class\"") || line.contains("\"style\""))),
            "SVG class/style written as a raw attribute:\n{code}"
        );
        assert!(code.contains(r#"["icon", _ctx.c], true)"#), "{code}");
        assert!(code.contains("_ctx.p, true)"), "{code}");
        assert!(
            code.contains("_setStyle(") && code.contains("_ctx.s)"),
            "{code}"
        );
        assert!(code.contains("{ color: _ctx.k })"), "{code}");
    }
}

#[test]
fn svg_bindings_use_inherited_namespace_instead_of_a_tag_list() {
    for tag in ["a", "filter", "feGaussianBlur", "view", "switch"] {
        let source = cstr!("<svg><{tag} :class=\"c\" :style=\"s\" :title=\"t\" /></svg>");
        for retained in [false, true] {
            let code = compile(&source, retained);
            assert!(code.contains("_ctx.c, true)"), "{tag}: {code}");
            assert!(code.contains("\"title\", _ctx.t, true)"), "{tag}: {code}");
            assert!(code.contains("_setStyle("), "{tag}: {code}");
        }
    }
}

#[test]
fn html_anchor_bindings_keep_html_semantics_at_svg_integration_points() {
    for source in [
        r#"<a :class="c" :style="s" :title="t" />"#,
        r#"<svg><foreignObject><a :class="c" :style="s" :title="t" /></foreignObject></svg>"#,
        r#"<svg><desc><a :class="c" :style="s" :title="t" /></desc></svg>"#,
        r#"<svg><title><a :class="c" :style="s" :title="t" /></title></svg>"#,
    ] {
        for retained in [false, true] {
            let code = compile(source, retained);
            assert!(code.contains("_ctx.c)"), "{code}");
            assert!(!code.contains("_ctx.c, true)"), "{code}");
            assert!(code.contains("_setProp("), "{code}");
            assert!(code.contains("\"title\", _ctx.t)"), "{code}");
        }
    }
}

#[test]
fn spread_props_on_svg_elements_are_attributes() {
    let mut packets = Vec::new();
    for source in [
        r#"<svg><circle v-bind="attrs"/></svg>"#,
        r#"<svg v-bind="attrs" :viewBox="box"><path/></svg>"#,
        r#"<div><span v-bind="attrs"></span></div>"#,
    ] {
        for retained in [false, true] {
            let allocator = Allocator::new();
            let output = compile_vapor(
                &allocator,
                source,
                VaporCompilerOptions {
                    prefix_identifiers: true,
                    davinci_retained_lane: retained,
                    ..Default::default()
                },
            );
            assert_eq!(output.error_messages.len(), 0, "{source}");
            packets.push(serde_json::json!({
                "source": source,
                "retained": retained,
                "code": output.code.as_str(),
                "templates": output.templates.iter().map(|template| template.as_str()).collect::<Vec<_>>(),
                "errors": [],
            }));
        }
    }
    insta::assert_snapshot!(
        serde_json::to_string_pretty(&packets).expect("complete SVG prop packets")
    );
}
