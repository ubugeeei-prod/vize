//! Inlay hints provider.
//!
//! Provides inlay hints for:
//! - Props destructure (show `#props.` prefix for destructured props in template and script)
//!
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]
//! Uses vize_croquis for proper scope analysis to accurately identify destructured props.

mod script;
mod template;

use tower_lsp::lsp_types::{InlayHint, Position, Range, Url};
use vize_croquis::{Analyzer, AnalyzerOptions};

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

        // Use vize_croquis analyzer for proper scope analysis
        let Some(ref script_setup) = descriptor.script_setup else {
            return hints;
        };

        // Analyze the script setup using croquis
        let mut analyzer = Analyzer::with_options(AnalyzerOptions {
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

            // Locate `const NAME =` in the script content. Anchoring on the
            // declaration keyword avoids matching usages inside expressions.
            let needle_const = vize_carton::cstr!("const {} =", source.name.as_str());
            let needle_let = vize_carton::cstr!("let {} =", source.name.as_str());
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
            let label = vize_carton::cstr!(": {}<{}>", wrapper, "_");
            hints.push(InlayHint {
                position,
                label: InlayHintLabel::String(label.to_string()),
                kind: Some(InlayHintKind::TYPE),
                text_edits: None,
                tooltip: Some(tower_lsp::lsp_types::InlayHintTooltip::String(
                    vize_carton::cstr!("Vue reactive binding ({})", wrapper).to_string(),
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::InlayHintService;
    use tower_lsp::lsp_types::{InlayHintLabel, Position, Range, Url};

    #[test]
    fn test_props_destructure_analysis() {
        let content = r#"<script setup lang="ts">
const { title, disabled } = defineProps<{
  title: string
  disabled?: boolean
}>()

console.log(title)
</script>

<template>
  <div>{{ title }}</div>
</template>"#;

        let uri = Url::parse("file:///test.vue").unwrap();
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 100,
                character: 0,
            },
        };

        let hints = InlayHintService::get_hints(content, &uri, range);

        // Should have hints for title in script (line 6) and template (line 10)
        assert!(!hints.is_empty(), "Should have inlay hints");

        // Verify all hints are #props.
        for hint in &hints {
            if let InlayHintLabel::String(label) = &hint.label {
                assert_eq!(label, "#props.");
            }
        }
    }

    #[test]
    fn test_props_destructure_with_alias() {
        let content = r#"<script setup lang="ts">
const { title: localTitle } = defineProps<{
  title: string
}>()

console.log(localTitle)
</script>

<template>
  <div>{{ localTitle }}</div>
</template>"#;

        let uri = Url::parse("file:///test.vue").unwrap();
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 100,
                character: 0,
            },
        };

        let hints = InlayHintService::get_hints(content, &uri, range);

        // Should have hints for localTitle (the alias), not title
        assert!(
            !hints.is_empty(),
            "Should have inlay hints for aliased prop"
        );
    }

    #[test]
    fn test_no_hints_in_define_props_type() {
        let content = r#"<script setup lang="ts">
const { title } = defineProps<{
  title: string
}>()
</script>

<template>
  <div>{{ title }}</div>
</template>"#;

        let uri = Url::parse("file:///test.vue").unwrap();
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 100,
                character: 0,
            },
        };

        let hints = InlayHintService::get_hints(content, &uri, range);

        // Check that no hints are in the defineProps type definition
        // (lines 1-3 in script, which is around line 1-4 in the file)
        for hint in &hints {
            assert!(
                hint.position.line > 3,
                "Hint should not be in defineProps type definition, found at line {}",
                hint.position.line
            );
        }
    }

    #[test]
    fn test_is_in_string() {
        assert!(!InlayHintService::is_in_string("foo bar", 4));
        assert!(InlayHintService::is_in_string("'foo bar'", 4));
        assert!(InlayHintService::is_in_string("\"foo bar\"", 4));
        assert!(!InlayHintService::is_in_string("\"foo\" bar", 6));
        assert!(InlayHintService::is_in_string("`foo bar`", 4));
    }

    #[test]
    fn test_reactive_binding_inlay_hint() {
        let content = r#"<script setup lang="ts">
import { ref, computed } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value * 2)
</script>
"#;
        let uri = Url::parse("file:///reactive.vue").unwrap();
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 100,
                character: 0,
            },
        };
        let hints = InlayHintService::get_hints(content, &uri, range);
        let labels: Vec<String> = hints
            .iter()
            .filter_map(|h| match &h.label {
                InlayHintLabel::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect();
        assert!(
            labels.iter().any(|s| s.contains("Ref")),
            "expected a Ref<_> inlay hint, got {labels:?}",
        );
        assert!(
            labels.iter().any(|s| s.contains("ComputedRef")),
            "expected a ComputedRef<_> inlay hint, got {labels:?}",
        );
    }

    #[test]
    fn test_no_hints_in_event_name_pattern() {
        // Test that "title" in "update:title" event name does not get a hint
        let content = r#"<script setup lang="ts">
const { title } = defineProps<{
  title: string
}>()

const emit = defineEmits<{
  (e: 'update:title', value: string): void
}>()
</script>

<template>
  <input :value="title" @update:title="emit('update:title', $event)" />
</template>"#;

        let uri = Url::parse("file:///test.vue").unwrap();
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 100,
                character: 0,
            },
        };

        let hints = InlayHintService::get_hints(content, &uri, range);

        // Should have hints for title in :value="title" and possibly template
        // But NOT for title in 'update:title' event names
        for hint in &hints {
            // Get the position in the content
            let line = hint.position.line as usize;
            let lines: Vec<&str> = content.lines().collect();
            if line < lines.len() {
                let line_content = lines[line];
                // Verify the hint is not on a line containing 'update:title' pattern
                // where title immediately follows a colon
                let char_pos = hint.position.character as usize;
                if char_pos > 0 && char_pos <= line_content.len() {
                    let before_char = line_content.as_bytes().get(char_pos - 1);
                    assert_ne!(
                        before_char,
                        Some(&b':'),
                        "Hint should not be placed after colon (event name pattern)"
                    );
                }
            }
        }
    }

    #[test]
    fn test_props_without_destructure_in_template() {
        // Test that props defined without destructuring also get hints in template
        let content = r#"<script setup lang="ts">
const props = defineProps<{
  title: string
  count: number
}>()

// In script, we access via props.title (no hint needed for 'title' alone)
console.log(props.title)
</script>

<template>
  <div>{{ title }}</div>
  <span>{{ count }}</span>
</template>"#;

        let uri = Url::parse("file:///test.vue").unwrap();
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 100,
                character: 0,
            },
        };

        let hints = InlayHintService::get_hints(content, &uri, range);

        // Should have hints for title and count in template (lines 11 and 12)
        // Even though props are not destructured
        let template_hints: Vec<_> = hints.iter().filter(|h| h.position.line >= 11).collect();

        assert!(
            !template_hints.is_empty(),
            "Should have hints for props in template even without destructuring"
        );
    }

    #[test]
    fn test_i18n_message_preview_without_script_setup() {
        let content = r#"<template>
  <p>{{ $t("auth.login") }}</p>
</template>
<i18n lang="json">
{
  "en": { "auth": { "login": "Log in" } }
}
</i18n>
"#;

        let uri = Url::parse("file:///test.vue").unwrap();
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 100,
                character: 0,
            },
        };

        let hints = InlayHintService::get_hints(content, &uri, range);
        let labels: Vec<&str> = hints
            .iter()
            .filter_map(|hint| match &hint.label {
                InlayHintLabel::String(label) => Some(label.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(labels, vec!["= Log in"]);
    }

    #[test]
    fn test_i18n_message_preview_from_workspace_json_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("src/components/LoginButton.vue");
        let locale_path = dir.path().join("src/locales/en.json");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::create_dir_all(locale_path.parent().unwrap()).unwrap();
        fs::write(&locale_path, r#"{ "auth": { "login": "Log in" } }"#).unwrap();

        let content = r#"<template>
  <p>{{ $t("auth.login") }}</p>
</template>
"#;
        fs::write(&source_path, content).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 100,
                character: 0,
            },
        };

        let hints = InlayHintService::get_hints(content, &uri, range);
        let labels: Vec<&str> = hints
            .iter()
            .filter_map(|hint| match &hint.label {
                InlayHintLabel::String(label) => Some(label.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(labels, vec!["= Log in"]);
    }
}
