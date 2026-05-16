//! Script-side inlay hint collection.
//!
//! Finds usages of destructured props in script setup code
//! and generates `#props.` prefix hints.
#![allow(clippy::disallowed_methods)]

use tower_lsp::lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, Position, Range};

use super::InlayHintService;
use crate::ide::offset_to_position;

impl InlayHintService {
    /// Collect inlay hints for props usages in script setup.
    pub(super) fn collect_script_props_hints(
        script: &str,
        script_offset: usize,
        full_content: &str,
        destructured_props: &[&str],
        define_props_end: usize,
        range: Range,
        hints: &mut Vec<InlayHint>,
    ) {
        // Find usages of destructured props after the defineProps call
        for &prop_name in destructured_props {
            Self::find_prop_usages_in_text(
                script,
                prop_name,
                script_offset,
                full_content,
                define_props_end,
                range,
                hints,
            );
        }
    }

    /// Find usages of a prop in plain text (for script).
    fn find_prop_usages_in_text(
        text: &str,
        prop_name: &str,
        base_offset: usize,
        full_content: &str,
        define_props_end: usize,
        range: Range,
        hints: &mut Vec<InlayHint>,
    ) {
        if prop_name.is_empty() || text.is_empty() {
            return;
        }

        let mut search_pos = 0;

        while let Some(found) = text[search_pos..].find(prop_name) {
            let abs_pos = search_pos + found;

            // Bounds check
            if abs_pos + prop_name.len() > text.len() {
                break;
            }

            // Calculate SFC offset
            let sfc_offset = base_offset + abs_pos;

            // Skip if within defineProps call (including type definition)
            if sfc_offset <= base_offset + define_props_end {
                search_pos = abs_pos + 1;
                continue;
            }

            // Check word boundaries
            let before_ok = abs_pos == 0
                || text
                    .as_bytes()
                    .get(abs_pos - 1)
                    .map(|&b| !Self::is_ident_char(b))
                    .unwrap_or(true);
            let after_ok = text
                .as_bytes()
                .get(abs_pos + prop_name.len())
                .map(|&b| !Self::is_ident_char(b))
                .unwrap_or(true);

            // Check it's not preceded by "props." already
            let not_already_prefixed =
                abs_pos < 6 || &text[abs_pos.saturating_sub(6)..abs_pos] != "props.";

            // Check it's not in a string literal (simple check for quotes)
            let not_in_string = !Self::is_in_string(text, abs_pos);

            // Check it's not a property access (preceded by .)
            let not_property_access = abs_pos == 0
                || text
                    .as_bytes()
                    .get(abs_pos - 1)
                    .map(|&b| b != b'.')
                    .unwrap_or(true);

            // Check it's not part of an event name pattern like "update:title" (preceded by :)
            let not_event_name_part = abs_pos == 0
                || text
                    .as_bytes()
                    .get(abs_pos - 1)
                    .map(|&b| b != b':')
                    .unwrap_or(true);

            if before_ok
                && after_ok
                && not_already_prefixed
                && not_in_string
                && not_property_access
                && not_event_name_part
            {
                // Bounds check for full_content
                if sfc_offset >= full_content.len() {
                    search_pos = abs_pos + 1;
                    continue;
                }

                let (line, character) = offset_to_position(full_content, sfc_offset);

                let position = Position { line, character };

                // Check if within requested range
                if Self::position_in_range(position, range) {
                    hints.push(InlayHint {
                        position,
                        label: InlayHintLabel::String("#props.".to_string()),
                        kind: Some(InlayHintKind::TYPE),
                        text_edits: None,
                        tooltip: Some(tower_lsp::lsp_types::InlayHintTooltip::String(
                            "Destructured from defineProps".to_string(),
                        )),
                        padding_left: None,
                        padding_right: Some(true),
                        data: None,
                    });
                }
            }

            search_pos = abs_pos + 1;
        }
    }

    /// Simple check if a position is inside a string literal.
    pub(super) fn is_in_string(text: &str, pos: usize) -> bool {
        if pos >= text.len() {
            return false;
        }

        let before = &text[..pos];
        let mut in_single = false;
        let mut in_double = false;
        let mut in_template = false;
        let mut prev_char = '\0';

        for c in before.chars() {
            if prev_char != '\\' {
                if c == '\'' && !in_double && !in_template {
                    in_single = !in_single;
                } else if c == '"' && !in_single && !in_template {
                    in_double = !in_double;
                } else if c == '`' && !in_single && !in_double {
                    in_template = !in_template;
                }
            }
            prev_char = c;
        }

        in_single || in_double || in_template
    }
}
