//! Attribute emission — `pug-attrs@3.0.0` with `pug-runtime@3.0.1`'s
//! `attr` / `classes` / `style` / `escape`, for evaluated values under
//! `terse` (the `html` doctype): every `class` source (shorthand `.x` and
//! `class=` alike) merges into one leading `class` attribute, the rest
//! follow in authored order, and pug's duplicate-name checks apply. A tag
//! with `&attributes` renders through the object format instead
//! ([`super::attr_object`]).

use alloc::vec::Vec as StdVec;

use vize_l0::{Span, String, cstr};
use vize_l1::markup::pug::{PugAttrItem, PugTag, PugTagPart};

use super::PugRendering;
use super::emit::Emitter;
use super::eval;
use super::literal::{cook_body, string_literal};
use super::value::{self, Value};

/// `pug_escape`: `"`, `&`, `<`, `>` to entities.
pub(super) fn escape(value: &str) -> String {
    escape_for(value, PugRendering::Pug)
}

/// [`escape`], or for the authored-order view only what a quoted Vue
/// attribute would read differently (see [`entity`]).
fn escape_for(value: &str, rendering: PugRendering) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        match entity(ch, chars.peek().copied(), rendering) {
            Some(entity) => out.push_str(entity),
            None => out.push(ch),
        }
    }
    out
}

/// The entity written for `ch` (followed by `next`), if any.
///
/// The authored-order view keeps a raw `<`, `>` and an `&` that cannot
/// start a character reference: Vue decodes both spellings to the same
/// value, and the authored bytes keep an expression's columns (#13295).
fn entity(ch: char, next: Option<char>, rendering: PugRendering) -> Option<&'static str> {
    let exact = rendering == PugRendering::Pug;
    match ch {
        '"' => Some("&quot;"),
        '&' if exact || next.is_some_and(|next| next.is_ascii_alphanumeric() || next == '#') => {
            Some("&amp;")
        }
        '<' if exact => Some("&lt;"),
        '>' if exact => Some("&gt;"),
        _ => None,
    }
}

pub(super) struct Entry<'s> {
    pub(super) key: String,
    /// The authored key, as the object format keeps it.
    pub(super) raw_key: &'s str,
    /// The authored key token, when it can be copied verbatim.
    pub(super) key_span: Option<Span>,
    pub(super) value: Value,
    /// A quoted literal's body, mapped verbatim when it renders unchanged.
    pub(super) body: Option<(&'s str, Span)>,
    /// The quote of a cooked literal `body`; `None` for an `#id` body.
    pub(super) quote: Option<char>,
    pub(super) escaped: bool,
    pub(super) span: Span,
}

/// A tag's evaluated attribute sources.
pub(super) struct Collected<'s> {
    /// Every `class` source: value, escaped, span.
    pub(super) classes: StdVec<(Value, bool, Span)>,
    pub(super) entries: StdVec<Entry<'s>>,
    /// How many entries precede the first authored `class` source.
    pub(super) class_at: Option<usize>,
    /// `&attributes(…)` expressions and their spans.
    pub(super) blocks: StdVec<(&'s str, Span)>,
}

impl Collected<'_> {
    /// pug's `attrs.length`: any class, id or attribute.
    pub(super) fn has_attrs(&self) -> bool {
        !self.classes.is_empty() || !self.entries.is_empty()
    }

    /// `pug_classes` over the class sources, each escaped as authored.
    pub(super) fn class_names(&self) -> String {
        let mut joined = String::default();
        for (value, escaped, _) in &self.classes {
            let name = value::classes(value);
            if name.is_empty() {
                continue;
            }
            let name = if *escaped { escape(&name) } else { name };
            if !joined.is_empty() {
                joined.push(' ');
            }
            joined.push_str(&name);
        }
        joined
    }
}

