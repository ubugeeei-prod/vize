//! Declared slots a `v-if` branch tests and then renders.
//!
//! `<span v-if="slots.tip"><slot name="tip" /></span>` renders `slots.tip`
//! inside the branch, so the condition is not an uncalled function check
//! (`TS2774`): vue-tsc's projection calls the slot inside the `if` block. The
//! outlet checks keep their own place; the authored condition block only gains
//! the same read of the slot it tests.

use super::reserved_props::rewrite_reserved_template_binding;
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use vize_carton::{CompactString, FxHashMap, append};
use vize_croquis::TemplateExpression;

/// The setup binding `<slot>` outlets read declared slots through.
const SLOTS_BINDING: &str = "slots";

/// Each authored `v-if` / `v-else-if` condition, by its template range, with
/// the `slots` members it tests that an outlet in its branch renders.
#[derive(Default)]
pub(crate) struct SlotConditionReads {
    by_condition: FxHashMap<(u32, u32), Vec<CompactString>>,
}

impl SlotConditionReads {
    pub(crate) fn is_empty(&self) -> bool {
        self.by_condition.is_empty()
    }

    /// Record that the condition at `range` renders slot `name`, when the
    /// condition reads that slot off `slots` with a member access.
    pub(crate) fn record(&mut self, range: (u32, u32), condition: &str, name: &str) {
        let Some(read) = slot_member_read(condition, name) else {
            return;
        };
        let reads = self.by_condition.entry(range).or_default();
        if !reads.contains(&read) {
            reads.push(read);
        }
    }

    /// Emit the reads of the condition `expr` inside its `if` block, rewritten
    /// the way the condition itself is.
    pub(super) fn append(
        &self,
        ts: &mut crate::virtual_ts::TextBuf,
        expr: &TemplateExpression,
        template_binding_access: &TemplateBindingAccess,
        indent: &str,
    ) {
        let Some(reads) = self.by_condition.get(&(expr.start, expr.end)) else {
            return;
        };
        for read in reads {
            let rewritten = rewrite_reserved_template_binding(read, template_binding_access);
            let read = rewritten.as_deref().unwrap_or(read.as_str());
            append!(*ts, "{indent}  void {read};\n");
        }
    }
}

/// `slots.<name>` or `slots?.<name>` as `condition` spells it, when the
/// condition reads slot `name` off the `slots` binding itself (not a member
/// named `slots` of another value).
fn slot_member_read(condition: &str, name: &str) -> Option<CompactString> {
    if !is_identifier(name) {
        return None;
    }
    let bytes = condition.as_bytes();
    for (start, _) in condition.match_indices(SLOTS_BINDING) {
        if start > 0
            && bytes
                .get(start - 1)
                .is_some_and(|&byte| is_identifier_byte(byte) || byte == b'.')
        {
            continue;
        }
        let after_binding = start + SLOTS_BINDING.len();
        let rest = condition.get(after_binding..).unwrap_or_default();
        let Some(member) = rest.strip_prefix("?.").or_else(|| rest.strip_prefix('.')) else {
            continue;
        };
        let Some(after_name) = member.strip_prefix(name) else {
            continue;
        };
        if after_name
            .as_bytes()
            .first()
            .is_some_and(|&byte| is_identifier_byte(byte))
        {
            continue;
        }
        let end = condition.len() - after_name.len();
        return condition.get(start..end).map(CompactString::new);
    }
    None
}

fn is_identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| is_identifier_byte(byte) && !byte.is_ascii_digit())
        && bytes.all(is_identifier_byte)
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

#[cfg(test)]
mod tests {
    use super::slot_member_read;

    #[test]
    fn reads_only_the_tested_member_of_the_slots_binding() {
        let read = |condition: &str, name: &str| slot_member_read(condition, name);
        assert_eq!(read("slots.tip", "tip").as_deref(), Some("slots.tip"));
        assert_eq!(read("slots?.tip", "tip").as_deref(), Some("slots?.tip"));
        assert_eq!(
            read("open && slots.tip", "tip").as_deref(),
            Some("slots.tip")
        );
        assert_eq!(read("slots.tipText", "tip"), None);
        assert_eq!(read("$slots.tip", "tip"), None);
        assert_eq!(read("mySlots.tip", "tip"), None);
        assert_eq!(read("props.slots.tip", "tip"), None);
        assert_eq!(read("slots['tip']", "tip"), None);
        assert_eq!(read("slots.item-header", "item-header"), None);
    }
}
