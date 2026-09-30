//! Markup profiles: the host rules a markup grammar is read under.
//!
//! Every difference between an HTML document (in-DOM template) and an SFC
//! template is one associated constant here, so the list lives in one place
//! and each branch folds away under static dispatch. The lexer reads
//! [`Profile::TOLERATE_DECLARATIONS`]; the remaining rules belong to the L1
//! tree builder and are declared now so the profile is complete.

/// Which profile a generic function was instantiated with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileKind {
    Document,
    Component,
}

/// Host rules for one markup profile. Implemented by zero-sized markers only.
pub trait Profile: Copy + Default + 'static {
    const KIND: ProfileKind;
    /// `<!DOCTYPE …>` and other top-level declarations are skipped silently
    /// instead of reporting `IncorrectlyOpenedComment`.
    const TOLERATE_DECLARATIONS: bool;
    /// Element names compare ASCII-case-insensitively (`<MyComp>` is
    /// `<mycomp>` once the browser has parsed the document).
    const FOLD_NAME_CASE: bool;
    /// `<x />` closes any element; otherwise only void and foreign elements.
    const SELF_CLOSING_ANY_ELEMENT: bool;
    /// HTML implied end tags (`<p>` closed by a block, `<li>` by `<li>`).
    const IMPLIED_END_TAGS: bool;
    /// The table content model (foster parenting, `is="vue:…"` workarounds).
    const TABLE_CONTENT_MODEL: bool;
}

/// HTML documents and in-DOM templates (petite-vue, standalone HTML).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Document;

/// SFC `<template>` content, the default Vue compile path.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Component;

impl Profile for Document {
    const KIND: ProfileKind = ProfileKind::Document;
    const TOLERATE_DECLARATIONS: bool = true;
    const FOLD_NAME_CASE: bool = true;
    const SELF_CLOSING_ANY_ELEMENT: bool = false;
    const IMPLIED_END_TAGS: bool = true;
    const TABLE_CONTENT_MODEL: bool = true;
}

impl Profile for Component {
    const KIND: ProfileKind = ProfileKind::Component;
    const TOLERATE_DECLARATIONS: bool = false;
    const FOLD_NAME_CASE: bool = false;
    const SELF_CLOSING_ANY_ELEMENT: bool = true;
    const IMPLIED_END_TAGS: bool = false;
    const TABLE_CONTENT_MODEL: bool = false;
}
