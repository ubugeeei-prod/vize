//! A Vue component template is built as written, as Vue 3.5's compiler builds
//! it: no implied end tags, nested form/anchor/button repair, adoption agency
//! or foster parenting. Documents and `html_tree_construction` keep them.

use super::{parse, parse_with_options};
use vize_l0::{Allocator, ErrorCode, String};
use vize_relief::{TemplateChildNode, options::ParserOptions};

/// The element tree as `tag(children)`, text as its content.
fn shape(children: &[TemplateChildNode<'_>]) -> String {
    let mut out = String::default();
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                out.push_str(element.tag);
                out.push('(');
                out.push_str(&shape(&element.children));
                out.push(')');
            }
            TemplateChildNode::Text(text) => out.push_str(text.content.trim()),
            _ => {}
        }
    }
    out
}

fn component(source: &str) -> String {
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, source);
    assert!(
        errors
            .iter()
            .all(|error| error.code != ErrorCode::ExtendPoint),
        "no tree-construction recovery: {source}: {errors:?}"
    );
    shape(&root.children)
}

fn repaired(source: &str) -> String {
    let allocator = Allocator::new();
    let options = ParserOptions {
        html_tree_construction: true,
        ..ParserOptions::default()
    };
    shape(&parse_with_options(&allocator, source, options).0.children)
}

#[test]
fn component_templates_keep_authored_nesting() {
    for (source, authored) in [
        (
            "<form><input><form><input></form></form>",
            "form(input()form(input()))",
        ),
        (
            r#"<a href="/">outer<a href="/foo">inner</a></a>"#,
            "a(outera(inner))",
        ),
        (
            "<button>aaa<button>bbb</button></button>",
            "button(aaabutton(bbb))",
        ),
        ("<p><ul><li>a</li></ul></p>", "p(ul(li(a)))"),
        ("<p><div>x</div></p>", "p(div(x))"),
        ("<p>first<p>second</p></p>", "p(firstp(second))"),
        ("<ul><li>c<li>d</ul>", "ul(li(cli(d)))"),
        ("<dl><dt>a<dd>b</dl>", "dl(dt(add(b)))"),
        (
            "<select><option>a<option>b</select>",
            "select(option(aoption(b)))",
        ),
        ("<table><div>x</div></table>", "table(div(x))"),
        (
            "<p><template><p>inner</p></template>outer</p>",
            "p(template(p(inner))outer)",
        ),
    ] {
        assert_eq!(component(source).as_str(), authored, "{source}");
    }
}

#[test]
fn html_tree_construction_keeps_the_browser_repairs() {
    for (source, browser) in [
        (
            "<form><input><form><input></form></form>",
            "form(input()input())",
        ),
        (
            r#"<a href="/">outer<a href="/foo">inner</a></a>"#,
            "a(outer)a(inner)",
        ),
        ("<p>first<p>second</p>", "p(first)p(second)"),
        ("<ul><li>c<li>d</ul>", "ul(li(c)li(d))"),
        ("<table><div>x</div></table>", "div(x)table()"),
        ("<b>aaa<i>bbb</b>ccc</i>", "b(aaai(bbb))i(ccc)"),
    ] {
        assert_eq!(repaired(source).as_str(), browser, "{source}");
    }
}

#[test]
fn misnested_formatting_is_reported_like_vue() {
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, "<b>aaa<i>bbb</b>ccc</i>");

    // Vue 3.5 reports the open `<i>` and the stray `</i>`; nothing is adopted.
    let codes: std::vec::Vec<_> = errors.iter().map(|error| error.code).collect();
    assert_eq!(codes, [ErrorCode::MissingEndTag, ErrorCode::InvalidEndTag]);
    assert_eq!(shape(&root.children).as_str(), "b(aaai(bbb))ccc");
}
