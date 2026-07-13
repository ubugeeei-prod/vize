//! Deterministic SFC opening-tag output helpers.

use std::borrow::Cow;

use vize_carton::FxHashMap;

use super::Block;

pub(super) fn document_prologue<'a>(
    source: &'a str,
    blocks: &[(usize, Block<'_>)],
) -> Option<&'a str> {
    let first_tag_start = blocks
        .iter()
        .map(|(_, block)| match block {
            Block::Script(block) => block.loc.tag_start,
            Block::Template(block) => block.loc.tag_start,
            Block::Style(block) => block.loc.tag_start,
            Block::Custom(block) => block.loc.tag_start,
        })
        .min()
        .unwrap_or(source.len());
    let prologue = source[..first_tag_start].trim();
    (!prologue.is_empty()).then_some(prologue)
}

pub(super) fn write_remaining_attrs(
    output: &mut Vec<u8>,
    attrs: &FxHashMap<Cow<'_, str>, Cow<'_, str>>,
    handled: &[&str],
) {
    let mut remaining_attrs: Vec<_> = attrs
        .iter()
        .filter(|(name, _)| !handled.contains(&name.as_ref()))
        .collect();
    remaining_attrs.sort_by(|(a, _), (b, _)| a.as_ref().cmp(b.as_ref()));

    for (name, value) in remaining_attrs {
        let value = if value.is_empty() {
            None
        } else {
            Some(value.as_ref())
        };
        write_attr(output, name, value);
    }
}

pub(super) fn write_attr(output: &mut Vec<u8>, name: &str, value: Option<&str>) {
    output.push(b' ');
    output.extend_from_slice(name.as_bytes());
    if let Some(value) = value {
        output.extend_from_slice(b"=\"");
        output.extend_from_slice(value.as_bytes());
        output.push(b'"');
    }
}
