//! Each host rule under both profiles: HTML parsing for Document, authored
//! Vue template nesting for Component, over the same original event tape.

use alloc::vec::Vec;
use vize_l0::{Allocator, SourceRoot, Span, String};

use super::{Refusal, construct_as};
use crate::markup::document::{DocumentHtmlNode, NativeDocument};
use crate::markup::{Component, Document, Profile};

type Shape = Vec<(&'static str, Option<usize>, bool)>;

fn envelope(body: &str) -> String {
    let mut source = String::from("<!DOCTYPE html><html><head></head><body>");
    source.push_str(body);
    source.push_str("</body></html>");
    source
}

/// Body descendants as `(name, parent, authored close)` in start-tag order.
fn shape<P: Profile>(body: &str) -> Result<Shape, Refusal> {
    let source = envelope(body);
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).expect("small source"));
    let tree = construct_as::<P>(&owner)?;
    let elements: Vec<_> = tree.elements().collect();
    Ok(elements
        .iter()
        .skip(3)
        .map(|element| {
            let parent = element.parent().and_then(|parent| {
                elements
                    .iter()
                    .position(|node| node.opening_span() == parent.opening_span())
            });
            (element.name(), parent, element.closing_span().is_some())
        })
        .collect())
}

#[test]
fn a_slash_closes_any_element_only_under_the_component_profile() {
    let body = "<div/><span></span>";
    assert_eq!(
        shape::<Component>(body).expect(body),
        [("div", Some(2), false), ("span", Some(2), true)]
    );
    assert_eq!(
        shape::<Document>(body),
        Err(Refusal::ImpliedEnd(Span::new(61, 65)))
    );
    let body = "<div/><span></span></div>";
    assert_eq!(
        shape::<Document>(body).expect(body),
        [("div", Some(2), true), ("span", Some(3), true)]
    );
    assert_eq!(
        shape::<Component>(body),
        Err(Refusal::ImpliedEnd(Span::new(61, 64)))
    );
}

#[test]
fn implied_end_tags_close_html_parents_only_under_the_document_profile() {
    let body = "<p>a<div>b</div><ul><li>c<li>d</ul><dl><dt>e<dd>f</dl>\
                <ruby>g<rt>h<rt>i</ruby><option>j<option>k";
    assert_eq!(
        shape::<Document>(body).expect(body),
        [
            ("p", Some(2), false),
            ("div", Some(2), true),
            ("ul", Some(2), true),
            ("li", Some(5), false),
            ("li", Some(5), false),
            ("dl", Some(2), true),
            ("dt", Some(8), false),
            ("dd", Some(8), false),
            ("ruby", Some(2), true),
            ("rt", Some(11), false),
            ("rt", Some(11), false),
            ("option", Some(2), false),
            ("option", Some(2), false),
        ]
    );
    assert!(matches!(
        shape::<Component>(body),
        Err(Refusal::ImpliedEnd(_))
    ));
    for body in ["<p><div></div></p>", "<ul><li><li></li></li></ul>"] {
        assert!(
            matches!(shape::<Document>(body), Err(Refusal::ImpliedEnd(_))),
            "{body}"
        );
        assert!(
            shape::<Component>(body)
                .expect(body)
                .iter()
                .all(|&(_, _, closed)| closed),
            "{body}"
        );
    }
    assert_eq!(
        shape::<Component>("<p><div></div></p>").expect("nested"),
        [("p", Some(2), true), ("div", Some(3), true)]
    );
}

#[test]
fn the_table_content_model_applies_only_under_the_document_profile() {
    let body = "<table><caption>c<colgroup><col><tbody><tr><td>a<td>b<tr><th>c\
                <tbody><tr><td><p>d</table>";
    assert_eq!(
        shape::<Document>(body).expect(body),
        [
            ("table", Some(2), true),
            ("caption", Some(3), false),
            ("colgroup", Some(3), false),
            ("col", Some(5), false),
            ("tbody", Some(3), false),
            ("tr", Some(7), false),
            ("td", Some(8), false),
            ("td", Some(8), false),
            ("tr", Some(7), false),
            ("th", Some(11), false),
            ("tbody", Some(3), false),
            ("tr", Some(13), false),
            ("td", Some(14), false),
            ("p", Some(15), false),
        ]
    );
    assert!(matches!(
        shape::<Component>(body),
        Err(Refusal::ImpliedEnd(_))
    ));
    for (body, refusal) in [
        (
            "<table><tr><td></td></tr></table>",
            Refusal::ImpliedStart(Span::new(48, 50)),
        ),
        (
            "<table><div></div></table>",
            Refusal::FosterParenting(Span::new(48, 51)),
        ),
        ("<tr></tr>", Refusal::IgnoredTag(Span::new(41, 43))),
    ] {
        assert_eq!(shape::<Document>(body), Err(refusal), "{body}");
        let authored = shape::<Component>(body).expect(body);
        assert!(authored.iter().all(|&(_, _, closed)| closed), "{body}");
    }
}

#[test]
fn html_names_fold_case_only_under_the_document_profile() {
    assert_eq!(
        shape::<Document>("<DIV></div>").expect("folded"),
        [("div", Some(2), true)]
    );
    assert_eq!(
        shape::<Component>("<DIV></div>"),
        Err(Refusal::UnsupportedElement(Span::new(41, 44)))
    );
}

#[test]
fn whitespace_after_body_joins_the_open_implied_end_element() {
    let source = "<!DOCTYPE html><html><head></head><body><p>a</body>\n</html>";
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(source).expect("small"));
    let tree = owner.html_structure().expect("implied p end");
    let elements: Vec<_> = tree.elements().collect();
    let [_, _, body, p] = elements.as_slice() else {
        panic!("four elements");
    };
    assert_eq!(p.closing_span(), None);
    let children: Vec<_> = body
        .child_nodes()
        .expect("body keeps its element child")
        .map(|node| match node.expect("original node") {
            DocumentHtmlNode::Element(element) => element.name(),
            _ => "node",
        })
        .collect();
    assert_eq!(children, ["p"]);
    assert_eq!(
        p.child_nodes().unwrap_err(),
        Refusal::OutsideBodyText(Span::new(51, 52))
    );
}
