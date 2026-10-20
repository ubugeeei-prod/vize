//! Implied end tags: L1 element nesting against the shipped parser.
//!
//! The standard lane follows HTML "in body" and "in table" start-tag closes;
//! the quirks lane keeps Vue compiler-core's authored nesting, which L1's
//! authored projection reproduces. Legacy `tbody`/`tr` owners with no source
//! bytes are inserted by L1->L2 lowering, so the comparison skips them.

use vize_armature::{
    ParserOptions, TemplateChildNode, TemplateSyntaxMode, parse_with_options_and_template_syntax,
};
use vize_l0::{Allocator, is_native_tag};
use vize_l1::{SurfaceChild, SurfaceTree, check_fidelity, parse_with_authored};

type Rows<'a> = Vec<(&'a str, usize)>;

fn surface<'a>(tree: &SurfaceTree<'a>) -> Rows<'a> {
    fn walk<'a>(children: &[SurfaceChild<'a>], depth: usize, rows: &mut Rows<'a>) {
        for child in children {
            if let SurfaceChild::Element(element) = child {
                rows.push((element.tag(), depth));
                walk(&element.children, depth + 1, rows);
            }
        }
    }
    let mut rows = Vec::new();
    walk(&tree.children, 0, &mut rows);
    rows
}

fn legacy<'a>(source: &'a str, allocator: &'a Allocator, syntax: TemplateSyntaxMode) -> Rows<'a> {
    fn walk<'a>(children: &[TemplateChildNode<'a>], depth: usize, rows: &mut Rows<'a>) {
        for child in children {
            if let TemplateChildNode::Element(element) = child {
                if element.loc.span.start == element.loc.span.end {
                    walk(&element.children, depth, rows);
                } else {
                    rows.push((element.tag, depth));
                    walk(&element.children, depth + 1, rows);
                }
            }
        }
    }
    let options = ParserOptions {
        is_native_tag: Some(is_native_tag),
        ..ParserOptions::default()
    };
    let (root, _) = parse_with_options_and_template_syntax(allocator, source, options, syntax);
    let mut rows = Vec::new();
    walk(&root.children, 0, &mut rows);
    rows
}

/// Both lanes agree on every source, normal and authored projections alike.
fn agrees(source: &str) {
    let allocator = Allocator::new();
    let (normal, authored, _) = parse_with_authored(&allocator, source);
    assert_eq!(check_fidelity(&normal), Ok(()), "{source}");
    let standard = legacy(source, &allocator, TemplateSyntaxMode::Standard);
    assert_eq!(surface(&normal), standard, "standard: {source}");
    let quirks = legacy(source, &allocator, TemplateSyntaxMode::Quirks);
    let authored = authored.as_ref().unwrap_or(&normal);
    assert_eq!(check_fidelity(authored), Ok(()), "{source}");
    assert_eq!(surface(authored), quirks, "quirks: {source}");
}

#[test]
fn in_body_optional_end_tags_match_both_lanes() {
    for source in [
        "<ul><li>a<li>b</li></ul>",
        "<ul><li>a<li>b</ul>",
        "<li>a<li>b",
        "<ul><li>a<li/>b</ul>",
        "<dl><dt>a<dd>b<dt>c</dl>",
        "<dl><dt>a<div><dd>b</div></dl>",
        "<dl><dt>a<Comp><dd>b</Comp></dl>",
        "<dl><dt>a<dl><dt>b</dl><dd>c</dl>",
        "<select><option>a<option>b</select>",
        "<select><optgroup><option>a<optgroup><option>b</select>",
        "<select><optgroup>a<optgroup>b</select>",
        "<select><option>a<optgroup><option>b</optgroup></select>",
        "<Select><Option>a<option>b</Select>",
        "<select><Option>a<optgroup>b</select>",
        "<option>a<option>b",
        "<option>a<optgroup>b",
        "<p>a<p>b",
        "<p>a<div>b</div>",
        "<p>a<div>b</div></p>",
        "<p>a<div/>b",
        "<p><span>a<div>b</div></span></p>",
        "<p>a<h1>b</h1>",
        "<p>a<pre>b</pre>",
        "<p>a<hr>b",
        "<p>a<table><tr><td>x</td></tr></table>",
        "<div><p>a<ul><li>b</li></ul></div>",
        "<p v-pre><div>x</div></p>",
        "<li><div><li>x</li></div></li>",
        "<li><span><li>x</li></span></li>",
        "<ul><li>a<ol><li>b</ol><li>c</ul>",
        "<ul><li>a<Comp/><li>b</ul>",
        "<ruby>a<rt>b<rt>c</ruby>",
    ] {
        agrees(source);
    }
}

