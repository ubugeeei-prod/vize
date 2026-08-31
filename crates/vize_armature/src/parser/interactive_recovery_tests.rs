use super::parse;
use vize_relief::{
    TemplateChildNode,
    errors::{CompilerError, ErrorCode},
};
use vize_s0::Allocator;

#[test]
fn nested_anchor_and_button_recovery_stays_recoverable() {
    let allocator = Allocator::new();
    let (anchor_root, anchor_errors) = parse(
        &allocator,
        r#"<a href="/">outer<a href="/foo">inner</a></a>"#,
    );
    assert_redundant_end_tag_notice(&anchor_errors, "a");
    assert!(anchor_errors.iter().all(CompilerError::is_recoverable));
    assert_eq!(anchor_root.children.len(), 2);
    assert!(matches!(&anchor_root.children[0], TemplateChildNode::Element(a) if a.tag == "a"));
    assert!(matches!(&anchor_root.children[1], TemplateChildNode::Element(a) if a.tag == "a"));

    let (button_root, button_errors) =
        parse(&allocator, "<button>aaa<button>bbb</button></button>");
    assert_redundant_end_tag_notice(&button_errors, "button");
    assert!(button_errors.iter().all(CompilerError::is_recoverable));
    assert_eq!(button_root.children.len(), 2);
    assert!(
        matches!(&button_root.children[0], TemplateChildNode::Element(button) if button.tag == "button")
    );
    assert!(
        matches!(&button_root.children[1], TemplateChildNode::Element(button) if button.tag == "button")
    );
}

#[test]
fn nested_anchor_recovery_consumes_only_matching_redundant_end_tag() {
    let allocator = Allocator::new();
    let (_, errors) = parse(
        &allocator,
        r#"<a href="/"><div><a href="/foo">inner</a></div></a></a>"#,
    );

    let invalid_end_tags = errors
        .iter()
        .filter(|error| error.code == ErrorCode::InvalidEndTag)
        .count();
    assert_eq!(
        invalid_end_tags, 1,
        "only the recovery-matched outer </a> is downgraded; an extra stray </a> stays hard: {errors:?}"
    );
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

fn assert_redundant_end_tag_notice(errors: &[CompilerError], tag: &str) {
    assert!(
        errors.iter().any(|error| {
            error.code == ErrorCode::ExtendPoint && error.message.contains("ignored this end tag")
        }),
        "the outer </{tag}> is redundant fallout from nested interactive-content recovery: {errors:?}"
    );
}
