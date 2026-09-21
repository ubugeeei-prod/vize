//! Native element output: the start tag, the content the element's
//! directives own (`v-html`, `v-text`, `<textarea v-model>`,
//! `<select v-model>`), and the end tag.

use vize_atelier_core::RuntimeHelper;
use vize_s0::cstr;
use vize_s1_to_s2::TransformContent;
use vize_s2::op as s2;

use super::{Emitter, Flags, Result, attrs, fallthrough, model, plan_source};
use crate::s4::string_plan::{
    SsrSegmentSource as Source, SsrStringPayloadKind, SsrStringSegment,
    SsrStringSegmentKind as Kind,
};
use crate::s4::{AdmissionFailure, LegacyReason};

/// Elements whose content model or runtime helpers are outside the plan
/// emitter: raw-text / RCDATA parents and `<template>`.
const REFUSED_TAGS: &[&str] = &[
    "template",
    "slot",
    "component",
    "script",
    "style",
    "title",
    "xmp",
    "iframe",
    "noembed",
    "noframes",
    "noscript",
    "plaintext",
];

/// What renders between the start and end tags.
enum Content<'r, 'a> {
    Children,
    /// `v-html`: the raw value, children ignored.
    Html(&'r s2::VueHtmlOp<'a>),
    /// `v-text`: the interpolated value, children ignored.
    Text(&'r s2::VueTextOp<'a>),
    /// `<textarea v-model>`: the interpolated model read.
    TextareaModel(&'r s2::ModelOp<'a>),
    /// `<select v-model>`: children see the model read.
    SelectModel(&'r s2::ModelOp<'a>),
}

impl<'r, 'a> Emitter<'_, 'r, 'a, '_, '_, '_> {
    pub(super) fn element(
        &mut self,
        open: SsrStringSegment<'r, 'a>,
        element: &'r s2::ElementOp<'a>,
        inherit: bool,
    ) -> Result<()> {
        let tag = plan_source(&open, SsrStringPayloadKind::TagName)?;
        if REFUSED_TAGS.contains(&tag) || !vize_s0::is_native_tag(tag) {
            return Err(LegacyReason::Element.into());
        }
        self.pos += 1;
        let attached_len = element.attributes.len() + element.bindings.len();
        let end = self.pos + attached_len;
        let attached = self
            .segments
            .get(self.pos..end)
            .ok_or(AdmissionFailure::Invalid(
                "string plan lost attached element segments",
            ))?;
        self.pos = end;
        attrs::admit(attached, open.fact, tag)?;
        let content = content(attached, tag);

        self.ctx.push_string_part_static("<");
        self.ctx.push_string_part_static(tag);
        if inherit {
            fallthrough::emit(self, attached, tag)?;
        } else {
            attrs::emit_inline(self, attached, tag)?;
        }
        let options = self.ctx.options;
        if let Some(scope_id) = &options.scope_id {
            self.ctx.push_string_part_static(" ");
            self.ctx.push_string_part_static(scope_id);
        }
        if tag == "option" {
            model::emit_option_selected(self, attached)?;
        }
        self.ctx.push_string_part_static(">");

        let void = vize_s0::is_void_tag(tag);
        if void {
            if !element.children.ops.is_empty() {
                return Err(LegacyReason::Structure.into());
            }
        } else {
            self.content(content)?;
        }
        self.close(
            Kind::CloseElement,
            |source| matches!(source, Source::Element(closed) if core::ptr::eq(*closed, element)),
        )?;
        if !void {
            self.ctx.push_string_part_static("</");
            self.ctx.push_string_part_static(tag);
            self.ctx.push_string_part_static(">");
        }
        Ok(())
    }

    fn content(&mut self, content: Content<'r, 'a>) -> Result<()> {
        let flags = Flags {
            as_fragment: false,
            disable_nested_fragments: false,
            inherit_attrs: false,
        };
        match content {
            Content::Children => self.children(flags),
            Content::Html(html) => {
                let value = html.value.as_ref().ok_or(LegacyReason::Binding)?;
                let exp = self.expr(value, TransformContent::Decoded)?;
                self.ctx.push_string_part_dynamic(&cstr!("({exp}) ?? ''"));
                self.skip_children()
            }
            Content::Text(text) => {
                let value = text.value.as_ref().ok_or(LegacyReason::Binding)?;
                let exp = self.expr(value, TransformContent::Decoded)?;
                self.ctx.use_ssr_helper(RuntimeHelper::SsrInterpolate);
                self.ctx
                    .push_string_part_dynamic(&cstr!("_ssrInterpolate({exp})"));
                self.skip_children()
            }
            Content::TextareaModel(model) => {
                let exp = self.expr(&model.contract.read, TransformContent::Decoded)?;
                self.ctx.use_ssr_helper(RuntimeHelper::SsrInterpolate);
                self.ctx
                    .push_string_part_dynamic(&cstr!("_ssrInterpolate({exp})"));
                self.skip_children()
            }
            Content::SelectModel(model) => {
                let exp = self.expr(&model.contract.read, TransformContent::Decoded)?;
                self.select_models.push(exp);
                let result = self.children(flags);
                self.select_models.pop();
                result
            }
        }
    }

    /// Consume a child list the legacy walker never renders (the element's
    /// directive owns its content), keeping the plan balanced.
    fn skip_children(&mut self) -> Result<()> {
        let mut depth = 0usize;
        while let Some(segment) = self.segments.get(self.pos) {
            match segment.kind {
                Kind::OpenElement
                | Kind::If
                | Kind::Branch
                | Kind::For
                | Kind::Component
                | Kind::SlotOutlet => depth += 1,
                Kind::CloseElement
                | Kind::CloseIf
                | Kind::CloseBranch
                | Kind::CloseFor
                | Kind::CloseComponent
                | Kind::CloseSlot => {
                    if depth == 0 {
                        return Ok(());
                    }
                    depth -= 1;
                }
                _ => {}
            }
            self.pos += 1;
        }
        Err(AdmissionFailure::Invalid(
            "string plan element is not closed",
        ))
    }
}

/// The legacy precedence: `v-html`, then `v-text`, then the form models.
fn content<'r, 'a>(attached: &[SsrStringSegment<'r, 'a>], tag: &str) -> Content<'r, 'a> {
    let bindings = || {
        attached.iter().filter_map(|segment| match segment.source {
            Source::Binding(binding) => Some(binding),
            _ => None,
        })
    };
    if let Some(html) = bindings().find_map(|binding| match binding {
        s2::BindingOp::VueHtml(html) => Some(&**html),
        _ => None,
    }) {
        return Content::Html(html);
    }
    if let Some(text) = bindings().find_map(|binding| match binding {
        s2::BindingOp::VueText(text) => Some(&**text),
        _ => None,
    }) {
        return Content::Text(text);
    }
    let model = bindings().find_map(|binding| match binding {
        s2::BindingOp::Model(model) => Some(&**model),
        _ => None,
    });
    match (tag, model) {
        ("textarea", Some(model)) => Content::TextareaModel(model),
        ("select", Some(model)) => Content::SelectModel(model),
        _ => Content::Children,
    }
}
