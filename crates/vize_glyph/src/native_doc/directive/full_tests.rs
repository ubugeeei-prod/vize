//! Original framing laws for Full heads without an argument.

use super::name_document;
use crate::native_doc::{TemplateRefusal, UnsupportedSyntax, document::Kind};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::{
    dialect::vue3::VueDirectives,
    markup::{DirectiveName, DirectivePrefix, DirectiveSyntax},
};

#[test]
fn full_names_and_complete_modifier_runs_borrow_original_pieces() {
    let allocator = Allocator::default();
    for (source, expected) in [
        ("é v-カスタム..keep", ["v-", "カスタム", "..keep"]),
        ("é v-if", ["v-", "if", ""]),
    ] {
        let text = source.get(3..).unwrap();
        let block = SourceRoot::new(source).unwrap().block(text, 3).unwrap();
        let head = VueDirectives.decompose(text, 3).unwrap().unwrap();
        assert_eq!(head.arg, None);
        let document = name_document(block, Some(head), &allocator).unwrap();
        let Kind::Concat(parts) = document.kind else {
            panic!("borrowed pieces")
        };
        assert_eq!(parts.len(), expected.len());
        let mut start = 3;
        for (part, expected) in parts.iter().zip(expected) {
            let Kind::Text(actual) = part.kind else {
                panic!("original text")
            };
            let original = source.get(start..start + expected.len()).unwrap();
            assert_eq!(actual, expected);
            assert!(core::ptr::eq(actual.as_ptr(), original.as_ptr()));
            start += expected.len();
        }
        assert_eq!(start, source.len());
    }
}

#[test]
fn forged_full_name_prefix_utf8_modifier_and_overflow_ranges_are_refused() {
    let allocator = Allocator::default();
    let source = "v-日本..keep";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let head = VueDirectives.decompose(source, 0).unwrap().unwrap();
    for altered in [
        DirectiveName {
            prefix: DirectivePrefix::Bind,
            ..head
        },
        DirectiveName {
            name: Span::new(1, 8),
            ..head
        },
        DirectiveName {
            name: Span::new(2, 3),
            modifiers: Span::new(3, block.end()),
            ..head
        },
        DirectiveName {
            name: Span::new(2, u32::MAX),
            modifiers: Span::new(u32::MAX, block.end()),
            ..head
        },
        DirectiveName {
            modifiers: Span::new(9, block.end()),
            ..head
        },
        DirectiveName {
            modifiers: Span::new(8, block.end() - 1),
            ..head
        },
    ] {
        assert!(matches!(
            name_document(block, Some(altered), &allocator),
            Err(TemplateRefusal::SourceMismatch { .. } | TemplateRefusal::Unsupported { .. })
        ));
    }
    let foreign = SourceRoot::new("vX日本..keep").unwrap().whole_block();
    assert!(matches!(
        name_document(foreign, Some(head), &allocator),
        Err(TemplateRefusal::SourceMismatch { .. })
    ));
}

#[test]
fn empty_full_names_missing_shorthands_and_incomplete_colons_stay_refusals() {
    let allocator = Allocator::default();
    for source in ["v-", "v-.prop", ":", ".", "@", "#"] {
        let block = SourceRoot::new(source).unwrap().whole_block();
        let head = VueDirectives.decompose(source, 0).unwrap();
        assert!(
            matches!(
                name_document(block, head, &allocator),
                Err(TemplateRefusal::Unsupported {
                    syntax: UnsupportedSyntax::Directive,
                    ..
                })
            ),
            "{source}"
        );
    }
    for source in ["v-bind:", "v-bind:.prop", "v-bind:["] {
        let block = SourceRoot::new(source).unwrap().whole_block();
        let head = VueDirectives.decompose(source, 0).unwrap();
        assert!(
            matches!(
                name_document(block, head, &allocator),
                Err(TemplateRefusal::SourceMismatch { .. })
            ),
            "{source}"
        );
    }
}
