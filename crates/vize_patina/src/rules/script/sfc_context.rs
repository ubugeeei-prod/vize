//! Cross-block SFC context available to a script rule.
//!
//! Script rules see a single `<script>` / `<script setup>` block and normally
//! cannot observe the rest of the file. Rules that correlate a script
//! declaration with `<template>` usage read the template from here.
//!
//! # Two channels, two directions of error
//!
//! [`SfcScriptContext::template_source`] is the **raw** template text. It is
//! what [`crate::rules::script::props_emits::template_emits`] scans, and that
//! scan is documented as one-directional: a name recovered from it only ever
//! *suppresses* an "unused" report, so an over-match (a call inside an HTML
//! comment, a coincidental attribute value) can at worst hide a finding.
//!
//! [`SfcScriptContext::template_root`] is the **parsed template AST**, for the
//! opposite direction: a rule that *creates* a finding from template evidence.
//! There an over-match is a false positive — a diagnostic at a location where
//! the user did nothing wrong — so raw text is not good enough. The AST
//! structurally excludes the cases a text scan cannot:
//!
//! * an HTML comment is a `Comment` node, never an expression;
//! * a text node is a `Text` node;
//! * a plain attribute is an `Attribute`, not a `Directive` carrying an
//!   expression;
//! * a `v-pre` region has its directives rewritten to attributes and its
//!   interpolations to text, so nothing inside it is compiled.
//!
//! Matching a *name* inside a recovered expression still needs an oxc parse of
//! that expression, so an occurrence inside a string literal cannot count; see
//! [`crate::rules::vue::no_mutating_props`] for the worked example.
//!
//! [`SfcScriptContext::template_offset`] is the byte offset of the template
//! block's content within the whole SFC source, so a rule reporting at a
//! *template* location can map an AST offset back to the file.
//!
//! The `Default` value (no template) is what standalone entry points (inline
//! HTML scripts, [`crate::rules::script::ScriptLinter::lint`]) pass, so rules
//! must treat a missing template as "no template usage observable", never as an
//! error. `template_root` is additionally `None` unless some enabled rule asked
//! for it through [`crate::rules::script::ScriptRule::uses_template_ast`], and
//! when the template failed to parse.

use vize_relief::RootNode;

/// Cross-block context handed to a script rule when the linted block is part of
/// an SFC. See the module documentation for how the three channels differ.
#[derive(Clone, Copy, Default)]
pub struct SfcScriptContext<'a> {
    /// Raw `<template>` block content, when the SFC declares a template.
    pub template_source: Option<&'a str>,
    /// Parsed `<template>` AST, when the SFC declares a template, some enabled
    /// rule requested it, and it parsed without fatal errors.
    pub template_root: Option<&'a RootNode<'a>>,
    /// Byte offset of the `<template>` content inside the SFC source, for
    /// reporting at a template location.
    pub template_offset: Option<u32>,
}

impl<'a> SfcScriptContext<'a> {
    /// The parsed template together with the offset needed to report inside it.
    ///
    /// Both are present or neither is useful, so rules that report at a
    /// template location take them as a pair rather than unwrapping twice.
    #[inline]
    pub fn template_ast(&self) -> Option<(&'a RootNode<'a>, u32)> {
        Some((self.template_root?, self.template_offset?))
    }
}
