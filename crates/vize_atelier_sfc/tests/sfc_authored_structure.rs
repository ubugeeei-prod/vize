//! A Vue component template keeps its authored structure, as Vue 3.5's
//! compiler builds it: a nested `form`, `a` or `button`, block content or a
//! `<p>` in a `<p>`, and content in a `<table>` compile as written, with every
//! handler. Vapor keeps the browser's repairs, because `innerHTML` reads its
//! templates (#13310). The expected SSR markup is Vue 3.5.35's `compileTemplate`
//! output for the same template, and the DOM trees are the ones Vue builds.

use vize_atelier_sfc::{
    SfcCompileOptions, SfcCompileResult, SfcError, SfcParseOptions, TemplateCompileOptions,
    compile_sfc, parse_sfc,
};

const SOURCE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/sfc-authored-structure/Layout.vue.txt"
);

fn compile(source: &str, ssr: bool, vapor: bool) -> Result<SfcCompileResult, SfcError> {
    let parse = SfcParseOptions {
        filename: "Layout.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("SFC");
    compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse,
            template: TemplateCompileOptions {
                ssr,
                vapor,
                ..Default::default()
            },
            vapor,
            ..Default::default()
        },
    )
}

fn code(source: &str, ssr: bool, vapor: bool) -> String {
    let result = compile(source, ssr, vapor).expect("compile");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    result.code.as_str().to_owned()
}

fn template(body: &str) -> String {
    format!("<template>\n  {body}\n</template>\n")
}

