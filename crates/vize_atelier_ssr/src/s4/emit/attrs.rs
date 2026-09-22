//! Attached-segment admission and the inline `name="value"` attribute shape
//! of non-root elements.

use vize_atelier_core::RuntimeHelper;
use vize_s0::{String, ToCompactString, cstr};
use vize_s1_to_s2::{TransformContent, decode_template_entities};
use vize_s2::expr::ExprRef;
use vize_s2::op::{self as s2, DynamicName};

use super::{Emitter, Result, model, plan_source, require_dynamic};
use crate::codegen::element::props::{merge_prop_values, quoted_js_string};
use vize_atelier_core::codegen::spanned::SpannedText;

use super::spans::{attribute_value_start, directive_value, expression_span};
use crate::codegen::helpers::escape_html_attr;
use crate::s4::string_plan::{
    SsrSegmentSource as Source, SsrStringPayloadKind, SsrStringSegment,
    SsrStringSegmentKind as Kind,
};
use crate::s4::{AdmissionFailure, LegacyReason};

pub(super) type Attached<'r, 'a> = [SsrStringSegment<'r, 'a>];

/// The name position of one admitted `v-bind`.
pub(super) enum BindName<'r, 'a> {
    /// `v-bind="object"`.
    Spread,
    Static(&'a str),
    /// `:[key]`: a bare identifier (rendered through the merged path).
    Dynamic(&'r DynamicName<'a>),
}

/// One admitted `v-bind`. `.camel` camelizes the key; `.prop` / `.attr`
/// render as the plain attribute on the server.
pub(super) struct Bind<'r, 'a> {
    pub(super) name: BindName<'r, 'a>,
    pub(super) camel: bool,
    pub(super) value: &'r ExprRef<'a>,
    /// The whole directive's authored range.
    pub(super) span: vize_s0::Span,
}

/// Admit an element's attached segments. Anything the plan emitter does not
/// own keeps the legacy lane; a segment that does not belong to its element
/// is a broken plan.
pub(super) fn admit(attached: &Attached<'_, '_>, owner_fact: u32, tag: &str) -> Result<()> {
    if attached
        .windows(2)
        .any(|pair| pair[0].span.start > pair[1].span.start)
    {
        return Err(AdmissionFailure::Invalid(
            "string plan attached segments are not in authored order",
        ));
    }
    for segment in attached {
        match (segment.kind, segment.source) {
            (Kind::StaticAttribute, Source::Attribute(_)) if segment.fact == owner_fact => {}
            (_, Source::Binding(binding)) => {
                match binding {
                    s2::BindingOp::Bind(_) => {
                        bind(binding)?;
                    }
                    s2::BindingOp::On(_) => {}
                    s2::BindingOp::Model(model) => model::admit(model, tag)?,
                    s2::BindingOp::VueShow(show) => admit_value(Some(&show.value))?,
                    s2::BindingOp::VueHtml(html) => admit_value(html.value.as_ref())?,
                    s2::BindingOp::VueText(text) => admit_value(text.value.as_ref())?,
                    // Rendered through `_ssrGetDirectiveProps`; a dynamic
                    // argument must be a bare identifier.
                    s2::BindingOp::VueDirective(directive) => {
                        if let Some(value) = &directive.value {
                            admit_value(Some(value))?;
                        }
                        if let Some(DynamicName::Dynamic(argument)) = &directive.argument {
                            admit_dynamic_key(argument)?;
                        }
                    }
                    // Server rendering drops them; their partition is static.
                    s2::BindingOp::VueOnce(_) | s2::BindingOp::VueCloak(_) => continue,
                    s2::BindingOp::VueMemo(_) => {}
                    _ => return Err(LegacyReason::Binding.into()),
                }
                require_dynamic(segment)?;
            }
            _ => {
                return Err(AdmissionFailure::Invalid(
                    "string plan attached segment does not belong to its element",
                ));
            }
        }
    }
    Ok(())
}

/// A directive value the transform rewrite can own. An opaque value (the S2
/// parse refused the raw text) still reaches the rewrite, which re-parses and
/// refuses exactly where the shipped transform reports it invalid.
pub(super) fn admit_value(value: Option<&ExprRef<'_>>) -> Result<()> {
    match value {
        Some(ExprRef::Js(_) | ExprRef::Opaque(_)) => Ok(()),
        Some(_) => Err(LegacyReason::ExpressionOrEncoding.into()),
        None => Err(LegacyReason::Binding.into()),
    }
}

pub(super) fn bind<'r, 'a>(binding: &'r s2::BindingOp<'a>) -> Result<Option<Bind<'r, 'a>>> {
    let s2::BindingOp::Bind(bind) = binding else {
        return Ok(None);
    };
    if bind
        .modifiers
        .iter()
        .any(|modifier| !matches!(*modifier, "camel" | "prop" | "attr"))
    {
        return Err(LegacyReason::Binding.into());
    }
    let name = match &bind.name {
        None => BindName::Spread,
        Some(DynamicName::Static(name)) => BindName::Static(name),
        Some(name @ DynamicName::Dynamic(argument)) => {
            admit_dynamic_key(argument)?;
            BindName::Dynamic(name)
        }
    };
    admit_value(bind.value.as_ref())?;
    match &bind.value {
        Some(value) => Ok(Some(Bind {
            name,
            camel: bind.modifiers.contains(&"camel"),
            value,
            span: bind.span,
        })),
        None => Err(LegacyReason::Binding.into()),
    }
}

/// A dynamic key the plan emitter owns: a bare identifier (`_ctx.<key>`, or
/// the scope-local name inside a `v-for` / slot scope).
pub(super) fn admit_dynamic_key(argument: &ExprRef<'_>) -> Result<()> {
    let source = argument.source();
    let simple = crate::codegen::element::props::is_valid_js_identifier(source)
        && !matches!(source, "true" | "false" | "null" | "undefined");
    if simple {
        Ok(())
    } else {
        Err(LegacyReason::Binding.into())
    }
}

/// The first static attribute named `name`, entity-decoded (`None` when the
/// first such attribute is valueless, as the legacy lookup returns).
pub(super) fn static_value(attached: &Attached<'_, '_>, name: &str) -> Option<String> {
    let attr = attached.iter().find_map(|segment| match segment.source {
        Source::Attribute(attr) if attr.name == name => Some(attr),
        _ => None,
    })?;
    attr.value.map(decode_template_entities)
}

/// The first static-name `v-bind:name` value.
pub(super) fn bound_value<'r, 'a>(
    attached: &Attached<'r, 'a>,
    name: &str,
) -> Option<&'r ExprRef<'a>> {
    attached.iter().find_map(|segment| match segment.source {
        Source::Binding(s2::BindingOp::Bind(bind)) => match bind.name {
            Some(DynamicName::Static(bound)) if bound == name => bind.value.as_ref(),
            _ => None,
        },
        _ => None,
    })
}

fn has_bound(attached: &Attached<'_, '_>, name: &str) -> bool {
    attached.iter().any(|segment| match segment.source {
        Source::Binding(s2::BindingOp::Bind(bind)) => {
            matches!(bind.name, Some(DynamicName::Static(bound)) if bound == name)
        }
        _ => false,
    })
}

/// The first `v-show` value.
fn show_value<'r, 'a>(attached: &Attached<'r, 'a>) -> Option<&'r ExprRef<'a>> {
    attached.iter().find_map(|segment| match segment.source {
        Source::Binding(s2::BindingOp::VueShow(show)) => Some(&show.value),
        _ => None,
    })
}

impl Emitter<'_, '_, '_, '_, '_, '_> {
    /// A rewritten `v-bind` value anchored at its authored expression: the
    /// directive's quoted value, or the expanded shorthand's expression.
    pub(super) fn bound_expression(&self, code: &str, bind: &Bind<'_, '_>) -> SpannedText {
        match directive_value(self.ctx.source, bind.span).or_else(|| expression_span(bind.value)) {
            Some(span) => self.ctx.spanned_expression(code, span),
            None => SpannedText::plain(code),
        }
    }

