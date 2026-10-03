use alloc::vec::Vec;
use vize_l0::{Allocator, SourceRoot, String};

use super::{DocumentHtmlRefusal as Refusal, NativeDocument};
use crate::markup::document::{DocumentLexicalRefusal, DocumentTreePolicy};

const CORPUS: &str = include_str!("../../../../tests/fixtures/document/html-structure.tsv");

fn root(source: &str) -> SourceRoot<'_> {
    SourceRoot::new(source).expect("original small document")
}

fn source(body: &str) -> String {
    let mut source = String::from("<!DOCTYPE html><html><head></head><body>");
    source.push_str(body);
    source.push_str("</body></html>");
    source
}

#[test]
fn original_element_ancestry_matches_the_real_browser_census() {
    let mut count = 0;
    for row in CORPUS.lines().filter(|line| !line.starts_with('#')) {
        let columns: Vec<_> = row.split('\t').collect();
        assert_eq!(columns.len(), 3);
        let allocator = Allocator::default();
        let owner = NativeDocument::lex_in(&allocator, root(columns[1]));
        let tree = owner.html_structure().expect(columns[0]);
        let elements: Vec<_> = tree.elements().collect();
        let actual: Vec<_> = elements
            .iter()
            .map(|element| {
                assert!(core::ptr::eq(element.owner(), &owner));
                let parent = element.parent().map(|parent| {
                    elements
                        .iter()
                        .position(|node| node.opening_span() == parent.opening_span())
                        .expect("original parent")
                });
                (element.name(), parent)
            })
            .collect();
        let expected: Vec<_> = columns[2]
            .split(',')
            .map(|entry| {
                let (name, parent) = entry.split_once(':').expect("real browser census entry");
                (
                    name,
                    (parent != "-")
                        .then(|| parent.parse::<usize>().expect("original parent order")),
                )
            })
            .collect();
        assert_eq!(actual, expected, "{}", columns[0]);
        assert_eq!(owner.unfinished_tree_policies().len(), 4);
        count += 1;
    }
    assert_eq!(count, 8);
}

#[test]
fn nonvoid_slash_really_keeps_the_original_html_parent_open() {
    let source = source("<DIV/><SPAN></span></dIv   ><br/><span></span>");
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, root(&source));
    let tree = owner.html_structure().expect("explicit complete HTML");
    let elements: Vec<_> = tree.elements().collect();
    let div = &elements[3];
    assert_eq!(div.name(), "div");
    assert_eq!(div.authored_name(), "DIV");
    assert_eq!(div.opening(), "<DIV/>");
    assert!(div.ignored_self_closing_slash());
    let close = div.closing_span().expect("original matching close");
    assert_eq!(close.slice(&source), "</dIv   >");
    assert_eq!(
        div.children().map(|child| child.name()).collect::<Vec<_>>(),
        ["span"]
    );
    assert_eq!(
        elements[4]
            .parent()
            .expect("actual div parent")
            .opening_span(),
        div.opening_span()
    );
    assert_eq!(elements[5].name(), "br");
    assert!(!elements[5].ignored_self_closing_slash());
    assert_eq!(elements[5].closing_span(), None);
    assert_eq!(
        elements[5].parent().expect("void sibling parent").name(),
        "body"
    );
}

#[test]
fn moving_the_original_owner_keeps_source_and_arena_custody() {
    let source = source("<div><span></span></div>");
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, root(&source));
    let moved = owner;
    let tree = moved.html_structure().expect("original moved owner");
    assert!(core::ptr::eq(tree.owner(), &moved));
    assert!(core::ptr::eq(tree.owner().allocator, &allocator));
    let div = tree.elements().nth(3).expect("original div");
    assert!(core::ptr::eq(div.owner(), &moved));
    let expected = source.find("<div>").expect("authored div");
    assert!(core::ptr::eq(
        div.opening().as_ptr(),
        source
            .get(expected..)
            .expect("original div suffix")
            .as_ptr()
    ));
}

