//! The Astro file container: a `---` fenced frontmatter script, then markup.

use vize_l0::{Allocator, Span, Vec};
use vize_l1::container::{Block, Container, ContainerError, ContainerErrorCode, ContainerFormat};

pub(crate) struct Astro;

impl ContainerFormat for Astro {
    fn split<'a>(&self, allocator: &'a Allocator, source: &'a str) -> Container<'a> {
        let mut blocks = Vec::new_in(&allocator);
        let mut errors = Vec::new_in(&allocator);
        let at = |offset: usize| u32::try_from(offset).unwrap_or(u32::MAX);
        let start = source.len() - source.trim_start().len();
        if source
            .get(start..)
            .is_some_and(|rest| rest.starts_with("---\n"))
        {
            let content = start + 4;
            // The search starts at the opening fence's line break: the script may be empty.
            let close = source
                .get(content - 1..)
                .and_then(|rest| rest.find("\n---"))
                .map(|found| content + found);
            blocks.push(Block {
                name: "frontmatter",
                open_tag: Span::new(at(start), at(content)),
                attrs: Vec::new_in(&allocator),
                content: Span::new(at(content), at(close.unwrap_or(source.len()))),
                close_tag: close.map(|close| Span::new(at(close), at(close + 3))),
            });
            if close.is_none() {
                errors.push(ContainerError {
                    code: ContainerErrorCode::MissingCloseTag,
                    offset: at(start),
                });
            }
        }
        Container {
            source,
            blocks,
            errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Astro;
    use vize_l0::Allocator;
    use vize_l1::container::ContainerFormat;

    #[test]
    fn the_frontmatter_is_one_block_and_the_split_is_lossless() {
        let source = "---\nconst a = 1;\n---\n\n<p>{a}</p>\n";
        let allocator = Allocator::default();
        let container = Astro.split(&allocator, source);
        assert!(container.errors.is_empty());
        let content = container.blocks.first().map(|block| block.content);
        assert_eq!(
            content.and_then(|span| source.get(span.start as usize..span.end as usize)),
            Some("const a = 1;\n")
        );
        let rebuilt: Vec<&str> = container
            .segments()
            .map(|segment| {
                let span = segment.span();
                source
                    .get(span.start as usize..span.end as usize)
                    .unwrap_or_default()
            })
            .collect();
        assert_eq!(rebuilt.concat(), source);
    }
}
