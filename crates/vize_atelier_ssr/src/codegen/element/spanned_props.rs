//! Span-carrying SSR prop objects (Davinci P3-9, S4).
//!
//! Element attribute objects (`_ssrRenderAttrs(_mergeProps({ id: "x" }, _attrs))`)
//! are assembled from [`VNodePropEntry`] values before they are written. An
//! entry built for a map-requesting compile carries the authored spans of its
//! key and value, and [`component_props_object_spanned`] lays the object out
//! exactly as [`component_props_object`](super::props::component_props_object)
//! does while keeping those anchors.

use vize_atelier_core::{
    AttributeNode, DirectiveNode, ExpressionNode, codegen::spanned::SpannedText,
};
use vize_s0::{Span, String};

use super::props::{component_prop_entry, push_component_prop_entry, push_js_object_key};
use super::{SsrCodegenContext, VNodePropEntry};

/// Authored spans of one generated prop entry.
#[derive(Clone, Debug)]
pub(crate) struct PropEntrySpans {
    /// Authored start of the key token (attribute name or `v-bind` argument),
    /// when the key is emitted verbatim.
    key: Option<u32>,
    /// The generated value, carrying its own anchors.
    value: SpannedText,
}

/// A static attribute entry whose generated `value` is the quoted attribute
/// text; with `spans`, the key and value map to the authored tokens.
pub(super) fn attribute_entry(attr: &AttributeNode, value: &str, spans: bool) -> VNodePropEntry {
    let mut entry = component_prop_entry(attr.name, value, false);
    if spans {
        let mut spanned = SpannedText::default();
        match &attr.value {
            Some(authored) if value.len() >= 2 => {
                spanned.push_str("\"");
                spanned.push_mapped(&value[1..value.len() - 1], authored.loc.span.start);
                spanned.push_str("\"");
            }
            _ => spanned.push_str(value),
        }
        entry.spans = Some(Box::new(PropEntrySpans {
            key: Some(attr.name_loc.span.start),
            value: spanned,
        }));
    }
    entry
}

/// A statically keyed entry whose value is an already spanned expression.
pub(crate) fn bound_entry(key: &str, key_start: Option<u32>, value: SpannedText) -> VNodePropEntry {
    let mut entry = component_prop_entry(key, value.as_str(), false);
    entry.spans = Some(Box::new(PropEntrySpans {
        key: key_start,
        value,
    }));
    entry
}

impl SsrCodegenContext<'_> {
    /// A statically keyed `v-bind` entry; with maps on, the key maps to the
    /// authored argument (when emitted verbatim) and the value to its
    /// expression.
    pub(super) fn bound_prop_entry(
        &self,
        key: &str,
        arg: &ExpressionNode<'_>,
        dir: &DirectiveNode<'_>,
        value: &str,
    ) -> VNodePropEntry {
        if !self.spans_enabled() {
            return component_prop_entry(key, value, false);
        }
        let verbatim = matches!(arg, ExpressionNode::Simple(simple) if simple.content == key);
        let span = dir
            .exp
            .as_ref()
            .map_or(Span::new(0, 0), |exp| exp.loc().span);
        bound_entry(
            key,
            verbatim.then(|| arg.loc().span.start),
            self.spanned_expression(value, span),
        )
    }
}

/// `callee(arg)` keeping `arg`'s anchors.
pub(crate) fn wrap_spanned(callee: &str, arg: &SpannedText) -> SpannedText {
    let mut out = SpannedText::plain(callee);
    out.push_str("(");
    out.push_spanned(arg);
    out.push_str(")");
    out
}

/// The object literal [`component_props_object`](super::props::component_props_object)
/// emits, with entry anchors kept.
pub(crate) fn component_props_object_spanned(entries: &[VNodePropEntry]) -> SpannedText {
    let mut out = SpannedText::plain("{ ");
    for (index, entry) in entries.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        match (&entry.spans, entry.dynamic) {
            (Some(spans), false) => {
                debug_assert_eq!(spans.value.as_str(), entry.value.as_str());
                let mut key = String::default();
                push_js_object_key(&mut key, &entry.key);
                let quote = usize::from(key.starts_with('"'));
                out.push_str(&key[..quote]);
                match spans.key {
                    Some(start) => out.push_mapped(&key[quote..], start),
                    None => out.push_str(&key[quote..]),
                }
                out.push_str(": ");
                out.push_spanned(&spans.value);
            }
            _ => {
                let mut text = String::default();
                push_component_prop_entry(&mut text, entry);
                out.push_str(&text);
            }
        }
    }
    out.push_str(" }");
    out
}

/// `_mergeProps(a, b, ...)` over spanned arguments.
pub(crate) fn merge_props_call(args: &[SpannedText]) -> SpannedText {
    let mut out = SpannedText::plain("_mergeProps(");
    for (index, arg) in args.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_spanned(arg);
    }
    out.push_str(")");
    out
}