/// The DOM render function's element tree as `depth:tag` rows, read from its
/// indentation. A hoisted element counts where the render function uses it.
fn tree(code: &str) -> String {
    let hoisted: Vec<(&str, &str)> = code
        .lines()
        .filter_map(|line| {
            let (name, value) = line.strip_prefix("const ")?.split_once(" = ")?;
            Some((name, element_tag(value)?))
        })
        .collect();
    let body = &code[code.find("function render").expect("render function")..];
    let mut rows = Vec::new();
    for line in body.lines() {
        let text = line.trim_start();
        let indent = line.len() - text.len();
        let name = text.trim_end_matches(',');
        let tag = element_tag(text).or_else(|| {
            hoisted
                .iter()
                .find(|(hoist, _)| *hoist == name)
                .map(|(_, tag)| *tag)
        });
        if let Some(tag) = tag {
            rows.push((indent, tag));
        }
    }
    let base = rows.iter().map(|(indent, _)| *indent).min().unwrap_or(0);
    rows.iter()
        .map(|(indent, tag)| format!("{}:{tag}", (indent - base) / 2))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The tag of the element a codegen line creates.
fn element_tag(text: &str) -> Option<&str> {
    let start = ["_createElementVNode(\"", "_createElementBlock(\""]
        .iter()
        .filter_map(|call| text.find(call).map(|index| index + call.len()))
        .min()?;
    let tag = &text[start..];
    Some(&tag[..tag.find('"')?])
}

#[test]
fn nested_and_block_content_stay_authored_like_vue() {
    assert_eq!(
        code(SOURCE, true, false),
        include_str!(
            "../../../tests/_fixtures/differential/compiler/sfc-authored-structure/module.expected.txt"
        )
    );

    let dom = code(SOURCE, false, false);
    assert_eq!(
        tree(&dom),
        "0:form 1:form 2:button 0:a 1:a 0:button 1:button 0:p 1:ul 2:li 0:p 1:div 1:p 0:table 1:div",
        "{dom}"
    );
    // Both forms and both buttons keep their own handlers.
    for handler in ["_ctx.save", "_ctx.search", "_ctx.outer", "_ctx.inner"] {
        assert!(dom.contains(handler), "{handler}: {dom}");
    }
}

#[test]
fn each_shape_matches_vue_markup_and_tree() {
    for (body, markup, vue_tree) in [
        (
            r#"<form @submit="a"><form @submit="b"><button>x</button></form></form>"#,
            "<form${_ssrRenderAttrs(_attrs)}><form><button>x</button></form></form>",
            "0:form 1:form 2:button",
        ),
        (
            r#"<a href="/x"><a href="/y">y</a></a>"#,
            r#"<a${_ssrRenderAttrs(_mergeProps({ href: "/x" }, _attrs))}><a href="/y">y</a></a>"#,
            "0:a 1:a",
        ),
        (
            r#"<button @click="a"><button @click="b">x</button></button>"#,
            "<button${_ssrRenderAttrs(_attrs)}><button>x</button></button>",
            "0:button 1:button",
        ),
        (
            "<p><ul><li>a</li></ul></p>",
            "<p${_ssrRenderAttrs(_attrs)}><ul><li>a</li></ul></p>",
            "0:p 1:ul 2:li",
        ),
        (
            "<p><div>x</div></p>",
            "<p${_ssrRenderAttrs(_attrs)}><div>x</div></p>",
            "0:p 1:div",
        ),
        (
            "<p>one<p>two</p></p>",
            "<p${_ssrRenderAttrs(_attrs)}>one<p>two</p></p>",
            "0:p 1:p",
        ),
        (
            "<table><div>x</div></table>",
            "<table${_ssrRenderAttrs(_attrs)}><div>x</div></table>",
            "0:table 1:div",
        ),
        (
            "<table>text<tr><td>c</td></tr></table>",
            "<table${_ssrRenderAttrs(_attrs)}>text<tr><td>c</td></tr></table>",
            "0:table 1:tr 2:td",
        ),
        (
            "<p v-pre><div><Child>{{ x }}</Child></div></p>",
            "<p${_ssrRenderAttrs(_attrs)}><div><Child>{{ x }}</Child></div></p>",
            "0:p 1:div",
        ),
    ] {
        let source = template(body);
        let ssr = code(&source, true, false);
        assert!(ssr.contains(&format!("_push(`{markup}`)")), "{body}: {ssr}");
        let dom = code(&source, false, false);
        assert_eq!(tree(&dom), vue_tree, "{body}: {dom}");
    }
}

/// Vue builds the same tree and also reports "Element is missing end tag.";
/// Vize accepts it, and `vue/html-end-tags` reports the omission instead.
#[test]
fn omitted_optional_end_tags_nest_like_vue_without_its_error() {
    for (body, markup, vue_tree) in [
        (
            "<ul><li>a<li>b</ul>",
            "<ul${_ssrRenderAttrs(_attrs)}><li>a<li>b</li></li></ul>",
            "0:ul 1:li 2:li",
        ),
        (
            "<dl><dt>a<dd>b</dl>",
            "<dl${_ssrRenderAttrs(_attrs)}><dt>a<dd>b</dd></dt></dl>",
            "0:dl 1:dt 2:dd",
        ),
        (
            "<select><option>a<option>b</select>",
            "<select${_ssrRenderAttrs(_attrs)}><option>a<option>b</option></option></select>",
            "0:select 1:option 2:option",
        ),
        (
            "<table><tr><td>a<td>b</tr></table>",
            "<table${_ssrRenderAttrs(_attrs)}><tr><td>a<td>b</td></td></tr></table>",
            "0:table 1:tr 2:td 3:td",
        ),
        (
            "<div><p>a<div>b</div></div>",
            "<div${_ssrRenderAttrs(_attrs)}><p>a<div>b</div></p></div>",
            "0:div 1:p 2:div",
        ),
    ] {
        let source = template(body);
        let ssr = code(&source, true, false);
        assert!(ssr.contains(&format!("_push(`{markup}`)")), "{body}: {ssr}");
        let dom = code(&source, false, false);
        assert_eq!(tree(&dom), vue_tree, "{body}: {dom}");
    }
}

#[test]
fn misnested_formatting_end_tags_fail_with_vues_errors() {
    let source = template("<b><i>x</b>y</i>");
    for ssr in [false, true] {
        let error = compile(&source, ssr, false).expect_err("Vue rejects it too");
        assert!(error.message.contains("MissingEndTag"), "{error:?}");
        assert!(error.message.contains("InvalidEndTag"), "{error:?}");
        assert!(!error.message.contains("adoption agency"), "{error:?}");
    }
}

/// Vapor still applies the browser's repairs, as before; Vue 3.6 Vapor splits
/// each invalid nesting into its own template instead (#13310).
#[test]
fn vapor_keeps_the_browsers_repairs() {
    let vapor = |body: &str| {
        let result = compile(&template(body), false, true).expect("compile");
        result.code.as_str().to_owned()
    };
    let anchors = vapor(r#"<a href="/x"><a href="/y">y</a></a>"#);
    assert!(
        anchors.contains(r#"_template("<a href=\"/x\"></a>")"#),
        "{anchors}"
    );
    assert!(
        anchors.contains(r#"_template("<a href=\"/y\">y</a>")"#),
        "{anchors}"
    );
    let items = vapor("<ul><li>a<li>b</ul>");
    assert!(items.contains("<ul><li>a</li><li>b</li></ul>"), "{items}");
    for body in [
        r#"<form @submit="a"><form @submit="b"><button>x</button></form></form>"#,
        "<p><ul><li>a</li></ul></p>",
        "<p>one<p>two</p></p>",
    ] {
        assert!(compile(&template(body), false, true).is_err(), "{body}");
    }
    assert!(compile(SOURCE, false, true).is_err(), "the whole case");
}
