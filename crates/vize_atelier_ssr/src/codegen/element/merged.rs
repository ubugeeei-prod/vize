//! The merged-props path for plain elements: `@vue/compiler-ssr` 3.5's
//! `ssrTransformElement` `needMergeProps` branch and its `buildSSRProps`.
//!
//! An element renders every attribute through one `_ssrRenderAttrs(...)` call
//! when it carries an object spread, a dynamic-key bind, or a custom
//! directive, and always on the fallthrough root (whose `_attrs` is an
//! injected object spread). The props follow Vue's `buildProps` order: source
//! order with each spread flushing the accumulated object, `_attrs` where the
//! injected bind sits, `v-show` moved last, and one
//! `_ssrGetDirectiveProps(...)` per custom directive after everything else.
//! A directive may also own the element's content through a `_temp` binding.

use super::props::{
    component_prop_entry, component_props_object, is_static_named_prop, normalize_prop_entries,
    quoted_js_string,
};
use super::spanned_props::{attribute_entry, component_props_object_spanned, merge_props_call};
use super::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, RuntimeHelper, SsrCodegenContext, String,
    TemplateChildNode, ToCompactString, VNodePropEntry, cstr,
};
use vize_atelier_core::codegen::document::EmitDocument;
use vize_s0::{camelize, is_builtin_directive};

/// The merged attributes of one element and the content a temp owns.
pub(super) struct MergedAttrs {
    pub(super) attrs: EmitDocument,
    pub(super) content: Option<String>,
}

/// Vue's `needMergeProps`: an object spread or dynamic-key bind
/// (`hasDynamicKeyVBind`), or a custom directive.
pub(super) fn needs_merged_props(el: &ElementNode) -> bool {
    el.props.iter().any(|prop| match prop {
        PropNode::Directive(dir) if dir.name == "bind" => !matches!(
            &dir.arg,
            Some(ExpressionNode::Simple(arg)) if arg.is_static
        ),
        PropNode::Directive(dir) => !is_builtin_directive(dir.name),
        PropNode::Attribute(_) => false,
    })
}

fn custom_directives<'n, 'a>(
    el: &'n ElementNode<'a>,
) -> impl Iterator<Item = &'n DirectiveNode<'a>> {
    el.props.iter().filter_map(|prop| match prop {
        PropNode::Directive(dir) if !is_builtin_directive(dir.name) => Some(dir.as_ref()),
        _ => None,
    })
}

fn has_directive(el: &ElementNode, name: &str) -> bool {
    el.props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == name))
}

impl<'a> SsrCodegenContext<'a> {
    fn collect_v_model_element_attr(
        &mut self,
        el: &ElementNode,
        dir: &DirectiveNode,
        entries: &mut std::vec::Vec<VNodePropEntry>,
        dynamic_model_exp: &mut Option<String>,
    ) {
        let Some(exp) = dir.exp.as_ref().map(|exp| self.expression_to_string(exp)) else {
            return;
        };

        if el.tag == "input" {
            if self.get_dynamic_bind_exp(el, "type").is_some() {
                *dynamic_model_exp = Some(exp);
                return;
            }

            let input_type = self.get_element_attr_value(el, "type");
            match input_type.as_deref() {
                Some("checkbox") => {
                    self.use_ssr_helper(RuntimeHelper::SsrLooseContain);
                    entries.push(component_prop_entry(
                        "checked",
                        &cstr!("(Array.isArray({exp}) ? _ssrLooseContain({exp}, null) : {exp})"),
                        false,
                    ));
                }
                Some("radio") => {
                    self.use_ssr_helper(RuntimeHelper::SsrLooseEqual);
                    let value = self
                        .get_element_attr_value(el, "value")
                        .map(|value| quoted_js_string(&value))
                        .unwrap_or_else(|| "null".to_compact_string());
                    entries.push(component_prop_entry(
                        "checked",
                        &cstr!("_ssrLooseEqual({exp}, {value})"),
                        false,
                    ));
                }
                _ => entries.push(component_prop_entry("value", &exp, false)),
            }
        }
    }

    /// `_ssrRenderAttrs(props[, "tag"])` for a merged element, plus the
    /// content a directive-props temp owns (`rawChildrenMap` upstream).
    pub(super) fn merged_element_attrs(
        &mut self,
        el: &ElementNode<'a>,
        inherit_attrs: bool,
    ) -> Option<MergedAttrs> {
        let mut args = self.merged_props_args(el, inherit_attrs);
        let has_directives = custom_directives(el).next().is_some();
        for dir in custom_directives(el) {
            let props = self.directive_props(dir);
            args.push(EmitDocument::from(props));
        }
        if args.is_empty() {
            return None;
        }
        let mut merged = self.merge_args(&args);
        let content_override = has_directive(el, "html") || has_directive(el, "text");
        let mut content = None;
        if el.tag == "textarea" {
            let existing = el.children.first();
            let interpolated = matches!(existing, Some(TemplateChildNode::Interpolation(_)))
                || has_directive(el, "model");
            if !content_override && !interpolated {
                let temp = self.alloc_temp();
                let fallback = match existing {
                    Some(TemplateChildNode::Text(text)) => quoted_js_string(text.content),
                    _ => "\"\"".to_compact_string(),
                };
                merged = assigned(&temp, &merged);
                self.use_ssr_helper(RuntimeHelper::SsrInterpolate);
                content = Some(cstr!(
                    "_ssrInterpolate((\"value\" in {temp}) ? {temp}.value : {fallback})"
                ));
            }
        } else if el.tag != "input" && has_directives && el.children.is_empty() && !content_override
        {
            let temp = self.alloc_temp();
            merged = assigned(&temp, &merged);
            self.use_ssr_helper(RuntimeHelper::SsrInterpolate);
            content = Some(cstr!(
                "(\"textContent\" in {temp}) ? _ssrInterpolate({temp}.textContent) : {temp}.innerHTML ?? ''"
            ));
        }
        self.use_ssr_helper(RuntimeHelper::SsrRenderAttrs);
        let tag_arg = if el.tag == "textarea" || el.tag.contains('-') {
            cstr!(", \"{}\"", el.tag)
        } else {
            String::default()
        };
        let mut attrs = EmitDocument::plain("_ssrRenderAttrs(");
        attrs.push_spanned(&merged);
        attrs.push_str(&tag_arg);
        attrs.push_str(")");
        Some(MergedAttrs { attrs, content })
    }

