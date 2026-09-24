//! VNode fallback control flow: conditional chains and `_renderList`
//! blocks.

use vize_atelier_core::RuntimeHelper;
use vize_davinci::id::NodeId;
use vize_s0::{FxHashSet, String, ToCompactString, cstr};
use vize_s1_to_s2::lower::WrapperKey;
use vize_s1_to_s2::{TransformContent, decode_template_entities};
use vize_s2::expr::ExprRef;
use vize_s2::op::{self as s2, DynamicName};

use super::vnode::array;
use super::{Emitter, Result};
use crate::codegen::element::props::quoted_js_string;
use crate::codegen::helpers::extract_destructure_params;
use crate::s4::string_plan::{SsrSegmentSource as Source, SsrStringSegmentKind as Kind};
use crate::s4::{AdmissionFailure, LegacyReason};

fn node(fact: u32) -> Result<NodeId> {
    NodeId::from_index(fact).ok_or(AdmissionFailure::Invalid(
        "string plan fact index is not a node id",
    ))
}

fn alias_source<'a>(expr: &ExprRef<'a>) -> Result<&'a str> {
    match expr {
        ExprRef::Js(js) => Ok(js.source),
        ExprRef::Opaque(opaque) => Ok(opaque.source),
        _ => Err(LegacyReason::ExpressionOrEncoding.into()),
    }
}

impl<'r, 'a> Emitter<'_, 'r, 'a, '_, '_, '_> {
    /// `(cond) ? a : (cond) ? b : _createCommentVNode("")`.
    pub(super) fn vnode_if(&mut self, if_op: &'r s2::IfOp<'a>) -> Result<String> {
        let fact = self.segment(self.pos)?.fact;
        let from_template = self
            .facts
            .wrappers
            .get(node(fact)?)
            .map(|wrappers| wrappers.from_template.clone())
            .unwrap_or_default();
        self.pos += 1;
        self.ctx.use_core_helper(RuntimeHelper::CreateComment);
        let mut out = String::default();
        for (index, branch) in if_op.branches.iter().enumerate() {
            self.close(
                Kind::Branch,
                |source| matches!(source, Source::Branch(open) if core::ptr::eq(*open, branch)),
            )?;
            if index > 0 {
                out.push_str(" : ");
            }
            if let Some(condition) = &branch.condition {
                let condition = self.expr(condition, TransformContent::Decoded)?;
                out.push('(');
                out.push_str(&condition);
                out.push_str(") ? ");
            }
            self.branch_key = self.branch_key_span(fact, index)?;
            let expressions = self.vnode_region();
            self.branch_key = None;
            let expressions = expressions?;
            let template = from_template.get(index).copied().unwrap_or(false);
            out.push_str(&self.vnode_branch(expressions, template));
            self.close(
                Kind::CloseBranch,
                |source| matches!(source, Source::Branch(open) if core::ptr::eq(*open, branch)),
            )?;
        }
        if if_op
            .branches
            .iter()
            .all(|branch| branch.condition.is_some())
        {
            out.push_str(" : _createCommentVNode(\"\")");
        }
        self.close(
            Kind::CloseIf,
            |source| matches!(source, Source::If(open) if core::ptr::eq(*open, if_op)),
        )?;
        Ok(out)
    }

    /// One branch body; an unwrapped `<template>` is the legacy lane's single
    /// template child, rendered as its own fragment.
    fn vnode_branch(&mut self, expressions: std::vec::Vec<String>, template: bool) -> String {
        if template {
            return self.vnode_template(expressions);
        }
        match expressions.as_slice() {
            [] => "_createCommentVNode(\"\")".to_compact_string(),
            [only] => only.clone(),
            _ => array(&expressions),
        }
    }

    /// `vnode_template_expression`: one child stays itself, several become a
    /// `_Fragment` VNode.
    fn vnode_template(&mut self, expressions: std::vec::Vec<String>) -> String {
        if let [only] = expressions.as_slice() {
            return only.clone();
        }
        self.ctx.use_core_helper(RuntimeHelper::CreateVNode);
        self.ctx.use_core_helper(RuntimeHelper::Fragment);
        cstr!("_createVNode(_Fragment, null, {})", array(&expressions))
    }