    /// `(({exp}) ? null : { display: "none" })` for the first `v-show`.
    pub(super) fn show_style(&self, attached: &Attached<'_, '_>) -> Result<Option<String>> {
        let Some(value) = show_value(attached) else {
            return Ok(None);
        };
        let exp = self.expr(value, TransformContent::Decoded)?;
        Ok(Some(cstr!("(({exp}) ? null : {{ display: \"none\" }})")))
    }
}

/// Non-root element attributes, in authored order.
pub(super) fn emit_inline(
    em: &mut Emitter<'_, '_, '_, '_, '_, '_>,
    attached: &Attached<'_, '_>,
    tag: &str,
) -> Result<()> {
    let dynamic_class = has_bound(attached, "class");
    let dynamic_style = has_bound(attached, "style");
    let explicit_style = dynamic_style
        || attached.iter().any(
            |segment| matches!(segment.source, Source::Attribute(attr) if attr.name == "style"),
        );
    for segment in attached {
        match segment.source {
            Source::Attribute(attr) => {
                let name = plan_source(segment, SsrStringPayloadKind::AttributeName)?;
                if (name == "class" && dynamic_class)
                    || (name == "style" && dynamic_style)
                    || vize_s0::is_reserved_prop(name)
                {
                    continue;
                }
                if name == "style"
                    && let Some(show) = em.show_style(attached)?
                {
                    let value = attr
                        .value
                        .map(|value| quoted_js_string(&decode_template_entities(value)))
                        .unwrap_or_else(|| "\"\"".to_compact_string());
                    let style_exp = merge_prop_values(std::vec![value, show]);
                    em.ctx.use_ssr_helper(RuntimeHelper::SsrRenderStyle);
                    em.ctx.push_string_part_static(" style=\"");
                    em.ctx
                        .push_string_part_dynamic(&cstr!("_ssrRenderStyle({style_exp})"));
                    em.ctx.push_string_part_static("\"");
                    continue;
                }
                em.ctx.push_string_part_static(" ");
                em.ctx.push_string_part_static_mapped(name, attr.span.start);
                if let Some(value) = attr.value {
                    em.ctx.push_string_part_static("=\"");
                    let decoded = escape_html_attr(&decode_template_entities(value));
                    match attribute_value_start(em.ctx.source, attr.span, name) {
                        Some(start) => em.ctx.push_string_part_static_mapped(&decoded, start),
                        None => em.ctx.push_string_part_static(&decoded),
                    }
                    em.ctx.push_string_part_static("\"");
                }
            }
            Source::Binding(binding) => match binding {
                s2::BindingOp::Bind(_) => {
                    let Some(bind) = bind(binding)? else {
                        continue;
                    };
                    let exp = em.expr(bind.value, TransformContent::Decoded)?;
                    let name = match bind.name {
                        BindName::Static(name) if bind.camel => Some(vize_s0::camelize(name)),
                        BindName::Static(name) => Some(name.to_compact_string()),
                        // Spreads and dynamic keys take the merged path.
                        BindName::Spread | BindName::Dynamic(_) => {
                            return Err(AdmissionFailure::Invalid(
                                "a merged-props bind reached the inline attribute path",
                            ));
                        }
                    };
                    super::inline_bind::emit(em, attached, &bind, name.as_deref(), exp)?;
                }
                s2::BindingOp::Model(model) => model::emit_inline(em, attached, model, tag)?,
                s2::BindingOp::VueShow(show) if !explicit_style => {
                    let exp = em.expr(&show.value, TransformContent::Decoded)?;
                    em.ctx.push_string_part_dynamic(&cstr!(
                        "(({exp}) ? \"\" : \" style=\\\"display: none;\\\"\")"
                    ));
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(())
}