    /// `buildProps` over one plain element: source-ordered segments, the
    /// fallthrough `_attrs`, then the moved `v-show` style, then the
    /// dynamic-model props of a modelled input whose type is only known at
    /// runtime.
    fn merged_props_args(
        &mut self,
        el: &ElementNode,
        inherit_attrs: bool,
    ) -> std::vec::Vec<EmitDocument> {
        let mut args = std::vec::Vec::new();
        let mut entries: std::vec::Vec<VNodePropEntry> = std::vec::Vec::new();
        let mut show = None;
        let mut dynamic_model = None;
        let has_spread = needs_dynamic_type(el);
        for prop in &el.props {
            match prop {
                PropNode::Attribute(attr) => {
                    let value = attr
                        .value
                        .as_ref()
                        .map(|value| quoted_js_string(value.content))
                        .unwrap_or_else(|| "\"\"".to_compact_string());
                    entries.push(attribute_entry(attr, &value, self.spans_enabled()));
                }
                PropNode::Directive(dir) => match dir.name {
                    "bind" => {
                        let value = dir
                            .exp
                            .as_ref()
                            .map(|exp| self.expression_to_string(exp))
                            .unwrap_or_else(|| "undefined".to_compact_string());
                        match &dir.arg {
                            None => {
                                self.flush_entries(&mut entries, &mut args);
                                args.push(EmitDocument::from(value));
                            }
                            Some(arg @ ExpressionNode::Simple(simple)) if simple.is_static => {
                                // SSR keys take `.camel` but never the client
                                // `.prop` / `.attr` prefixes (`transformBind`
                                // skips them `inSSR`).
                                let key = self.expression_to_string(arg);
                                let camel = dir.modifiers.iter().any(|m| m.content == "camel");
                                let key = if camel { camelize(&key) } else { key };
                                entries.push(self.bound_prop_entry(&key, arg, dir, &value));
                            }
                            Some(arg) => {
                                let key = self.dynamic_arg_to_string(arg);
                                entries.push(component_prop_entry(&key, &value, true));
                            }
                        }
                    }
                    "model" => {
                        let runtime_type = has_spread
                            && !el
                                .props
                                .iter()
                                .any(|prop| is_static_named_prop(prop, "type"));
                        if runtime_type {
                            dynamic_model =
                                dir.exp.as_ref().map(|exp| self.expression_to_string(exp));
                        } else {
                            self.collect_v_model_element_attr(
                                el,
                                dir,
                                &mut entries,
                                &mut dynamic_model,
                            );
                        }
                    }
                    "show" => {
                        show = dir.exp.as_ref().map(|exp| self.expression_to_string(exp));
                    }
                    _ => {}
                },
            }
        }
        self.flush_entries(&mut entries, &mut args);
        if inherit_attrs {
            args.push(EmitDocument::plain("_attrs"));
        }
        if let Some(exp) = show {
            let style = cstr!("(({exp}) ? null : {{ display: \"none\" }})");
            args.push(EmitDocument::from(component_props_object(&[
                component_prop_entry("style", &style, false),
            ])));
        }
        if let Some(model) = dynamic_model {
            self.use_ssr_helper(RuntimeHelper::SsrGetDynamicModelProps);
            let existing: String = if args.is_empty() {
                "{}".to_compact_string()
            } else {
                self.merge_args(&args).as_str().into()
            };
            args.push(EmitDocument::from(cstr!(
                "_ssrGetDynamicModelProps({existing}, {model})"
            )));
        }
        args
    }

    fn flush_entries(
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
    pub(super) fn merge_args(&mut self, args: &[EmitDocument]) -> EmitDocument {
        if let [only] = args {
            return only.clone();
        }
        self.use_core_helper(RuntimeHelper::MergeProps);
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

/// An `<input v-model>` whose type may arrive through a spread or dynamic
/// key: `ssrTransformModel` leaves its value to `ssrGetDynamicModelProps`.
fn needs_dynamic_type(el: &ElementNode) -> bool {
    el.tag == "input"
        && el.props.iter().any(|prop| {
            matches!(
                prop,
                PropNode::Directive(dir) if dir.name == "bind" && !matches!(
                    &dir.arg,
                    Some(ExpressionNode::Simple(arg)) if arg.is_static
                )
            )
        })
}
