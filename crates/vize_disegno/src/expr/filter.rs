//! `vue.filter` - Vue 2 pipe-filter expressions
//! (`{{ msg | capitalize }}`, `:id="raw | formatId"`).
//!
//! Filters were removed in Vue 3, where `|` is bitwise-OR. The payload
//! is a dialect expression, not a binding: interpolations and bind
//! values both carry one, so it sits on [`super::ExprRef`] rather than
//! [`crate::op::BindingOp`]. Semantics are the Vue 2 rewrite
//! (`a | f` → `_filter_f(a)`); the S2 pass that legalizes this payload
//! is the one that emits that spelling. Until then the payload is the
//! authored chain, never a guessed JS parse of the whole text — `|` as
//! bitwise-OR would be the Vue 3 reading of the same bytes.

use vize_carton::{Allocator, Span, Vec};

use super::{ExprRef, JsExpr, OpaqueExpr};

/// One Vue 2 filter application in a chain (`capitalize`, `f(b, c)`).
#[derive(Debug, Clone, Copy)]
pub struct VueFilterApp<'a> {
    /// The filter's identifier (`capitalize`, `format-id`).
    pub name: &'a str,
    /// Call-style arguments inside the parentheses, excluding the
    /// closing `)`; `None` for a bare name. Empty `Some("")` is `f()`.
    pub args: Option<&'a str>,
    /// The authored segment after the pipe, trimmed.
    pub raw: &'a str,
    /// The segment's range in the compiled file.
    pub span: Span,
}

/// `vue.filter` - a base expression with an ordered filter chain
/// (outermost last), covering the whole authored pipe text.
#[derive(Debug)]
pub struct VueFilterExpr<'a> {
    /// The exact authored text, pipes included.
    pub source: &'a str,
    /// The authored range of [`Self::source`].
    pub span: Span,
    /// The base expression (left of the first top-level pipe), admitted
    /// through the shared JS rule.
    pub base: ExprRef<'a>,
    /// Filters in authored order, outermost last.
    pub filters: Vec<'a, VueFilterApp<'a>>,
}

impl<'a> VueFilterExpr<'a> {
    /// Split `source` as a Vue 2 filter chain and admit the base.
    ///
    /// Returns `None` when there is no top-level pipe, or when any
    /// filter name is not an identifier — the shipped rewrite bails
    /// out and leaves the original expression untouched, and so do we.
    /// Total over the base: refused base text becomes the classified
    /// escape, never a panic.
    #[must_use]
    pub fn parse_in(allocator: &'a Allocator, source: &'a str, span: Span) -> Option<&'a Self> {
        let split = split_filters(source)?;
        let mut filters = Vec::new_in(&allocator);
        for segment in split.filters {
            filters.push(parse_app(segment, span)?);
        }
        if filters.is_empty() {
            return None;
        }
        let base_off = u32::try_from(split.base_offset).unwrap_or(u32::MAX);
        let base_start = span.start.saturating_add(base_off);
        let base_len = u32::try_from(split.base.len()).unwrap_or(u32::MAX);
        let base_span = Span::new(base_start, base_start.saturating_add(base_len));
        let base = match JsExpr::parse_in(allocator, split.base, base_span) {
            Ok(js) => ExprRef::Js(js),
            Err(reason) => ExprRef::Opaque(allocator.alloc(OpaqueExpr {
                reason,
                source: split.base,
                span: base_span,
            })),
        };
        Some(allocator.alloc(Self {
            source,
            span,
            base,
            filters,
        }))
    }
}

/// One trimmed filter segment and its byte offset in the original text.
struct Segment<'a> {
    text: &'a str,
    offset: usize,
}

struct Split<'a> {
    base: &'a str,
    base_offset: usize,
    filters: alloc::vec::Vec<Segment<'a>>,
}

