use super::Parser;
use vize_l0::Allocator;

#[test]
fn opted_in_spans_bind_the_original_source_and_preserve_the_public_result() {
    let source = "<div v-pre><span><Child is=\"vue:Child\"></Child></span></div><Child></Child>";
    let arena = Allocator::new();
    let (root, errors, frozen) = Parser::new(&arena, source).parse_with_frozen_elements();
    let (control, control_errors) = Parser::new(&arena, source).parse();
    assert_eq!(vize_l0::cstr!("{root:?}"), vize_l0::cstr!("{control:?}"));
    assert_eq!(
        vize_l0::cstr!("{errors:?}"),
        vize_l0::cstr!("{control_errors:?}")
    );
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(
        frozen
            .spans
            .iter()
            .map(|span| span.slice(source))
            .collect::<std::vec::Vec<_>>(),
        ["<div v-pre>", "<span>", "<Child is=\"vue:Child\">"],
    );
    assert_eq!(frozen.spans_for(root.source, source), Some(frozen.spans));
    let copy = source.to_owned();
    assert_eq!(frozen.spans_for(&copy, source), None);
    assert_eq!(frozen.spans_for(root.source, &copy), None);
}

#[test]
fn opening_ownership_survives_html_scope_recovery_without_changing_recovery() {
    let source = "<p v-pre><div><Child></Child></div><p></p>";
    let arena = Allocator::new();
    let (root, errors, frozen) = Parser::new(&arena, source).parse_with_frozen_elements();
    let (control, control_errors) = Parser::new(&arena, source).parse();
    assert_eq!(vize_l0::cstr!("{root:?}"), vize_l0::cstr!("{control:?}"));
    assert_eq!(
        vize_l0::cstr!("{errors:?}"),
        vize_l0::cstr!("{control_errors:?}")
    );
    assert_eq!(
        frozen
            .spans
            .iter()
            .map(|span| span.slice(source))
            .collect::<std::vec::Vec<_>>(),
        ["<p v-pre>", "<div>"],
    );
    assert!(
        frozen
            .spans
            .windows(2)
            .all(|pair| pair[0].start < pair[1].start)
    );
}

#[test]
fn normal_opt_in_is_empty_and_default_parser_disables_collection() {
    let source = "<Child :id=\"active\"></Child>";
    let arena = Allocator::new();
    assert!(Parser::new(&arena, source).frozen_elements.is_none());
    let (root, errors, frozen) = Parser::new(&arena, source).parse_with_frozen_elements();
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(frozen.spans_for(root.source, source), Some(&[][..]));
}
