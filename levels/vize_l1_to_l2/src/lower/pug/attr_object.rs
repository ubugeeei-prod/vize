//! `&attributes` and a mixin call's `attributes`: `pug-attrs@3.0.0`'s
//! object format, `pug_merge` and `pug_attrs` from `pug-runtime@3.0.1`.
//!
//! A tag with `&attributes(…)` renders `pug_attrs(pug_merge([{own
//! attributes}, …blocks]), terse)`; a call passes the same object (or its
//! lone block, unmerged) to the mixin as `attributes`. `pug_merge` writes
//! into its first object, so a merge into an object the template can read
//! again is refused rather than modelled.

use alloc::vec::Vec as StdVec;

use vize_l0::{Span, String};

use super::attrs::{Collected, Entry, escape};
use super::emit::Emitter;
use super::eval::{self, Refusal};
use super::ops::is_array_index;
use super::value::{self, Value};

/// An attribute object with the span each key came from.
pub(super) type Attrs = StdVec<(String, Value, Span)>;

/// `pug_escape`: a value whose text needs no escaping is returned as it
/// is (non-strings included); otherwise its escaped text.
fn escape_value(value: Value) -> Value {
    let text = value.to_js_string();
    if text.contains(['"', '&', '<', '>']) {
        Value::Str(escape(&text))
    } else {
        value
    }
}

fn set(attrs: &mut Attrs, key: &str, value: Value, span: Span) {
    match attrs.iter_mut().find(|(bound, ..)| bound == key) {
        Some((_, slot, at)) => {
            *slot = value;
            *at = span;
        }
        None => attrs.push((String::from(key), value, span)),
    }
}

/// `pug_style` with the `;` `pug_merge` appends.
fn style_run(value: &Value) -> String {
    let mut style = value::style(value);
    if !style.is_empty() && !style.ends_with(';') {
        style.push(';');
    }
    style
}

/// The own enumerable keys `for…in` visits.
fn entries_of(value: Value) -> Result<StdVec<(String, Value)>, Refusal> {
    match value {
        Value::Object(entries) => Ok(entries),
        Value::Str(text) if !text.is_empty() => Err(String::from(
            "`&attributes` would spread a string's characters; pass an object",
        )),
        Value::Array(items) if !items.is_empty() => Err(String::from(
            "`&attributes` would spread an array's indices; pass an object",
        )),
        _ => Ok(StdVec::new()),
    }
}

/// `pug_merge(target, source)`: `class` concatenates, `style` joins,
/// the rest overwrite.
fn merge(target: &mut Attrs, source: StdVec<(String, Value)>, span: Span) {
    for (key, value) in source {
        match key.as_str() {
            "class" => {
                let mut list = match target.iter().find(|(bound, ..)| bound == "class") {
                    Some((_, Value::Array(items), _)) => items.clone(),
                    Some((_, value, _)) if value.truthy() => alloc::vec![value.clone()],
                    _ => StdVec::new(),
                };
                match value {
                    Value::Array(items) => list.extend(items),
                    value if value.truthy() => list.push(value),
                    _ => {}
                }
                let at = target
                    .iter()
                    .find(|(bound, ..)| bound == "class")
                    .map_or(span, |(.., at)| *at);
                set(target, "class", Value::Array(list), at);
            }
            "style" => {
                let mut style = target
                    .iter()
                    .find(|(bound, ..)| bound == "style")
                    .map_or_else(String::default, |(_, value, _)| style_run(value));
                style.push_str(&style_run(&value));
                set(target, "style", Value::Str(style), span);
            }
            _ => set(target, &key, value, span),
        }
    }
}

impl<'t> Emitter<'t> {
    /// The tag's own attributes in pug-attrs' object format: the merged
    /// class string first, then each attribute, escaped values through
    /// `pug_escape`.
    fn own_object(&mut self, collected: &Collected<'_>) -> Option<Attrs> {
        let mut attrs = Attrs::new();
        if let Some((_, _, span)) = collected.classes.first() {
            attrs.push((
                String::from("class"),
                Value::Str(collected.class_names()),
                *span,
            ));
        }
        for Entry {
            raw_key,
            value,
            escaped,
            span,
            ..
        } in &collected.entries
        {
            if is_array_index(raw_key) {
                let reason = "pug's attribute object would list it before the others";
                self.error(*span, super::refusal::attribute(raw_key, reason));
                return None;
            }
            let value = if *raw_key == "style" {
                Value::Str(value::style(value))
            } else {
                value.clone()
            };
            let value = if *escaped { escape_value(value) } else { value };
            set(&mut attrs, raw_key, value, *span);
        }
        Some(attrs)
    }

