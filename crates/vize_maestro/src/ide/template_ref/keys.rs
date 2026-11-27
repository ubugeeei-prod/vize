//! Template-ref keys as one renamable name.
//!
//! `useTemplateRef('box')` and `ref="box"` are linked only by the string
//! `box`, so no TypeScript symbol connects them. This module treats the key
//! itself as the symbol: references list every place it is spelled, and a
//! rename rewrites all of them together. The binding the call is assigned to
//! is an ordinary script symbol and keeps its name.
//!
//! The key is renamed only when every spelling is known. A component that
//! names refs some other way (`$refs`, a dynamic `:ref`, a computed
//! `useTemplateRef` argument, a string with escapes) gets no answer here, so
//! a rename never leaves one spelling behind.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, Expression, ImportDeclarationSpecifier, Program, Statement,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use tower_lsp::lsp_types::{Location, Range, TextEdit, WorkspaceEdit};
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

use super::range_from_offsets;
use crate::ide::IdeContext;

/// Modules whose `useTemplateRef` export is Vue's.
const VUE_MODULES: [&str; 4] = ["vue", "@vue/runtime-core", "@vue/runtime-dom", "#imports"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Site {
    Script,
    Template,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Spelling {
    key: String,
    start: usize,
    end: usize,
    site: Site,
}

/// The key under the cursor and every place the component spells it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemplateRefKey {
    pub(crate) name: String,
    selected: (usize, usize),
    /// Source offsets of the key text, without quotes, in source order.
    spellings: Vec<(usize, usize)>,
}

impl TemplateRefKey {
    pub(crate) fn selected_range(&self, content: &str) -> Range {
        range_from_offsets(content, self.selected.0, self.selected.1)
    }

    pub(crate) fn locations(&self, ctx: &IdeContext<'_>) -> Vec<Location> {
        self.spellings
            .iter()
            .map(|&(start, end)| Location {
                uri: ctx.uri.clone(),
                range: range_from_offsets(&ctx.content, start, end),
            })
            .collect()
    }

    pub(crate) fn rename(&self, ctx: &IdeContext<'_>, new_name: &str) -> Option<WorkspaceEdit> {
        if !is_editable_key(new_name) {
            return None;
        }
        let edits = self
            .spellings
            .iter()
            .map(|&(start, end)| TextEdit {
                range: range_from_offsets(&ctx.content, start, end),
                new_text: new_name.to_string(),
            })
            .collect();
        Some(WorkspaceEdit {
            changes: Some([(ctx.uri.clone(), edits)].into_iter().collect()),
            document_changes: None,
            change_annotations: None,
        })
    }
}

/// The template-ref key at the cursor, when the cursor is on a static
/// `ref="key"` value or a `useTemplateRef('key')` argument and the key is
/// passed to `useTemplateRef` at least once.
pub(crate) fn key_at_offset(ctx: &IdeContext<'_>) -> Option<TemplateRefKey> {
    let spellings = spellings(ctx)?;
    let selected = spellings
        .iter()
        .find(|spelling| spelling.start <= ctx.offset && ctx.offset <= spelling.end)?;
    let same_key = || spellings.iter().filter(|other| other.key == selected.key);
    if !same_key().any(|spelling| spelling.site == Site::Script) {
        return None;
    }
    Some(TemplateRefKey {
        name: selected.key.clone(),
        selected: (selected.start, selected.end),
        spellings: same_key()
            .map(|spelling| (spelling.start, spelling.end))
            .collect(),
    })
}

/// Every key spelling in the component, or `None` when some ref is named in
/// a way this module cannot follow.
fn spellings(ctx: &IdeContext<'_>) -> Option<Vec<Spelling>> {
    // `$refs.box`, `$refs['box']` and `this.$refs` name keys too.
    if ctx.content.contains("$refs") {
        return None;
    }
    let descriptor = ctx.descriptor()?;
    let mut spellings = Vec::new();
    for script in [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
        .into_iter()
        .flatten()
    {
        script_spellings(
            script.content.as_ref(),
            script.lang.as_deref(),
            script.loc.start,
            &mut spellings,
        )?;
    }
    if let Some(template) = descriptor.template.as_ref() {
        if !vize_l1::container::lang::is_html_template(template.lang.as_deref()) {
            return None;
        }
        template_spellings(
            template.content.as_ref(),
            template.loc.start,
            &mut spellings,
        )?;
    }
    spellings.sort_by_key(|spelling| spelling.start);
    Some(spellings)
}

fn script_spellings(
    content: &str,
    lang: Option<&str>,
    base: usize,
    spellings: &mut Vec<Spelling>,
) -> Option<()> {
    let source_type = match lang {
        Some("tsx") => SourceType::tsx(),
        Some("jsx") => SourceType::jsx(),
        Some("js") => SourceType::mjs(),
        _ => SourceType::ts(),
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, content, source_type).parse();
    // A recovered parse may have dropped a call.
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let mut collector = ScriptCollector {
        content,
        base,
        callees: callee_names(&parsed.program),
        spellings,
        unknown: false,
    };
    collector.visit_program(&parsed.program);
    (!collector.unknown).then_some(())
}

/// Local names that call Vue's `useTemplateRef`. An unimported
/// `useTemplateRef` is an auto-import and counts too.
fn callee_names<'a>(program: &Program<'a>) -> Vec<&'a str> {
    let mut names = Vec::new();
    let mut bare_is_foreign = false;
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        let from_vue = VUE_MODULES.contains(&import.source.value.as_str());
        for specifier in import.specifiers.iter().flatten() {
            let local = specifier.local().name.as_str();
            let is_use_template_ref = matches!(
                specifier,
                ImportDeclarationSpecifier::ImportSpecifier(named)
                    if named.imported.name().as_str() == "useTemplateRef"
            );
            if from_vue && is_use_template_ref {
                names.push(local);
            } else if local == "useTemplateRef" {
                bare_is_foreign = true;
            }
        }
    }
    if !bare_is_foreign && !names.contains(&"useTemplateRef") {
        names.push("useTemplateRef");
    }
    names
}

