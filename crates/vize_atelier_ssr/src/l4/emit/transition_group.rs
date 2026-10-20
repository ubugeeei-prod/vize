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
            disable_nested_fragments: !self.ctx.options.vapor,
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
mod tests {
    //! Original directive facts and a deliberately contradictory target partition.
    #![expect(
        clippy::expect_used,
        reason = "exact original plan custody in negative controls"
    )]

    use super::admit_directives;
    use crate::l4::AdmissionFailure;
    use crate::l4::string_plan::{SsrSegmentSource as Source, lower_l2_to_string_plan};
    use vize_l0::Allocator;
    use vize_l2::op as l2;
    use vize_l2_to_l3::PartitionKind;

    fn error_text(error: AdmissionFailure) -> &'static str {
        match error {
            AdmissionFailure::Invalid(message) => message,
            AdmissionFailure::Unsupported(_) => "unsupported directive operand",
        }
    }

    #[test]
    fn original_group_directives_require_their_dynamic_fact_even_without_a_value() {
        for (source, authored, has_value) in [
            (
                r#"<TransitionGroup tag="ul" v-example><li /></TransitionGroup>"#,
                "v-example",
                false,
            ),
            (
                r#"<transition-group tag="ul" v-example:[name].active="value"><li /></transition-group>"#,
                "v-example:[name].active=\"value\"",
                true,
            ),
        ] {
            let allocator = Allocator::new();
            let (tree, errors) = vize_l1::markup::parse(&allocator, source);
            assert_eq!(errors.len(), 0);
            let lowered = vize_l1_to_l2::lower(&allocator, &tree, &errors);
            let partitioned = vize_l2_to_l3::lower(&allocator, &lowered.root);
            let planned =
                lower_l2_to_string_plan(&allocator, &lowered.root, &partitioned.partition);
            assert_eq!(planned.errors.len(), 0);
            let original = planned
                .plan
                .segments
                .iter()
                .find_map(|segment| match segment.source {
                    Source::Component(component) => Some((segment.fact, component)),
                    _ => None,
                })
                .expect("original group owner");
            let binding = original
                .1
                .bindings
                .iter()
                .find_map(|binding| match binding {
                    l2::BindingOp::VueDirective(directive) => Some((binding, directive)),
                    _ => None,
                })
                .expect("original directive operand");
            let segment = planned.plan.segments.iter().find(|segment| {
            matches!(segment.source, Source::Binding(owner) if core::ptr::eq(owner, binding.0))
        }).copied().expect("same original directive in the plan");
            assert_eq!(segment.partition, PartitionKind::Dynamic);
            assert_eq!(segment.span, binding.1.span);
            assert_eq!(binding.1.value.is_some(), has_value);
            assert_eq!(
                source.get(segment.span.start as usize..segment.span.end as usize),
                Some(authored)
            );
            assert_eq!(
                admit_directives(core::slice::from_ref(&segment), original.0, original.1.name)
                    .map_err(error_text),
                Ok(())
            );
            let mut contradictory = segment;
            contradictory.partition = PartitionKind::Static;
            assert_eq!(
                admit_directives(
                    core::slice::from_ref(&contradictory),
                    original.0,
                    original.1.name
                )
                .map_err(error_text),
                Err("partition fact classifies an expression segment as static")
            );
        }
    }
}
