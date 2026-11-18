//! Borrowed, immutable CSS metadata generated from the pinned Microsoft catalog.
//! Request paths neither decode JSON nor initialize a runtime catalog.
//!
//! The catalog holds no pointers. Every text is a `u16` id into one string
//! table, and every list is a run of a flat table. A table of `&'static str`
//! and nested slices costs a 16-byte pointer, plus a 24-byte dynamic
//! relocation on Linux, for each of its 30,000 references. That is well over a
//! megabyte of the CLI binary for 300 KB of text.

use std::cmp::Ordering;

mod at_rules;
mod baselines;
mod colors;
mod descriptors;
mod lists;
mod properties;
mod pseudo_classes;
mod pseudo_elements;
mod references;
mod starts;
mod text;
mod values;

/// No text: the id of an absent optional field.
const NONE: u16 = u16::MAX;

/// A run of a flat table: its first index and its length.
#[derive(Clone, Copy, Debug)]
struct Run(u16, u16);

impl Run {
    fn of<T>(self, table: &'static [T]) -> &'static [T] {
        let start = usize::from(self.0);
        table
            .get(start..start + usize::from(self.1))
            .unwrap_or_default()
    }
}

fn text(id: u16) -> &'static str {
    let index = usize::from(id);
    match (starts::STARTS.get(index), starts::STARTS.get(index + 1)) {
        (Some(&start), Some(&end)) => match (usize::try_from(start), usize::try_from(end)) {
            (Ok(start), Ok(end)) => text::TEXT.get(start..end).unwrap_or_default(),
            _ => "",
        },
        _ => "",
    }
}

fn optional(id: u16) -> Option<&'static str> {
    (id != NONE).then(|| text(id))
}

/// A list of catalog texts, in authored order.
#[derive(Clone, Copy, Debug)]
pub(super) struct CssStrings(&'static [u16]);

impl CssStrings {
    pub fn is_empty(self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(self) -> impl Iterator<Item = &'static str> {
        self.0.iter().map(|&id| text(id))
    }

    pub fn contains(self, value: &str) -> bool {
        self.iter().any(|item| item == value)
    }
}

/// Original named documentation reference.
#[derive(Clone, Copy, Debug)]
pub(super) struct CssReference {
    name: u16,
    url: u16,
}

impl CssReference {
    pub fn name(&self) -> &'static str {
        text(self.name)
    }

    pub fn url(&self) -> &'static str {
        text(self.url)
    }
}

/// Original availability metadata; status is the source's `high`, `low` or `false`.
#[derive(Clone, Copy, Debug)]
pub(super) struct CssBaseline {
    pub status: &'static str,
    pub low_date: Option<&'static str>,
    pub high_date: Option<&'static str>,
}

/// A stored [`CssBaseline`]: text ids.
#[derive(Clone, Copy, Debug)]
struct Baseline(u16, u16, u16);

fn baseline(index: u16) -> Option<CssBaseline> {
    let Baseline(status, low_date, high_date) = *baselines::BASELINES.get(usize::from(index))?;
    Some(CssBaseline {
        status: text(status),
        low_date: optional(low_date),
        high_date: optional(high_date),
    })
}

/// Original property-owned or named color metadata; authored value order stays intact.
#[derive(Clone, Copy, Debug)]
pub(super) struct CssValue {
    name: u16,
    description: u16,
    browsers: Run,
    baseline: u16,
}

impl CssValue {
    pub fn name(&self) -> &'static str {
        text(self.name)
    }

    pub fn description(&self) -> Option<&'static str> {
        optional(self.description)
    }

    pub fn browsers(&self) -> CssStrings {
        CssStrings(self.browsers.of(&lists::LISTS))
    }

    pub fn baseline(&self) -> Option<CssBaseline> {
        baseline(self.baseline)
    }
}

/// Complete original entry metadata; missing documentation is represented explicitly.
#[derive(Clone, Copy, Debug)]
pub(super) struct CssEntry {
    name: u16,
    description: u16,
    syntax: u16,
    references: Run,
    values: Run,
    at_rule: u16,
    status: u16,
    restrictions: Run,
    browsers: Run,
    baseline: u16,
    relevance: Option<u16>,
    descriptors: Run,
    entry_type: u16,
}

impl CssEntry {
    pub fn name(&self) -> &'static str {
        text(self.name)
    }

    pub fn description(&self) -> Option<&'static str> {
        optional(self.description)
    }

    pub fn syntax(&self) -> Option<&'static str> {
        optional(self.syntax)
    }

    pub fn references(&self) -> &'static [CssReference] {
        self.references.of(&references::REFERENCES)
    }

    pub fn values(&self) -> &'static [CssValue] {
        self.values.of(&values::VALUES)
    }

    pub fn at_rule(&self) -> Option<&'static str> {
        optional(self.at_rule)
    }

    pub fn status(&self) -> Option<&'static str> {
        optional(self.status)
    }

    pub fn restrictions(&self) -> CssStrings {
        CssStrings(self.restrictions.of(&lists::LISTS))
    }

    pub fn browsers(&self) -> CssStrings {
        CssStrings(self.browsers.of(&lists::LISTS))
    }

    pub fn baseline(&self) -> Option<CssBaseline> {
        baseline(self.baseline)
    }

    pub fn relevance(&self) -> Option<u16> {
        self.relevance
    }

    pub fn descriptors(&self) -> &'static [CssEntry] {
        self.descriptors.of(&descriptors::DESCRIPTORS)
    }

    pub fn entry_type(&self) -> Option<&'static str> {
        optional(self.entry_type)
    }
}

pub(super) fn properties() -> &'static [CssEntry] {
    &properties::ENTRIES
}

pub(super) fn property(name: &str) -> Option<&'static CssEntry> {
    lookup(properties(), name)
}

pub(super) fn pseudo_classes() -> &'static [CssEntry] {
    &pseudo_classes::ENTRIES
}

pub(super) fn pseudo_class(name: &str) -> Option<&'static CssEntry> {
    lookup(pseudo_classes(), name)
}

pub(super) fn pseudo_elements() -> &'static [CssEntry] {
    &pseudo_elements::ENTRIES
}

pub(super) fn pseudo_element(name: &str) -> Option<&'static CssEntry> {
    lookup(pseudo_elements(), name)
}

pub(super) fn at_rules() -> &'static [CssEntry] {
    &at_rules::ENTRIES
}

pub(super) fn at_rule(name: &str) -> Option<&'static CssEntry> {
    lookup(at_rules(), name)
}

pub(super) fn colors() -> &'static [CssValue] {
    &colors::VALUES
}

pub(super) fn color(name: &str) -> Option<&'static CssValue> {
    colors()
        .binary_search_by(|value| ascii_compare(value.name(), name))
        .ok()
        .and_then(|index| colors().get(index))
}

/// Binary search keeps ASCII case-insensitive CSS lookup allocation-free.
fn lookup(entries: &'static [CssEntry], name: &str) -> Option<&'static CssEntry> {
    entries
        .binary_search_by(|entry| ascii_compare(entry.name(), name))
        .ok()
        .and_then(|index| entries.get(index))
}

fn ascii_compare(left: &str, right: &str) -> Ordering {
    left.bytes()
        .map(|byte| byte.to_ascii_lowercase())
        .cmp(right.bytes().map(|byte| byte.to_ascii_lowercase()))
}
