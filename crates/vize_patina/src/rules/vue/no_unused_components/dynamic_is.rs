//! Detection of a dynamic `is` binding, the rule's whole-file suppression.

use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

/// Whether the template binds `is` to something other than a string literal,
/// anywhere.
///
/// `eslint-plugin-vue`'s `vue/no-unused-components` defaults
/// `ignoreWhenBindingPresent` to `true` and stops reporting for the whole file
/// when it sees one, because a dynamic `<component :is="resolved">` can render
/// any registered component and the rule cannot tell which. Reporting anyway is
/// how a legitimately-used component gets called unused: the shape is common
/// enough in real code to dominate this rule's output (#3223).
///
/// A literal (`:is="'MyPanel'"`) is exempt: it names its component, so the
/// registration is still checkable. A static `is="MyPanel"` attribute is not a
/// binding at all and does not suppress anything.
///
/// The walk is an explicit worklist rather than recursion: the template parser
/// accepts up to `MAX_ELEMENT_NESTING_DEPTH` nested elements, and a stack
/// overflow on a valid file would take the LSP process down with it.
pub(super) fn has_dynamic_is_binding<'a>(nodes: &'a [TemplateChildNode<'a>]) -> bool {
    let mut worklist: Vec<&'a [TemplateChildNode<'a>]> = vec![nodes];
    while let Some(children) = worklist.pop() {
        for node in children {
            match node {
                TemplateChildNode::Element(element) => {
                    if element.props.iter().any(is_dynamic_is_prop) {
                        return true;
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
    false
}

fn is_dynamic_is_prop(prop: &PropNode<'_>) -> bool {
    let PropNode::Directive(directive) = prop else {
        return false;
    };
    if directive.name != "bind" {
        return false;
    }
    // A dynamic argument (`:[name]="x"`) can resolve to `is`, so it counts.
    let Some(ExpressionNode::Simple(argument)) = directive.arg.as_ref() else {
        return true;
    };
    if !argument.is_static {
        return true;
    }
    if argument.content != "is" {
        return false;
    }
    !matches!(
        directive.exp.as_ref(),
        Some(ExpressionNode::Simple(expression)) if names_a_component(expression.content)
    )
}

/// Whether a directive expression statically names a component.
///
/// The expression is parsed rather than inspected by its delimiters: matching
/// quotes get `` `${name}` `` wrong in one direction (interpolated, so the name
/// is only known at runtime) and `('MyPanel')` wrong in the other (a literal
/// wearing parentheses).
fn names_a_component(content: &str) -> bool {
    let allocator = Allocator::default();
    let Ok(expression) = Parser::new(&allocator, content, SourceType::ts())
        .with_options(ParseOptions {
            preserve_parens: false,
            ..ParseOptions::default()
        })
        .parse_expression()
    else {
        return false;
    };
    match expression {
        Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        _ => false,
    }
}
