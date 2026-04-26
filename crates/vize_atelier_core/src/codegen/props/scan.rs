//! Single-pass prop metadata collection for code generation.

use crate::ast::{DirectiveNode, ExpressionNode, PropNode};
use crate::options::BindingType;

use super::super::context::CodegenContext;
use super::directives::is_supported_directive;
use super::events::get_von_event_key;
use vize_carton::{FxHashMap, String};

#[derive(Default)]
pub(super) struct EventNameCounts {
    first_key: Option<String>,
    first_count: usize,
    many: Option<FxHashMap<String, usize>>,
}

impl EventNameCounts {
    #[inline]
    pub(super) fn count(&self, key: &str) -> usize {
        if let Some(counts) = &self.many {
            return counts.get(key).copied().unwrap_or(0);
        }

        if self.first_key.as_deref() == Some(key) {
            self.first_count
        } else {
            0
        }
    }

    fn observe(&mut self, key: String) {
        if let Some(counts) = &mut self.many {
            *counts.entry(key).or_insert(0) += 1;
            return;
        }

        if let Some(first_key) = &self.first_key {
            if first_key.as_str() == key.as_str() {
                self.first_count += 1;
                return;
            }

            let first_key = self.first_key.take().expect("first event key");
            let mut counts = FxHashMap::default();
            counts.insert(first_key, self.first_count);
            counts.insert(key, 1);
            self.first_count = 0;
            self.many = Some(counts);
            return;
        }

        self.first_key = Some(key);
        self.first_count = 1;
    }
}

pub(super) struct PropsScan<'props> {
    pub(super) static_class: Option<&'props str>,
    pub(super) static_style: Option<&'props str>,
    pub(super) has_vbind_obj: bool,
    pub(super) has_von_obj: bool,
    pub(super) has_other: bool,
    has_dynamic_key: bool,
    has_dynamic_vmodel: bool,
    has_dynamic_class: bool,
    has_dynamic_style: bool,
    visible_prop_count: usize,
    visible_class_attrs: usize,
    visible_style_attrs: usize,
    has_normalizer: bool,
    has_inline_handler: bool,
    pub(super) event_counts: EventNameCounts,
}

