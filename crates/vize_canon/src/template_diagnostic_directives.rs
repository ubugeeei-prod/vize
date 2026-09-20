//! Authored-node ownership for Vue diagnostic comments, shared by all clients.

use std::ops::Range;
use vize_atelier_core::{ParserOptions, parser::parse_with_options};
use vize_atelier_sfc::{SfcParseOptions, SfcTemplateBlock, parse_sfc};
use vize_carton::Allocator;
use vize_relief::TemplateChildNode;

/// Vue's unused expect-error diagnostic uses TypeScript's directive code.
pub const UNUSED_EXPECT_ERROR_CODE: u32 = 2578;
/// Shared wording for CLI and editor diagnostics.
pub const UNUSED_EXPECT_ERROR_MESSAGE: &str = "Unused '@vue-expect-error' directive.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectivePolicy {
    Ignore,
    Expect,
    Skip,
}

/// Byte ranges belong to the authored SFC, never to generated helper text.
#[derive(Debug)]
pub struct TemplateDiagnosticDirective {
    pub comment: Range<usize>,
    pub token: Range<usize>,
    pub targets: Vec<Range<usize>>,
    pub policy: DirectivePolicy,
    used: bool,
}

#[derive(Debug, Default)]
pub struct TemplateDiagnosticDirectives {
    directives: Vec<TemplateDiagnosticDirective>,
}

impl TemplateDiagnosticDirectives {
    /// Parse only actual template comments. Text in scripts, attributes or
    /// interpolated strings cannot establish a diagnostic directive.
    pub fn for_sfc(source: &str) -> Self {
        if !source.contains("@vue-") {
            return Self::default();
        }
        let Ok(descriptor) = parse_sfc(source, SfcParseOptions::default()) else {
            return Self::default();
        };
        descriptor
            .template
            .as_ref()
            .map_or_else(Self::default, Self::for_template)
    }

    pub fn for_template(template: &SfcTemplateBlock<'_>) -> Self {
        if !template.content.contains("@vue-") {
            return Self::default();
        }
        let allocator = Allocator::new();
        let (root, _) = parse_with_options(
            &allocator,
            &template.content,
            ParserOptions {
                comments: true,
                ..ParserOptions::default()
            },
        );
        let mut result = Self::default();
        result.visit(&root.children, template.loc.start);
        result
    }

    pub fn directives(&self) -> &[TemplateDiagnosticDirective] {
        &self.directives
    }

    /// Mark the owning expectation when an authored diagnostic is suppressed.
    /// Ignore/expect affect the next node itself; skip affects its whole subtree.
    pub fn suppresses(&mut self, offset: usize) -> bool {
        for directive in &mut self.directives {
            if directive
                .targets
                .iter()
                .any(|range| range.contains(&offset))
            {
                directive.used = true;
                return true;
            }
        }
        false
    }

    pub fn unused_expectations(&self) -> impl Iterator<Item = &Range<usize>> {
        self.directives.iter().filter_map(|directive| {
            (directive.policy == DirectivePolicy::Expect && !directive.used)
                .then_some(&directive.comment)
        })
    }

    fn visit(&mut self, nodes: &[TemplateChildNode<'_>], base: usize) {
        let mut pending = Vec::new();
        // `@vue-generic` arguments are checked as part of the node they
        // instantiate, so the node's directive owns their diagnostics too.
        let mut generic_comments = Vec::new();
        for node in nodes {
            if let TemplateChildNode::Comment(comment) = node {
                let trimmed = comment.content.trim_start();
                if trimmed.starts_with("@vue-generic") {
                    generic_comments.push(
                        base + comment.loc.span.start as usize
                            ..base + comment.loc.span.end as usize,
                    );
                    continue;
                }
                let Some((token, policy)) = [
                    ("@vue-expect-error", DirectivePolicy::Expect),
                    ("@vue-ignore", DirectivePolicy::Ignore),
                    ("@vue-skip", DirectivePolicy::Skip),
                ]
                .into_iter()
                .find(|(token, _)| {
                    trimmed.strip_prefix(token).is_some_and(|tail| {
                        tail.chars()
                            .next()
                            .is_none_or(|ch| !ch.is_alphanumeric() && ch != '-' && ch != '_')
                    })
                }) else {
                    continue;
                };
                let start = base + comment.loc.span.start as usize;
                let token_start = start + 4 + comment.content.len() - trimmed.len();
                pending.push(TemplateDiagnosticDirective {
                    comment: start..base + comment.loc.span.end as usize,
                    token: token_start..token_start + token.len(),
                    targets: Vec::new(),
                    policy,
                    used: false,
                });
                continue;
            }
            if matches!(node, TemplateChildNode::Text(text) if text.content.trim().is_empty()) {
                continue;
            }
            let start = base + node.loc().span.start as usize;
            let end = base + node.loc().span.end as usize;
            let children = match node {
                TemplateChildNode::Element(element) => Some(element.children.as_slice()),
                _ => None,
            };
            let skip = pending
                .iter()
                .any(|directive| directive.policy == DirectivePolicy::Skip);
            // The upstream context gives ignore precedence, and the last expect
            // owns diagnostics when several comments precede the same node.
            let selected = pending
                .iter()
                .rposition(|directive| directive.policy == DirectivePolicy::Skip)
                .or_else(|| {
                    pending
                        .iter()
                        .rposition(|directive| directive.policy == DirectivePolicy::Ignore)
                })
                .or_else(|| {
                    pending
                        .iter()
                        .rposition(|directive| directive.policy == DirectivePolicy::Expect)
                });
            if let Some(index) = selected {
                let mut directive = pending.swap_remove(index);
                if skip {
                    directive.targets.push(start..base + subtree_end(node));
                } else {
                    // Element locations own their opening tag only. Extending
                    // them to a later sibling can accidentally claim a nested
                    // element's descendants, whose locations are also shallow.
                    directive.targets.push(start..end);
                }
                directive.targets.append(&mut generic_comments);
                self.directives.push(directive);
            }
            pending.clear();
            generic_comments.clear();
            if !skip && let Some(children) = children {
                self.visit(children, base);
            }
        }
        // Keep unattached expectations observable rather than silently accepting
        // a comment that no longer checks any authored code after an edit.
        self.directives.extend(pending);
    }
}

fn subtree_end(node: &TemplateChildNode<'_>) -> usize {
    let own_end = node.loc().span.end as usize;
    match node {
        TemplateChildNode::Element(element) => element
            .children
            .iter()
            .map(subtree_end)
            .fold(own_end, usize::max),
        _ => own_end,
    }
}

#[cfg(test)]
mod tests;
