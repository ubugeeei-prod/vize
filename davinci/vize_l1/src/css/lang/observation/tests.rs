use super::*;
use core::ptr;
use vize_l0::SourceRoot;

fn assert_same_style(language: StyleLanguage, block: SourceBlock<'_>) {
    let observed = StyleObservation::read(language, block);
    let direct = language.structure(block.source(), block.start());
    let actual = observed.structure();
    assert!(ptr::eq(actual, observed.structure()));
    assert!(ptr::eq(actual.source(), block.source()));
    assert_eq!(actual.is_balanced(), direct.is_balanced());
    assert_eq!(actual.is_terminated(), direct.is_terminated());
    assert_eq!(actual.comments(), direct.comments());
    assert_eq!(actual.nodes(), direct.nodes());
    for (node, other) in actual.nodes().iter().zip(direct.nodes()) {
        assert!(block.contains_block_span(node.span()));
        assert_eq!(actual.kind(node), direct.kind(other));
        assert_eq!(actual.code(node.prelude()), direct.code(other.prelude()));
        assert_eq!(actual.text(node.span()), direct.text(other.span()));
    }
    for comment in actual.comments() {
        assert!(block.contains_block_span(comment.span()));
        assert_eq!(actual.text(comment.span()), direct.text(comment.span()));
    }
    let classes = language.authored_classes(block.source());
    assert_eq!(observed.authored_classes(), classes.as_ref());
    if let Some(first) = observed.authored_classes() {
        assert!(ptr::eq(first, observed.authored_classes().unwrap()));
    }
}

#[test]
fn shared_classes_and_structure_keep_each_dialects_source_and_recovery() {
    use StyleLanguage::{Css, Less, Sass, Scss, Stylus};
    for (lang, text) in [
        (Css, ".first, :global(.external) { /* css */ color:red }"),
        (Scss, ".card { &__label { color: var(--fg); } .x-#{1} {} }"),
        (Less, ".root { .m() { .hidden {} } &-wide { width: 2px; } }"),
        (Sass, ".card\n  &__text\n    color: red\n  // comment\n"),
        (Stylus, ".tile\n  &-body\n    color red\n"),
        (Stylus, ".tile { color: red; }"),
        (Css, "/* unterminated"),
        (Scss, ".unclosed { color: red;"),
        (Less, ".continued { @import 'rest'; &:hover {} }"),
        (Sass, "@if $enabled\n  .possible\n    color: red\n"),
        (Css, ""),
    ] {
        let doc = vize_l0::cstr!("前\n<style>{text}</style>後");
        let start = "前\n<style>".len();
        let slice = &doc[start..start + text.len()];
        let block = SourceRoot::new(&doc)
            .unwrap()
            .block(slice, start as u32)
            .unwrap();
        assert_same_style(lang, block);
    }
}

#[test]
fn unreadable_and_partly_known_classes_are_distinct() {
    let invalid = SourceRoot::new("/*").unwrap().whole_block();
    let invalid = StyleObservation::read(StyleLanguage::Css, invalid);
    assert!(!invalid.structure().is_terminated());
    assert!(invalid.authored_classes().is_none());
    let text = ".kept {} .unknown-#{$x} {}";
    let partial = StyleObservation::read(
        StyleLanguage::Scss,
        SourceRoot::new(text).unwrap().whole_block(),
    );
    let classes = partial.authored_classes().unwrap();
    assert!(!classes.is_complete());
    assert!(classes.names().iter().any(|name| name == "kept"));
    assert!(!partial.structure().nodes().is_empty());
}

mod corpus;
