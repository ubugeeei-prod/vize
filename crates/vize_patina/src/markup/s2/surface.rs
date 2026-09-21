//! S1 surface reads for the S2 facade: authored attributes the S2 op tree
//! consumed or dropped, located on the lossless S1 tree instead of a private
//! re-scan of the source (architecture: surface-shaped consumers read S1).
//!
//! S1 is a contiguous partition of the source, so an element is found by its
//! start offset (the S2 op span starts at the S1 element's `<tag` token) with a
//! binary search per children level, and every attribute extent is recovered
//! from the tokens the parser kept — the lowering's own extent rule.

use vize_s0::Span;
use vize_s1::{Attribute, Element, SurfaceChild, SurfaceTree, Token};

/// Byte offset of a slice of `source`.
#[inline]
pub(in crate::markup) fn offset_in(source: &str, slice: &str) -> u32 {
    let base = source.as_ptr() as usize;
    let at = slice.as_ptr() as usize;
    debug_assert!(
        at >= base && at <= base + source.len(),
        "S1 handed the facade a slice outside the lowered source"
    );
    u32::try_from(at.saturating_sub(base)).unwrap_or(u32::MAX)
}

#[inline]
fn token_end(source: &str, token: &Token<'_>) -> u32 {
    offset_in(source, token.text) + token.text.len() as u32
}

pub(in crate::markup) fn child_start(source: &str, child: &SurfaceChild<'_>) -> u32 {
    match child {
        SurfaceChild::Element(element) => offset_in(source, element.open.lt_name.text),
        SurfaceChild::Interpolation(node) => offset_in(source, node.open.text),
        SurfaceChild::Text(token)
        | SurfaceChild::Comment(token)
        | SurfaceChild::Cdata(token)
        | SurfaceChild::ProcessingInstruction(token)
        | SurfaceChild::Unexpected(token) => offset_in(source, token.text),
    }
}

/// The S1 element whose `<tag` token starts at `start`.
pub(in crate::markup) fn element_at<'a>(
    tree: &'a SurfaceTree<'a>,
    start: u32,
) -> Option<&'a Element<'a>> {
    let siblings = siblings_at(tree, start)?;
    let index = siblings.partition_point(|child| child_start(tree.source, child) < start);
    match siblings.get(index)? {
        SurfaceChild::Element(element) => Some(element),
        _ => None,
    }
}

/// The S1 sibling list holding the element whose `<tag` token starts at
/// `start`.
pub(in crate::markup) fn siblings_at<'a>(
    tree: &'a SurfaceTree<'a>,
    start: u32,
) -> Option<&'a [SurfaceChild<'a>]> {
    let source = tree.source;
    let mut children: &'a [SurfaceChild<'a>] = &tree.children;
    loop {
        let index = children
            .partition_point(|child| child_start(source, child) <= start)
            .checked_sub(1)?;
        let SurfaceChild::Element(element) = &children[index] else {
            return None;
        };
        if offset_in(source, element.open.lt_name.text) == start {
            return Some(children);
        }
        children = &element.children;
    }
}

/// The range a token covers.
pub(in crate::markup) fn token_range(source: &str, token: &Token<'_>) -> crate::ir::ByteRange {
    let start = offset_in(source, token.text);
    crate::ir::ByteRange::new(start, start + token.text.len() as u32)
}

/// One authored attribute's extent: name through the end of its value
/// (closing quote included when present) — the lowering's `attr_span`.
pub(in crate::markup) fn attr_span(source: &str, attr: &Attribute<'_>) -> Span {
    let start = offset_in(source, attr.name.text);
    let end = match &attr.value {
        Some(value) => match &value.close_quote {
            Some(close) => token_end(source, close),
            None => token_end(source, &value.content),
        },
        None => match &attr.eq {
            Some(eq) => token_end(source, eq),
            None => token_end(source, &attr.name),
        },
    };
    Span::new(start, end)
}

/// The authored value text of an attribute, when it has a value node.
#[inline]
pub(in crate::markup) fn attr_value<'a>(attr: &Attribute<'a>) -> Option<&'a str> {
    attr.value.as_ref().map(|value| value.content.text)
}

