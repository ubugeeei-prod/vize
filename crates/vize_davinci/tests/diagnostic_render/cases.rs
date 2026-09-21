//! The TS-53 renderer cases: realistic Vue diagnostics chosen so that, between
//! them, every layout path renders — single and hanging labels, multi-line
//! spans sharing and crossing margin columns, elision, both fix styles, wide
//! and control characters, CRLF, end of file, every severity, codes and none.
//!
//! Producer text arrives localized (Patina localizes at report time), so each
//! case supplies its messages in all three locales through [`tr`].

mod layout;
mod markup;
mod text;

use vize_davinci::diagnostic::{Diagnostic, DiagnosticPart, PartKind, Severity, Stage};
use vize_s0::Span;
use vize_s0::i18n::Locale;

/// Diagnostics as a case produces them: `(code, diagnostic)`.
pub type Produced = Vec<(Option<&'static str>, Diagnostic)>;

pub struct Case {
    pub name: &'static str,
    pub path: &'static str,
    pub source: &'static str,
    pub diagnostics: fn(Locale) -> Produced,
}

pub const ALL: &[&Case] = &[
    &markup::V_FOR_KEY,
    &markup::DUPLICATE_ATTRIBUTE,
    &markup::V_IF_WITH_V_FOR,
    &text::WIDE_CHARACTERS,
    &layout::MULTILINE_ELEMENT,
    &layout::MULTILINE_ATTRIBUTE,
    &layout::NESTED_MULTILINE,
    &markup::SHORTHAND_FIX,
    &text::CONTROL_CHARACTERS,
    &text::END_OF_FILE,
    &markup::SEVERITIES,
    &layout::LONG_SPAN,
    &layout::MULTI_LINE_LABELS,
    &text::EDGE_CASES,
];

/// The text for `locale`.
pub fn tr(locale: Locale, en: &'static str, ja: &'static str, zh: &'static str) -> &'static str {
    match locale {
        Locale::En => en,
        Locale::Ja => ja,
        Locale::Zh => zh,
    }
}

/// The `nth` (zero-based) occurrence of `needle` in `source`.
pub fn nth(source: &str, needle: &str, nth: usize) -> Span {
    let mut from = 0;
    for _ in 0..nth {
        from += source[from..].find(needle).expect("occurrence exists") + needle.len();
    }
    let start = from + source[from..].find(needle).expect("occurrence exists");
    Span::new(start as u32, (start + needle.len()) as u32)
}

/// The first occurrence of `needle` in `source`.
pub fn span(source: &str, needle: &str) -> Span {
    nth(source, needle, 0)
}

/// From the start of `open` to the end of the first `close` after it.
pub fn between(source: &str, open: &str, close: &str) -> Span {
    let start = span(source, open).start as usize;
    let end = start + source[start..].find(close).expect("close exists") + close.len();
    Span::new(start as u32, end as u32)
}

/// The empty span just after `span`.
pub const fn after(span: Span) -> Span {
    Span::new(span.end, span.end)
}

pub fn diagnostic(severity: Severity, span: Span, message: &str) -> Diagnostic {
    Diagnostic::new(severity, Stage::Semantic, span, message)
}

pub fn primary(span: Span, label: &str) -> DiagnosticPart {
    DiagnosticPart::new(PartKind::Primary, span, label)
}

pub fn secondary(span: Span, label: &str) -> DiagnosticPart {
    DiagnosticPart::new(PartKind::Secondary, span, label)
}

pub fn help(span: Span, text: &str) -> DiagnosticPart {
    DiagnosticPart::new(PartKind::Help, span, text)
}

pub fn fix(span: Span, replacement: &str) -> DiagnosticPart {
    DiagnosticPart::new(PartKind::Suggestion, span, replacement)
}
