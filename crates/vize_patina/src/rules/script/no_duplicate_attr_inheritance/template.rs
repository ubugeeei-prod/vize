//! Attribute forwarding references from the existing validated template AST.
//! The pinned upstream rule reports the first unbound $attrs reference in each
//! argumentless v-bind, including nested elements. Ordinary fragments are
//! excluded; a complete v-if / v-else-if / v-else root group is comparable.

use oxc_allocator::Allocator;
use oxc_ast::ast::{ArrowFunctionExpression, BindingIdentifier};
use oxc_ast_visit::{Visit, walk::walk_arrow_function_expression};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_l0::cstr;
use vize_relief::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, RootNode, TemplateChildNode,
};

pub(super) struct AttrsSpread {
    pub(super) start: u32,
    pub(super) end: u32,
}

pub(super) fn attrs_spreads(root: &RootNode<'_>) -> Vec<AttrsSpread> {
    let elements: Vec<_> = root
        .children
        .iter()
        .filter_map(|child| match child {
            TemplateChildNode::Element(element) => Some(&**element),
            _ => None,
        })
        .collect();
    if elements.len() > 1 && !conditional_group(&elements) {
        return Vec::new();
    }
    let mut findings = Vec::new();
    collect(&root.children, root.source, false, &mut findings);
    findings
}

fn conditional_group(elements: &[&ElementNode<'_>]) -> bool {
    elements.len() > 1
        && elements
            .first()
            .is_some_and(|element| has_directive(element, "if"))
        && elements.last().is_some_and(|element| {
            has_directive(element, "else-if") || has_directive(element, "else")
        })
        && elements
            .iter()
            .skip(1)
            .take(elements.len().saturating_sub(2))
            .all(|element| has_directive(element, "else-if"))
}

fn has_directive(element: &ElementNode<'_>, name: &str) -> bool {
    element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == name))
}

fn collect(
    children: &[TemplateChildNode<'_>],
    source: &str,
    shadowed: bool,
    out: &mut Vec<AttrsSpread>,
) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                let shadowed = shadowed
                    || element.props.iter().any(|prop| match prop {
                        PropNode::Directive(dir) if dir.name == "for" => {
                            for_shadows_attrs(dir, source)
                        }
                        PropNode::Directive(dir) if dir.name == "slot" => {
                            dir.exp.as_ref().is_some_and(|exp| {
                                binding_shadows_attrs(expression_source(exp, source))
                            })
                        }
                        _ => false,
                    });
                if !shadowed {
                    for prop in &element.props {
                        let PropNode::Directive(dir) = prop else {
                            continue;
                        };
                        if dir.name != "bind" || dir.arg.is_some() {
                            continue;
                        }
                        let Some(exp) = dir.exp.as_ref() else {
                            continue;
                        };
                        let Some(text) =
                            source.get(exp.loc().span.start as usize..exp.loc().span.end as usize)
                        else {
                            continue;
                        };
                        if let Some((start, end)) = first_attrs_reference(text) {
                            out.push(AttrsSpread {
                                start: exp.loc().span.start + start,
                                end: exp.loc().span.start + end,
                            });
                        }
                    }
                }
                collect(&element.children, source, shadowed, out);
            }
            TemplateChildNode::If(node) => {
                for branch in &node.branches {
                    collect(&branch.children, source, shadowed, out);
                }
            }
            TemplateChildNode::For(node) => {
                let shadowed = shadowed
                    || [
                        node.value_alias.as_ref(),
                        node.key_alias.as_ref(),
                        node.object_index_alias.as_ref(),
                    ]
                    .into_iter()
                    .flatten()
                    .any(|alias| binding_shadows_attrs(expression_source(alias, source)));
                collect(&node.children, source, shadowed, out);
            }
            _ => {}
        }
    }
}

fn for_shadows_attrs(directive: &DirectiveNode<'_>, source: &str) -> bool {
    if let Some(parsed) = directive.for_parse_result.as_ref() {
        return [
            parsed.value.as_ref(),
            parsed.key.as_ref(),
            parsed.index.as_ref(),
        ]
        .into_iter()
        .flatten()
        .any(|alias| binding_shadows_attrs(expression_source(alias, source)));
    }
    // The cached raw template may not carry structural for aliases yet.
    // Reuse the shared scope parser so escaped names are bindings while the
    // iterable source remains a read; unknown bindings stay conservative.
    directive.exp.as_ref().is_some_and(|exp| {
        vize_croquis::drawer::parse_v_for_scope_expression(expression_source(exp, source))
            .is_none_or(|aliases| {
                aliases
                    .value_bindings
                    .iter()
                    .any(|name| name.as_str() == "$attrs")
                    || aliases
                        .key_alias
                        .as_ref()
                        .is_some_and(|name| name.as_str() == "$attrs")
                    || aliases
                        .index_alias
                        .as_ref()
                        .is_some_and(|name| name.as_str() == "$attrs")
            })
    })
}

fn expression_source<'a>(exp: &'a ExpressionNode<'a>, source: &'a str) -> &'a str {
    match exp {
        ExpressionNode::Simple(simple) => simple.content,
        ExpressionNode::Compound(compound) => compound.loc.span.slice(source),
    }
}

fn binding_shadows_attrs(pattern: &str) -> bool {
    if !pattern.contains("$attrs") && !pattern.contains('\\') {
        return false;
    }
    let allocator = Allocator::default();
    let wrapped = cstr!("({pattern}) => {{}}");
    let parsed = Parser::new(&allocator, &wrapped, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return true;
    }
    let mut visitor = AttrsBinding {
        entered: false,
        found: false,
    };
    visitor.visit_program(&parsed.program);
    visitor.found
}

struct AttrsBinding {
    entered: bool,
    found: bool,
}

impl<'a> Visit<'a> for AttrsBinding {
    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if self.entered {
            return;
        }
        self.entered = true;
        walk_arrow_function_expression(self, arrow);
    }

    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        self.found |= identifier.name.as_str() == "$attrs";
    }
}

fn first_attrs_reference(source: &str) -> Option<(u32, u32)> {
    if !source.contains("$attrs") && !source.contains('\\') {
        return None;
    }
    if source.trim() == "$attrs" {
        let start = (source.len() - source.trim_start().len()) as u32;
        return Some((start, start + 6));
    }
    let allocator = Allocator::default();
    let wrapped = cstr!("({source})");
    let parsed = Parser::new(&allocator, &wrapped, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let built = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .with_build_nodes(true)
        .build(&parsed.program);
    if !built.diagnostics.is_empty() {
        return None;
    }
    let semantic = &built.semantic;
    let scoping = semantic.scoping();
    scoping
        .root_unresolved_references_ids()
        .flatten()
        .filter_map(|id| {
            let reference = scoping.get_reference(id);
            if semantic.reference_name(reference) != "$attrs" {
                return None;
            }
            let span = semantic.reference_span(reference);
            Some((span.start.checked_sub(1)?, span.end.checked_sub(1)?))
        })
        .min_by_key(|&(start, _)| start)
}
