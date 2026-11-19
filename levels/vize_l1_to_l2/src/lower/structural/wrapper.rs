//! The `<template v-if>` wrapper-key channel (P2-9 series 5): the
//! branch key the legacy transform lifts off the wrapper element
//! (`extract_key_prop` runs on the branch node itself, template
//! wrappers included). The wrapper unwraps at lowering, so its key has
//! no op to ride — it is captured here as a fact channel the `v-if`
//! pass folds into branch-key facts — and the remaining wrapper
//! attributes stay recorded drops.

use alloc::vec::Vec as StdVec;
use vize_l2::analysis::fragment_facts::{FragmentAttr, FragmentClass, FragmentKey};
use vize_l2::provenance::ProvenanceSink;

use vize_l0::id::NodeId;
use vize_l0::{Span, String, cstr};
use vize_l1::markup::Element;

use super::super::cx::{Cx, attr_slice, attr_span};
use super::super::directive::{Arg, AttrForm, Head};
use super::super::element::{Analyzed, BranchKind, attr_text};
use super::super::expr::expr_at;

/// The first key-ish attribute of a `<template v-if>` wrapper: a static
/// `key` attribute or a static-argument `:key` binding, in authored
/// order — the legacy `extract_key_prop` predicate. A dynamic-argument
/// `:[key]` is **not** captured (the legacy lane's arg-content match
/// admits it — a recorded quirk the differential lane counts rather
/// than imitates).
pub(crate) fn capture_wrapper_key<'a>(
    cx: &mut Cx<'a, impl ProvenanceSink>,
    element: &Element<'a>,
    analyzed: &Analyzed<'a>,
) -> Option<(usize, FragmentKey)> {
    for (index, (attr, form)) in element.open.attrs.iter().zip(&analyzed.forms).enumerate() {
        if Some(index) == analyzed.branch.map(|(idx, _)| idx) || Some(index) == analyzed.vfor {
            continue;
        }
        match form {
            AttrForm::Static if attr.name.text == "key" => {
                return Some((
                    index,
                    FragmentKey::Static {
                        value: attr_text(attr).map(String::from),
                        span: attr_span(cx, attr),
                    },
                ));
            }
            AttrForm::Directive(directive)
                if directive.head == Head::Bind && directive.arg == Some(Arg::Static("key")) =>
            {
                let source = match attr_text(attr) {
                    Some(text) => String::from(text.trim()),
                    // The parser's same-name expansion: a valueless
                    // `:key` reads `key`.
                    None => String::from("key"),
                };
                return Some((
                    index,
                    FragmentKey::Dynamic {
                        source,
                        span: attr_span(cx, attr),
                    },
                ));
            }
            _ => {}
        }
    }
    None
}

/// Keep a dynamic wrapper `:key` on the lint expression index.
///
/// Unwrapping stores the key as an owned fragment fact, so template
/// expression rules never see it. The attribute value is still a source
/// slice here. Static keys and a valueless `:key` are not expressions.
pub(crate) fn retain_dynamic_wrapper_key<'a>(
    cx: &mut Cx<'a, impl ProvenanceSink>,
    element: &Element<'a>,
    index: usize,
    key: &FragmentKey,
) {
    if !cx.retain_consumed_expressions {
        return;
    }
    if !matches!(key, FragmentKey::Dynamic { .. }) {
        return;
    }
    let Some(text) = element.open.attrs.get(index).and_then(attr_text) else {
        return;
    };
    cx.consumed_expressions.push(expr_at(cx, text));
}