impl<'props> PropsScan<'props> {
    pub(super) fn new<'ast>(
        ctx: &CodegenContext,
        props: &'props [PropNode<'ast>],
        skip_is: bool,
    ) -> Self {
        let mut scan = Self {
            static_class: None,
            static_style: None,
            has_vbind_obj: false,
            has_von_obj: false,
            has_other: false,
            has_dynamic_key: false,
            has_dynamic_vmodel: false,
            has_dynamic_class: false,
            has_dynamic_style: false,
            visible_prop_count: 0,
            visible_class_attrs: 0,
            visible_style_attrs: 0,
            has_normalizer: false,
            has_inline_handler: false,
            event_counts: EventNameCounts::default(),
        };

        for prop in props {
            scan.observe_other_prop(prop);

            let visible = !skip_is || !is_is_prop(prop);
            match prop {
                PropNode::Attribute(attr) => {
                    if attr.name == "class" {
                        if scan.static_class.is_none() {
                            scan.static_class = attr.value.as_ref().map(|v| v.content.as_str());
                        }
                        if visible {
                            scan.visible_class_attrs += 1;
                        }
                    } else if attr.name == "style" {
                        if scan.static_style.is_none() {
                            scan.static_style = attr.value.as_ref().map(|v| v.content.as_str());
                        }
                        if visible {
                            scan.visible_style_attrs += 1;
                        }
                    }

                    if visible {
                        scan.visible_prop_count += 1;
                    }
                }
                PropNode::Directive(dir) => {
                    scan.observe_directive(ctx, dir);
                    if visible && is_supported_directive(dir) {
                        scan.visible_prop_count += 1;
                    }
                }
            }
        }

        scan
    }

    #[inline]
    pub(super) fn needs_normalize(&self) -> bool {
        self.has_dynamic_vmodel || self.has_dynamic_key
    }

    #[inline]
    pub(super) fn skip_static_class(&self) -> bool {
        self.static_class.is_some() && self.has_dynamic_class
    }

    #[inline]
    pub(super) fn skip_static_style(&self) -> bool {
        self.static_style.is_some() && self.has_dynamic_style
    }

    #[inline]
    pub(super) fn visible_count(&self, has_scope_id: bool) -> usize {
        let mut count = self.visible_prop_count;
        if self.skip_static_class() {
            count = count.saturating_sub(self.visible_class_attrs);
        }
        if self.skip_static_style() {
            count = count.saturating_sub(self.visible_style_attrs);
        }
        if has_scope_id {
            count += 1;
        }
        count
    }

    #[inline]
    pub(super) fn multiline(&self, has_scope_id: bool) -> bool {
        self.visible_count(has_scope_id) > 1 || self.has_normalizer || self.has_inline_handler
    }

    fn observe_other_prop<'ast>(&mut self, prop: &PropNode<'ast>) {
        if self.has_other {
            return;
        }

        self.has_other = match prop {
            PropNode::Attribute(_) => true,
            PropNode::Directive(dir) => {
                if (dir.name == "bind" || dir.name == "on") && dir.arg.is_none() {
                    false
                } else {
                    is_supported_directive(dir)
                }
            }
        };
    }

    fn observe_directive(&mut self, ctx: &CodegenContext, dir: &DirectiveNode<'_>) {
        match dir.name.as_str() {
            "bind" => {
                if dir.arg.is_none() {
                    self.has_vbind_obj = true;
                    return;
                }

                if let Some(ExpressionNode::Simple(exp)) = &dir.arg {
                    if !exp.is_static {
                        self.has_dynamic_key = true;
                    }

                    if exp.content == "class" {
                        self.has_dynamic_class = true;
                        self.has_normalizer = true;
                    } else if exp.content == "style" {
                        self.has_dynamic_style = true;
                        self.has_normalizer = true;
                    }
                }
            }
            "on" => {
                if dir.arg.is_none() {
                    self.has_von_obj = true;
                }
                if !self.has_inline_handler && has_inline_handler(ctx, dir) {
                    self.has_inline_handler = true;
                }
                if let Some(event_key) = get_von_event_key(dir) {
                    self.event_counts.observe(event_key);
                }
            }
            "model" => {
                self.has_dynamic_vmodel |= dir.arg.as_ref().is_some_and(|arg| match arg {
                    ExpressionNode::Simple(exp) => !exp.is_static,
                    ExpressionNode::Compound(_) => true,
                });
            }
            "text" => {
                self.has_normalizer = true;
            }
            _ => {}
        }
    }
}

fn is_is_prop(prop: &PropNode<'_>) -> bool {
    match prop {
        PropNode::Attribute(attr) => attr.name == "is",
        PropNode::Directive(dir) => {
            dir.name == "bind"
                && matches!(&dir.arg, Some(ExpressionNode::Simple(exp)) if exp.content == "is")
        }
    }
}

fn has_inline_handler(ctx: &CodegenContext, dir: &DirectiveNode<'_>) -> bool {
    if ctx.cache_handlers_in_current_scope()
        && dir.exp.is_some()
        && !is_setup_const_handler(ctx, dir)
    {
        return true;
    }

    if dir.modifiers.iter().any(|m| {
        let name = m.content.as_str();
        !matches!(name, "capture" | "once" | "passive")
    }) {
        return true;
    }

    dir.exp.as_ref().is_some_and(|exp| {
        if let ExpressionNode::Simple(simple) = exp {
            let content = simple.content.as_str();
            content.contains('(')
                || content.contains('+')
                || content.contains('-')
                || content.contains('=')
                || content.contains(' ')
        } else {
            false
        }
    })
}

fn is_setup_const_handler(ctx: &CodegenContext, dir: &DirectiveNode<'_>) -> bool {
    dir.exp.as_ref().is_some_and(|exp| {
        if let ExpressionNode::Simple(simple) = exp {
            if !simple.is_static {
                let content = simple.content.trim();
                if crate::transforms::is_simple_identifier(content) {
                    return ctx
                        .options
                        .binding_metadata
                        .as_ref()
                        .and_then(|metadata| metadata.bindings.get(content))
                        .is_some_and(|binding| *binding == BindingType::SetupConst);
                }
            }
        }
        false
    })
}
