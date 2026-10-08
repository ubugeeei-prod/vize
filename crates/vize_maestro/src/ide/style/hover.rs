//! Selected property/value/selector documentation uses the same static resolver.

use super::{
    context::{self, CssContext},
    data, documentation,
    resolve::Selected,
    values, vue,
};
use crate::ide::{IdeContext, markup::markdown_content};
use tower_lsp::lsp_types::{Hover, HoverContents};

pub(crate) fn hover(ctx: &IdeContext<'_>, index: usize) -> Option<Hover> {
    let (content, base, standard_css) = super::region(ctx, index)?;
    let offset = ctx.offset - base;
    // Retain the old Vue feature range behavior while avoiding a word allocation.
    if let Some((start, end)) = crate::ide::token_span_at_offset(content, offset, |ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, b'_' | b'-' | b'$' | b':')
    }) && let Some(feature) = vue::feature(content.get(start..end)?)
    {
        return Some(Hover {
            contents: HoverContents::Markup(markdown_content(vue::markdown(feature))),
            range: None,
        });
    }
    if !standard_css {
        return None;
    }
    let (selected, span) = match context::at(content, offset) {
        CssContext::Property { span, .. } => (
            Selected::Entry("property", data::property(content.get(span.0..span.1)?)?),
            span,
        ),
        CssContext::Pseudo { span, .. } => {
            let name = content.get(span.0..span.1)?;
            (
                Selected::Entry(
                    "pseudo",
                    data::pseudo_class(name).or_else(|| data::pseudo_element(name))?,
                ),
                span,
            )
        }
        CssContext::AtRule { span, .. } => (
            Selected::Entry("at-rule", data::at_rule(content.get(span.0..span.1)?)?),
            span,
        ),
        CssContext::Value { property, span, .. } => {
            let property = data::property(property)?;
            let name = content.get(span.0..span.1)?;
            let selected = if let Some(value) = property
                .values
                .iter()
                .find(|value| value.name.eq_ignore_ascii_case(name))
            {
                Selected::Value(property, value)
            } else if let Some(value) = values::lookup(&values::WIDE, name) {
                Selected::Wide(property, value)
            } else if let Some(value) = values::lookup(&values::FUNCTIONS, name) {
                if value.name == "calc" && !values::accepts_calc(property) {
                    return None;
                }
                Selected::Function(property, value)
            } else if values::accepts_colors(property) {
                Selected::Color(property, data::color(name)?)
            } else {
                return None;
            };
            (selected, span)
        }
        CssContext::Unknown => return None,
    };
    Some(Hover {
        contents: HoverContents::Markup(markdown_content(documentation::markdown(selected))),
        range: Some(super::range(ctx, base, span)),
    })
}
