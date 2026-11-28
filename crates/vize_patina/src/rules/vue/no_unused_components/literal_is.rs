//! Components a template names through `is`, which count as used.

use super::{ComponentCandidate, NoUnusedComponents};
use oxc_ast::ast::Expression;
use vize_l0::CompactString;
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

/// Drop the candidates the template names through `is`.
///
/// `eslint-plugin-vue`'s `vue/no-unused-components` counts a component as
/// used when a literal names it in `:is` / `v-bind:is` / `v-is`
/// (`:is="'MyPanel'"`, `:is="('MyPanel')"`), and when a static `is="MyPanel"`
/// or `is="vue:MyPanel"` attribute names it. A template literal without
/// expressions (`` :is="`MyPanel`" ``) names its component as plainly, so it
/// counts too. A non-literal `v-is` can render any registered component,
/// which, like a dynamic `:is`, leaves nothing to report.
///
/// The caller has already returned for a dynamic `is` binding, so every
/// `:is` reaching here is a literal. The walk only runs when a candidate is
/// otherwise unused, so files without one do no extra work.
pub(super) fn retain_unnamed_by_is<'a>(
    nodes: &'a [TemplateChildNode<'a>],
    candidates: &mut Vec<ComponentCandidate>,
) {
    if candidates.is_empty() {
        return;
    }
    let Some(names) = names_through_is(nodes) else {
        candidates.clear();
        return;
    };
    candidates.retain(|candidate| {
        !names.iter().any(|used| {
            NoUnusedComponents::component_name_matches(used.as_str(), candidate.name.as_str())
        })
    });
}

/// Names from every `is`, or `None` when a `v-is` is not a literal.
///
/// The walk is an explicit worklist rather than recursion: the template parser
/// accepts up to `MAX_ELEMENT_NESTING_DEPTH` nested elements, and a stack
/// overflow on a valid file would take the LSP process down with it.
fn names_through_is<'a>(nodes: &'a [TemplateChildNode<'a>]) -> Option<Vec<CompactString>> {
    let mut names = Vec::new();
    let mut worklist: Vec<&'a [TemplateChildNode<'a>]> = vec![nodes];
    while let Some(children) = worklist.pop() {
        for node in children {
            match node {
                TemplateChildNode::Element(element) => {
                    for prop in &element.props {
                        match prop_is(prop) {
                            PropIs::None => {}
                            PropIs::Dynamic => return None,
                            PropIs::Named(name) => names.push(name),
                        }
                    }
                    worklist.push(&element.children);
                }
                TemplateChildNode::If(node) => worklist.extend(
                    node.branches
                        .iter()
                        .map(|branch| branch.children.as_slice()),
                ),
                TemplateChildNode::IfBranch(branch) => worklist.push(&branch.children),
                TemplateChildNode::For(node) => worklist.push(&node.children),
                _ => {}
            }
        }
    }
    Some(names)
}

enum PropIs {
    None,
    Dynamic,
    Named(CompactString),
}

fn prop_is(prop: &PropNode<'_>) -> PropIs {
    let directive = match prop {
        PropNode::Attribute(attribute) => {
            if attribute.name != "is" {
                return PropIs::None;
            }
            let Some(value) = attribute.value.as_ref() else {
                return PropIs::None;
            };
            let name = value.content.strip_prefix("vue:").unwrap_or(value.content);
            return PropIs::Named(CompactString::new(name));
        }
        PropNode::Directive(directive) => directive,
    };
    let Some(ExpressionNode::Simple(expression)) = directive.exp.as_ref() else {
        return PropIs::None;
    };
    let is_binding = directive.name == "bind"
        && matches!(
            directive.arg.as_ref(),
            Some(ExpressionNode::Simple(argument)) if argument.is_static && argument.content == "is"
        );
    if !is_binding && directive.name != "is" {
        return PropIs::None;
    }
    // Without a retained parse the expression's text is not a lone
    // expression, so it names nothing and, as before, suppresses nothing.
    let Some(parsed) = expression.js_ast.and_then(|ast| ast.as_expression()) else {
        return PropIs::None;
    };
    literal_component_name(parsed.ast).map_or(PropIs::Dynamic, PropIs::Named)
}

/// The component a directive expression statically names, if it does.
///
/// The template's parse-once AST answers rather than the expression's
/// delimiters: matching quotes get `` `${name}` `` wrong in one direction
/// (interpolated, so the name is only known at runtime) and `('MyPanel')`
/// wrong in the other (a literal wearing parentheses).
fn literal_component_name(expression: &Expression<'_>) -> Option<CompactString> {
    match expression.without_parentheses() {
        Expression::StringLiteral(literal) => Some(CompactString::new(literal.value.as_str())),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            let quasi = template.quasis.first()?;
            let value = quasi.value.cooked.unwrap_or(quasi.value.raw);
            Some(CompactString::new(value.as_str()))
        }
        _ => None,
    }
}