struct ScriptCollector<'s, 'a> {
    content: &'s str,
    base: usize,
    callees: Vec<&'a str>,
    spellings: &'s mut Vec<Spelling>,
    unknown: bool,
}

impl<'a> Visit<'a> for ScriptCollector<'_, 'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &call.callee
            && self.callees.contains(&callee.name.as_str())
        {
            match call.arguments.first() {
                Some(Argument::StringLiteral(literal)) => {
                    let start = literal.span.start as usize + 1;
                    let end = (literal.span.end as usize).saturating_sub(1);
                    // An escaped key has no single span to rewrite.
                    if self.content.get(start..end) == Some(literal.value.as_str()) {
                        self.spellings.push(Spelling {
                            key: literal.value.to_string(),
                            start: self.base + start,
                            end: self.base + end,
                            site: Site::Script,
                        });
                    } else {
                        self.unknown = true;
                    }
                }
                _ => self.unknown = true,
            }
        }
        walk::walk_call_expression(self, call);
    }
}

fn template_spellings(content: &str, base: usize, spellings: &mut Vec<Spelling>) -> Option<()> {
    let allocator = vize_l0::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, content);
    if errors.iter().any(|error| !error.is_recoverable()) {
        return None;
    }
    collect_template(&root.children, content, base, spellings)
}

fn collect_template(
    children: &[TemplateChildNode<'_>],
    content: &str,
    base: usize,
    spellings: &mut Vec<Spelling>,
) -> Option<()> {
    for child in children {
        let TemplateChildNode::Element(element) = child else {
            continue;
        };
        for prop in &element.props {
            match prop {
                PropNode::Attribute(attribute) if attribute.name == "ref" => {
                    let value = attribute.value.as_ref()?;
                    let (start, end) = unquoted(
                        content,
                        value.loc.span.start as usize,
                        value.loc.span.end as usize,
                    )?;
                    // An entity-encoded key has no single span to rewrite.
                    if content.get(start..end)? != value.content {
                        return None;
                    }
                    spellings.push(Spelling {
                        key: value.content.to_string(),
                        start: base + start,
                        end: base + end,
                        site: Site::Template,
                    });
                }
                // `:ref` may evaluate to any key.
                PropNode::Directive(directive)
                    if directive.name == "bind"
                        && matches!(
                            &directive.arg,
                            Some(ExpressionNode::Simple(arg)) if arg.content == "ref"
                        ) =>
                {
                    return None;
                }
                _ => {}
            }
        }
        collect_template(&element.children, content, base, spellings)?;
    }
    Some(())
}

/// The span inside an attribute value's quotes.
fn unquoted(content: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    let raw = content.get(start..end)?;
    match raw.as_bytes().first() {
        Some(&quote @ (b'"' | b'\''))
            if raw.len() >= 2 && raw.as_bytes().last() == Some(&quote) =>
        {
            Some((start + 1, end - 1))
        }
        Some(b'"' | b'\'') => None,
        _ => Some((start, end)),
    }
}

/// A key that can be written between either quote kind without escaping.
fn is_editable_key(key: &str) -> bool {
    !key.is_empty()
        && !key
            .chars()
            .any(|ch| ch.is_whitespace() || matches!(ch, '"' | '\'' | '`' | '\\' | '<' | '>' | '&'))
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests;
