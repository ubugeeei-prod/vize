//! The merged-props path (`codegen::element::merged` over plan segments):
//! `@vue/compiler-ssr` 3.5's `needMergeProps` branch. The fallthrough root and
//! any element with an object spread, a dynamic-key bind, or a custom
//! directive render every attribute through one `_ssrRenderAttrs(...)` call:
//! source-ordered segments with each spread flushing the accumulated object,
//! `_attrs`, the moved `v-show` style, the runtime-typed model props, then one
//! `_ssrGetDirectiveProps(...)` per custom directive. A directive (or a merged
//! `<textarea>`) may own the element's content through a `_temp` binding.

use vize_atelier_core::RuntimeHelper;
use vize_s0::{String, ToCompactString, camelize, cstr};
use vize_s1_to_s2::{TransformContent, decode_template_entities};
use vize_s2::op::{self as s2, DynamicName};

use super::attrs::{Attached, BindName, bind};
use super::spans::{argument_start, attribute_value_start};
use super::{Emitter, Result, model, plan_source};
use crate::codegen::element::VNodePropEntry;
use crate::codegen::element::props::{
    component_prop_entry, component_props_object, normalize_prop_entries, quoted_js_string,
};
use crate::codegen::element::spanned_props::{
    bound_entry, component_props_object_spanned, merge_props_call,
};
use crate::s4::string_plan::{
    SsrSegmentSource as Source, SsrStringPayloadKind, SsrStringSegmentKind as Kind,
};
use vize_atelier_core::codegen::document::EmitDocument;

/// The merged attributes of one element and the content a temp owns.
pub(super) struct Merged {
    pub(super) attrs: EmitDocument,
    pub(super) content: Option<String>,
}

/// Vue's `needMergeProps`: an object spread or dynamic-key bind, or a custom
/// directive.
pub(super) fn needs_merged(attached: &Attached<'_, '_>) -> bool {
    attached.iter().any(|segment| match segment.source {
        Source::Binding(s2::BindingOp::Bind(bind)) => {
            !matches!(bind.name, Some(DynamicName::Static(_)))
        }
        Source::Binding(s2::BindingOp::VueDirective(_)) => true,
        _ => false,
    })
}