    /// Each `&attributes(…)` value, in order; `None` after an error.
    fn block_values(&mut self, collected: &Collected<'_>) -> Option<StdVec<(Value, Span)>> {
        let mut values = StdVec::with_capacity(collected.blocks.len());
        for (source, span) in &collected.blocks {
            match eval::expression(&self.scope, source) {
                Ok(evaluated) => values.push((evaluated.value, *span)),
                Err(reason) => {
                    self.error(*span, super::refusal::and_attributes(&reason));
                    return None;
                }
            }
        }
        Some(values)
    }

    /// `pug_merge([own, …blocks])` as pug builds it; with no own
    /// attributes the first block is the merge target, so it must be a
    /// fresh object literal.
    fn merged(&mut self, collected: &Collected<'_>, own: Option<Attrs>) -> Option<Attrs> {
        let values = self.block_values(collected)?;
        let mut values = values.into_iter();
        let mut target = match own {
            Some(own) => own,
            None => {
                let (first, span) = values.next()?;
                let literal = collected
                    .blocks
                    .first()
                    .is_some_and(|(source, _)| eval::is_object_literal(source));
                if values.len() > 0 && !literal {
                    let reason = "pug_merge would modify the first object in place";
                    self.error(span, super::refusal::and_attributes(reason));
                    return None;
                }
                self.entries(first, span)?
                    .into_iter()
                    .map(|(key, value)| (key, value, span))
                    .collect()
            }
        };
        for (value, span) in values {
            let entries = self.entries(value, span)?;
            merge(&mut target, entries, span);
        }
        Some(target)
    }

    fn entries(&mut self, value: Value, span: Span) -> Option<StdVec<(String, Value)>> {
        if value.has_function() {
            let reason = super::refusal::FUNCTION_VALUE;
            self.error(span, super::refusal::and_attributes(reason));
            return None;
        }
        match entries_of(value) {
            Ok(entries) => Some(entries),
            Err(reason) => {
                self.error(span, super::refusal::and_attributes(&reason));
                None
            }
        }
    }

    /// A tag with `&attributes`: `pug_attrs(pug_merge([…]), terse)`.
    pub(super) fn attribute_blocks(&mut self, collected: &Collected<'_>) {
        let own = if collected.has_attrs() {
            match self.own_object(collected) {
                Some(own) => Some(own),
                None => return,
            }
        } else {
            None
        };
        if let Some(attrs) = self.merged(collected, own) {
            self.render_object(&attrs);
        }
    }

    /// The `attributes` a mixin call passes: `undefined` without
    /// attributes; `None` after an error.
    pub(super) fn call_attributes(&mut self, collected: &Collected<'_>) -> Option<Value> {
        let own = if collected.has_attrs() {
            Some(self.own_object(collected)?)
        } else {
            None
        };
        let object = |attrs: Attrs| {
            Value::Object(
                attrs
                    .into_iter()
                    .map(|(key, value, _)| (key, value))
                    .collect(),
            )
        };
        match (own, collected.blocks.len()) {
            (None, 0) => Some(Value::Undefined),
            (Some(own), 0) => Some(object(own)),
            // A lone block is passed as it is.
            (None, 1) => self
                .block_values(collected)?
                .into_iter()
                .next()
                .map(|(value, _)| value),
            (own, _) => self.merged(collected, own).map(object),
        }
    }

    /// `pug_attrs(attrs, terse)`: `class` through `pug_classes` and first,
    /// the rest through `pug_attr` unescaped, in order.
    fn render_object(&mut self, attrs: &Attrs) {
        if let Some((key, _, span)) = attrs.iter().find(|(_, value, _)| value.has_function()) {
            let message = super::refusal::attribute(key, super::refusal::FUNCTION_VALUE);
            self.error(*span, message);
            return;
        }
        let class = attrs.iter().find(|(key, ..)| key == "class");
        let rest = attrs.iter().filter(|(key, ..)| key != "class");
        for (key, value, span) in class.into_iter().chain(rest) {
            let value = if key == "class" {
                pug_classes(value)
            } else {
                value.clone()
            };
            self.attribute(&Entry {
                key: key.clone(),
                raw_key: "",
                key_span: None,
                value,
                body: None,
                escaped: false,
                span: *span,
            });
        }
    }
}

/// `pug_classes(value)` without escaping: arrays and objects become
/// their class string; anything else is returned as `val || ''`.
fn pug_classes(value: &Value) -> Value {
    match value {
        Value::Array(_) | Value::Object(_) => Value::Str(value::classes(value)),
        value if value.truthy() => value.clone(),
        _ => Value::str(""),
    }
}
