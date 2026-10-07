//! Template writes only suppress advice; token ambiguity is conservative.
use super::{SfcScriptContext, analysis::Inventory};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{CompactString, FxHashSet};
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

pub(super) fn template_writes(sfc: SfcScriptContext<'_>) -> FxHashSet<CompactString> {
    let mut writes = FxHashSet::default();
    if let Some(root) = sfc.template_root {
        collect_template_writes(&root.children, root.source, &mut writes);
    }
    writes
}

fn collect_template_writes(
    children: &[TemplateChildNode<'_>],
    source: &str,
    writes: &mut FxHashSet<CompactString>,
) {
    for child in children {
        if let TemplateChildNode::Interpolation(interpolation) = child {
            let text = match &interpolation.content {
                ExpressionNode::Simple(simple) => simple.content,
                ExpressionNode::Compound(compound) => compound.loc.span.slice(source),
            };
            collect_expression_writes(text, writes);
        }
        if let TemplateChildNode::Element(element) = child {
            for prop in &element.props {
                if let PropNode::Attribute(attribute) = prop
                    && attribute.name == "ref"
                    && let Some(value) = &attribute.value
                {
                    writes.insert(CompactString::new(value.content));
                }
                if let PropNode::Directive(directive) = prop
                    && let Some(expression) = &directive.exp
                {
                    let text = match expression {
                        ExpressionNode::Simple(simple) => simple.content,
                        ExpressionNode::Compound(compound) => compound.loc.span.slice(source),
                    };
                    let is_bound_ref = directive.name == "bind" && directive.arg.as_ref().is_some_and(|argument| {
                        matches!(argument, ExpressionNode::Simple(simple) if simple.is_static && simple.content == "ref")
                    });
                    if directive.name == "model" || is_bound_ref {
                        collect_tokens(text, writes);
                    } else {
                        collect_expression_writes(text, writes);
                    }
                }
            }
            collect_template_writes(&element.children, source, writes);
        }
    }
}

fn collect_expression_writes(text: &str, writes: &mut FxHashSet<CompactString>) {
    if !(text.contains('=')
        || text.contains("++")
        || text.contains("--")
        || text.contains("delete ")
        || text.contains('('))
    {
        return;
    }
    let allocator = oxc_allocator::Allocator::default();
    let parsed = Parser::new(
        &allocator,
        text,
        SourceType::default().with_typescript(true),
    )
    .parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        // Unknown expressions cannot establish absence of writes.
        collect_tokens(text, writes);
    } else {
        writes.extend(Inventory::collect(&parsed.program).written_names());
    }
}

fn collect_tokens(text: &str, writes: &mut FxHashSet<CompactString>) {
    // Over-collecting tokens can only suppress advice, never create findings.
    for token in text.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$')) {
        if !token.is_empty() {
            writes.insert(CompactString::new(token));
        }
    }
}