    /// `(_openBlock(true), _createBlock(_Fragment, null, _renderList(...),
    /// 128 | 256))`.
    pub(super) fn vnode_for(&mut self, for_op: &'r s2::ForOp<'a>) -> Result<String> {
        let fact = self.segment(self.pos)?.fact;
        self.pos += 1;
        for helper in [
            RuntimeHelper::RenderList,
            RuntimeHelper::OpenBlock,
            RuntimeHelper::CreateBlock,
            RuntimeHelper::Fragment,
        ] {
            self.ctx.use_core_helper(helper);
        }
        let binding = &for_op.binding;
        let source = self.expr(&binding.source, TransformContent::Decoded)?;
        let value = alias_source(&binding.value)?;
        let key = binding.key.as_ref().map(alias_source).transpose()?;
        let index = binding.index.as_ref().map(alias_source).transpose()?;
        let aliases: std::vec::Vec<&str> =
            [Some(value), key, index].into_iter().flatten().collect();

        let mark = self.exprs.enter_for([Some(value), key, index]);
        let mut params = FxHashSet::default();
        for alias in &aliases {
            extract_destructure_params(alias.trim(), &mut params);
        }
        self.scoped_params.push(params);
        let body = self.vnode_for_body(fact, &for_op.region.ops);
        self.scoped_params.pop();
        self.exprs.leave(mark);
        let (body, keyed) = body?;
        self.close(
            Kind::CloseFor,
            |source| matches!(source, Source::For(open) if core::ptr::eq(*open, for_op)),
        )?;
        let flag = if keyed {
            "128 /* KEYED_FRAGMENT */"
        } else {
            "256 /* UNKEYED_FRAGMENT */"
        };
        let aliases = aliases.join(", ");
        Ok(cstr!(
            "(_openBlock(true), _createBlock(_Fragment, null, _renderList({source}, ({aliases}) => {{ return {body} }}), {flag}))"
        ))
    }

    /// The loop body and whether its single child carries a key.
    fn vnode_for_body(&mut self, fact: u32, ops: &'r [s2::Op<'a>]) -> Result<(String, bool)> {
        let wrapper = self.facts.for_wrappers.get(node(fact)?);
        let Some(wrapper) = wrapper else {
            let key = match ops {
                [s2::Op::Element(element)] => {
                    self.item_key(&element.attributes, &element.bindings)?
                }
                [s2::Op::Component(component)] => {
                    self.item_key(&component.attributes, &component.bindings)?
                }
                _ => None,
            };
            let expressions = self.vnode_region()?;
            return Ok((self.vnode_branch(expressions, false), key.is_some()));
        };
        let key = match &wrapper.key {
            None => None,
            Some(WrapperKey::Static { value, .. }) => Some(match value {
                Some(value) => quoted_js_string(&decode_template_entities(value)),
                None => "\"\"".to_compact_string(),
            }),
            Some(WrapperKey::Dynamic { source, .. }) => Some(self.text_expr(source)?),
        };
        let Some(key) = key else {
            let expressions = self.vnode_region()?;
            return Ok((self.vnode_template(expressions), false));
        };
        if let [s2::Op::Element(element)] = ops {
            return Ok((self.vnode_element(element, Some(&key))?, true));
        }
        let children = self.vnode_region()?;
        let children = if children.is_empty() {
            "[]".to_compact_string()
        } else {
            array(&children)
        };
        Ok((
            cstr!(
                "(_openBlock(), _createBlock(_Fragment, {{ key: {key} }}, {children}, 64 /* STABLE_FRAGMENT */))"
            ),
            true,
        ))
    }

    /// The first `key` attribute or `:key` binding of a loop item.
    fn item_key(
        &self,
        attributes: &[s2::Attribute<'_>],
        bindings: &[s2::BindingOp<'_>],
    ) -> Result<Option<String>> {
        let attribute = attributes.iter().find(|attr| attr.name == "key");
        let binding = bindings.iter().find_map(|binding| match binding {
            s2::BindingOp::Bind(bind) if matches!(bind.name, Some(DynamicName::Static("key"))) => {
                Some(bind)
            }
            _ => None,
        });
        match (attribute, binding) {
            (Some(attr), Some(bind)) if bind.span.start < attr.span.start => {
                self.bound_key_value(bind).map(Some)
            }
            (Some(attr), _) => Ok(Some(match attr.value {
                Some(value) => quoted_js_string(&decode_template_entities(value)),
                None => "\"\"".to_compact_string(),
            })),
            (None, Some(bind)) => self.bound_key_value(bind).map(Some),
            (None, None) => Ok(None),
        }
    }

    fn bound_key_value(&self, bind: &s2::BindOp<'_>) -> Result<String> {
        let value = bind.value.as_ref().ok_or(LegacyReason::Binding)?;
        self.expr(value, TransformContent::Decoded)
    }
}
