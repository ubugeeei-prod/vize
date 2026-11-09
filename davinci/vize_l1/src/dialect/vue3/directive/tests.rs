//! Literal names use the head boundary without interpreting the argument.

use super::{VueDirectives, frozen_attribute_name};
use crate::markup::directive::{ArgSyntax, DirectiveSyntax};
use vize_l0::Allocator;

#[test]
fn directive_argument_boundaries_match_vue_tokenizer_states() {
    // Oracle: compiler-core baseParse, including recovered error 27 for `:[`.
    for (authored, expected_arg, expected_modifiers) in [
        (":[a]b", Some((false, "a]")), ""),
        (":foo[bar]", Some((true, "foo[bar]")), ""),
        (":[", Some((false, "")), ""),
        (":[a[b]].camel", Some((false, "a[b]")), ".camel"),
        (":title.camel", Some((true, "title")), ".camel"),
        (":[title].camel", Some((false, "title")), ".camel"),
        (
            "#[(slot=>slot)(outside)]",
            Some((false, "(slot=>slot)(outside)")),
            "",
        ),
        (
            "#[(other=>slot)(outside)].once",
            Some((false, "(other=>slot)(outside)")),
            ".once",
        ),
    ] {
        let offset = 17;
        let decomposed = VueDirectives.decompose(authored, offset).unwrap().unwrap();
        let content = |span: vize_l0::Span| {
            authored
                .get((span.start - offset) as usize..(span.end - offset) as usize)
                .unwrap()
        };
        let actual_arg = decomposed.arg.map(|arg| match arg {
            ArgSyntax::Static(span) => (true, content(span)),
            ArgSyntax::Dynamic(span) => (false, content(span)),
        });
        assert_eq!(actual_arg, expected_arg, "{authored}");
        assert_eq!(
            content(decomposed.modifiers),
            expected_modifiers,
            "{authored}"
        );
    }
}

#[test]
fn frozen_longhand_names_keep_quoted_brackets_and_all_modifiers() {
    let allocator = Allocator::new();
    for (authored, expected) in [
        (
            "v-bind:[keys['name]']].camel.stop",
            "v-bind[keys['name]']].camel.stop",
        ),
        (
            "v-on:[keys[\"event]\"]].once.capture",
            "v-on[keys[\"event]\"]].once.capture",
        ),
        ("v-例:[鍵['名]']].修飾", "v-例[鍵['名]']].修飾"),
        ("v-bind:[unterminated..camel", "v-bind[unterminated..camel"),
        ("v-bind:id..camel", "v-bindid..camel"),
    ] {
        assert_eq!(frozen_attribute_name(&allocator, authored), expected);
    }
}

#[test]
fn frozen_plain_and_shorthand_names_keep_the_original_borrow() {
    let allocator = Allocator::new();
    for authored in [
        "is",
        ":[keys['name]']].camel.stop",
        ".id.camel",
        "@[keys['event]']].once",
        "#default",
        "v-pre",
        "v-bind.camel:literal",
    ] {
        let frozen = frozen_attribute_name(&allocator, authored);
        assert_eq!(frozen, authored);
        assert!(core::ptr::eq(frozen, authored));
    }
}

#[test]
fn directive_kind_reads_the_decomposed_head() {
    use super::DirectiveKind;
    for (authored, expected) in [
        ("v-if", DirectiveKind::If),
        ("v-else-if", DirectiveKind::ElseIf),
        ("v-else", DirectiveKind::Else),
        ("v-for", DirectiveKind::For),
        ("v-model.trim", DirectiveKind::Model),
        ("v-slot:item", DirectiveKind::Slot),
        ("#item", DirectiveKind::Slot),
        ("v-bind:title.camel", DirectiveKind::Bind),
        (":title", DirectiveKind::Bind),
        (".title", DirectiveKind::Bind),
        ("v-on:click", DirectiveKind::On),
        ("@click.stop", DirectiveKind::On),
        ("v-html", DirectiveKind::Html),
        ("v-text", DirectiveKind::Text),
        ("v-show", DirectiveKind::Show),
        ("v-cloak", DirectiveKind::Cloak),
        ("v-once", DirectiveKind::Once),
        ("v-pre", DirectiveKind::Pre),
        ("v-memo", DirectiveKind::Memo),
        ("v-focus:[arg].mod", DirectiveKind::Custom),
        ("v-iffy", DirectiveKind::Custom),
        ("v-", DirectiveKind::Custom),
    ] {
        // The head spans are absolute: resolve them against the whole source.
        let source = ["<p ", authored, ">"].concat();
        let head = VueDirectives.decompose(authored, 3).unwrap().unwrap();
        assert_eq!(DirectiveKind::of(&head, &source), expected, "{authored}");
    }
}