#[test]
fn scope_boundaries_and_non_elements_stop_the_search() {
    for source in [
        "<ul><li><ul><li>x</li></ul></li></ul>",
        "<li><Comp><li>x</li></Comp></li>",
        "<li>a<slot><li>b</slot>",
        "<li>a<component :is=\"x\"><li>b</component>",
        "<ul><li>a<template #x><li>b</template></ul>",
        "<p><Comp><div>x</div></Comp></p>",
        "<p><template v-if=\"x\"><div>x</div></template></p>",
        "<p><button><div>x</div></button></p>",
        "<p>a<button>b<div>c</div></button></p>",
        "<div><table><tr><td><p>a<div>b</div></td></tr></table></div>",
        // Components keep their authored spelling; only lowercase HTML closes.
        "<p>a<P>b",
        "<LI>a<li>b",
        "<p>a<Div>b</Div>",
        "<p>a<DIV>b</DIV>",
        // Foreign content is not HTML until an integration point.
        "<svg><p>x<p>y</svg>",
        "<svg><li>a<li>b</svg>",
        "<ul><li>a<svg><li>b</svg></ul>",
        "<ul><li>a<svg><foreignObject><li>b</foreignObject></svg></ul>",
        "<math><mi><p>a<p>b</mi></math>",
        "<a><a>x</a></a>",
    ] {
        agrees(source);
    }
}

#[test]
fn table_rows_and_row_groups_close_their_open_sibling() {
    for source in [
        "<table><tr><td>a</td></tr></table>",
        "<table><tr>a<tr>b</table>",
        "<table><tbody><tr>a<tr>b</tbody></table>",
        "<table><tbody><tr>a</tbody><tbody><tr>b<tr>c</table>",
        "<table><thead><tr><th>a</th></tr><tbody><tr><td>b</td></tr></table>",
        "<table><thead><tr>a<tbody><tr>b</table>",
        "<table><thead><tr><th>a<tbody><tr><td>b</table>",
        "<table><colgroup><col></colgroup><tr><td>a</td></tr></table>",
        "<table><colgroup><tr>a<tr>b</table>",
        "<table><tbody><colgroup><tr>a<tr>b</table>",
        "<table><tbody><tr><thead>x</table>",
        "<table><tbody><tr><td>a<td>b</tr></tbody></table>",
        "<table><tr><td>a<td>b</tr></table>",
        "<table><tbody><tr><td>a<tr><td>b</tbody></table>",
        "<table><tbody><tr><td><tr>x</table>",
        "<table><td>a</td></table>",
        "<table><caption>c<tr><td>a</td></tr></table>",
        "<table><template v-for=\"r in rs\"><tr>a<tr>b</template></table>",
        "<table><tr v-for=\"r in rs\"><td v-for=\"c in r\">{{ c }}</td></tr></table>",
        "<table><tbody><tr><td><table><tr>a<tr>b</table></td></tr></tbody></table>",
    ] {
        agrees(source);
    }
}

/// Shapes L1 cannot or does not yet take from the standard lane. Each row is
/// the L1 surface; the quirks lane still matches the authored projection.
#[test]
fn standard_lane_divergences_stay_explicit() {
    for (source, expected) in [
        // Foster parenting moves nodes before the table; L1 keeps source order.
        (
            "<table><div>x</div><tr><td>a</td></tr></table>",
            &[("table", 0), ("div", 1), ("tr", 1), ("td", 2)][..],
        ),
        // A nested `form` start tag is ignored by the standard lane.
        ("<form><form>x</form></form>", &[("form", 0), ("form", 1)]),
        // A self-closing nested interactive element keeps its scope.
        ("<a>x<a/>y</a>", &[("a", 0), ("a", 1)]),
        // L1 cannot tell a plain table `template` from a structural one.
        (
            "<table><template><tr>a<tr>b</template></table>",
            &[("table", 0), ("template", 1), ("tr", 2), ("tr", 3)],
        ),
    ] {
        let allocator = Allocator::new();
        let (normal, authored, _) = parse_with_authored(&allocator, source);
        assert_eq!(check_fidelity(&normal), Ok(()), "{source}");
        assert_eq!(surface(&normal), expected, "{source}");
        assert_ne!(
            legacy(source, &allocator, TemplateSyntaxMode::Standard),
            expected,
            "now agrees, move to the agreeing corpus: {source}"
        );
        let authored = authored.as_ref().unwrap_or(&normal);
        assert_eq!(
            surface(authored),
            legacy(source, &allocator, TemplateSyntaxMode::Quirks),
            "{source}"
        );
    }
}
