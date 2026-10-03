//! Borrowed documents for the first typed Vue directive-head family.

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
    if !matches!(head.prefix, DirectivePrefix::Bind | DirectivePrefix::Prop) {
        return Err(unsupported());
    }
    if block.start().checked_add(1) != Some(argument.start) || argument.start == argument.end {
        return Err(unsupported());
    }
    if head.name != Span::new(block.start(), block.start())
        || head.modifiers != Span::new(argument.end, block.end())
    {
        return Err(TemplateRefusal::SourceMismatch {
            offset: block.start() as usize,
        });
    }
    let mut parts = Vec::new_in(&allocator);
    for span in [
        Span::new(block.start(), argument.start),
        argument,
        head.modifiers,
    ] {
        if !block.contains_block_span(span) {
            return Err(TemplateRefusal::SourceMismatch {
                offset: block.start() as usize,
            });
        }
        let text = block
            .root_source()
            .get(span.start as usize..span.end as usize)
            .ok_or(TemplateRefusal::SourceMismatch {
                offset: block.start() as usize,
            })?;
        parts.push(Doc::text(text));
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
        markup::{ArgSyntax, DirectiveName, DirectiveSyntax},
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
}