/// The body of a quoted literal token, its span and its quote.
fn literal_body<'s>(
    emitter: &Emitter<'_>,
    value: &vize_l1::markup::Token<'s>,
) -> Option<(&'s str, Span, char)> {
    let text = value.text;
    if !emitter.is_authored(text) {
        return None;
    }
    let quote = text
        .chars()
        .next()
        .filter(|ch| matches!(ch, '"' | '\'' | '`'))?;
    let body = text.get(1..)?.strip_suffix(quote)?;
    let start = emitter.offset(value) + 1;
    Some((body, Span::new(start, start + body.len() as u32), quote))
}

impl<'t> Emitter<'t> {
    pub(super) fn attributes(&mut self, tag: &'t PugTag<'t>) {
        let collected = self.collect(tag);
        if !collected.blocks.is_empty() {
            self.attribute_blocks(&collected);
            return;
        }
        // pug hoists the merged class attribute to the front; the
        // authored-order rendering keeps it where it was written.
        let entries = &collected.entries;
        let split = match self.rendering {
            super::PugRendering::Pug => 0,
            super::PugRendering::AuthoredOrder => {
                collected.class_at.unwrap_or(0).min(entries.len())
            }
        };
        let (before, after) = entries.split_at_checked(split).unwrap_or((entries, &[]));
        for entry in before {
            self.attribute(entry);
        }
        if let Some((_, _, span)) = collected.classes.first() {
            let joined = collected.class_names();
            if !joined.is_empty() {
                self.synth(" class=\"", *span);
                self.synth(&joined, *span);
                self.synth("\"", *span);
            }
        }
        for entry in after {
            self.attribute(entry);
        }
    }

