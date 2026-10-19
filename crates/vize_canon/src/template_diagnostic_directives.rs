//! Authored-node ownership for Vue diagnostic comments, shared by all clients.

use std::ops::Range;
use vize_atelier_core::{ParserOptions, parser::parse_with_options};
use vize_atelier_sfc::{
    BlockLocation, SfcDescriptor, SfcParseOptions, SfcTemplateBlock, parse_sfc,
};
use vize_carton::Allocator;
use vize_relief::{RootNode, TemplateChildNode};

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
        Self::for_descriptor(source, &descriptor)
    }

    /// Template comments plus SFC-level comments placed directly before a
    /// block. Like `@ts-expect-error` for the next line, a block directive
    /// owns only that block's opening tag (`module`, `generic`, `lang`
    /// diagnostics); `@vue-skip` owns the whole block.
    pub fn for_descriptor(source: &str, descriptor: &SfcDescriptor<'_>) -> Self {
        let mut result = descriptor
            .template
            .as_ref()
            .map_or_else(Self::default, Self::for_template);
        if !source.contains("@vue-") {
            return result;
        }
        let mut blocks: Vec<&BlockLocation> = descriptor
            .template
            .iter()
            .map(|block| &block.loc)
            .chain(descriptor.script.iter().map(|block| &block.loc))
            .chain(descriptor.script_setup.iter().map(|block| &block.loc))
            .chain(descriptor.styles.iter().map(|block| &block.loc))
            .chain(descriptor.custom_blocks.iter().map(|block| &block.loc))
            .collect();
        blocks.sort_by_key(|loc| loc.tag_start);
        let mut gap_start = 0;
        for loc in blocks {
            let mut pending = Vec::new();
            let mut cursor = gap_start;
            while let Some(open) = source
                .get(cursor..loc.tag_start)
                .and_then(|gap| gap.find("<!--"))
            {
                let content_start = cursor + open + 4;
                let Some(close) = source
                    .get(content_start..loc.tag_start)
                    .and_then(|rest| rest.find("-->"))
                else {
                    break;
                };
                let end = content_start + close + 3;
                let content = source.get(content_start..end - 3).unwrap_or_default();
                pending.extend(comment_directive(content, cursor + open..end));
                cursor = end;
            }
            if let Some(mut directive) = take_owner(&mut pending) {
                let target_end = if directive.policy == DirectivePolicy::Skip {
                    loc.tag_end
                } else {
                    loc.start
                };
                directive.targets.push(loc.tag_start..target_end);
                result.directives.push(directive);
            }
            gap_start = loc.tag_end.max(loc.end);
        }
        result
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
        Self::for_root(&root, template.loc.start)
    }

    /// Reuse an already parsed, comment-retaining template root. The policy
    /// walk is shared with `for_template`; callers must not reparse or invent
    /// another comment grammar to decide which authored node owns an expect.
    pub(crate) fn for_root(root: &RootNode<'_>, base: usize) -> Self {
        let mut result = Self::default();
        result.visit(&root.children, base);
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
                pending.extend(comment_directive(
                    comment.content,
                    base + comment.loc.span.start as usize..base + comment.loc.span.end as usize,
                ));
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
            if let Some(mut directive) = take_owner(&mut pending) {
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

/// The directive an HTML comment's `content` declares, if any.
fn comment_directive(content: &str, comment: Range<usize>) -> Option<TemplateDiagnosticDirective> {
    let trimmed = content.trim_start();
    let (token, policy) = [
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
    })?;
    let token_start = comment.start + 4 + content.len() - trimmed.len();
    Some(TemplateDiagnosticDirective {
        comment,
        token: token_start..token_start + token.len(),
        targets: Vec::new(),
        policy,
        used: false,
    })
}

/// The upstream context gives ignore precedence, and the last expect owns
/// diagnostics when several comments precede the same node.
fn take_owner(
    pending: &mut Vec<TemplateDiagnosticDirective>,
) -> Option<TemplateDiagnosticDirective> {
    let last = |policy| {
        pending
            .iter()
            .rposition(|directive: &TemplateDiagnosticDirective| directive.policy == policy)
    };
    let index = last(DirectivePolicy::Skip)
        .or_else(|| last(DirectivePolicy::Ignore))
        .or_else(|| last(DirectivePolicy::Expect))?;
    Some(pending.swap_remove(index))
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
mod tests {
    #![expect(clippy::string_slice, reason = "tests assert by panicking")]
    use super::*;

    #[test]
    fn multiline_own_node_errors_do_not_consume_child_or_sibling_expectations() {
        let source = "<template><!-- @vue-expect-error -->\n<div\n :x=\"outer\">{{ child }}<!-- @vue-expect-error -->{{ inner }}</div>{{ sibling }}</template>";
        let mut plan = TemplateDiagnosticDirectives::for_sfc(source);
        assert_eq!(plan.directives().len(), 2);
        assert!(!plan.suppresses(source.find("child").unwrap()));
        assert_eq!(plan.unused_expectations().count(), 2);
        assert!(plan.suppresses(source.find("outer").unwrap()));
        assert!(plan.suppresses(source.find("inner").unwrap()));
        assert!(!plan.suppresses(source.find("sibling").unwrap()));
        assert_eq!(plan.unused_expectations().count(), 0);
    }

    #[test]
    fn skip_owns_whole_subtree_but_ignore_does_not() {
        for (directive, children_suppressed) in [("skip", true), ("ignore", false)] {
            let source = vize_carton::cstr!(
                "<template><!-- @vue-{directive} --><div :x=\"outer\">{{ child }}</div>{{ sibling }}</template>"
            );
            let mut plan = TemplateDiagnosticDirectives::for_sfc(&source);
            assert!(plan.suppresses(source.find("outer").unwrap()));
            assert_eq!(
                plan.suppresses(source.find("child").unwrap()),
                children_suppressed,
                "{source}: {plan:?}"
            );
            assert!(!plan.suppresses(source.find("sibling").unwrap()));
            assert_eq!(plan.unused_expectations().count(), 0);
        }
    }

    #[test]
    fn comments_inside_a_skipped_subtree_do_not_report_unused_errors() {
        let source = "<template><!-- @vue-skip --><div><!-- @vue-expect-error -->{{ valid }}</div></template>";
        let plan = TemplateDiagnosticDirectives::for_sfc(source);
        assert_eq!(plan.directives().len(), 1);
        assert_eq!(plan.unused_expectations().count(), 0);
    }

    #[test]
    fn an_outer_expectation_never_claims_nested_descendants_between_siblings() {
        let source = "<template><!-- @vue-expect-error --><div :x=\"outer\"><span>{{ nested }}</span><p>{{ other }}</p></div></template>";
        let mut plan = TemplateDiagnosticDirectives::for_sfc(source);
        assert!(!plan.suppresses(source.find("nested").unwrap()));
        assert!(!plan.suppresses(source.find("other").unwrap()));
        assert_eq!(plan.unused_expectations().count(), 1);
        assert!(plan.suppresses(source.find("outer").unwrap()));
    }

    #[test]
    fn token_ownership_excludes_script_strings_attributes_and_comment_prose() {
        for source in [
            "<script setup>const s = '<!-- @vue-ignore -->';</script><template>{{ missing }}</template>",
            "<template><div title=\"<!-- @vue-ignore -->\">{{ missing }}</div></template>",
            "<template><!-- prose @vue-ignore -->{{ missing }}</template>",
            "<template><!-- @vue-ignore_more -->{{ missing }}</template>",
            "<template><!-- @vue-ignore-more -->{{ missing }}</template>",
            "<template>{{ '<!-- @vue-ignore -->' }}{{ missing }}</template>",
        ] {
            let plan = TemplateDiagnosticDirectives::for_sfc(source);
            assert!(plan.directives().is_empty(), "{source}");
        }
    }

    #[test]
    fn empty_expectations_retain_the_authored_comment_range() {
        let source = "<template>😀\r\n<!--  @vue-expect-error reason -->\r\n</template>";
        let plan = TemplateDiagnosticDirectives::for_sfc(source);
        let directive = &plan.directives()[0];
        assert_eq!(&source[directive.token.clone()], "@vue-expect-error");
        assert_eq!(
            &source[directive.comment.clone()],
            "<!--  @vue-expect-error reason -->"
        );
        assert_eq!(plan.unused_expectations().count(), 1);
    }

    #[test]
    fn a_node_directive_owns_its_generic_argument_comment_in_either_order() {
        for source in [
            "<template><!-- @vue-expect-error --><!-- @vue-generic {boolean} --><Comp />{{ sibling }}</template>",
            "<template><!-- @vue-generic {boolean} --><!-- @vue-expect-error --><Comp />{{ sibling }}</template>",
        ] {
            let mut plan = TemplateDiagnosticDirectives::for_sfc(source);
            assert_eq!(plan.directives().len(), 1, "{source}");
            assert!(plan.suppresses(source.find("boolean").unwrap()), "{source}");
            assert!(
                !plan.suppresses(source.find("sibling").unwrap()),
                "{source}"
            );
            assert_eq!(plan.unused_expectations().count(), 0, "{source}");
        }
        // Without a directive the generic comment's diagnostics stay reportable.
        let source = "<template><!-- @vue-generic {boolean} --><Comp /></template>";
        let mut plan = TemplateDiagnosticDirectives::for_sfc(source);
        assert_eq!(plan.directives().len(), 0);
        assert!(!plan.suppresses(source.find("boolean").unwrap()));
    }

    #[test]
    fn borrowed_root_preserves_the_complete_canonical_policy_with_offsets() {
        for newline in ["\n", "\r\n"] {
            for comments in [
                "<!-- @vue-expect-error -->",
                "<!-- @vue-expect-error --><!-- @vue-ignore -->",
                "<!-- @vue-expect-error --><!-- @vue-skip -->",
                "<!-- @vue-expect-error --><!-- @vue-generic {string} -->",
                "<!-- prose @vue-expect-error -->",
            ] {
                let source = vize_carton::cstr!(
                    "<script setup>const prefix = '😀';</script>{newline}<template>日本{newline}{comments}<Comp id=\"bad\">{{{{ child }}}}</Comp><!-- @vue-expect-error --></template>"
                );
                let descriptor = parse_sfc(&source, SfcParseOptions::default()).unwrap();
                let template = descriptor.template.as_ref().unwrap();
                let allocator = Allocator::new();
                let (root, errors) = parse_with_options(
                    &allocator,
                    &template.content,
                    ParserOptions {
                        comments: true,
                        ..Default::default()
                    },
                );
                assert!(errors.is_empty(), "{errors:?}");
                let original = TemplateDiagnosticDirectives::for_template(template);
                let borrowed = TemplateDiagnosticDirectives::for_root(&root, template.loc.start);
                assert_eq!(
                    vize_carton::cstr!("{original:?}"),
                    vize_carton::cstr!("{borrowed:?}")
                );
            }
        }
    }
}
