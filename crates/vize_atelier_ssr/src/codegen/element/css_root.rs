//! Suspense root-child emission, shared with ordinary SSR roots.

use super::super::css_vars::RootCssVars;
use super::{ElementNode, ElementType, ExpressionNode, PropNode, RuntimeHelper, SsrCodegenContext};
use vize_atelier_core::TemplateChildNode;

fn template_slot<'n, 'a>(
    child: &'n TemplateChildNode<'a>,
) -> Option<&'n vize_atelier_core::DirectiveNode<'a>> {
    let TemplateChildNode::Element(el) = child else {
        return None;
    };
    if el.tag_type != ElementType::Template {
        return None;
    }
    el.props.iter().find_map(|prop| match prop {
        PropNode::Directive(dir) if dir.name == "slot" => Some(dir.as_ref()),
        _ => None,
    })
}

impl<'a> SsrCodegenContext<'a> {
    /// Process Vue's built-in <Suspense> component.
    ///
    /// The SSR renderer has a dedicated helper for Suspense. Rendering it through
    /// `ssrRenderComponent(resolveComponent("Suspense"))` makes Vue attempt a
    /// runtime component lookup and leaves Nuxt root components empty.
    pub(super) fn process_suspense(&mut self, el: &ElementNode<'a>, css_vars: bool) {
        self.flush_push();
        self.use_ssr_helper(RuntimeHelper::SsrRenderSuspense);

        self.push_indent();
        self.push("_ssrRenderSuspense(_push, {\n");
        self.indent_level += 1;
        if el.children.iter().any(|child| {
            template_slot(child).is_some_and(|slot| match &slot.arg {
                None => false,
                Some(ExpressionNode::Simple(arg)) if arg.is_static => arg.content != "default",
                Some(_) => true,
            })
        }) {
            self.process_suspense_template_slots(el, css_vars);
            self.indent_level -= 1;
            self.push_indent();
            self.push("})\n");
            return;
        }
        self.push_indent();
        self.push("default: () => {\n");
        self.indent_level += 1;

        let old_parts = std::mem::take(&mut self.current_template_parts);
        self.process_children_with_fallthrough_attrs_and_css_vars(
            &el.children,
            false,
            false,
            false,
            false,
            RootCssVars {
                enabled: css_vars,
                template_wrapper: true,
            },
        );
        self.flush_push();
        self.current_template_parts = old_parts;

        self.indent_level -= 1;
        self.push_indent();
        self.push("},\n");
        self.push_indent();
        self.push("_: 1\n");
        self.indent_level -= 1;
        self.push_indent();
        self.push("})\n");
    }

    fn process_suspense_template_slots(&mut self, el: &ElementNode<'a>, css_vars: bool) {
        let mut implicit = std::vec::Vec::new();
        let mut dynamic = false;
        for child in &el.children {
            let Some(slot) = template_slot(child) else {
                implicit.push(child);
                continue;
            };
            let TemplateChildNode::Element(template) = child else {
                continue;
            };
            self.push_indent();
            match &slot.arg {
                Some(ExpressionNode::Simple(arg)) if arg.is_static => {
                    if super::props::is_simple_identifier(arg.content) {
                        self.push_mapped(arg.content, arg.loc.span.start);
                    } else {
                        let key = super::props::quoted_js_string(arg.content);
                        let inner = key
                            .strip_prefix('"')
                            .and_then(|key| key.strip_suffix('"'))
                            .unwrap_or_default();
                        self.push_then_mapped("\"", inner, arg.loc.span.start);
                        self.push("\"");
                    }
                }
                Some(arg) => {
                    dynamic = true;
                    let key = self.expression_to_string(arg);
                    self.push("[");
                    self.push_expression_text(&key, arg.loc().span);
                    self.push("]");
                }
                None => self.push("default"),
            }
            self.push(": () => {\n");
            self.indent_level += 1;
            let old_parts = std::mem::take(&mut self.current_template_parts);
            self.process_children_with_fallthrough_attrs_and_css_vars(
                &template.children,
                false,
                false,
                false,
                false,
                RootCssVars {
                    enabled: css_vars,
                    template_wrapper: true,
                },
            );
            self.flush_push();
            self.current_template_parts = old_parts;
            self.indent_level -= 1;
            self.push_indent();
            self.push("},\n");
        }
        super::normalize_implicit_slot_children(&mut implicit);
        if !implicit.is_empty() {
            self.push_indent();
            self.push("default: () => {\n");
            self.indent_level += 1;
            let old_parts = std::mem::take(&mut self.current_template_parts);
            for child in implicit {
                self.process_child(
                    child,
                    false,
                    false,
                    false,
                    RootCssVars {
                        enabled: css_vars,
                        template_wrapper: true,
                    },
                );
            }
            self.flush_push();
            self.current_template_parts = old_parts;
            self.indent_level -= 1;
            self.push_indent();
            self.push("},\n");
        }
        self.push_indent();
        self.push(if dynamic { "_: 2\n" } else { "_: 1\n" });
    }
}