    /// Evaluate a tag's head parts, reporting pug's duplicate-name errors
    /// and every value without a build-time value.
    pub(super) fn collect(&mut self, tag: &'t PugTag<'t>) -> Collected<'t> {
        let mut collected = Collected {
            classes: StdVec::new(),
            entries: StdVec::new(),
            class_at: None,
            blocks: StdVec::new(),
        };
        let mut names: StdVec<&str> = StdVec::new();
        for part in &tag.parts {
            if collected.class_at.is_none() {
                collected.class_at =
                    first_class_offset(part).map(|offset| collected.entries.len() + offset);
            }
            match part {
                PugTagPart::Class(token) => {
                    let value = Value::str(sigil_body(token.text));
                    collected.classes.push((value, false, self.span(token)));
                }
                PugTagPart::Id(token) => {
                    let span = self.span(token);
                    if names.contains(&"id") {
                        self.error(
                            span,
                            String::from("Duplicate attribute \"id\" is not allowed."),
                        );
                    }
                    names.push("id");
                    let body = sigil_body(token.text);
                    collected.entries.push(Entry {
                        key: String::from("id"),
                        raw_key: "id",
                        key_span: None,
                        value: Value::str(body),
                        body: self
                            .is_authored(token.text)
                            .then(|| (body, Span::new(span.start + 1, span.end))),
                        quote: None,
                        escaped: false,
                        span,
                    });
                }
                PugTagPart::AndAttributes(token) => {
                    let source = token.text.get("&attributes".len()..).unwrap_or_default();
                    let source = source
                        .strip_prefix('(')
                        .and_then(|inner| inner.strip_suffix(')'))
                        .unwrap_or(source);
                    collected.blocks.push((source, self.span(token)));
                }
                PugTagPart::Attrs(group) => {
                    for item in &group.items {
                        let PugAttrItem::Attr(attr) = item else {
                            continue;
                        };
                        let key = attr.key();
                        let span = self.span(&attr.name);
                        // Synthesized bytes map to the whole `name=value`.
                        let whole = attr
                            .value
                            .as_ref()
                            .map_or(span, |value| Span::new(span.start, self.span(value).end));
                        if key != "class" {
                            if names.contains(&key) {
                                let message =
                                    cstr!("Duplicate attribute \"{key}\" is not allowed.");
                                self.error(span, message);
                            }
                            names.push(key);
                        }
                        // A bare attribute, and a value that reads a template
                        // binding, take pug's runtime path.
                        let (value, constant) = match &attr.value {
                            None => (Value::Bool(true), false),
                            Some(value) => match eval::expression(&self.scope, value.text) {
                                Ok(evaluated) if evaluated.value.has_function() => {
                                    let value_span = self.span(value);
                                    let reason = super::refusal::FUNCTION_VALUE;
                                    self.error(value_span, super::refusal::attribute(key, reason));
                                    continue;
                                }
                                Ok(evaluated) => (evaluated.value, evaluated.constant),
                                Err(reason) => {
                                    let value_span = self.span(value);
                                    self.error(value_span, super::refusal::attribute(key, &reason));
                                    continue;
                                }
                            },
                        };
                        let escaped = !attr.unescaped();
                        if key == "class" {
                            collected.classes.push((value, escaped, whole));
                            continue;
                        }
                        // The runtime path, `pug.attr("KEY", value, …)`, pastes
                        // the key into a JavaScript string, so it renders
                        // cooked, and one that is no valid string body breaks
                        // pug's build.
                        let rendered = if !constant {
                            match cook_double_quoted(key) {
                                Some(cooked) => cooked,
                                None => {
                                    let message =
                                        cstr!("pug cannot compile the attribute name `{key}`");
                                    self.error(span, message);
                                    continue;
                                }
                            }
                        } else {
                            String::from(key)
                        };
                        let key_span = (rendered.as_str() == attr.name.text
                            && self.is_authored(attr.name.text))
                        .then_some(span);
                        let literal = attr.value.as_ref().and_then(|v| literal_body(self, v));
                        collected.entries.push(Entry {
                            key: rendered,
                            raw_key: key,
                            key_span,
                            value,
                            body: literal.map(|(body, span, _)| (body, span)),
                            quote: literal.map(|(.., quote)| quote),
                            escaped,
                            span: whole,
                        });
                    }
                }
            }
        }
        collected
    }

    /// `pug_attr(key, value, escaped, terse = true)`, with `style`'s
    /// `pug_style` string coercion first.
    pub(super) fn attribute(&mut self, entry: &Entry<'_>) {
        let value = if entry.key == "style" {
            Value::Str(value::style(&entry.value))
        } else {
            entry.value.clone()
        };
        let (text, quote) = match value {
            Value::Bool(false) | Value::Null | Value::Undefined => return,
            ref other if !other.truthy() && matches!(entry.key.as_str(), "class" | "style") => {
                return;
            }
            Value::Bool(true) => (None, "\""),
            Value::Str(text) => (
                Some(if entry.escaped {
                    escape_for(&text, self.rendering)
                } else {
                    text
                }),
                "\"",
            ),
            other => {
                let Some(json) = other.json() else {
                    return;
                };
                if entry.escaped {
                    (Some(escape_for(&json, self.rendering)), "\"")
                } else if json.contains('"') {
                    (
                        Some(String::from(json.replace('\'', "&#39;").as_str())),
                        "'",
                    )
                } else {
                    (Some(json), "\"")
                }
            }
        };
        self.synth(" ", entry.span);
        match entry.key_span {
            Some(span) => {
                self.html.push_str(&entry.key);
                self.map.push_verbatim(entry.key.len(), span.start);
            }
            None => self.synth(&entry.key, entry.span),
        }
        if let Some(text) = text {
            self.synth(if quote == "'" { "='" } else { "=\"" }, entry.span);
            match entry.body {
                Some((body, span)) if body == text.as_str() => {
                    self.html.push_str(body);
                    self.map.push_verbatim(body.len(), span.start);
                }
                Some((body, span)) if self.cooked_body(entry, body, span, &text) => {}
                _ => self.synth(&text, entry.span),
            }
            self.synth(quote, entry.span);
        }
    }

    /// A literal `body` whose cooked, escaped chars render exactly `text`:
    /// unchanged runs stay verbatim and each rewritten char maps to the
    /// bytes that spell it, so the value keeps its authored columns (#13295).
    fn cooked_body(&mut self, entry: &Entry<'_>, body: &str, span: Span, text: &str) -> bool {
        let Some(quote) = entry.quote else {
            return false;
        };
        let mut chars = StdVec::with_capacity(body.len());
        if cook_body(body, quote, |ch, raw| chars.push((ch, raw))).is_none() {
            return false;
        }
        let escape = entry.escaped.then_some(self.rendering);
        let mut buffer = [0; 4];
        let mut rendered = 0;
        for (index, (ch, _)) in chars.iter().enumerate() {
            let next = chars.get(index + 1).map(|(next, _)| *next);
            let piece = rendered_char(*ch, next, escape, &mut buffer);
            if text.get(rendered..rendered + piece.len()) != Some(piece) {
                return false;
            }
            rendered += piece.len();
        }
        if rendered != text.len() {
            return false;
        }
        if !self.include_output(text.len(), entry.span) {
            return true;
        }
        let mut run: Option<core::ops::Range<usize>> = None;
        for (index, (ch, raw)) in chars.iter().cloned().enumerate() {
            let next = chars.get(index + 1).map(|(next, _)| *next);
            let piece = rendered_char(ch, next, escape, &mut buffer);
            if body.get(raw.clone()) == Some(piece) {
                match run.take() {
                    Some(open) if open.end == raw.start => run = Some(open.start..raw.end),
                    open => {
                        if let Some(open) = open {
                            self.body_run(body, span, open);
                        }
                        run = Some(raw);
                    }
                }
                continue;
            }
            if let Some(open) = run.take() {
                self.body_run(body, span, open);
            }
            self.html.push_str(piece);
            self.map.push_synth(
                piece.len(),
                span.start + raw.start as u32,
                span.start + raw.end as u32,
            );
        }
        if let Some(open) = run {
            self.body_run(body, span, open);
        }
        true
    }

    /// Copy `body[run]` verbatim.
    fn body_run(&mut self, body: &str, span: Span, run: core::ops::Range<usize>) {
        let text = body.get(run.clone()).unwrap_or_default();
        self.html.push_str(text);
        self.map
            .push_verbatim(text.len(), span.start + run.start as u32);
    }
}

