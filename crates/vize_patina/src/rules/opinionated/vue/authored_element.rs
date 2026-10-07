//! Prove authored start/end tags from existing element and child ranges.

use std::ops::Range;
use vize_relief::{ElementNode, Namespace, TemplateChildNode};

pub(super) fn opening<'a>(source: &'a str, element: &ElementNode<'_>) -> Option<&'a str> {
    let span = element.loc.span;
    let authored = source.get(span.start as usize..span.end as usize)?;
    if !authored.starts_with('<')
        || authored.get(1..1 + element.tag.len())? != element.tag
        || !authored
            .as_bytes()
            .get(1 + element.tag.len())
            .is_some_and(|b| space(*b) || matches!(b, b'/' | b'>'))
    {
        return None;
    }
    let mut quote = None;
    for (index, byte) in authored.bytes().enumerate().skip(1) {
        match (quote, byte) {
            (Some(current), byte) if current == byte => quote = None,
            (Some(_), _) => {}
            (None, b'\'' | b'"') => quote = Some(byte),
            (None, b'>') => {
                if index + 1 != authored.len() {
                    return None;
                }
                let delimiter = authored
                    .get(..index)?
                    .trim_end_matches([' ', '\t', '\r', '\n', '\u{c}']);
                if delimiter.ends_with('/') {
                    let slash = span.start as usize + delimiter.len() - 1;
                    // An unquoted attribute value can contain the final slash.
                    if element
                        .props
                        .iter()
                        .any(|prop| prop.loc().span.end as usize > slash)
                    {
                        return None;
                    }
                }
                return Some(authored);
            }
            (None, b'<') => return None,
            _ => {}
        }
    }
    None
}

pub(super) fn closing(source: &str, element: &ElementNode<'_>) -> Option<Range<usize>> {
    closing_with_depth(source, element, 0)
}

fn closing_with_depth(
    source: &str,
    element: &ElementNode<'_>,
    depth: usize,
) -> Option<Range<usize>> {
    if depth >= 64 {
        return None;
    }
    let authored = opening(source, element)?;
    if authored.trim_end_matches('>').trim_end().ends_with('/') {
        return None;
    }
    let mut cursor = element.loc.span.end as usize;
    for child in element.children.iter() {
        let span = child.loc().span;
        source.get(span.start as usize..span.end as usize)?;
        if !source.get(cursor..span.start as usize)?.bytes().all(space) {
            return None;
        }
        cursor = match child {
            TemplateChildNode::Element(nested) => {
                let opening = opening(source, nested)?;
                if nested.is_self_closing
                    || (nested.ns == Namespace::Html && vize_l0::is_void_tag(nested.tag))
                    || opening.trim_end_matches('>').trim_end().ends_with('/')
                {
                    nested.loc.span.end as usize
                } else {
                    closing_with_depth(source, nested, depth + 1)?.end
                }
            }
            _ => span.end as usize,
        };
    }
    while source
        .as_bytes()
        .get(cursor)
        .is_some_and(|byte| space(*byte))
    {
        cursor += 1;
    }
    let start = cursor;
    let suffix = source.get(cursor..)?;
    if !suffix.starts_with("</") || suffix.get(2..2 + element.tag.len())? != element.tag {
        return None;
    }
    cursor += 2 + element.tag.len();
    while source
        .as_bytes()
        .get(cursor)
        .is_some_and(|byte| space(*byte))
    {
        cursor += 1;
    }
    (source.as_bytes().get(cursor) == Some(&b'>')).then_some(start..cursor + 1)
}

fn space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | 12)
}
