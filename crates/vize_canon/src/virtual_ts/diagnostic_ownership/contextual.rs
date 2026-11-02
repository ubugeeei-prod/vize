//! Reuse the parser-owned expression AST to refuse contextual echo proofs.

use super::independent_reads::{ReadValue, collect_candidates};
use oxc_ast::ast::{ArrowFunctionExpression, Class, Function};
use oxc_ast_visit::Visit;
use oxc_syntax::scope::ScopeFlags;
use vize_carton::{FxHashMap, FxHashSet, profile};

#[derive(Default)]
pub(crate) struct ContextualPropFacts {
    pub(crate) unproven: FxHashSet<(u32, u32)>,
    pub(crate) reads: FxHashMap<(u32, u32), ReadValue>,
}
use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};

pub(crate) fn contextual_prop_facts(root: &RootNode<'_>) -> ContextualPropFacts {
    profile!("canon.virtual_ts.input_ownership.retained_context", {
        let mut facts = ContextualPropFacts::default();
        visit_children(&root.children, &mut facts);
        facts
    })
}

fn visit_children(children: &[TemplateChildNode<'_>], facts: &mut ContextualPropFacts) {
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
                            if let Some(retained) =
                                simple.js_ast.and_then(|ast| ast.as_expression())
                                && retained.raw == simple.content
                                && let Some(reads) = collect_candidates(retained.ast, retained.raw)
                            {
                                facts.reads.insert(
                                    (directive.loc.span.start, directive.loc.span.end),
                                    reads,
                                );
                            }
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
                        facts
                            .unproven
                            .insert((directive.loc.span.start, directive.loc.span.end));
                    }
                }
                visit_children(&element.children, facts);
            }
            TemplateChildNode::If(node) => {
                for branch in &node.branches {
                    visit_children(&branch.children, facts);
                }
            }
            TemplateChildNode::For(node) => visit_children(&node.children, facts),
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
                contextual_prop_facts(&root).unproven,
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
                contextual_prop_facts(&root).unproven,
                expected.into_iter().collect::<FxHashSet<_>>()
            );
        }
    }
}
