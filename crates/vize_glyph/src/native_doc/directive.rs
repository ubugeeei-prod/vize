//! Borrowed documents for typed Vue heads with complete static arguments.

use vize_l0::{Allocator, SourceBlock, Span, Vec};
use vize_l1::markup::{ArgSyntax, DirectiveName, DirectivePrefix};

use super::{Doc, TemplateRefusal, UnsupportedSyntax};

pub(super) fn name_document<'a>(
    block: SourceBlock<'a>,
    head: Option<DirectiveName>,
    allocator: &'a Allocator,
) -> Result<Doc<'a>, TemplateRefusal> {
    let Some(head) = head else {
        return Ok(Doc::text(block.source()));
    };
    let unsupported = || TemplateRefusal::Unsupported {
        offset: block.start() as usize,
        syntax: UnsupportedSyntax::Directive,
    };
    let Some(ArgSyntax::Static(argument)) = head.arg else {
        return Err(unsupported());
    };
    if argument.start == argument.end {
        return Err(unsupported());
    }
    let mismatch = || TemplateRefusal::SourceMismatch {
        offset: block.start() as usize,
    };
    if head.modifiers != Span::new(argument.end, block.end()) {
        return Err(mismatch());
    }
    let spelling = match head.prefix {
        DirectivePrefix::Full => "v-",
        DirectivePrefix::Bind => ":",
        DirectivePrefix::Prop => ".",
        DirectivePrefix::On => "@",
        DirectivePrefix::Slot => "#",
    };
    let (prefix, name, separator) = match head.prefix {
        DirectivePrefix::Full => {
            if head.name.start == head.name.end {
                return Err(unsupported());
            }
            if block.start().checked_add(2) != Some(head.name.start)
                || head.name.end.checked_add(1) != Some(argument.start)
            {
                return Err(mismatch());
            }
            (
                Span::new(block.start(), head.name.start),
                Some(head.name),
                Some(Span::new(head.name.end, argument.start)),
            )
        }
        _ => {
            if block.start().checked_add(1) != Some(argument.start)
                || head.name != Span::new(block.start(), block.start())
            {
                return Err(mismatch());
            }
            (Span::new(block.start(), argument.start), None, None)
        }
    };
    let project = |span| {
        if !block.contains_block_span(span) {
            return Err(mismatch());
        }
        block
            .root_source()
            .get(span.start as usize..span.end as usize)
            .ok_or_else(mismatch)
    };
    if project(prefix)? != spelling || separator.is_some_and(|span| project(span) != Ok(":")) {
        return Err(TemplateRefusal::SourceMismatch {
            offset: block.start() as usize,
        });
    }
    let mut parts = Vec::new_in(&allocator);
    for span in [
        Some(prefix),
        name,
        separator,
        Some(argument),
        Some(head.modifiers),
    ]
    .into_iter()
    .flatten()
    {
        parts.push(Doc::text(project(span)?));
    }
    Ok(Doc::concat(parts))
}

#[cfg(test)]
mod tests {
    use super::name_document;
    use crate::native_doc::TemplateRefusal;
    use crate::native_doc::document::Kind;
    use vize_l0::{Allocator, SourceRoot, Span};
    use vize_l1::{
        dialect::vue3::VueDirectives,
        markup::{ArgSyntax, DirectiveName, DirectivePrefix, DirectiveSyntax},
    };

    #[test]
    fn original_typed_unicode_argument_pieces_borrow_the_checked_source_block() {
        let allocator = Allocator::default();
        let source = "é :日本.camel";
        let text = source.get(3..).unwrap();
        let block = SourceRoot::new(source).unwrap().block(text, 3).unwrap();
        let head = VueDirectives
            .decompose(text, block.start())
            .unwrap()
            .unwrap();
        assert_eq!(head.arg, Some(ArgSyntax::Static(Span::new(4, 10))));
        assert_eq!(head.modifiers, Span::new(10, 16));
        let document = name_document(block, Some(head), &allocator).unwrap();
        let Kind::Concat(parts) = document.kind else {
            panic!("borrowed pieces")
        };
        assert_eq!(parts.len(), 3);
        for (part, span) in parts
            .iter()
            .zip([Span::new(3, 4), Span::new(4, 10), Span::new(10, 16)])
        {
            let Kind::Text(actual) = part.kind else {
                panic!("source text")
            };
            let expected = source.get(span.start as usize..span.end as usize).unwrap();
            assert_eq!(actual, expected);
            assert!(core::ptr::eq(actual.as_ptr(), expected.as_ptr()));
        }
    }

