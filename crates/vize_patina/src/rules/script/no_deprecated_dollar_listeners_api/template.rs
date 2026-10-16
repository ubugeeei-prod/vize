//! Genuine unbound template references, including lexical v-for/slot scopes.
//! Text, comments, literal attributes and v-pre regions never enter this walker.
use oxc_allocator::Allocator;
use oxc_ast::ast::BindingIdentifier;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{SourceType, Span};
use vize_l0::cstr;
use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};

pub(super) fn references(root: &RootNode<'_>) -> Vec<Span> {
    let mut out = Vec::new();
    walk(&root.children, root.source, false, &mut out);
    out
}

fn text<'a>(exp: &'a ExpressionNode<'_>, source: &'a str) -> &'a str {
    exp.loc().span.slice(source)
}

fn walk(children: &[TemplateChildNode<'_>], source: &str, shadowed: bool, out: &mut Vec<Span>) {
    for child in children {
        match child {
            TemplateChildNode::Interpolation(node) if !shadowed => scan(
                text(&node.content, source),
                node.content.loc().span.start,
                false,
                out,
            ),
            TemplateChildNode::Element(element) => {
                let mut local = shadowed;
                for prop in &element.props {
                    if let PropNode::Directive(dir) = prop
                        && dir.name == "for"
                        && let Some(exp) = &dir.exp
                    {
                        let body = text(exp, source);
                        if let Some(scope) =
                            vize_croquis::drawer::parse_v_for_scope_expression(body)
                        {
                            if !shadowed && let Some(index) = body.rfind(scope.source.as_str()) {
                                scan(
                                    scope.source.as_str(),
                                    exp.loc().span.start + index as u32,
                                    false,
                                    out,
                                );
                            }
                            local |= scope
                                .value_bindings
                                .iter()
                                .any(|name| name.as_str() == "$listeners")
                                || scope.key_alias.as_deref() == Some("$listeners")
                                || scope.index_alias.as_deref() == Some("$listeners");
                        } else {
                            // Invalid binding syntax cannot prove an unbound instance reference.
                            local = true;
                        }
                    }
                }
                for prop in &element.props {
                    let PropNode::Directive(dir) = prop else {
                        continue;
                    };
                    if !local
                        && let Some(arg) = &dir.arg
                        && let ExpressionNode::Simple(arg) = arg
                        && !arg.is_static
                    {
                        scan(arg.loc.span.slice(source), arg.loc.span.start, false, out);
                    }
                    if let Some(exp) = &dir.exp {
                        if dir.name == "slot" {
                            if !local {
                                // Default initializers in a slot binding can still reference
                                // the component; semantic resolution excludes the bindings.
                                scan_binding(text(exp, source), exp.loc().span.start, out);
                            }
                        } else if dir.name != "for" && !local {
                            scan(
                                text(exp, source),
                                exp.loc().span.start,
                                dir.name == "on",
                                out,
                            );
                        }
                    }
                }
                for prop in &element.props {
                    if let PropNode::Directive(dir) = prop
                        && dir.name == "slot"
                        && let Some(exp) = &dir.exp
                    {
                        local |= binds_listeners(text(exp, source));
                    }
                }
                walk(&element.children, source, local, out);
            }
            TemplateChildNode::If(node) => {
                for branch in &node.branches {
                    walk(&branch.children, source, shadowed, out);
                }
            }
            TemplateChildNode::For(node) => {
                if !shadowed {
                    scan(
                        text(&node.source, source),
                        node.source.loc().span.start,
                        false,
                        out,
                    );
                }
                let local = shadowed
                    || [
                        node.value_alias.as_ref(),
                        node.key_alias.as_ref(),
                        node.object_index_alias.as_ref(),
                    ]
                    .into_iter()
                    .flatten()
                    .any(|exp| binds_listeners(text(exp, source)));
                walk(&node.children, source, local, out);
            }
            _ => {}
        }
    }
}

fn binds_listeners(pattern: &str) -> bool {
    let allocator = Allocator::default();
    let wrapped = cstr!("({pattern}) => {{}}");
    let parsed = Parser::new(&allocator, &wrapped, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return true;
    }
    let mut visitor = Binding(false);
    visitor.visit_program(&parsed.program);
    visitor.0
}
struct Binding(bool);
impl<'a> Visit<'a> for Binding {
    fn visit_binding_identifier(&mut self, id: &BindingIdentifier<'a>) {
        self.0 |= id.name.as_str() == "$listeners";
    }
}

fn scan_binding(source: &str, base: u32, out: &mut Vec<Span>) {
    scan_wrapped(&cstr!("({source}) => {{}}"), base, 1, out);
}
fn scan(source: &str, base: u32, statements: bool, out: &mut Vec<Span>) {
    if !source.contains("$listeners") && !source.contains('\\') {
        return;
    }
    if statements {
        scan_wrapped(source, base, 0, out);
    } else {
        scan_wrapped(&cstr!("({source})"), base, 1, out);
    }
}
fn scan_wrapped(source: &str, base: u32, prefix: u32, out: &mut Vec<Span>) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return;
    }
    let built = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program);
    if !built.diagnostics.is_empty() {
        return;
    }
    let semantic = &built.semantic;
    let scoping = semantic.scoping();
    let mut spans: Vec<_> = scoping
        .root_unresolved_references_ids()
        .flatten()
        .filter_map(|id| {
            let reference = scoping.get_reference(id);
            if semantic.reference_name(reference) != "$listeners" {
                return None;
            }
            let span = semantic.reference_span(reference);
            Some(Span::new(
                base + span.start.checked_sub(prefix)?,
                base + span.end.checked_sub(prefix)?,
            ))
        })
        .collect();
    spans.sort_by_key(|span| span.start);
    out.extend(spans);
}