/// One cooked char (followed by `next`) as `pug_attr` renders it,
/// escaped for `escape` when the attribute is.
fn rendered_char(
    ch: char,
    next: Option<char>,
    escape: Option<PugRendering>,
    buffer: &mut [u8; 4],
) -> &str {
    match escape.and_then(|rendering| entity(ch, next, rendering)) {
        Some(entity) => entity,
        None => ch.encode_utf8(buffer),
    }
}

/// Within one head part, how many entries precede its first `class`
/// source (`None` when the part has none). On a template that derives
/// without errors every non-class attribute becomes an entry.
fn first_class_offset(part: &PugTagPart<'_>) -> Option<usize> {
    match part {
        PugTagPart::Class(_) => Some(0),
        PugTagPart::Attrs(group) => {
            let keys = group.items.iter().filter_map(|item| match item {
                PugAttrItem::Attr(attr) => Some(attr.key()),
                PugAttrItem::Unexpected(_) => None,
            });
            keys.clone()
                .any(|key| key == "class")
                .then(|| keys.take_while(|key| *key != "class").count())
        }
        PugTagPart::Id(_) | PugTagPart::AndAttributes(_) => None,
    }
}

/// The cooked value of `"key"` as a JavaScript string literal.
fn cook_double_quoted(key: &str) -> Option<String> {
    string_literal(&cstr!("\"{key}\""), '"')
}

/// A `.class` / `#id` shorthand token without its one-byte sigil.
fn sigil_body(text: &str) -> &str {
    text.get(1..).unwrap_or_default()
}
