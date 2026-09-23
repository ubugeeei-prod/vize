//! Attribute emission — `pug-attrs@3.0.0` with `pug-runtime@3.0.1`'s
//! `attr` / `classes` / `style` / `escape`, for folded constants under
//! `terse` (the `html` doctype): every `class` source (shorthand `.x` and
//! `class=` alike) merges into one leading `class` attribute, the rest
//! follow in authored order, and pug's duplicate-name checks apply.

use alloc::vec::Vec as StdVec;

use vize_s0::{Span, String, cstr};
use vize_s1::pug::{PugAttrItem, PugTag, PugTagPart};

use super::emit::Emitter;
use super::literal::{Lit, evaluate};

/// `pug_escape`: `"`, `&`, `<`, `>` to entities.
pub(super) fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("&quot;"),
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            other => out.push(other),
        }
    }
    out
}

struct Entry<'s> {
    key: String,
    /// The authored key token, when it can be copied verbatim.
    key_span: Option<Span>,
    value: Lit,
    /// A quoted literal's body, mapped verbatim when it renders unchanged.
    body: Option<(&'s str, Span)>,
    escaped: bool,
    span: Span,
}

/// The body of a quoted literal token and its span.
fn literal_body<'s>(emitter: &Emitter<'_>, value: &vize_s1::Token<'s>) -> Option<(&'s str, Span)> {
    let text = value.text;
    let quote = text
        .chars()
        .next()
        .filter(|ch| matches!(ch, '"' | '\'' | '`'))?;
    let body = text.get(1..)?.strip_suffix(quote)?;
    let start = emitter.offset(value) + 1;
    Some((body, Span::new(start, start + body.len() as u32)))
}

impl Emitter<'_> {
    pub(super) fn attributes(&mut self, tag: &PugTag<'_>) {
        let mut classes: StdVec<(Lit, bool, Span)> = StdVec::new();
        let mut entries: StdVec<Entry<'_>> = StdVec::new();
        let mut names: StdVec<&str> = StdVec::new();
        // How many entries precede the first authored `class` source.
        let mut class_at = None;
        for part in &tag.parts {
            if class_at.is_none() {
                class_at = first_class_offset(part).map(|offset| entries.len() + offset);
            }
            match part {
                PugTagPart::Class(token) => {
                    classes.push((
                        Lit::Str(String::from(sigil_body(token.text))),
                        false,
                        self.span(token),
                    ));
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
                    let value = Lit::Str(String::from(sigil_body(token.text)));
                    let body = Some((sigil_body(token.text), Span::new(span.start + 1, span.end)));
                    entries.push(Entry {
                        key: String::from("id"),
                        key_span: None,
                        value,
                        body,
                        escaped: false,
                        span,
                    });
                }
                PugTagPart::AndAttributes(token) => {
                    let span = self.span(token);
                    self.error(span, String::from(super::refusal::AND_ATTRIBUTES));
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
                        let value = match &attr.value {
                            None => Lit::Bool(true),
                            Some(value) => match evaluate(value.text) {
                                Some(folded) => folded,
                                None => {
                                    let message = cstr!(
                                        "pug attribute `{key}` has an executable value; Vue \
                                         templates accept only constant pug attribute values \
                                         (use `:{key}` for bindings)"
                                    );
                                    let value_span = self.span(value);
                                    self.error(value_span, message);
                                    continue;
                                }
                            },
                        };
                        let escaped = !attr.unescaped();
                        if key == "class" {
                            classes.push((value, escaped, whole));
                            continue;
                        }
                        // A bare attribute takes pug's runtime path,
                        // `pug.attr("KEY", true, …)`: the key is pasted into
                        // a JavaScript string, so it renders cooked, and one
                        // that is no valid string body breaks pug's build.
                        let rendered = if attr.value.is_none() {
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
                        let key_span = (rendered.as_str() == attr.name.text).then_some(span);
                        let body = attr.value.as_ref().and_then(|v| literal_body(self, v));
                        entries.push(Entry {
                            key: rendered,
                            key_span,
                            value,
                            body,
                            escaped,
                            span: whole,
                        });
                    }
                }
            }
        }
        // pug hoists the merged class attribute to the front; the
        // authored-order rendering keeps it where it was written.
        let split = match self.rendering {
            super::PugRendering::Pug => 0,
            super::PugRendering::AuthoredOrder => class_at.unwrap_or(0).min(entries.len()),
        };
        let (before, after) = entries.split_at_checked(split).unwrap_or((&entries, &[]));
        for entry in before {
            self.attribute(entry);
        }
        self.classes(&classes);
        for entry in after {
            self.attribute(entry);
        }
    }

    /// `pug_classes` over the collected sources, then `pug_attr('class')`.
    fn classes(&mut self, classes: &[(Lit, bool, Span)]) {
        let mut joined = String::default();
        for (value, escaped, _) in classes {
            if !value.truthy() {
                continue;
            }
            let name = value.to_js_string();
            let name = if *escaped { escape(&name) } else { name };
            if !joined.is_empty() {
                joined.push(' ');
            }
            joined.push_str(&name);
        }
        let Some((_, _, span)) = classes.first() else {
            return;
        };
        if !joined.is_empty() {
            self.synth(" class=\"", *span);
            self.synth(&joined, *span);
            self.synth("\"", *span);
        }
    }

    /// `pug_attr(key, value, escaped, terse = true)`, with `style`'s
    /// `pug_style` string coercion first.
    fn attribute(&mut self, entry: &Entry<'_>) {
        let value = if entry.key == "style" {
            if entry.value.truthy() {
                Lit::Str(entry.value.to_js_string())
            } else {
                Lit::Str(String::default())
            }
        } else {
            entry.value.clone()
        };
        let text = match value {
            Lit::Bool(false) | Lit::Null | Lit::Undefined => return,
            Lit::Str(ref text)
                if text.is_empty() && matches!(entry.key.as_str(), "class" | "style") =>
            {
                return;
            }
            Lit::Bool(true) => None,
            Lit::Str(text) | Lit::Int(text) => Some(text),
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
            let text = if entry.escaped { escape(&text) } else { text };
            self.synth("=\"", entry.span);
            match entry.body {
                Some((body, span)) if body == text.as_str() => {
                    self.html.push_str(body);
                    self.map.push_verbatim(body.len(), span.start);
                }
                _ => self.synth(&text, entry.span),
            }
            self.synth("\"", entry.span);
        }
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
    match evaluate(&cstr!("\"{key}\"")) {
        Some(Lit::Str(cooked)) => Some(cooked),
        _ => None,
    }
}

/// A `.class` / `#id` shorthand token without its one-byte sigil.
fn sigil_body(text: &str) -> &str {
    text.get(1..).unwrap_or_default()
}
