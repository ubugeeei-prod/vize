//! A rule-local lexical stack. Sibling scopes and other rules' scope metadata
//! are untouched; each authored declaration is checked once in attribute order.

use super::{
    bindings::{self, Binding},
    frozen,
};
use crate::context::LintContext;
use vize_l0::{CompactString, FxHashSet};
use vize_relief::{ElementNode, PropNode, TemplateChildNode};

#[derive(Default)]
struct Scope {
    visible: FxHashSet<CompactString>,
    introduced: Vec<CompactString>,
}

pub(super) fn check<'a>(ctx: &mut LintContext<'a>, children: &[TemplateChildNode<'a>]) {
    walk(ctx, children, &mut Scope::default());
}

fn walk<'a>(ctx: &mut LintContext<'a>, children: &[TemplateChildNode<'a>], scope: &mut Scope) {
    vize_l0::recursion::ensure_sufficient_stack(|| {
        for child in children {
            match child {
                TemplateChildNode::Element(element) => {
                    if frozen::opens(element, ctx.source) {
                        continue;
                    }
                    let start = scope.introduced.len();
                    element_bindings(ctx, element, scope);
                    walk(ctx, &element.children, scope);
                    scope.leave(start);
                }
                TemplateChildNode::For(node) => {
                    let start = scope.introduced.len();
                    // A lowered list has already consumed its authored v-for.
                    // Its aliases retain their own source locations.
                    for alias in [
                        node.value_alias.as_ref(),
                        node.key_alias.as_ref(),
                        node.object_index_alias.as_ref(),
                    ]
                    .into_iter()
                    .flatten()
                    {
                        for binding in bindings::alias(ctx.allocator(), alias, ctx.source) {
                            scope.declare(ctx, binding);
                        }
                    }
                    walk(ctx, &node.children, scope);
                    scope.leave(start);
                }
                TemplateChildNode::If(node) => {
                    for branch in &node.branches {
                        walk(ctx, &branch.children, scope);
                    }
                }
                TemplateChildNode::IfBranch(node) => walk(ctx, &node.children, scope),
                _ => {}
            }
        }
    });
}

fn element_bindings<'a>(ctx: &mut LintContext<'a>, element: &ElementNode<'a>, scope: &mut Scope) {
    for prop in &element.props {
        let declarations = match prop {
            PropNode::Directive(directive) => {
                let Some(expression) = &directive.exp else {
                    continue;
                };
                match directive.name {
                    "for" => bindings::for_expression(ctx.allocator(), expression, ctx.source),
                    "slot" => bindings::slot(expression, ctx.source),
                    _ => continue,
                }
            }
            PropNode::Attribute(attribute)
                if attribute.name == "slot-scope"
                    || (attribute.name == "scope" && element.tag == "template") =>
            {
                let Some(value) = &attribute.value else {
                    continue;
                };
                bindings::legacy(ctx.allocator(), value.content, &value.loc, ctx.source)
            }
            _ => continue,
        };
        for declaration in declarations {
            scope.declare(ctx, declaration);
        }
    }
}

impl Scope {
    fn declare(&mut self, ctx: &mut LintContext<'_>, binding: Binding) {
        if self.visible.insert(binding.name.clone()) {
            self.introduced.push(binding.name);
        } else {
            ctx.warn_with_help(
                ctx.t_fmt(
                    "vue/no-template-shadow.message",
                    &[("name", binding.name.as_str())],
                ),
                &binding.loc,
                ctx.t("vue/no-template-shadow.help"),
            );
        }
    }

    fn leave(&mut self, start: usize) {
        for name in self.introduced.drain(start..) {
            self.visible.remove(&name);
        }
    }
}
