//! Template-side inlay hint collection.
//!
//! Finds usages of props in template mustache expressions and
//! Vue directive attributes, generating `#props.` prefix hints.

use tower_lsp::lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, Position, Range};

use super::InlayHintService;
use super::expr_regions::code_regions;
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

        while let Some(start) = template.get(pos..).and_then(|rest| rest.find("{{")) {
            let abs_start = pos + start + 2; // Skip "{{"

            if let Some((expr, _)) = template
                .get(abs_start..)
                .and_then(|rest| rest.split_once("}}"))
            {
                let abs_end = abs_start + expr.len();

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

        while let Some(remaining) = template.get(pos..).filter(|rest| !rest.is_empty()) {
            // Find next directive or shorthand
            let mut next_match: Option<(usize, usize, char)> = None; // (position, skip_len, quote_char)

            // Check for shorthand patterns: :prop=", @event=", #slot="
            for (i, c) in remaining.char_indices() {
                if (c == ':' || c == '@' || c == '#') && i + 1 < remaining.len() {
                    // Check if followed by identifier and ="
                    let after = remaining.get(i + 1..).unwrap_or_default();
                    if let Some((attr_name, _)) = after.split_once('=') {
                        let eq_pos = attr_name.len();
                        // Validate it's a valid attribute name (alphanumeric, -, _)
                        if !attr_name.is_empty()
                            && attr_name
                                .chars()
                                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':')
                        {
                            let quote_start = i + 1 + eq_pos + 1;
                            if let Some(&quote @ (b'"' | b'\'')) =
                                remaining.as_bytes().get(quote_start)
                            {
                                next_match = Some((i, quote_start + 1, quote as char));
                                break;
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
                        if let Some(eq_pos) =
                            remaining.get(pattern_end..).and_then(|rest| rest.find('='))
                        {
                            let quote_pos = pattern_end + eq_pos + 1;
                            if let Some(&quote @ (b'"' | b'\'')) =
                                remaining.as_bytes().get(quote_pos)
                            {
                                let new_match = (found, quote_pos + 1, quote as char);
                                if next_match
                                    .as_ref()
                                    .is_none_or(|current| new_match.0 < current.0)
                                {
                                    next_match = Some(new_match);
                                }
                            }
                        }
                    } else if let Some(&quote) = pattern.as_bytes().last() {
                        // v-if=", etc. - pattern already includes the quote
                        let new_match = (found, pattern_end, quote as char);
                        if next_match
                            .as_ref()
                            .is_none_or(|current| new_match.0 < current.0)
                        {
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
            if let Some((expr, _)) = template
                .get(abs_start..)
                .and_then(|rest| rest.split_once(quote))
            {
                let abs_end = abs_start + expr.len();

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

        // Only executable code can reference a prop: matching text inside
        // string literals, template-literal text, or comments must not hint
        // (a static `tag--size-` chunk is not a use of `size`).
        let regions = code_regions(expr);
        let mut region_iter = regions.iter();
        let mut region = match region_iter.next() {
            Some(&region) => region,
            None => return,
        };
        let mut search_pos = region.0;
        // Resume one character past each match so overlapping names are found.
        let step = prop_name.chars().next().map_or(1, char::len_utf8);

        while let Some(found) = expr.get(search_pos..).and_then(|rest| rest.find(prop_name)) {
            let abs_pos = search_pos + found;

            // Bounds check
            if abs_pos + prop_name.len() > expr.len() {
                break;
            }

            // Advance to the code region containing this match; skip the
            // match when it starts outside code or crosses a region edge.
            while abs_pos >= region.1 {
                region = match region_iter.next() {
                    Some(&next) => next,
                    None => return,
                };
            }
            if abs_pos < region.0 {
                search_pos = region.0;
                continue;
            }
            if abs_pos + prop_name.len() > region.1 {
                search_pos = abs_pos + step;
                continue;
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
            let not_already_prefixed = !expr
                .get(..abs_pos)
                .is_some_and(|before| before.ends_with("props."));

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
                    search_pos = abs_pos + step;
                    continue;
                }

                let (line, character) = offset_to_position(full_content, sfc_offset);

                let position = Position { line, character };

                // Check if within requested range
                if Self::position_in_range(position, range) {
                    hints.push(InlayHint {
                        position,
                        label: InlayHintLabel::String("#props.".to_owned()),
                        kind: Some(InlayHintKind::TYPE),
                        text_edits: None,
                        tooltip: Some(tower_lsp::lsp_types::InlayHintTooltip::String(
                            "Destructured from defineProps".to_owned(),
                        )),
                        padding_left: None,
                        padding_right: Some(true),
                        data: None,
                    });
                }
            }

            search_pos = abs_pos + step;
        }
    }
}