    #[test]
    fn forged_argument_projection_cannot_split_utf8_or_escape_the_source_block() {
        let allocator = Allocator::default();
        let source = ":日本.camel";
        let block = SourceRoot::new(source).unwrap().whole_block();
        let head = VueDirectives.decompose(source, 0).unwrap().unwrap();
        for end in [2, block.end() + 1] {
            let altered = DirectiveName {
                arg: Some(ArgSyntax::Static(Span::new(1, end))),
                modifiers: Span::new(end, block.end()),
                ..head
            };
            assert!(matches!(
                name_document(block, Some(altered), &allocator),
                Err(TemplateRefusal::SourceMismatch { .. })
            ));
        }
    }

    #[test]
    fn full_name_separator_and_other_static_prefixes_borrow_original_pieces() {
        let cases: [(&str, &[&str]); 3] = [
            (
                "é v-カスタム:日本..camel",
                &["v-", "カスタム", ":", "日本", "..camel"],
            ),
            ("é @更新.once", &["@", "更新", ".once"]),
            ("é #見出し", &["#", "見出し", ""]),
        ];
        let allocator = Allocator::default();
        for (source, expected) in cases {
            let text = source.get(3..).unwrap();
            let block = SourceRoot::new(source).unwrap().block(text, 3).unwrap();
            let head = VueDirectives.decompose(text, 3).unwrap().unwrap();
            let document = name_document(block, Some(head), &allocator).unwrap();
            let Kind::Concat(parts) = document.kind else {
                panic!("borrowed pieces")
            };
            assert_eq!(parts.len(), expected.len());
            let mut start = 3;
            for (part, expected) in parts.iter().zip(expected) {
                let Kind::Text(actual) = part.kind else {
                    panic!("source text")
                };
                let original = source.get(start..start + expected.len()).unwrap();
                assert_eq!(actual, *expected);
                assert!(core::ptr::eq(actual.as_ptr(), original.as_ptr()));
                start += expected.len();
            }
            assert_eq!(start, source.len());
        }
    }

    #[test]
    fn forged_full_prefix_name_separator_argument_and_modifier_ranges_are_refused() {
        let allocator = Allocator::default();
        let source = "v-bind:日本.camel";
        let block = SourceRoot::new(source).unwrap().whole_block();
        let head = VueDirectives.decompose(source, 0).unwrap().unwrap();
        for altered in [
            DirectiveName {
                prefix: DirectivePrefix::Bind,
                ..head
            },
            DirectiveName {
                name: Span::new(1, 6),
                ..head
            },
            DirectiveName {
                name: Span::new(2, 5),
                arg: Some(ArgSyntax::Static(Span::new(6, 13))),
                ..head
            },
            DirectiveName {
                arg: Some(ArgSyntax::Static(Span::new(7, 8))),
                modifiers: Span::new(8, block.end()),
                ..head
            },
            DirectiveName {
                modifiers: Span::new(12, block.end()),
                ..head
            },
            DirectiveName {
                arg: Some(ArgSyntax::Static(Span::new(7, u32::MAX))),
                modifiers: Span::new(u32::MAX, block.end()),
                ..head
            },
        ] {
            assert!(matches!(
                name_document(block, Some(altered), &allocator),
                Err(TemplateRefusal::SourceMismatch { .. })
            ));
        }
        let foreign = SourceRoot::new("vXbind:日本.camel").unwrap().whole_block();
        assert!(matches!(
            name_document(foreign, Some(head), &allocator),
            Err(TemplateRefusal::SourceMismatch { .. })
        ));
    }
}
