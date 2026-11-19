//! Borrowed, immutable CSS metadata generated from the pinned Microsoft catalog.
//! Request paths neither decode JSON nor initialize a runtime catalog.

use std::cmp::Ordering;

mod at_rules_0;
mod at_rules_index;
mod colors;
mod properties_0;
mod properties_1;
mod properties_2;
mod properties_3;
mod properties_index;
mod pseudo_classes_0;
mod pseudo_classes_index;
mod pseudo_elements_0;
mod pseudo_elements_index;

/// Original named documentation reference.
#[derive(Clone, Copy, Debug)]
pub(super) struct CssReference {
    pub name: &'static str,
    pub url: &'static str,
}

/// Original availability metadata; status is the source's `high`, `low` or `false`.
#[derive(Clone, Copy, Debug)]
pub(super) struct CssBaseline {
    pub status: &'static str,
    pub low_date: Option<&'static str>,
    pub high_date: Option<&'static str>,
}

/// Original property-owned or named color metadata; authored value order stays intact.
#[derive(Clone, Copy, Debug)]
pub(super) struct CssValue {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub browsers: &'static [&'static str],
    pub baseline: Option<CssBaseline>,
}

/// Complete original entry metadata; missing documentation is represented explicitly.
#[derive(Clone, Copy, Debug)]
pub(super) struct CssEntry {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub syntax: Option<&'static str>,
    pub references: &'static [CssReference],
    pub values: &'static [CssValue],
    pub at_rule: Option<&'static str>,
    pub status: Option<&'static str>,
    pub restrictions: &'static [&'static str],
    pub browsers: &'static [&'static str],
    pub baseline: Option<CssBaseline>,
    pub relevance: Option<u16>,
    pub descriptors: &'static [CssEntry],
    pub entry_type: Option<&'static str>,
}

pub(super) fn properties() -> &'static [&'static CssEntry] {
    &properties_index::ENTRIES
}

pub(super) fn property(name: &str) -> Option<&'static CssEntry> {
    lookup(properties(), name)
}

pub(super) fn pseudo_classes() -> &'static [&'static CssEntry] {
    &pseudo_classes_index::ENTRIES
}

pub(super) fn pseudo_class(name: &str) -> Option<&'static CssEntry> {
    lookup(pseudo_classes(), name)
}

pub(super) fn pseudo_elements() -> &'static [&'static CssEntry] {
    &pseudo_elements_index::ENTRIES
}

pub(super) fn pseudo_element(name: &str) -> Option<&'static CssEntry> {
    lookup(pseudo_elements(), name)
}

pub(super) fn at_rules() -> &'static [&'static CssEntry] {
    &at_rules_index::ENTRIES
}

pub(super) fn at_rule(name: &str) -> Option<&'static CssEntry> {
    lookup(at_rules(), name)
}

pub(super) fn colors() -> &'static [CssValue] {
    &colors::VALUES
}

pub(super) fn color(name: &str) -> Option<&'static CssValue> {
    colors()
        .binary_search_by(|value| ascii_compare(value.name, name))
        .ok()
        .and_then(|index| colors().get(index))
}

/// Binary search keeps ASCII case-insensitive CSS lookup allocation-free.
fn lookup(entries: &'static [&'static CssEntry], name: &str) -> Option<&'static CssEntry> {
    entries
        .binary_search_by(|entry| ascii_compare(entry.name, name))
        .ok()
        .and_then(|index| entries.get(index).copied())
}

fn ascii_compare(left: &str, right: &str) -> Ordering {
    left.bytes()
        .map(|byte| byte.to_ascii_lowercase())
        .cmp(right.bytes().map(|byte| byte.to_ascii_lowercase()))
}
