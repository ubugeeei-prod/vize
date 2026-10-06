//! Source-owned Vue builtin hints when native type checking is unavailable.

use super::InlayHintService;
use crate::utils::offset_to_position_str;
use tower_lsp::lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, InlayHintTooltip, Range};
use vize_croquis::{Croquis, reactivity::ReactiveKind};

impl InlayHintService {
    pub(super) fn collect_builtin_hints(
        content: &str,
        script_offset: usize,
        croquis: &Croquis,
        range: Range,
        hints: &mut Vec<InlayHint>,
    ) {
        if range.start > range.end {
            return;
        }
        for fact in croquis.types.builtin_reactive_types() {
            // A later declaration cannot lend its identifier span to an old fact.
            if croquis.binding_spans.get(fact.name()) != Some(&fact.span()) {
                continue;
            }
            let Some(offset) = script_offset.checked_add(fact.span().1 as usize) else {
                continue;
            };
            if offset > content.len() || !content.is_char_boundary(offset) {
                continue;
            }
            let position = offset_to_position_str(content, offset);
            if !Self::position_in_range(position, range) {
                continue;
            }
            let wrapper = match fact.kind() {
                ReactiveKind::Ref => "Ref",
                ReactiveKind::Computed => "ComputedRef",
                _ => continue,
            };
            hints.push(InlayHint {
                position,
                label: InlayHintLabel::String(
                    vize_l0::cstr!(": {wrapper}<{}>", fact.value_type()).into(),
                ),
                kind: Some(InlayHintKind::TYPE),
                text_edits: None,
                tooltip: Some(InlayHintTooltip::String(
                    vize_l0::cstr!("Vue reactive binding ({wrapper})").into(),
                )),
                padding_left: Some(true),
                padding_right: None,
                data: None,
            });
        }
    }
}
