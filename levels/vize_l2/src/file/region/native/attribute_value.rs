//! Observe and park the complete value at its one original header visit.

use super::NativeVisibility;
use crate::artifact::{ElementAllocation, RegionBuilder};
use crate::file::region::FileRegion;
use crate::file::{NativeFileAttributeValue, NativeFileAttributeValueState as State};
use crate::lang::js::NativeTemplateIssueKind as Kind;
use crate::op::Attribute;
use core::{
    ops::{DerefMut, Range},
    ptr::NonNull,
};
use vize_l0::{Span, id::NodeId};
use vize_l1::markup::{NativeAttribute, NativeElement, NativeTemplateComponent};

pub(super) struct PreparedAttributes<'a> {
    range: Range<usize>,
    storage: NonNull<[Attribute<'a>]>,
}

impl<'a: 'b, 'b, R> FileRegion<'_, 'b, 'a, R, NativeVisibility>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    /// Authenticate the original static type while its exact header storage is
    /// prepared. Only normal Element close later attaches this observation.
    pub(super) fn prepared_checkbox_type(
        &self,
        prepared: &PreparedAttributes<'a>,
        attributes: &[Attribute<'a>],
        selected: &NativeTemplateComponent<'a>,
        original: &NativeElement<'_, 'a>,
    ) -> bool {
        let [attribute] = attributes else {
            return false;
        };
        if original.attributes().len() != 2
            || attribute.name != "type"
            || attribute.value != Some("checkbox")
            || !core::ptr::eq(
                prepared.storage.as_ptr(),
                NonNull::from(attributes).as_ptr(),
            )
            || prepared.range.end.checked_sub(prepared.range.start) != Some(1)
        {
            return false;
        }
        let Some(record) = self.facts.native_attribute_values.get(prepared.range.start) else {
            return false;
        };
        let Some(observation) = record.observation() else {
            return false;
        };
        let Some(original_type) = original
            .attributes()
            .find(|value| value.surface().name.text == "type")
        else {
            return false;
        };
        if observation.admitted_for(selected, original_type).is_none() {
            return false;
        }
        matches!(record.state, State::Pending)
            && record.slot == 0
            && record.element.is_none()
            && record.attribute == Some(NonNull::from(attribute))
            && core::ptr::eq(record.name, attribute.name)
            && attribute
                .value
                .is_some_and(|text| core::ptr::eq(text, observation.source().text()))
            && core::ptr::eq(observation.source().authored_root(), self.source)
            && observation.source().decode_map().is_none()
    }

    pub(super) fn observe_attribute_value(
        &mut self,
        selected: &NativeTemplateComponent<'a>,
        original: NativeAttribute<'_, 'a>,
        slot: usize,
    ) -> Result<Attribute<'a>, Kind> {
        let block = selected.component().block();
        self.with_walk(block.span(), |region| {
            #[cfg(test)]
            interruption::trip(interruption::Fault::BeforeObserve);
            let name = original.surface().name.text;
            let observation = selected.observe_attribute_value(original.reborrow());
            let index = region.facts.native_attribute_values.len();
            region
                .facts
                .native_attribute_values
                .push(NativeFileAttributeValue {
                    observation,
                    state: State::Pending,
                    slot,
                    name,
                    element: None,
                    attribute: None,
                });
            #[cfg(test)]
            interruption::trip(interruption::Fault::AfterPark);
            // Park before any short join, source projection or name/value policy.
            let result = (|| {
                let record = region
                    .facts
                    .native_attribute_values
                    .get(index)
                    .ok_or(Kind::InvalidEvent)?;
                let value =
                    record
                        .observation
                        .as_ref()
                        .map_err(|failure| Kind::AttributeValue {
                            span: failure
                                .value_span()
                                .or_else(|| failure.name_span())
                                .unwrap_or(block.span()),
                            kind: failure.kind(),
                        })?;
                let joined = value
                    .admitted_for(selected, original)
                    .ok_or(Kind::InvalidEvent)?;
                let source = joined.observation().source();
                if !core::ptr::eq(source.authored_root(), region.source) {
                    return Err(Kind::InvalidEvent);
                }
                if name.is_empty()
                    || matches!(name, "style" | "key" | "ref" | "is")
                    || source.text().bytes().any(|byte| {
                        byte == 0
                            || (name != "class" && byte == b'\r' && source.decode_map().is_some())
                    })
                {
                    return Err(Kind::UnsupportedChild);
                }
                // Class keeps the original once-decoded value, including
                // encoded CR. Qualified target normalization consumes HTML
                // whitespace; ordinary attribute CR policy stays unchanged.
                Ok(Attribute {
                    name,
                    value: Some(source.text()),
                    span: Span::new(value.name_span().start, value.full_value_span().end),
                })
            })();
            if let Err(kind) = result {
                region
                    .facts
                    .native_attribute_values
                    .get_mut(index)
                    .ok_or(Kind::InvalidEvent)?
                    .state = State::Refused(kind);
            }
            result
        })
    }

    pub(super) fn prepare_attribute_storage(
        &mut self,
        start: usize,
        attributes: &[Attribute<'a>],
    ) -> Result<PreparedAttributes<'a>, Kind> {
        let end = self.facts.native_attribute_values.len();
        for record in self
            .facts
            .native_attribute_values
            .get_mut(start..end)
            .ok_or(Kind::InvalidEvent)?
        {
            let attribute = attributes.get(record.slot).ok_or(Kind::InvalidEvent)?;
            let value = record.observation().ok_or(Kind::InvalidEvent)?;
            if !matches!(record.state, State::Pending)
                || !core::ptr::eq(attribute.name, record.name)
                || !attribute
                    .value
                    .is_some_and(|text| core::ptr::eq(text, value.source().text()))
            {
                return Err(Kind::InvalidEvent);
            }
            record.attribute = Some(NonNull::from(attribute));
        }
        Ok(PreparedAttributes {
            range: start..end,
            storage: NonNull::from(attributes),
        })
    }

    pub(super) fn attach_attribute_values(
        &mut self,
        prepared: PreparedAttributes<'a>,
        node: NodeId,
        allocation: ElementAllocation<'a>,
    ) -> Result<(), Kind> {
        #[cfg(test)]
        interruption::trip(interruption::Fault::AfterClose);
        if !allocation.matches_storage(prepared.storage) {
            return Err(Kind::InvalidEvent);
        }
        for record in self
            .facts
            .native_attribute_values
            .get_mut(prepared.range)
            .ok_or(Kind::InvalidEvent)?
        {
            if !matches!(record.state, State::Pending)
                || record.attribute.is_none()
                || record.element.is_some()
            {
                return Err(Kind::InvalidEvent);
            }
            record.element = Some(allocation.pointer());
            record.state = State::Attached {
                node,
                slot: record.slot,
            };
        }
        #[cfg(test)]
        interruption::trip(interruption::Fault::AfterAttach);
        Ok(())
    }
}

#[cfg(test)]
mod interruption;
#[cfg(test)]
mod tests;