/// Top-level `|` split, mirroring `@vue/compiler-core`'s `parseFilter`:
/// pipes inside strings, template literals, regex literals, or
/// `()[]{}` nesting are not separators, and `||` is never a filter.
fn split_filters(exp: &str) -> Option<Split<'_>> {
    let bytes = exp.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    let mut in_template = false;
    let mut in_regex = false;
    let mut curly: u32 = 0;
    let mut square: u32 = 0;
    let mut paren: u32 = 0;
    let mut last_filter_index = 0usize;
    let mut base: Option<Segment<'_>> = None;
    let mut filters = alloc::vec::Vec::new();
    let mut prev: u8 = 0;
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        if in_single {
            if c == b'\'' && prev != b'\\' {
                in_single = false;
            }
        } else if in_double {
            if c == b'"' && prev != b'\\' {
                in_double = false;
            }
        } else if in_template {
            if c == b'`' && prev != b'\\' {
                in_template = false;
            }
        } else if in_regex {
            if c == b'/' && prev != b'\\' {
                in_regex = false;
            }
        } else if c == b'|'
            && bytes.get(i + 1).copied() != Some(b'|')
            && (i == 0 || bytes[i - 1] != b'|')
            && curly == 0
            && square == 0
            && paren == 0
        {
            if base.is_none() {
                last_filter_index = i + 1;
                base = Some(trimmed_segment(exp, 0, i));
            } else {
                filters.push(trimmed_segment(exp, last_filter_index, i));
                last_filter_index = i + 1;
            }
        } else {
            match c {
                b'"' => in_double = true,
                b'\'' => in_single = true,
                b'`' => in_template = true,
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => square += 1,
                b']' => square = square.saturating_sub(1),
                b'{' => curly += 1,
                b'}' => curly = curly.saturating_sub(1),
                _ => {}
            }
            if c == b'/' {
                let mut j = i as isize - 1;
                let mut p: Option<char> = None;
                while j >= 0 {
                    let pc = exp[j as usize..].chars().next().unwrap_or(' ');
                    if pc != ' ' {
                        p = Some(pc);
                        break;
                    }
                    j -= 1;
                }
                if p.is_none_or(|pc| !is_valid_division_char(pc)) {
                    in_regex = true;
                }
            }
        }
        prev = c;
        i += 1;
    }
    let base = base?;
    if last_filter_index != 0 {
        filters.push(trimmed_segment(exp, last_filter_index, exp.len()));
    }
    if filters.is_empty() {
        return None;
    }
    Some(Split {
        base: base.text,
        base_offset: base.offset,
        filters,
    })
}

fn trimmed_segment(exp: &str, start: usize, end: usize) -> Segment<'_> {
    let raw = &exp[start..end];
    let text = raw.trim();
    let offset = start + raw.len() - raw.trim_start().len();
    Segment { text, offset }
}

fn parse_app(segment: Segment<'_>, parent: Span) -> Option<VueFilterApp<'_>> {
    let raw = segment.text;
    if raw.is_empty() {
        return None;
    }
    let (name, args) = match raw.find('(') {
        None => (raw, None),
        Some(idx) => {
            let name = raw[..idx].trim();
            let inner = raw[idx + 1..].strip_suffix(')').unwrap_or(&raw[idx + 1..]);
            (name, Some(inner))
        }
    };
    if !is_filter_name(name) {
        return None;
    }
    let start = parent
        .start
        .saturating_add(u32::try_from(segment.offset).ok()?);
    let end = start.saturating_add(u32::try_from(raw.len()).ok()?);
    Some(VueFilterApp {
        name,
        args,
        raw,
        span: Span::new(start, end),
    })
}

/// Vue resolves whatever name appears; require a non-empty identifier
/// (with `-` mapped to `_`, matching the shipped `toValidAssetId` gate)
/// so we never emit `_filter_(` for malformed input.
fn is_filter_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' && first != '$' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$' || c == '-')
}

fn is_valid_division_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || matches!(c, ')' | '.' | '+' | '-' | '$' | ']')
}

const _: () = assert!(!core::mem::needs_drop::<VueFilterExpr<'static>>());
const _: () = assert!(!core::mem::needs_drop::<VueFilterApp<'static>>());

/// See [`crate::op`] for the guard rationale.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<VueFilterApp<'_>>() == 56);
    assert!(core::mem::size_of::<VueFilterExpr<'_>>() == 64);
};

#[cfg(test)]
mod tests {
    use super::{VueFilterExpr, split_filters};
    use vize_carton::{Allocator, Span};

    #[test]
    fn no_pipe_is_none() {
        assert!(split_filters("message").is_none());
        assert!(split_filters("a + b").is_none());
        assert!(split_filters("a || b").is_none());
    }

    #[test]
    fn nested_and_quoted_pipes_are_not_filters() {
        assert!(split_filters("'x | y'").is_none());
        assert!(split_filters("`x | y`").is_none());
        assert!(split_filters("[a | b]").is_none());
        assert!(split_filters("f(a | b)").is_none());
        assert!(split_filters("{ a: b | c }").is_none());
    }

    #[test]
    fn a_chain_splits_base_then_filters() {
        let split = split_filters("msg | capitalize | f(b)").expect("chain");
        assert_eq!(split.base, "msg");
        assert_eq!(split.filters.len(), 2);
        assert_eq!(split.filters[0].text, "capitalize");
        assert_eq!(split.filters[1].text, "f(b)");
    }

    #[test]
    fn parse_in_admits_the_base_and_keeps_call_args() {
        let allocator = Allocator::default();
        let expr = VueFilterExpr::parse_in(&allocator, "raw | formatId(x)", Span::new(0, 17))
            .expect("filter");
        assert_eq!(expr.source, "raw | formatId(x)");
        assert_eq!(expr.base.source(), "raw");
        assert_eq!(expr.filters.len(), 1);
        assert_eq!(expr.filters[0].name, "formatId");
        assert_eq!(expr.filters[0].args, Some("x"));
    }

    #[test]
    fn a_malformed_filter_name_refuses_the_chain() {
        let allocator = Allocator::default();
        assert!(VueFilterExpr::parse_in(&allocator, "a | ", Span::new(0, 4)).is_none());
    }
}
