//! Expand Vue same-name bindings while preserving the public prop and modifiers.

use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};
use vize_s0::FxHashMap;

use crate::ide::IdeContext;

type Spans = FxHashMap<(usize, usize), (usize, usize)>;

pub(super) fn shorthands(ctx: &IdeContext<'_>) -> Option<Spans> {
    let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, Default::default()).ok()?;
    let mut spans = FxHashMap::default();
    let Some(template) = descriptor.template else {
        return Some(spans);
    };
    let allocator = vize_s0::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, &template.content);
    if !errors.is_empty() {
        return None;
    }
    collect(&root.children, template.loc.start, &mut spans);
    Some(spans)
}

fn collect(children: &[TemplateChildNode<'_>], offset: usize, spans: &mut Spans) {
    for child in children {
        if let TemplateChildNode::Element(element) = child {
            for prop in &element.props {
                if let PropNode::Directive(directive) = prop
                    && directive.name == "bind"
                    && directive.shorthand
                    && let Some(ExpressionNode::Simple(arg)) = &directive.arg
                    && arg.is_static
                {
                    spans.insert(
                        (
                            offset + arg.loc.span.start as usize,
                            offset + arg.loc.span.end as usize,
                        ),
                        (
                            offset + directive.loc.span.start as usize,
                            offset + directive.loc.span.end as usize,
                        ),
                    );
                }
            }
            collect(&element.children, offset, spans);
        }
    }
}
