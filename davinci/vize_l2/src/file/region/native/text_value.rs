//! One original root event: guard, observe, park, join, mint and attach.

use super::{NativeVisibility, RootRegion};
use crate::file::NativeTextValuePolicyError as Policy;
use crate::file::{NativeFileTextValue, NativeFileTextValueState as State};
use crate::lang::js::NativeTemplateIssueKind as Kind;
use vize_l0::id::NodeId;
use vize_l1::{
    TokenStatus,
    markup::{NativeChild, NativeTemplateComponent},
};

#[cfg(test)]
mod interruption;
#[cfg(test)]
mod tests;

pub(super) fn construct<'a>(
    selected: &NativeTemplateComponent<'a>,
    child: NativeChild<'_, 'a>,
    region: &mut RootRegion<'_, 'a, NativeVisibility>,
    cursor: usize,
    expected: usize,
) -> Result<NodeId, Kind> {
    // The caller arms the sticky whole-walk guard before even this observation.
    #[cfg(test)]
    interruption::before_observe();
    let observation = selected.observe_text_value(child.reborrow());
    region.facts.native_text_values.push(NativeFileTextValue {
        observation,
        state: State::Pending,
        text: None,
    });
    #[cfg(test)]
    interruption::after_park();
    let result = (|| {
        let [record] = region.facts.native_text_values.as_slice() else {
            return Err(Kind::InvalidEvent);
        };
        let observation =
            record
                .observation
                .as_ref()
                .map_err(|failure| Kind::TextValuePreparation {
                    span: failure.span().unwrap_or(failure.block().span()),
                    kind: failure.kind(),
                })?;
        let joined = observation
            .admitted_for(selected, child)
            .ok_or(Kind::InvalidEvent)?;
        let child = joined.child();
        let source = observation.source();
        if child.parent_element().is_some()
            || cursor != 0
            || expected != 1
            || child.ordinal() != cursor
            || source.span() != selected.component().block().span()
            || observation.token().status != TokenStatus::Present
        {
            return Err(Kind::TextValuePolicy(Policy::RootExtent));
        }
        if !observation.token().leading.is_empty() {
            return Err(Kind::TextValuePolicy(Policy::Leading));
        }
        if selected.ordinary().is_some() || selected.setup().is_some() || selected.has_styles() {
            return Err(Kind::TextValuePolicy(Policy::ScriptOrStyle));
        }
        if observation.raw_text().is_empty() || source.text().is_empty() {
            return Err(Kind::TextValuePolicy(Policy::Empty));
        }
        if observation.raw_text().contains('\0') || source.text().contains('\0') {
            return Err(Kind::TextValuePolicy(Policy::Control));
        }
        if whitespace(observation.raw_text()) {
            return Err(Kind::TextValuePolicy(Policy::RawWhitespace));
        }
        if whitespace(source.text()) {
            return Err(Kind::TextValuePolicy(Policy::DecodedWhitespace));
        }
        let (node, allocation) = region
            .region
            .native_text(source.text(), source.span())
            .map_err(Kind::Artifact)?;
        #[cfg(test)]
        interruption::after_mint();
        let [record] = region.facts.native_text_values.as_mut_slice() else {
            return Err(Kind::InvalidEvent);
        };
        record.text = Some(allocation.pointer());
        record.state = State::Attached(node);
        #[cfg(test)]
        interruption::after_attach();
        Ok(node)
    })();
    if let Err(kind) = result
        && let Some(record) = region.facts.native_text_values.last_mut()
    {
        record.state = State::Refused(kind);
    }
    result
}

fn whitespace(text: &str) -> bool {
    text.bytes()
        .any(|byte| matches!(byte, b'\t' | b'\n' | b'\x0c' | b'\r' | b' '))
}
