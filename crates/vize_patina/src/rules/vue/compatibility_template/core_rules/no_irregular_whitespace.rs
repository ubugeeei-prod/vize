//! `vue/no-irregular-whitespace` compatibility rule.
//!
//! As in eslint-plugin-vue at its default options, the script blocks and the
//! template are scanned as text, a run of irregular spaces reports once, and
//! string literals are skipped. Comments, regular expressions, template
//! literals, attribute values and text are checked; the `skip*` options are
//! not configurable.

use super::super::core_checks::{TemplateGoal, template_goal};
use super::super::origin::Origin;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::StringLiteral;
use oxc_ast_visit::Visit;
use vize_l1::markup::entity::EntityContext;
use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-irregular-whitespace",
    description: "Disallow irregular whitespace in script blocks and templates",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow irregular whitespace in script blocks and templates.
pub struct NoIrregularWhitespace;

impl Rule for NoIrregularWhitespace {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let (Some(descriptor), Some(scripts)) = (ctx.sfc_descriptor(), ctx.sfc_scripts()) else {
            return;
        };
        let blocks = [
            (descriptor.script.as_ref(), scripts.script()),
            (descriptor.script_setup.as_ref(), scripts.script_setup()),
        ];
        let mut scans = Vec::new();
        for (block, parsed) in blocks {
            let Some(block) = block else {
                continue;
            };
            // A failed parse has no reliable string ranges, so that block is
            // left alone. The other script block is still scanned.
            let Some(parsed) =
                parsed.filter(|parsed| !parsed.panicked && parsed.diagnostics.is_empty())
            else {
                continue;
            };
            let mut strings = StringLiterals::default();
            strings.offset = block.loc.start as u32;
            strings.visit_program(&parsed.program);
            scans.push((block.loc.start, block.loc.end, strings.ranges));
        }
        for (start, end, strings) in scans {
            report_irregular_whitespace(ctx, start, end, &strings);
        }
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let mut strings = Vec::new();
        for child in &root.children {
            collect_template_strings(ctx, child, &mut strings);
        }
        let end = ctx.source.len();
        report_irregular_whitespace(ctx, 0, end, &strings);
    }
}

fn collect_template_strings<'a>(
    ctx: &mut LintContext<'a>,
    node: &TemplateChildNode<'a>,
    strings: &mut Vec<ByteRange>,
) {
    let children = match node {
        TemplateChildNode::Element(element) => {
            for prop in &element.props {
                if let PropNode::Directive(directive) = prop {
                    for expression in [&directive.exp, &directive.arg].into_iter().flatten() {
                        expression_strings(
                            ctx,
                            expression,
                            EntityContext::Attribute,
                            Some(directive.name),
                            strings,
                        );
                    }
                }
            }
            &element.children
        }
        TemplateChildNode::Interpolation(interpolation) => {
            expression_strings(
                ctx,
                &interpolation.content,
                EntityContext::Text,
                None,
                strings,
            );
            return;
        }
        TemplateChildNode::If(if_node) => {
            for branch in &if_node.branches {
                for child in &branch.children {
                    collect_template_strings(ctx, child, strings);
                }
            }
            return;
        }
        TemplateChildNode::IfBranch(branch) => &branch.children,
        TemplateChildNode::For(for_node) => &for_node.children,
        _ => return,
    };
    for child in children {
        collect_template_strings(ctx, child, strings);
    }
}

fn expression_strings<'a>(
    ctx: &mut LintContext<'a>,
    expression: &ExpressionNode<'a>,
    entity_context: EntityContext,
    directive: Option<&str>,
    strings: &mut Vec<ByteRange>,
) {
    let ExpressionNode::Simple(simple) = expression else {
        return;
    };
    let Some(origin) = Origin::new(ctx.source, simple.content, simple.loc.span, entity_context)
    else {
        return;
    };
    let Some(goal) = template_goal(ctx, simple, directive) else {
        return;
    };
    let mut literals = StringLiterals::default();
    match goal {
        TemplateGoal::Expression(expression) => literals.visit_expression(expression),
        TemplateGoal::SlotParameters(parameters) => literals.visit_formal_parameters(parameters),
        TemplateGoal::VFor(retained) => {
            literals.offset = retained.parameters_offset;
            literals.visit_formal_parameters(retained.parameters);
            literals.offset = retained.source_offset;
            literals.visit_expression(retained.source);
        }
        TemplateGoal::Handler(program) => literals.visit_program(program),
    }
    strings.extend(
        literals
            .ranges
            .into_iter()
            .filter_map(|range| origin.range(oxc_span::Span::new(range.start, range.end))),
    );
}

/// String literal ranges, shifted by `offset`.
#[derive(Default)]
struct StringLiterals {
    offset: u32,
    ranges: Vec<ByteRange>,
}

impl<'a> Visit<'a> for StringLiterals {
    fn visit_string_literal(&mut self, literal: &StringLiteral<'a>) {
        self.ranges.push(ByteRange {
            start: literal.span.start.saturating_add(self.offset),
            end: literal.span.end.saturating_add(self.offset),
        });
    }
}

/// `IRREGULAR_WHITESPACE` of the eslint-plugin-vue rule, line terminators aside.
fn is_irregular_space(character: char) -> bool {
    matches!(
        character,
        '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{00A0}' | '\u{1680}' | '\u{180E}' | '\u{2000}'
            ..='\u{200B}' | '\u{202F}' | '\u{205F}' | '\u{3000}' | '\u{FEFF}'
    )
}

/// Report each run of irregular spaces and each irregular line terminator in
/// `source[start..end]` that does not start inside a string literal.
fn report_irregular_whitespace(
    ctx: &mut LintContext<'_>,
    start: usize,
    end: usize,
    strings: &[ByteRange],
) {
    let Some(text) = ctx.source.get(start..end) else {
        return;
    };
    let mut found = Vec::new();
    let mut run: Option<(usize, usize)> = None;
    for (index, character) in text.char_indices() {
        let at = start + index;
        let next = at + character.len_utf8();
        if is_irregular_space(character) {
            run = Some(run.map_or((at, next), |(run_start, _)| (run_start, next)));
            continue;
        }
        found.extend(run.take());
        if matches!(character, '\u{2028}' | '\u{2029}') {
            found.push((at, next));
        }
    }
    found.extend(run);
    for (run_start, run_end) in found {
        let (Ok(run_start), Ok(run_end)) = (u32::try_from(run_start), u32::try_from(run_end))
        else {
            continue;
        };
        if strings
            .iter()
            .any(|string| string.start <= run_start && run_start < string.end)
        {
            continue;
        }
        ctx.warn_at_with_help(
            "Unexpected irregular whitespace",
            ByteRange {
                start: run_start,
                end: run_end,
            },
            "Replace the irregular whitespace with a regular space or remove it",
        );
    }
}
