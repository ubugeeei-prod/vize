//! Template writes only suppress advice; token ambiguity is conservative.
use super::{SfcScriptContext, analysis::Inventory};
use oxc_ast_visit::Visit;
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
                    if directive.name == "model" {
                        // Any identifier-shaped token can only suppress a warning;
                        // over-collecting computed indexes is deliberately safe.
                        for token in
                            text.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'))
                        {
                            if !token.is_empty() {
                                writes.insert(CompactString::new(token));
                            }
                        }
                    } else if text.contains('=')
                        || text.contains("++")
                        || text.contains("--")
                        || text.contains("delete ")
                        || text.contains('(')
                    {
                        let allocator = oxc_allocator::Allocator::default();
                        let parsed = Parser::new(
                            &allocator,
                            text,
                            SourceType::default().with_typescript(true),
                        )
                        .parse();
                        // Unknown expressions cannot establish absence of writes.
                        if parsed.panicked || !parsed.diagnostics.is_empty() {
                            for token in
                                text.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'))
                            {
                                if !token.is_empty() {
                                    writes.insert(CompactString::new(token));
                                }
                            }
                        } else {
                            let inventory = Inventory::collect(&parsed.program);
                            writes.extend(inventory.written_names());
                        }
                    }
                }
            }
            collect_template_writes(&element.children, source, writes);
        }
    }
}
