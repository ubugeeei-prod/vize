//! Independent static template-ref reads over a freshly parsed template.

use std::collections::BTreeSet;
use vize_carton::{Allocator, CompactString};
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

pub(super) fn evaluate(source: &str) -> BTreeSet<CompactString> {
    let allocator = Allocator::default();
    let (root, _) = vize_armature::Parser::new(&allocator, source).parse();
    let mut reads = BTreeSet::new();
    visit(&root.children, &mut reads);
    reads
}

fn visit(children: &[TemplateChildNode<'_>], reads: &mut BTreeSet<CompactString>) {
    for node in children {
        match node {
            TemplateChildNode::Element(element) => {
                for prop in &element.props {
                    match prop {
                        PropNode::Attribute(attr) if attr.name == "ref" => {
                            if let Some(value) = &attr.value {
                                reads.insert(CompactString::new(value.content));
                            }
                        }
                        PropNode::Directive(dir)
                            if dir.name == "bind"
                                && matches!(&dir.arg, Some(ExpressionNode::Simple(arg)) if arg.content == "ref") =>
                        {
                            if let Some(ExpressionNode::Simple(exp)) = &dir.exp
                                && let Some(js) = exp.js_ast.and_then(|js| js.as_expression())
                                && let oxc_ast::ast::Expression::StringLiteral(value) = js.ast
                            {
                                reads.insert(CompactString::new(value.value.as_str()));
                            }
                        }
                        _ => {}
                    }
                }
                visit(&element.children, reads);
            }
            TemplateChildNode::If(node) => {
                for branch in &node.branches {
                    visit(&branch.children, reads);
                }
            }
            TemplateChildNode::IfBranch(node) => visit(&node.children, reads),
            TemplateChildNode::For(node) => visit(&node.children, reads),
            _ => {}
        }
    }
}
