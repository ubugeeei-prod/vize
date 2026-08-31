use vize_armature::{Allocator, CompilerError, ErrorCode, TemplateChildNode, parse};

const NESTED_ANCHOR_RECOVERY: &str =
    "Nested anchor start tag closed the previous anchor before inserting the new one.";
const NESTED_BUTTON_RECOVERY: &str =
    "Nested button start tag closed the previous button before inserting the new one.";
const IGNORED_END_TAG_RECOVERY: &str = "HTML tree construction ignored this end tag because the element was already closed before a nested start tag.";

#[test]
fn nested_anchor_and_button_are_split_without_hard_errors() {
    let allocator = Allocator::new();
    let (anchor_root, anchor_errors) = parse(
        &allocator,
        r#"<a href="/">outer<a href="/foo">inner</a></a>"#,
    );

    assert!(
        anchor_errors
            .iter()
            .any(|e| e.code == ErrorCode::ExtendPoint && e.message == NESTED_ANCHOR_RECOVERY)
    );
    assert!(
        anchor_errors
            .iter()
            .any(|e| e.code == ErrorCode::ExtendPoint && e.message == IGNORED_END_TAG_RECOVERY),
        "the outer </a> is redundant fallout from the nested-anchor recovery: {anchor_errors:?}"
    );
    assert!(
        anchor_errors.iter().all(CompilerError::is_recoverable),
        "nested anchor recovery must not leave a hard parser error: {anchor_errors:?}"
    );
    assert_eq!(anchor_root.children.len(), 2);
    assert!(matches!(&anchor_root.children[0], TemplateChildNode::Element(a) if a.tag == "a"));
    assert!(matches!(&anchor_root.children[1], TemplateChildNode::Element(a) if a.tag == "a"));

    let (button_root, button_errors) =
        parse(&allocator, "<button>aaa<button>bbb</button></button>");
    assert!(
        button_errors
            .iter()
            .any(|e| e.code == ErrorCode::ExtendPoint && e.message == NESTED_BUTTON_RECOVERY)
    );
    assert!(
        button_errors
            .iter()
            .any(|e| e.code == ErrorCode::ExtendPoint && e.message == IGNORED_END_TAG_RECOVERY),
        "the outer </button> is redundant fallout from the nested-button recovery: {button_errors:?}"
    );
    assert!(
        button_errors.iter().all(CompilerError::is_recoverable),
        "nested button recovery must not leave a hard parser error: {button_errors:?}"
    );
    assert_eq!(button_root.children.len(), 2);
    assert!(
        matches!(&button_root.children[0], TemplateChildNode::Element(button) if button.tag == "button")
    );
    assert!(
        matches!(&button_root.children[1], TemplateChildNode::Element(button) if button.tag == "button")
    );
}

#[test]
fn direct_nested_interactive_end_tag_closes_live_inner_before_redundant_outer() {
    let allocator = Allocator::new();
    for (source, tag) in [
        ("<a><a>x</a></a>", "a"),
        ("<button><button>x</button></button>", "button"),
    ] {
        let (root, errors) = parse(&allocator, source);
        assert!(
            errors.iter().all(CompilerError::is_recoverable),
            "{source}: direct nested interactive recovery must not leave hard parser errors: {errors:?}"
        );
        assert_eq!(
            ignored_end_tag_count(&errors),
            1,
            "{source}: only the redundant outer end tag should be ignored"
        );
        assert_eq!(root.children.len(), 2);
        let TemplateChildNode::Element(outer) = &root.children[0] else {
            panic!("{source}: first child is the implicitly closed outer element");
        };
        let TemplateChildNode::Element(inner) = &root.children[1] else {
            panic!("{source}: second child is the live inner element");
        };
        assert_eq!(outer.tag, tag);
        assert!(outer.children.is_empty());
        assert_eq!(inner.tag, tag);
        assert!(matches!(&inner.children[0], TemplateChildNode::Text(text) if text.content == "x"));
    }
}

#[test]
fn nested_interactive_recovery_consumes_descendant_end_tags() {
    let allocator = Allocator::new();
    for source in [
        r#"<a href="/"><div><a href="/foo">inner</a></div></a>"#,
        "<button><div><button>bbb</button></div></button>",
    ] {
        let (_, errors) = parse(&allocator, source);
        assert!(
            errors.iter().all(CompilerError::is_recoverable),
            "{source}: descendant end tags popped by the nested interactive-content recovery must not stay hard: {errors:?}"
        );
    }
}

#[test]
fn nested_interactive_recovery_does_not_close_same_named_ancestors() {
    let allocator = Allocator::new();
    for source in [
        r#"<div><a href="/"><div><a href="/foo">inner</a></div></a></div>"#,
        "<div><button><div><button>bbb</button></div></button></div>",
    ] {
        let (root, errors) = parse(&allocator, source);
        assert!(
            errors.iter().all(CompilerError::is_recoverable),
            "{source}: redundant descendant end tags from nested interactive-content recovery must not pop same-named ancestors: {errors:?}"
        );
        assert_eq!(
            root.children.len(),
            1,
            "{source}: the outer ancestor should stay open until its authored end tag"
        );
    }
}

#[test]
fn nested_interactive_recovery_keeps_extra_end_tag_hard() {
    let allocator = Allocator::new();
    let (_, errors) = parse(
        &allocator,
        r#"<a href="/"><div><a href="/foo">inner</a></div></a></a>"#,
    );

    assert_eq!(
        invalid_end_tag_count(&errors),
        1,
        "only the recovery-matched outer </a> is downgraded; an extra stray </a> stays hard: {errors:?}"
    );
}

#[test]
fn nested_interactive_recovery_keeps_out_of_order_end_tag_hard() {
    let allocator = Allocator::new();
    let (_, errors) = parse(
        &allocator,
        r#"<a href="/"><div><a href="/foo">inner</a></a></div>"#,
    );

    assert_eq!(
        invalid_end_tag_count(&errors),
        1,
        "the redundant end-tag recovery must not consume out-of-order authored tags: {errors:?}"
    );
}

fn invalid_end_tag_count(errors: &[CompilerError]) -> usize {
    errors
        .iter()
        .filter(|error| error.code == ErrorCode::InvalidEndTag)
        .count()
}

fn ignored_end_tag_count(errors: &[CompilerError]) -> usize {
    errors
        .iter()
        .filter(|error| {
            error.code == ErrorCode::ExtendPoint && error.message == IGNORED_END_TAG_RECOVERY
        })
        .count()
}
