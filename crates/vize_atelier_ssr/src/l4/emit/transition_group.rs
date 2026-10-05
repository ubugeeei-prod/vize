//! The original group tag and attachments on the selected string-plan path.

use vize_atelier_core::codegen::document::EmitDocument;
use vize_l1_to_l2::{TransformContent, decode_template_entities};
use vize_l2::op::{self as l2, DynamicName};

use super::attrs::Attached;
use super::spans::attribute_value_start;
use super::{Emitter, Flags, Result};
use crate::l4::string_plan::SsrSegmentSource as Source;

enum Tag {
    Static(vize_l0::String, u32),
    Dynamic(EmitDocument),
}

/// Newly rendered custom directive operands must retain their dynamic fact.
pub(super) fn admit_directives(
    attached: &Attached<'_, '_>,
    owner_fact: u32,
    name: &str,
) -> Result<()> {
    for segment in attached {
        if matches!(
            segment.source,
            Source::Binding(l2::BindingOp::VueDirective(_))
        ) {
            super::attrs::admit(core::slice::from_ref(segment), owner_fact, name)?;
        }
    }
    Ok(())
}

impl Emitter<'_, '_, '_, '_, '_, '_> {
    pub(super) fn transition_group(
        &mut self,
        attached: &Attached<'_, '_>,
        name: &str,
        inherit: bool,
        css_vars: bool,
    ) -> Result<()> {
        let index = attached.iter().position(|segment| match segment.source {
            Source::Attribute(attr) => attr.name == "tag",
            Source::Binding(l2::BindingOp::Bind(bind)) => {
                matches!(bind.name, Some(DynamicName::Static("tag")))
            }
            _ => false,
        });
        let tag = match index.and_then(|index| attached.get(index)) {
            Some(segment) => match segment.source {
                Source::Attribute(attr) => attr.value.map(|value| {
                    Tag::Static(
                        decode_template_entities(value),
                        attribute_value_start(self.ctx.source, attr.span, "tag")
                            .unwrap_or(attr.span.start),
                    )
                }),
                Source::Binding(binding) => match super::attrs::bind(binding)? {
                    Some(bind) => {
                        let value = self.expr(bind.value, TransformContent::Decoded)?;
                        Some(Tag::Dynamic(self.bound_expression(&value, &bind)))
                    }
                    None => None,
                },
                _ => None,
            },
            None => None,
        };
        if let Some(tag) = &tag {
            self.ctx.push_string_part_static("<");
            self.group_tag(tag);
            let attrs: std::vec::Vec<_> = attached
                .iter()
                .copied()
                .enumerate()
                .filter_map(|(at, segment)| (Some(at) != index).then_some(segment))
                .collect();
            if let Some(merged) = self.group_attrs(&attrs, name, inherit, css_vars && !inherit)? {
                self.ctx.push_string_part_dynamic_spanned(merged.attrs);
            }
            if let Some(scope) = &self.ctx.options.scope_id {
                self.ctx.push_string_part_static(" ");
                self.ctx.push_string_part_static(scope);
            }
            self.ctx.push_string_part_static(">");
        }
        self.children(Flags {
            as_fragment: tag.is_none(),
            disable_nested_fragments: true,
            disable_comments: true,
            inherit_attrs: false,
            css_vars: false,
        })?;
        if let Some(tag) = &tag {
            self.ctx.push_string_part_static("</");
            self.group_tag(tag);
            self.ctx.push_string_part_static(">");
        }
        Ok(())
    }

    fn group_tag(&mut self, tag: &Tag) {
        match tag {
            Tag::Static(value, start) => self.ctx.push_string_part_static_mapped(value, *start),
            Tag::Dynamic(value) => self.ctx.push_string_part_dynamic_spanned(value.clone()),
        }
    }
}

#[cfg(test)]
mod tests;
