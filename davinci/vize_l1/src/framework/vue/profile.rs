//! The Vue SFC `<template>` profile of the htmlish grammar.

use crate::markup::profile::{Component, Profile, ProfileKind};

/// A Vue SFC `<template>`: the neutral [`Component`] profile plus Vue's
/// attribute boundary. In Vue a closing quote ends an attribute, so
/// `a="1"b` is two attributes with no error. HTML requires whitespace there.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VueTemplate;

impl Profile for VueTemplate {
    const KIND: ProfileKind = Component::KIND;
    const TOLERATE_DECLARATIONS: bool = Component::TOLERATE_DECLARATIONS;
    const FOLD_NAME_CASE: bool = Component::FOLD_NAME_CASE;
    const SELF_CLOSING_ANY_ELEMENT: bool = Component::SELF_CLOSING_ANY_ELEMENT;
    const REQUIRE_ATTRIBUTE_WHITESPACE: bool = false;
    const IMPLIED_END_TAGS: bool = Component::IMPLIED_END_TAGS;
    const SELECT_CONTENT_MODEL: bool = Component::SELECT_CONTENT_MODEL;
    const TABLE_CONTENT_MODEL: bool = Component::TABLE_CONTENT_MODEL;
}
