//! Template-side inlay hint collection.
//!
//! Finds usages of props in template mustache expressions and
//! Vue directive attributes, generating `#props.` prefix hints.
#![allow(clippy::disallowed_methods)]

use tower_lsp::lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, Position, Range};

use super::InlayHintService;
use crate::ide::offset_to_position;

impl InlayHintService {
    /// Collect inlay hints for props usages in template.
    pub(super) fn collect_template_props_hints(
        template: &str,
        template_offset: usize,
        full_content: &str,
        destructured_props: &[&str],
        range: Range,
        hints: &mut Vec<InlayHint>,
    ) {
        // Find mustache expressions {{ ... }}
        Self::collect_mustache_hints(
            template,
            template_offset,
            full_content,
            destructured_props,
            range,
            hints,
        );

        // Find Vue directive expressions (:prop="...", v-bind:prop="...", @event="...", v-if="...", etc.)
        Self::collect_directive_hints(
            template,
            template_offset,
            full_content,
            destructured_props,
            range,
            hints,
        );
    }

    /// Collect hints from mustache expressions {{ ... }}.
    fn collect_mustache_hints(
        template: &str,
        template_offset: usize,
        full_content: &str,
        destructured_props: &[&str],
        range: Range,
        hints: &mut Vec<InlayHint>,
    ) {
        let mut pos = 0;

        while let Some(start) = template[pos..].find("{{") {
            let abs_start = pos + start + 2; // Skip "{{"

            if let Some(end) = template[abs_start..].find("}}") {
                let abs_end = abs_start + end;
                let expr = &template[abs_start..abs_end];

                for &prop in destructured_props {
                    Self::find_prop_usages_in_expr(
                        expr,
                        prop,
                        template_offset + abs_start,
                        full_content,
                        range,
                        hints,
                    );
                }

                pos = abs_end + 2;
            } else {
                break;
            }
        }
    }

    /// Collect hints from Vue directive attributes.
    fn collect_directive_hints(
        template: &str,
        template_offset: usize,
        full_content: &str,
        destructured_props: &[&str],
        range: Range,
        hints: &mut Vec<InlayHint>,
    ) {
        // Patterns for Vue directives:
        // :prop="...", v-bind:prop="...", @event="...", v-on:event="..."
        // v-if="...", v-else-if="...", v-for="...", v-show="...", v-model="..."
        // v-slot:name="...", #name="..."

        let directive_patterns = [
            "v-if=\"",
            "v-if='",
            "v-else-if=\"",
            "v-else-if='",
            "v-for=\"",
            "v-for='",
            "v-show=\"",
            "v-show='",
            "v-model=\"",
            "v-model='",
            "v-bind:",
            "v-on:",
            "v-slot:",
        ];

        let mut pos = 0;

        while pos < template.len() {
            let remaining = &template[pos..];

            // Find next directive or shorthand
            let mut next_match: Option<(usize, usize, char)> = None; // (position, skip_len, quote_char)

            // Check for shorthand patterns: :prop=", @event=", #slot="
            for (i, c) in remaining.char_indices() {
                if (c == ':' || c == '@' || c == '#') && i + 1 < remaining.len() {
                    // Check if followed by identifier and ="
                    let after = &remaining[i + 1..];
                    if let Some(eq_pos) = after.find('=') {
                        let attr_name = &after[..eq_pos];
                        // Validate it's a valid attribute name (alphanumeric, -, _)
                        if !attr_name.is_empty()
                            && attr_name
                                .chars()
                                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':')
                        {
                            let quote_start = i + 1 + eq_pos + 1;
                            if quote_start < remaining.len() {
                                let quote = remaining.as_bytes()[quote_start] as char;
                                if quote == '"' || quote == '\'' {
                                    next_match = Some((i, quote_start + 1, quote));
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // Check for full directive patterns
            for pattern in &directive_patterns {
                if let Some(found) = remaining.find(pattern) {
                    let pattern_end = found + pattern.len();
                    if pattern.ends_with(':') {
                        // v-bind:, v-on:, v-slot: - need to find ="
                        if let Some(eq_pos) = remaining[pattern_end..].find('=') {
                            let quote_pos = pattern_end + eq_pos + 1;
                            if quote_pos < remaining.len() {
                                let quote = remaining.as_bytes()[quote_pos] as char;
                                if quote == '"' || quote == '\'' {
                                    let new_match = (found, quote_pos + 1, quote);
                                    if next_match.is_none()
                                        || new_match.0 < next_match.as_ref().unwrap().0
                                    {
                                        next_match = Some(new_match);
                                    }
                                }
                            }
                        }
                    } else {
                        // v-if=", etc. - pattern already includes the quote
                        let quote = pattern.chars().last().unwrap();
                        let new_match = (found, pattern_end, quote);
                        if next_match.is_none() || new_match.0 < next_match.as_ref().unwrap().0 {
                            next_match = Some(new_match);
                        }
                    }
                }
            }

            let Some((_, expr_start, quote)) = next_match else {
                break;
            };

            let abs_start = pos + expr_start;

            // Find closing quote
            if let Some(end) = template[abs_start..].find(quote) {
                let abs_end = abs_start + end;
                let expr = &template[abs_start..abs_end];

                for &prop in destructured_props {
                    Self::find_prop_usages_in_expr(
                        expr,
                        prop,
                        template_offset + abs_start,
                        full_content,
                        range,
                        hints,
                    );
                }

                pos = abs_end + 1;
            } else {
                pos = abs_start + 1;
            }
        }
    }

    /// Find usages of a prop in an expression and add hints.
    pub(super) fn find_prop_usages_in_expr(
        expr: &str,
        prop_name: &str,
        base_offset: usize,
        full_content: &str,
        range: Range,
        hints: &mut Vec<InlayHint>,
    ) {
        if prop_name.is_empty() || expr.is_empty() {
            return;
        }

        let mut search_pos = 0;

        while let Some(found) = expr[search_pos..].find(prop_name) {
            let abs_pos = search_pos + found;

            // Bounds check
            if abs_pos + prop_name.len() > expr.len() {
                break;
            }

            // Check word boundaries
            let before_ok = abs_pos == 0
                || expr
                    .as_bytes()
                    .get(abs_pos - 1)
                    .map(|&b| !Self::is_ident_char(b))
                    .unwrap_or(true);
            let after_ok = expr
                .as_bytes()
                .get(abs_pos + prop_name.len())
                .map(|&b| !Self::is_ident_char(b))
                .unwrap_or(true);

            // Check it's not preceded by "props." already
            let not_already_prefixed =
                abs_pos < 6 || &expr[abs_pos.saturating_sub(6)..abs_pos] != "props.";

            // Check it's not a property access (preceded by .)
            let not_property_access = abs_pos == 0
                || expr
                    .as_bytes()
                    .get(abs_pos - 1)
                    .map(|&b| b != b'.')
                    .unwrap_or(true);

            // Check it's not part of an event name pattern like "update:title" (preceded by :)
            let not_event_name_part = abs_pos == 0
                || expr
                    .as_bytes()
                    .get(abs_pos - 1)
                    .map(|&b| b != b':')
                    .unwrap_or(true);

            if before_ok
                && after_ok
                && not_already_prefixed
                && not_property_access
                && not_event_name_part
            {
                let sfc_offset = base_offset + abs_pos;

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
}
