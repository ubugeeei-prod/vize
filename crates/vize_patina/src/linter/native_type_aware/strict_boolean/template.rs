//! Conditions use their own template AST spans and the checker projection.
use super::queries::{Collector, Condition};
use crate::markup::surface_source::{DecodedSource, directive, element_verbatim};
use oxc_allocator::Allocator;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use vize_l1::markup::{SurfaceChild, SurfaceTree};

pub(super) fn collect(root: &SurfaceTree<'_>, offset: u32, out: &mut Vec<Condition>) {
    children(root, &root.children, offset, false, out);
}

fn expression(source: &DecodedSource<'_>, offset: u32, out: &mut Vec<Condition>) {
    let text = source.text();
    let allocator = Allocator::default();
    let Ok(parsed) = Parser::new(&allocator, text, SourceType::ts()).parse_expression() else {
        return;
    };
    let mut collector = Collector {
        conditions: Vec::new(),
    };
    collector.condition(&parsed);
    collector.visit_expression(&parsed);
    out.extend(collector.conditions.into_iter().filter_map(|condition| {
        let authored = source.authored_span(Span::new(condition.start, condition.end))?;
        Some(Condition {
            start: offset + source.start + authored.start,
            end: offset + source.start + authored.end,
        })
    }));
}

fn children(
    root: &SurfaceTree<'_>,
    nodes: &[SurfaceChild<'_>],
    offset: u32,
    inherited_verbatim: bool,
    out: &mut Vec<Condition>,
) {
    for node in nodes {
        if let SurfaceChild::Element(element) = node {
            let verbatim = element_verbatim(element, inherited_verbatim);
            if !verbatim {
                for attribute in &element.open.attrs {
                    if let Some(directive) = directive(root, attribute, false)
                        && matches!(directive.name, "if" | "else-if" | "show")
                        && let Some(value) = directive.value
                    {
                        expression(&value, offset, out);
                    }
                }
            }
            children(root, &element.children, offset, verbatim, out);
        }
    }
}
