//! The Svelte file container: top-level `<script>` and `<style>` blocks.
//!
//! A block opens at the start of a line. Every other byte is markup, which
//! the L1 markup parser reads in place, so spans stay file-absolute.

use vize_l0::{Allocator, Span, Vec};
use vize_l1::container::{Block, Container, ContainerError, ContainerErrorCode, ContainerFormat};

pub(crate) struct Svelte;

impl ContainerFormat for Svelte {
    fn split<'a>(&self, allocator: &'a Allocator, source: &'a str) -> Container<'a> {
        let mut blocks = Vec::new_in(&allocator);
        let mut errors = Vec::new_in(&allocator);
        let mut cursor = 0;
        while let Some((name, start)) = next_block(source, cursor) {
            let at = |offset: usize| u32::try_from(offset).unwrap_or(u32::MAX);
            let Some(gt) = source.get(start..).and_then(|rest| rest.find('>')) else {
                errors.push(ContainerError {
                    code: ContainerErrorCode::UnterminatedOpenTag,
                    offset: at(start),
                });
                break;
            };
            let content = start + gt + 1;
            let close = source
                .get(content..)
                .and_then(|rest| rest.find(&*vize_l0::cstr!("</{name}>")))
                .map(|found| content + found);
            let end = close.map_or(source.len(), |close| close + name.len() + 3);
            blocks.push(Block {
                name,
                open_tag: Span::new(at(start), at(content)),
                attrs: Vec::new_in(&allocator),
                content: Span::new(at(content), at(close.unwrap_or(source.len()))),
                close_tag: close.map(|close| Span::new(at(close), at(end))),
            });
            if close.is_none() {
                errors.push(ContainerError {
                    code: ContainerErrorCode::MissingCloseTag,
                    offset: at(start),
                });
            }
            cursor = end;
        }
        Container {
            source,
            blocks,
            errors,
        }
    }
}

/// The next `<script` or `<style` that opens a line at or after `from`.
fn next_block(source: &str, from: usize) -> Option<(&str, usize)> {
    let mut line = from;
    loop {
        let rest = source.get(line..)?;
        for name in ["script", "style"] {
            let opens = rest
                .strip_prefix('<')
                .and_then(|tag| tag.strip_prefix(name))
                .is_some_and(|after| after.starts_with(['>', ' ', '\t', '\n']));
            if opens {
                return Some((source.get(line + 1..line + 1 + name.len())?, line));
            }
        }
        line += rest.find('\n')? + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::Svelte;
    use vize_l0::Allocator;
    use vize_l1::container::ContainerFormat;

    #[test]
    fn blocks_split_losslessly_around_the_markup() {
        let source = "<script>\nlet a = 1;\n</script>\n\n<p>{a}</p>\n<style>p{}</style>\n";
        let allocator = Allocator::default();
        let container = Svelte.split(&allocator, source);
        let names: Vec<&str> = container.blocks.iter().map(|block| block.name).collect();
        assert_eq!(names, ["script", "style"]);
        assert!(container.errors.is_empty());
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
