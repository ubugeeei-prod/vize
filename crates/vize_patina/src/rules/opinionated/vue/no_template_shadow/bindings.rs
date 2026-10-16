//! Authored template declarations: retained modern slots, and original-slice
//! parameter goals for legacy slots and v-for aliases. No synthetic wrappers.

use oxc_ast::ast::{BindingPattern, FormalParameters};
use oxc_span::Span;
use vize_armature::{parser::retain_slot_parameters_in, patterns::attribute_source_span};
use vize_l0::{Allocator, CompactString, FxHashSet};
use vize_relief::{ExpressionNode, RetainedJsAst, SourceLocation};

pub(super) struct Binding {
    pub name: CompactString,
    pub loc: SourceLocation,
}

pub(super) fn slot(expression: &ExpressionNode<'_>, source: &str) -> Vec<Binding> {
    let ExpressionNode::Simple(node) = expression else {
        return Vec::new();
    };
    let Some(retained) = node.js_ast.filter(|ast| ast.raw == node.content) else {
        return Vec::new();
    };
    declarations(retained, 0, node.content, &node.loc, source).unwrap_or_default()
}

pub(super) fn legacy<'a>(
    allocator: &'a Allocator,
    content: &'a str,
    loc: &SourceLocation,
    source: &str,
) -> Vec<Binding> {
    declarations(
        retain_slot_parameters_in(allocator, content),
        0,
        content,
        loc,
        source,
    )
    .unwrap_or_default()
}

pub(super) fn for_expression<'a>(
    allocator: &'a Allocator,
    expression: &ExpressionNode<'a>,
    source: &str,
) -> Vec<Binding> {
    let ExpressionNode::Simple(node) = expression else {
        return Vec::new();
    };
    let Some(retained) =
        vize_croquis::drawer::retain_v_for_binding_parameters(allocator, node.content)
    else {
        return Vec::new();
    };
    parameter_declarations(
        &retained.parameters,
        retained.offset,
        node.content,
        &node.loc,
        source,
    )
    .unwrap_or_default()
}

pub(super) fn alias<'a>(
    allocator: &'a Allocator,
    expression: &ExpressionNode<'a>,
    source: &str,
) -> Vec<Binding> {
    let ExpressionNode::Simple(node) = expression else {
        return Vec::new();
    };
    legacy(allocator, node.content, &node.loc, source)
}

fn declarations(
    retained: RetainedJsAst<'_>,
    offset: u32,
    decoded: &str,
    loc: &SourceLocation,
    source: &str,
) -> Option<Vec<Binding>> {
    let parameters = retained.as_slot_parameters()?.ok()?;
    parameter_declarations(parameters, offset, decoded, loc, source)
}

fn parameter_declarations(
    parameters: &FormalParameters<'_>,
    offset: u32,
    decoded: &str,
    loc: &SourceLocation,
    source: &str,
) -> Option<Vec<Binding>> {
    let authored = source.get(loc.span.start as usize..loc.span.end as usize)?;
    let mut bindings = Vec::new();
    let mut valid = true;
    collect_parameters(parameters, &mut |name, start, end| {
        let Some(span) =
            attribute_source_span(authored, decoded, Span::new(offset + start, offset + end))
        else {
            valid = false;
            return;
        };
        bindings.push(Binding {
            name: CompactString::new(name),
            loc: SourceLocation::new(loc.span.start + span.start, loc.span.start + span.end),
        });
    });
    if !valid {
        return None;
    }
    unique(bindings)
}

fn unique(bindings: Vec<Binding>) -> Option<Vec<Binding>> {
    let mut names = FxHashSet::default();
    let valid = bindings
        .iter()
        .all(|binding| names.insert(binding.name.as_str()));
    drop(names);
    valid.then_some(bindings)
}

fn collect_parameters(parameters: &FormalParameters<'_>, visit: &mut impl FnMut(&str, u32, u32)) {
    for parameter in &parameters.items {
        collect_pattern(&parameter.pattern, visit);
    }
    if let Some(rest) = &parameters.rest {
        collect_pattern(&rest.rest.argument, visit);
    }
}

fn collect_pattern(pattern: &BindingPattern<'_>, visit: &mut impl FnMut(&str, u32, u32)) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => visit(
            identifier.name.as_str(),
            identifier.span.start,
            identifier.span.end,
        ),
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_pattern(&property.value, visit);
            }
            if let Some(rest) = &object.rest {
                collect_pattern(&rest.argument, visit);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_pattern(element, visit);
            }
            if let Some(rest) = &array.rest {
                collect_pattern(&rest.argument, visit);
            }
        }
        BindingPattern::AssignmentPattern(default) => collect_pattern(&default.left, visit),
    }
}
