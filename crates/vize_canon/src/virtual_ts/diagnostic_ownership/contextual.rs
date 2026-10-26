//! Reuse the parser-owned expression AST to refuse contextual echo proofs.

use oxc_ast::ast::{ArrowFunctionExpression, Class, Function};
use oxc_ast_visit::Visit;
use oxc_syntax::scope::ScopeFlags;
use vize_carton::{FxHashSet, profile};
use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};

pub(crate) fn unproven_prop_ranges(root: &RootNode<'_>) -> FxHashSet<(u32, u32)> {
    profile!("canon.virtual_ts.input_ownership.retained_context", {
        let mut unproven = FxHashSet::default();
        visit_children(&root.children, &mut unproven);
        unproven
    })
}

fn visit_children(children: &[TemplateChildNode<'_>], unproven: &mut FxHashSet<(u32, u32)>) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                for prop in &element.props {
                    let PropNode::Directive(directive) = prop else {
                        continue;
                    };
                    if !matches!(directive.name, "bind" | "model") {
                        continue;
                    }
                    let safe = directive.exp.as_ref().is_some_and(|expression| {
                        let ExpressionNode::Simple(simple) = expression else {
                            return false;
                        };
                        // Preserve the existing broad callback classifier. Its
                        // excluded values already cannot receive an IGNORE;
                        // the EXPECT relocation proof handles geometry alone.
                        if crate::virtual_ts::scope::contains_inline_function_prop_value(
                            simple.content,
                        ) {
                            return true;
                        }
                        let Some(retained) = simple.js_ast.and_then(|ast| ast.as_expression())
                        else {
                            return false;
                        };
                        if retained.raw != simple.content {
                            return false;
                        }
                        let mut contextual = Contextual(false);
                        contextual.visit_expression(retained.ast);
                        !contextual.0
                    });
                    if !safe {
                        unproven.insert((directive.loc.span.start, directive.loc.span.end));
                    }
                }
                visit_children(&element.children, unproven);
            }
            TemplateChildNode::If(node) => {
                for branch in &node.branches {
                    visit_children(&branch.children, unproven);
                }
            }
            TemplateChildNode::For(node) => visit_children(&node.children, unproven),
            _ => {}
        }
    }
}

struct Contextual(bool);
impl<'a> Visit<'a> for Contextual {
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {
        self.0 = true;
    }
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {
        self.0 = true;
    }
    fn visit_class(&mut self, _: &Class<'a>) {
        self.0 = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_methods_getters_and_missing_asts_cannot_prove_echoes() {
        for newline in ["\n", "\r\n"] {
            let source = "<!-- 日本 🧭 -->\n<Comp :value=\"{ pick(field) { return field.missing } }\" />\n<Comp :value=\"{ get value() { return missing } }\" />\n<Comp :value=\"{ role: null, tags: [] }\" />\n<Comp :value=\"items[0].length\" />".replace('\n', newline);
            let allocator = vize_carton::Allocator::new();
            let (mut root, errors) = vize_armature::parse(&allocator, &source);
            assert!(errors.is_empty());
            let expected = root
                .children
                .iter()
                .filter_map(|node| {
                    let TemplateChildNode::Element(element) = node else {
                        return None;
                    };
                    let PropNode::Directive(directive) = element.props.first()? else {
                        return None;
                    };
                    Some((directive.loc.span.start, directive.loc.span.end))
                })
                .collect::<Vec<_>>();
            assert_eq!(
                unproven_prop_ranges(&root),
                expected.iter().take(2).copied().collect::<FxHashSet<_>>()
            );
            for child in &mut root.children {
                if let TemplateChildNode::Element(element) = child {
                    for prop in &mut element.props {
                        if let PropNode::Directive(directive) = prop
                            && let Some(ExpressionNode::Simple(simple)) = directive.exp.as_mut()
                        {
                            simple.js_ast = None;
                        }
                    }
                }
            }
            assert_eq!(
                unproven_prop_ranges(&root),
                expected.into_iter().collect::<FxHashSet<_>>()
            );
        }
    }
}