fn has_binding(attached: &Attached<'_, '_>, test: impl Fn(&s2::BindingOp<'_>) -> bool) -> bool {
    attached.iter().any(|segment| match segment.source {
        Source::Binding(binding) => test(binding),
        _ => false,
    })
}

/// A static `type` spelling: the attribute or a static-name bind.
fn has_static_type(attached: &Attached<'_, '_>) -> bool {
    attached.iter().any(|segment| match segment.source {
        Source::Attribute(attr) => attr.name == "type",
        Source::Binding(s2::BindingOp::Bind(bind)) => {
            matches!(bind.name, Some(DynamicName::Static("type")))
        }
        _ => false,
    })
}

impl Emitter<'_, '_, '_, '_, '_, '_> {
    /// `_ssrRenderAttrs(props[, "tag"])` plus temp-owned content. The cursor
    /// sits on the element's first child.
    pub(super) fn merged_attrs(
        &mut self,
        attached: &Attached<'_, '_>,
        tag: &str,
        inherit: bool,
    ) -> Result<Option<Merged>> {
        let mut args = self.merged_props_args(attached, tag, inherit)?;
        let mut directives = 0usize;
        for segment in attached {
            if let Source::Binding(s2::BindingOp::VueDirective(directive)) = segment.source {
                let props = self.directive_props(directive)?;
                args.push(EmitDocument::from(props));
                directives += 1;
            }
        }
        if args.is_empty() {
            return Ok(None);
        }
        let mut merged = self.merge(&args);
        let content_override = has_binding(attached, |binding| {
            matches!(
                binding,
                s2::BindingOp::VueHtml(_) | s2::BindingOp::VueText(_)
            )
        });
        let children = self.direct_children(self.pos)?;
        let mut content = None;
        if tag == "textarea" {
            let first = children.first().map(|&at| self.segments[at]);
            // Legacy reads the first child only: a merged text-led run has
            // no single legacy child to mirror.
            if let Some(segment) = first
                && segment.kind == Kind::DynamicText
                && let Source::Interpolation(interpolation) = segment.source
                && super::text::compound_parts(self.facts.texts, &segment, interpolation)?
                    .is_some_and(|parts| parts.first().is_some_and(|part| !part.dynamic))
            {
                return Err(crate::s4::LegacyReason::Structure.into());
            }
            let interpolated = matches!(first, Some(segment) if segment.kind == Kind::DynamicText)
                || has_binding(attached, |binding| {
                    matches!(binding, s2::BindingOp::Model(_))
                });
            if !content_override && !interpolated {
                let fallback = match first {
                    Some(segment) if segment.kind == Kind::Text => {
                        let text = plan_source(&segment, SsrStringPayloadKind::Text)?;
                        quoted_js_string(&decode_template_entities(text))
                    }
                    _ => "\"\"".to_compact_string(),
                };
                let temp = self.ctx.alloc_temp();
                merged = assigned(&temp, &merged);
                self.ctx.use_ssr_helper(RuntimeHelper::SsrInterpolate);
                content = Some(cstr!(
                    "_ssrInterpolate((\"value\" in {temp}) ? {temp}.value : {fallback})"
                ));
            }
        } else if tag != "input" && directives > 0 && children.is_empty() && !content_override {
            let temp = self.ctx.alloc_temp();
            merged = assigned(&temp, &merged);
            self.ctx.use_ssr_helper(RuntimeHelper::SsrInterpolate);
            content = Some(cstr!(
                "(\"textContent\" in {temp}) ? _ssrInterpolate({temp}.textContent) : {temp}.innerHTML ?? ''"
            ));
        }
        self.ctx.use_ssr_helper(RuntimeHelper::SsrRenderAttrs);
        let tag_arg = if tag == "textarea" || tag.contains('-') {
            cstr!(", \"{tag}\"")
        } else {
            String::default()
        };
        let mut attrs = EmitDocument::plain("_ssrRenderAttrs(");
        attrs.push_spanned(&merged);
        attrs.push_str(&tag_arg);
        attrs.push_str(")");
        Ok(Some(Merged { attrs, content }))
    }

    fn merged_props_args(
        &mut self,
        attached: &Attached<'_, '_>,
        tag: &str,
        inherit: bool,
    ) -> Result<std::vec::Vec<EmitDocument>> {
        let spans = self.ctx.spans_enabled();
        let mut args = std::vec::Vec::new();
        let mut entries: std::vec::Vec<VNodePropEntry> = std::vec::Vec::new();
        let mut show = None;
        let mut dynamic_model = None;
        let runtime_type = tag == "input"
            && has_binding(
                attached,
                |binding| matches!(binding, s2::BindingOp::Bind(bind) if !matches!(bind.name, Some(DynamicName::Static(_)))),
            )
            && !has_static_type(attached);
        for segment in attached {
            match segment.source {
                Source::Attribute(attr) => {
                    let value = attr
                        .value
                        .map(|value| quoted_js_string(&decode_template_entities(value)))
                        .unwrap_or_else(|| "\"\"".to_compact_string());
                    let name = plan_source(segment, SsrStringPayloadKind::AttributeName)?;
                    entries.push(if spans {
                        static_entry(self, attr, name, &value)
                    } else {
                        component_prop_entry(name, &value, false)
                    });
                }
                Source::Binding(binding) => match binding {
                    s2::BindingOp::Bind(_) => {
                        let Some(bind) = bind(binding)? else {
                            continue;
                        };
                        let value = self.expr(bind.value, TransformContent::Decoded)?;
                        match bind.name {
                            BindName::Spread => {
                                self.flush(&mut entries, &mut args);
                                args.push(EmitDocument::from(value));
                            }
                            BindName::Static(name) => {
                                let key = if bind.camel {
                                    camelize(name)
                                } else {
                                    name.to_compact_string()
                                };
                                entries.push(if spans {
                                    // The key maps to the authored argument only
                                    // when emitted verbatim, as the walker's.
                                    let key_start = (key == name)
                                        .then(|| argument_start(self.ctx.source, bind.span, name))
                                        .flatten();
                                    bound_entry(
                                        &key,
                                        key_start,
                                        self.bound_expression(&value, &bind),
                                    )
                                } else {
                                    component_prop_entry(&key, &value, false)
                                });
                            }
                            BindName::Dynamic(name) => {
                                let key = self.dynamic_key(name)?;
                                entries.push(component_prop_entry(&key, &value, true));
                            }
                        }
                    }
                    s2::BindingOp::Model(model) if runtime_type => {
                        dynamic_model =
                            Some(self.expr(&model.contract.read, TransformContent::Decoded)?);
                    }
                    s2::BindingOp::Model(model) => model::collect_root(
                        self,
                        attached,
                        model,
                        tag,
                        &mut entries,
                        &mut dynamic_model,
                    )?,
                    s2::BindingOp::VueShow(show_op) => {
                        show = Some(self.expr(&show_op.value, TransformContent::Decoded)?);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        self.flush(&mut entries, &mut args);
        if inherit {
            args.push(EmitDocument::plain("_attrs"));
        }
        if let Some(exp) = show {
            let style = cstr!("(({exp}) ? null : {{ display: \"none\" }})");
            args.push(EmitDocument::from(component_props_object(&[
                component_prop_entry("style", &style, false),
            ])));
        }
        if let Some(model) = dynamic_model {
            self.ctx
                .use_ssr_helper(RuntimeHelper::SsrGetDynamicModelProps);
            let existing: String = if args.is_empty() {
                "{}".to_compact_string()
            } else {
                self.merge(&args).as_str().into()
            };
            args.push(EmitDocument::from(cstr!(
                "_ssrGetDynamicModelProps({existing}, {model})"
            )));
        }
        Ok(args)
    }

    fn flush(
        &mut self,
        entries: &mut std::vec::Vec<VNodePropEntry>,
        args: &mut std::vec::Vec<EmitDocument>,
    ) {
        if entries.is_empty() {
            return;
        }
        let normalized = normalize_prop_entries(core::mem::take(entries));
        args.push(component_props_object_spanned(&normalized));
    }

    /// One argument stays itself; several merge through `_mergeProps`.
    pub(super) fn merge(&mut self, args: &[EmitDocument]) -> EmitDocument {
        if let [only] = args {
            return only.clone();
        }
        self.ctx.use_core_helper(RuntimeHelper::MergeProps);
        merge_props_call(args)
    }
}

/// `temp = merged`, keeping `merged`'s anchors.
fn assigned(temp: &str, merged: &EmitDocument) -> EmitDocument {
    let mut out = EmitDocument::plain(temp);
    out.push_str(" = ");
    out.push_spanned(merged);
    out
}

/// A static attribute's props-object entry with its key and value anchored
/// at the authored tokens, as the AST walker builds it.
fn static_entry(
    em: &Emitter<'_, '_, '_, '_, '_, '_>,
    attr: &s2::Attribute<'_>,
    name: &str,
    value: &str,
) -> VNodePropEntry {
    let start = attr
        .value
        .and_then(|_| attribute_value_start(em.ctx.source, attr.span, name));
    let spanned = match start {
        Some(start) if value.len() >= 2 => {
            let mut spanned = EmitDocument::plain("\"");
            spanned.push_mapped(&value[1..value.len() - 1], start);
            spanned.push_str("\"");
            spanned
        }
        _ => EmitDocument::plain(value),
    };
    bound_entry(name, Some(attr.span.start), spanned)
}
