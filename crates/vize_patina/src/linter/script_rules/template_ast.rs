//! One `<template>` parse per SFC, shared by every template-aware script rule.
//!
//! A `script/*` rule that creates a finding from template evidence needs the
//! template **AST**, not the raw text (see
//! [`crate::rules::script::SfcScriptContext`]). Parsing it per rule would
//! reparse the same block once for every such rule, so it is parsed here at
//! most once per SFC and handed to all of them through the shared context.
//!
//! The parse is skipped entirely unless some enabled rule declares
//! [`crate::rules::script::ScriptRule::uses_template_ast`], keeping the common
//! configuration — where no template-aware script rule is on — exactly as cheap
//! as before.

use vize_armature::Parser;
use vize_atelier_sfc::SfcDescriptor;
use vize_relief::RootNode;
use vize_s0::{Allocator, profile};

use crate::linter::Linter;

/// The parsed `<template>` block together with its offset in the SFC source.
pub(super) struct TemplateAst<'a> {
    pub(super) root: RootNode<'a>,
    pub(super) offset: u32,
}

/// Parse the descriptor's `<template>` block for the script rules that need it.
///
/// Returns `None` when the SFC has no template, when no enabled rule reads the
/// AST, or when the template has a fatal parse error — a partial AST is not
/// evidence, and reporting from one would invent findings on a file whose real
/// problem is already reported by the template parse pass.
pub(super) fn parse_for_script_rules<'alloc, 'source>(
    linter: &Linter,
    descriptor: &'source SfcDescriptor<'source>,
    allocator: &'alloc Allocator,
) -> Option<TemplateAst<'alloc>>
where
    'source: 'alloc,
{
    if !super::active_builtin_script_rule_entries(linter)
        .any(|entry| super::resolved_rule(linter, entry).uses_template_ast())
    {
        return None;
    }
    let template = descriptor.template.as_ref()?;
    let (root, parse_errors) = profile!(
        "patina.script_rule.template_parse",
        Parser::new(allocator, template.content.as_ref()).parse()
    );
    if Linter::has_fatal_template_parse_errors(&parse_errors) {
        return None;
    }
    Some(TemplateAst {
        root,
        offset: template.loc.start as u32,
    })
}
