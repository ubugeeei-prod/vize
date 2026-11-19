//! The jade 1.x profile (`<template lang="jade">`).
//!
//! Vue's toolchain renders `lang="jade"` with the legacy `jade` package,
//! not pug. Where jade 1.x and the pinned pug 3 differ, the profile keeps
//! the author's intent and says so, never silently:
//!
//! - `- each` / `- for`: jade's iteration, accepted (pug 3 refuses the
//!   dash), with a fix that drops it.
//! - a bodiless `mixin name(args)`: jade's legacy call, accepted, with a
//!   fix to `+name(args)`.
//! - `each x of xs`: iterated as pug 3 does, warning that jade 1.x renders
//!   a literal `<each>` element instead.
//! - `#{…}` inside a quoted attribute value: kept as written (pug 3), with a
//!   warning that jade 1.x interpolates it at build time — with no component
//!   data, to `undefined` — and a fix that binds the attribute.
//!
//! Every warning names `vue/prefer-pug-lang`, the lint rule that reports
//! them with the template's `lang`.

use vize_l0::{Span, String, cstr};
use vize_l1::markup::pug::{PugAttr, PugRefused};

use super::emit::Emitter;

/// How to configure or silence a jade profile warning.
pub(super) const HINT: &str = "Rewrite it in pug 3 syntax and set the template's lang to \"pug\"; \
                               the `vue/prefer-pug-lang` lint rule reports every jade-only \
                               construct left and can be configured or turned off.";

impl<'t> Emitter<'t> {
    /// `- each` / `- for` under the jade profile.
    pub(super) fn jade_dash(&mut self, head: &'t PugRefused<'t>) {
        if !self.is_jade() {
            return;
        }
        let leading = head.head.leading;
        let Some(dash) = leading.rfind('-') else {
            return;
        };
        let tail = leading.get(dash..).unwrap_or_default();
        if !tail.get(1..).unwrap_or_default().bytes().all(|b| b == b' ') {
            return;
        }
        let span = self.text_span(tail);
        let keyword = if head.head.text.starts_with("for") {
            "for"
        } else {
            "each"
        };
        let message = cstr!(
            "`- {keyword}` is jade 1.x iteration; pug 3 removed the dash. Vize iterates it as \
             jade does."
        );
        self.jade_warning(span, message, Some((span, String::default())));
    }

    /// `each x of xs` under the jade profile.
    pub(super) fn jade_each_of(&mut self, head: &'t PugRefused<'t>) {
        if !self.is_jade() {
            return;
        }
        let span = self.span(&head.head);
        let message = String::from(
            "jade 1.x has no `each … of`: Vue's jade toolchain renders a literal `<each>` element \
             here. Vize iterates it as pug 3 does.",
        );
        self.jade_warning(span, message, None);
    }

    /// A bodiless `mixin name(args)`: jade 1.x calls the mixin. Returns
    /// whether the profile reads it as a call.
    pub(super) fn jade_legacy_call(&mut self, definition: &'t PugRefused<'t>) -> bool {
        if !self.is_jade() || !definition.block.is_empty() {
            return false;
        }
        let text = definition.head.text;
        let call = text
            .strip_prefix("mixin")
            .map(|rest| rest.trim_start_matches(' '))
            .unwrap_or(text);
        let span = self.span(&definition.head);
        let message = cstr!(
            "`{text}` without a body is jade 1.x's legacy mixin call; pug 3 removed it. Vize \
             calls the mixin as jade does: write `+{call}`."
        );
        self.jade_warning(span, message, Some((span, cstr!("+{call}"))));
        true
    }

    /// `#{…}` inside a quoted attribute value under the jade profile.
    pub(super) fn jade_attribute(&mut self, attr: &'t PugAttr<'t>) {
        if !self.is_jade() {
            return;
        }
        let Some(value) = attr.value.as_ref() else {
            return;
        };
        let text = value.text;
        let Some(quote) = text.chars().next().filter(|ch| matches!(ch, '"' | '\'')) else {
            return;
        };
        let Some(body) = text.get(1..).and_then(|rest| rest.strip_suffix(quote)) else {
            return;
        };
        let mut pieces = alloc::vec::Vec::new();
        let mut from = 0;
        while let Some(at) = body.get(from..).and_then(|rest| rest.find("#{")) {
            let start = from + at;
            let escaped = start > 0 && body.as_bytes().get(start - 1) == Some(&b'\\');
            let Some(close) = body.get(start + 2..).and_then(|rest| rest.find('}')) else {
                break;
            };
            let end = start + 2 + close + 1;
            if !escaped {
                pieces.push((start, end));
            }
            from = end;
        }
        let Some(&(first, _)) = pieces.first() else {
            return;
        };
        let value_span = self.span(value);
        let body_start = value_span.start + 1;
        let key = attr.key();
        let fix = bound_attribute(key, body, attr.unescaped()).map(|bound| {
            let whole = Span::new(self.span(&attr.name).start, value_span.end);
            (whole, bound)
        });
        for (index, &(start, end)) in pieces.iter().enumerate() {
            let span = Span::new(body_start + start as u32, body_start + end as u32);
            let written = body.get(start..end).unwrap_or_default();
            let message = cstr!(
                "jade 1.x interpolates `{written}` in the `{key}` attribute at build time, with no \
                 component data, so Vue's jade toolchain renders \"undefined\" there. Vize keeps \
                 the text as written; bind the attribute to interpolate on every render."
            );
            // One fix rewrites the whole attribute; it rides on the first.
            let fix = if index == 0 && first == start {
                fix.clone()
            } else {
                None
            };
            self.jade_warning(span, message, fix);
        }
    }
}

/// `key="a#{b}c"` as `:key="`a${b}c`"`, when that is a faithful rewrite:
/// a plain escaped attribute whose body holds no backtick, `${`, escape or
/// double quote.
fn bound_attribute(key: &str, body: &str, unescaped: bool) -> Option<String> {
    let plain_key = key
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'));
    if unescaped || !plain_key || body.contains(['`', '"', '\\']) || body.contains("${") {
        return None;
    }
    let literal = body.replace("#{", "${");
    Some(cstr!(":{key}=\"`{literal}`\""))
}

#[cfg(test)]
mod tests;