/// A directive spelling read off an authored attribute name without
/// allocating — the shapes `vize_s1_to_s2`'s classifier splits (`v-name`,
/// `:`/`.` bind, `@` on, `#` slot, `[dynamic]` arguments, dot modifiers).
#[derive(Clone, Copy)]
pub(in crate::markup) struct SurfaceDirective<'a> {
    /// Directive name without prefix (`bind`, `on`, `slot`, `if`, `pin`).
    pub name: &'a str,
    /// Argument text (the bracket contents for a dynamic argument).
    pub arg: Option<&'a str>,
    /// Whether the argument is a static name.
    pub arg_static: bool,
    /// The dot-separated modifier tail, without its leading dot.
    pub modifiers: &'a str,
    /// Whether the spelling is the `.prop` dot shorthand, which carries a
    /// leading synthesized `prop` modifier (the shipped parser's rule).
    prop_shorthand: bool,
}

impl<'a> SurfaceDirective<'a> {
    /// Split an authored attribute name; `None` for a plain attribute.
    pub(in crate::markup) fn parse(name: &'a str) -> Option<Self> {
        for (prefix, head) in [(":", "bind"), (".", "bind"), ("@", "on"), ("#", "slot")] {
            if let Some(rest) = name.strip_prefix(prefix) {
                let mut directive = Self::with_arg_first(head, rest);
                directive.prop_shorthand = prefix == "."
                    && !directive
                        .modifiers
                        .split('.')
                        .any(|modifier| modifier == "prop");
                return Some(directive);
            }
        }
        let rest = name.strip_prefix("v-")?;
        let head_end = rest.find([':', '.']).unwrap_or(rest.len());
        let head = &rest[..head_end];
        let tail = &rest[head_end..];
        Some(match tail.strip_prefix(':') {
            Some(after_colon) => Self::with_arg_first(head, after_colon),
            None => Self {
                name: head,
                arg: None,
                arg_static: true,
                modifiers: tail.strip_prefix('.').unwrap_or(""),
                prop_shorthand: false,
            },
        })
    }

    fn with_arg_first(name: &'a str, text: &'a str) -> Self {
        if let Some(inner_and_rest) = text.strip_prefix('[') {
            let mut depth = 1usize;
            for (index, byte) in inner_and_rest.bytes().enumerate() {
                match byte {
                    b'[' => depth += 1,
                    b']' => {
                        depth -= 1;
                        if depth == 0 {
                            let rest = &inner_and_rest[index + 1..];
                            return Self {
                                name,
                                arg: Some(&inner_and_rest[..index]),
                                arg_static: false,
                                modifiers: rest.strip_prefix('.').unwrap_or(""),
                                prop_shorthand: false,
                            };
                        }
                    }
                    _ => {}
                }
            }
            return Self {
                name,
                arg: Some(inner_and_rest),
                arg_static: false,
                modifiers: "",
                prop_shorthand: false,
            };
        }
        let arg_end = text.find('.').unwrap_or(text.len());
        let arg = &text[..arg_end];
        Self {
            name,
            arg: (!arg.is_empty()).then_some(arg),
            arg_static: true,
            modifiers: text[arg_end..].strip_prefix('.').unwrap_or(""),
            prop_shorthand: false,
        }
    }

    /// Whether this spelling is a structural directive the facade consumes
    /// into a scope (`v-if` / `v-else-if` / `v-else` / `v-for`).
    pub(in crate::markup) fn is_structural(&self) -> bool {
        matches!(self.name, "if" | "else-if" | "else" | "for")
    }

    /// Visit the non-empty modifier segments in authored order.
    pub(in crate::markup) fn walk_modifiers(&self, visitor: &mut impl FnMut(&'a str)) {
        if self.prop_shorthand {
            visitor("prop");
        }
        for modifier in self.modifiers.split('.') {
            if !modifier.is_empty() {
                visitor(modifier);
            }
        }
    }
}
