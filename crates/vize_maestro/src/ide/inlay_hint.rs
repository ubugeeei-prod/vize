//! Inlay hints provider.
//!
//! Provides inlay hints for:
//! - Props destructure (show `#props.` prefix for destructured props in template and script)
//!
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]
//! Uses vize_croquis for proper scope analysis to accurately identify destructured props.

mod expr_regions;
mod script;
mod template;

#[cfg(test)]
mod binding_hint_tests;
#[cfg(test)]
mod i18n_hint_tests;
#[cfg(test)]
mod prop_hint_tests;

use tower_lsp::lsp_types::{InlayHint, Position, Range, Url};
use vize_croquis::{Drawer, DrawerOptions};

use crate::ide::ecosystem;
use crate::ide::offset_to_position;

/// Inlay hint service.
pub struct InlayHintService;

impl InlayHintService {
    /// Get inlay hints for a document range.
    pub fn get_hints(content: &str, uri: &Url, range: Range) -> Vec<InlayHint> {
        Self::get_hints_with_ecosystem(content, uri, range, true)
    }

    /// Get inlay hints for a document range with optional ecosystem helpers.
    pub fn get_hints_with_ecosystem(
        content: &str,
        uri: &Url,
        range: Range,
        ecosystem_enabled: bool,
    ) -> Vec<InlayHint> {
        let mut hints = Vec::new();

        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
            return hints;
        };

        if ecosystem_enabled {
            ecosystem::i18n::collect_inlay_hints(
                content,
                &descriptor,
                Some(uri),
                range,
                &mut hints,
            );
        }

        // Use the Croquis drawer for proper scope analysis.
        let Some(ref script_setup) = descriptor.script_setup else {
            return hints;
        };

        // Analyze the script setup using croquis
        let mut analyzer = Drawer::with_options(DrawerOptions {
            analyze_script: true,
            ..Default::default()
        });
        analyzer.analyze_script_setup(&script_setup.content);
        let croquis = analyzer.finish();

        // Get all prop names from defineProps (for template hints)
        let all_prop_names: Vec<String> = croquis
            .macros
            .props()
            .iter()
            .map(|p| p.name.to_string())
            .collect();

        // Get props destructure info from the analysis (for script hints)
        let props_destructure = croquis.macros.props_destructure();

        // Collect local names of destructured props (for script)
        let destructured_local_names: Vec<&str> = props_destructure
            .map(|pd| pd.bindings.values().map(|b| b.local.as_str()).collect())
            .unwrap_or_default();

        // Get the defineProps call span to skip hints within the type definition
        let define_props_end = croquis
            .macros
            .define_props()
            .map(|call| call.end as usize)
            .unwrap_or(0);

        // Find usages of destructured props in script setup (only destructured ones)
        if !destructured_local_names.is_empty() {
            Self::collect_script_props_hints(
                &script_setup.content,
                script_setup.loc.start,
                content,
                &destructured_local_names,
                define_props_end,
                range,
                &mut hints,
            );
        }

        // Find usages of props in template (all props are available in template)
        if let Some(ref template) = descriptor.template
            && !all_prop_names.is_empty()
        {
            let prop_refs: Vec<&str> = all_prop_names.iter().map(|s| s.as_str()).collect();
            Self::collect_template_props_hints(
                &template.content,
                template.loc.start,
                content,
                &prop_refs,
                range,
                &mut hints,
            );
        }

        // Reactive-binding inlay hints: show `: Ref<…>` / `: ComputedRef<…>`
        // after `const X = ref(...)` / `const X = computed(() => ...)` so the
        // editor surfaces the inferred wrapper without requiring hover.
        Self::collect_reactive_binding_hints(
            &script_setup.content,
            script_setup.loc.start,
            content,
            &croquis,
            range,
            &mut hints,
        );

        hints
    }

    /// Append inlay hints for reactive binding declarations.
    fn collect_reactive_binding_hints(
        script: &str,
        script_offset: usize,
        full_content: &str,
        croquis: &vize_croquis::Croquis,
        range: Range,
        hints: &mut Vec<InlayHint>,
    ) {
        use tower_lsp::lsp_types::{InlayHintKind, InlayHintLabel, Position};
        use vize_croquis::reactivity::ReactiveKind;

        for source in croquis.reactivity.sources() {
            // Only attach the hint to ref-family bindings; reactive() objects
            // are direct, no wrapper to surface.
            let wrapper = match source.kind {
                ReactiveKind::Ref | ReactiveKind::ShallowRef | ReactiveKind::ToRef => "Ref",
                ReactiveKind::Computed => "ComputedRef",
                _ => continue,
            };
            // Resolve the inner type via the same heuristic that completion
            // uses for the .value shortcut. Falls back to `_` when the source
            // is too dynamic to infer (e.g. `ref(props.bar)`).
            let value_type = crate::ide::completion::infer_reactive_value_type(
                script,
                source.name.as_str(),
                source.kind,
            )
            .unwrap_or_else(|| "_".to_string());

            // Locate `const NAME =` in the script content. Anchoring on the
            // declaration keyword avoids matching usages inside expressions.
            let needle_const = vize_s0::cstr!("const {} =", source.name.as_str());
            let needle_let = vize_s0::cstr!("let {} =", source.name.as_str());
            let pos_in_script = script
                .find(needle_const.as_str())
                .map(|p| p + needle_const.len())
                .or_else(|| {
                    script
                        .find(needle_let.as_str())
                        .map(|p| p + needle_let.len())
                });
            let Some(pos_in_script) = pos_in_script else {
                continue;
            };
            // Anchor the hint at the position right after the binding name
            // (just before the `=`). That keeps the inlay rendered between
            // the identifier and the initializer.
            let name_end_in_script = {
                let mut walk = pos_in_script - " =".len();
                while walk > 0 && script.as_bytes()[walk - 1] == b' ' {
                    walk -= 1;
                }
                walk
            };
            let sfc_offset = script_offset + name_end_in_script;
            if sfc_offset > full_content.len() {
                continue;
            }
            let (line, character) = offset_to_position(full_content, sfc_offset);
            let position = Position { line, character };
            if !Self::position_in_range(position, range) {
                continue;
            }
            let label = vize_s0::cstr!(": {}<{}>", wrapper, value_type.as_str());
            hints.push(InlayHint {
                position,
                label: InlayHintLabel::String(label.to_string()),
                kind: Some(InlayHintKind::TYPE),
                text_edits: None,
                tooltip: Some(tower_lsp::lsp_types::InlayHintTooltip::String(
                    vize_s0::cstr!("Vue reactive binding ({})", wrapper).to_string(),
                )),
                padding_left: Some(true),
                padding_right: None,
                data: None,
            });
        }
    }

    /// Check if a position is within a range.
    fn position_in_range(pos: Position, range: Range) -> bool {
        if pos.line < range.start.line || pos.line > range.end.line {
            return false;
        }
        if pos.line == range.start.line && pos.character < range.start.character {
            return false;
        }
        if pos.line == range.end.line && pos.character > range.end.character {
            return false;
        }
        true
    }

    fn is_ident_char(c: u8) -> bool {
        c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
    }
}