/// Static attributes that ride on a lowered `<template v-for>` fragment.
pub(crate) fn capture_wrapper_attrs<'a>(
    cx: &mut Cx<'a, impl ProvenanceSink>,
    element: &Element<'a>,
    analyzed: &Analyzed<'a>,
    captured_key: Option<usize>,
) -> (StdVec<usize>, StdVec<FragmentAttr>, Option<FragmentClass>) {
    let mut indexes = StdVec::new();
    let mut attributes = StdVec::new();
    let mut class = None;
    for (index, (attr, form)) in element.open.attrs.iter().zip(&analyzed.forms).enumerate() {
        if Some(index) == analyzed.branch.map(|(idx, _)| idx)
            || Some(index) == analyzed.vfor
            || Some(index) == captured_key
        {
            continue;
        }
        match form {
            AttrForm::Static if attr.name.text == "class" => {
                indexes.push(index);
                let entry = class.get_or_insert_with(|| FragmentClass {
                    static_value: None,
                    dynamic_source: None,
                    span: attr_span(cx, attr),
                });
                entry.static_value = Some(String::from(
                    attr.value.as_ref().map_or("", |value| value.content.text),
                ));
            }
            AttrForm::Static => {
                indexes.push(index);
                attributes.push(FragmentAttr {
                    name: String::from(attr.name.text),
                    value: attr
                        .value
                        .as_ref()
                        .map(|value| String::from(value.content.text)),
                    span: attr_span(cx, attr),
                });
            }
            AttrForm::Directive(directive)
                if directive.head == Head::Bind
                    && directive.arg == Some(Arg::Static("class"))
                    && directive.modifiers.is_empty() =>
            {
                let source = match attr_text(attr) {
                    Some(text) if text.trim().is_empty() => continue,
                    Some(text) => String::from(text.trim()),
                    None => String::from("class"),
                };
                indexes.push(index);
                let entry = class.get_or_insert_with(|| FragmentClass {
                    static_value: None,
                    dynamic_source: None,
                    span: attr_span(cx, attr),
                });
                entry.dynamic_source = Some(source);
                entry.span = attr_span(cx, attr);
            }
            _ => {}
        }
    }
    (indexes, attributes, class)
}

/// The first runtime directive (`v-show` or a custom directive) on a
/// `<template v-for>` wrapper; the shipped lane keeps it on the fragment.
pub(crate) fn runtime_directive(
    cx: &Cx<'_, impl ProvenanceSink>,
    element: &Element<'_>,
    analyzed: &Analyzed<'_>,
) -> Option<Span> {
    element
        .open
        .attrs
        .iter()
        .zip(&analyzed.forms)
        .find(|(_, form)| {
            matches!(form, AttrForm::Directive(directive) if matches!(directive.head, Head::Show | Head::Custom))
        })
        .map(|(attr, _)| attr_span(cx, attr))
}

/// The `v-once` on the `<template v-if>` that carries a chain. It caches the
/// whole chain, so it is a wrapper fact and not a dropped attribute. Vue
/// ignores it on a `v-else-if` / `v-else` carrier, where it stays a drop.
pub(crate) fn chain_once_attr(element: &Element<'_>, analyzed: &Analyzed<'_>) -> Option<usize> {
    if matches!(
        analyzed.branch,
        Some((_, BranchKind::ElseIf | BranchKind::Else))
    ) {
        return None;
    }
    element
        .open
        .attrs
        .iter()
        .position(|attr| attr.name.text == "v-once")
}

/// Record one captured wrapper key under provenance.
pub(crate) fn record_wrapper_key(
    cx: &mut Cx<'_, impl ProvenanceSink>,
    node: Option<NodeId>,
    key: &FragmentKey,
    branch: usize,
) {
    let (FragmentKey::Static { span, .. } | FragmentKey::Dynamic { span, .. }) = key;
    let spelling = cx
        .source
        .get(span.start as usize..span.end as usize)
        .unwrap_or("key");
    cx.record(
        "lower.branch-wrapper-key",
        node,
        spelling,
        || cstr!("wrapper key branch={branch}"),
        *span,
    );
}

/// A `<template>` wrapper the lowering unwraps has no op to carry its
/// remaining attributes; each is dropped under an `Info` diagnostic and
/// a record — dropping is never silent. `captured` names the wrapper-key
/// attribute the caller lifted into the wrapper-key channel (P2-9
/// series 5), and `extra_captured` the attributes another wrapper fact
/// carries; neither is a drop.
pub(crate) fn record_template_drops_except<'a>(
    cx: &mut Cx<'a, impl ProvenanceSink>,
    element: &Element<'a>,
    analyzed: &Analyzed<'a>,
    captured: Option<usize>,
    extra_captured: &[usize],
) {
    for (index, attr) in element.open.attrs.iter().enumerate() {
        if Some(index) == analyzed.branch.map(|(idx, _)| idx)
            || Some(index) == analyzed.vfor
            || Some(index) == captured
            || extra_captured.contains(&index)
        {
            continue;
        }
        cx.info(
            attr_span(cx, attr),
            cstr!(
                "`{}` on an unwrapped `<template>` wrapper has no L2 home; it is dropped under provenance",
                attr.name.text
            ),
        );
        cx.record(
            "drop.template-attribute",
            None,
            attr_slice(cx, attr),
            String::default,
            attr_span(cx, attr),
        );
    }
}