#[test]
fn implicit_envelopes_and_text_insertion_modes_refuse_completion() {
    for source in [
        "<DIV/><SPAN></SPAN></DIV>",
        "<!DOCTYPE html><html><body></body></html>",
        "&#32;<!DOCTYPE html><html><head></head><body></body></html>",
        "<!DOCTYPE html><html><head>&#32;</head><body></body></html>",
        "<!DOCTYPE html><html><head>nonspace</head><body></body></html>",
        "<!DOCTYPE html><html><head></head><body></body>tail</html>",
        "<!DOCTYPE html><html><head></head><body></body></html>\u{a0}",
    ] {
        let allocator = Allocator::default();
        let owner = NativeDocument::lex_in(&allocator, root(source));
        assert_eq!(
            owner.html_structure().unwrap_err(),
            Refusal::ExplicitEnvelope,
            "{source}"
        );
    }
}

#[test]
fn vue_flavored_callbacks_cannot_hide_real_html_structure() {
    for (body, refusal) in [
        ("{{ '<span>' }}", Refusal::VueSyntax),
        (
            "<div :[a>b]=x></div>",
            Refusal::Lexical(DocumentLexicalRefusal::RecoveredSyntax),
        ),
        ("<div v-scope=state></div>", Refusal::VueSyntax),
    ] {
        let source = source(body);
        let allocator = Allocator::default();
        let owner = NativeDocument::lex_in(&allocator, root(&source));
        assert_eq!(owner.html_structure().unwrap_err(), refusal, "{body}");
    }
}

#[test]
fn unimplemented_tree_policies_and_foreign_raw_formatting_are_explicit() {
    for (body, policy) in [
        ("<table></table>", DocumentTreePolicy::TableContentModel),
        ("<p></p>", DocumentTreePolicy::ImpliedEndTags),
    ] {
        let source = source(body);
        let allocator = Allocator::default();
        let owner = NativeDocument::lex_in(&allocator, root(&source));
        assert_eq!(
            owner.html_structure().unwrap_err(),
            Refusal::UnsupportedPolicy(policy)
        );
    }
    for body in [
        "<svg></svg>",
        "<math></math>",
        "<script></script>",
        "<SCRIPT></SCRIPT>",
        "<b></b>",
        "<template></template>",
    ] {
        let source = source(body);
        let allocator = Allocator::default();
        let owner = NativeDocument::lex_in(&allocator, root(&source));
        assert!(
            matches!(owner.html_structure(), Err(Refusal::UnsupportedElement(_))),
            "{body}"
        );
    }
}

#[test]
fn original_mismatched_ends_and_non_html_close_frames_cannot_mint_a_tree() {
    for body in ["<div><span></div></span>", "<div/>", "<br></br>"] {
        let source = source(body);
        let allocator = Allocator::default();
        let owner = NativeDocument::lex_in(&allocator, root(&source));
        assert!(
            matches!(owner.html_structure(), Err(Refusal::ImpliedEnd(_))),
            "{body}"
        );
    }
    for body in ["<div></ div>", "<div/ ></div>"] {
        let source = source(body);
        let allocator = Allocator::default();
        let owner = NativeDocument::lex_in(&allocator, root(&source));
        assert!(
            matches!(owner.html_structure(), Err(Refusal::InvalidFrame(_))),
            "{body}"
        );
    }
}

#[test]
fn lexical_recovery_and_pending_structure_never_become_html_completion() {
    for source in ["<!>", "<div a=\"", "{{"] {
        let allocator = Allocator::default();
        let owner = NativeDocument::lex_in(&allocator, root(source));
        assert!(
            matches!(owner.html_structure(), Err(Refusal::Lexical(_))),
            "{source}"
        );
    }
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(
        &allocator,
        root("<!DOCTYPE html><html><head></head><body><div>"),
    );
    assert_eq!(
        owner.html_structure().unwrap_err(),
        Refusal::PendingStructure
    );
    assert!(!matches!(
        owner.html_structure(),
        Err(Refusal::Lexical(DocumentLexicalRefusal::PendingSyntax))
    ));
}

#[test]
fn the_original_pinned_petite_vue_document_still_reports_unsupported_structure() {
    let source = include_str!("../../../../tests/fixtures/document/petite-vue-svg.html");
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, root(source));
    assert!(owner.normal_completion().is_ok());
    assert!(matches!(
        owner.html_structure(),
        Err(Refusal::UnsupportedElement(_))
    ));
    assert_eq!(owner.unfinished_tree_policies().len(), 4);
}
