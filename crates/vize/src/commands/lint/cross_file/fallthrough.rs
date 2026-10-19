//! Call-site warning for attrs that a statically resolved child cannot inherit.

use std::path::PathBuf;

use super::super::routes::config::{CrossFileRuleSettings, configure};
use super::sfc::{CrossFileSourceOffsets, FallthroughRoot};
use vize_croquis_cf::{CrossFileAnalyzer, CrossFileResult, FileId};
use vize_l0::{FxHashMap, cstr};
use vize_patina::{HelpLevel, LintDiagnostic, LintResult};

pub(in crate::commands::lint) const RULE: &str = "vue/cross-file-attrs-fallthrough";

pub(super) fn apply<S: AsRef<str>>(
    files: &[(PathBuf, S)],
    analyzer: &CrossFileAnalyzer,
    analysis: &CrossFileResult,
    indexes: &FxHashMap<FileId, usize>,
    offsets: &FxHashMap<FileId, CrossFileSourceOffsets>,
    results: &mut [LintResult],
    (help_level, settings): (HelpLevel, Option<&CrossFileRuleSettings<'_>>),
) {
    for usage in &analysis.fallthrough_usage_facts {
        let Some(&parent_index) = indexes.get(&usage.parent_file_id) else {
            continue;
        };
        // A same-named component elsewhere in the project is not evidence for
        // this call site. Only trust the parent's resolved import.
        if vize_croquis_cf::facts::imported_render_target(
            analyzer.registry(),
            usage.parent_file_id,
            usage.component_name.as_str(),
        ) != Some(usage.child_file_id)
        {
            continue;
        }
        let Some(child) = analyzer.get_analysis(usage.child_file_id) else {
            continue;
        };
        let Some(child_offsets) = offsets.get(&usage.child_file_id) else {
            continue;
        };
        let reason = if child.template_info.inherit_attrs_disabled {
            "inheritAttrs: false"
        } else {
            match child_offsets.fallthrough_root {
                Some(FallthroughRoot::Teleport) => "a Teleport root",
                Some(FallthroughRoot::Fragment) => "a fragment root",
                Some(FallthroughRoot::Text) => "a text root",
                None => continue,
            }
        };
        if child.template_info.uses_attrs
            || child.template_info.binds_attrs_explicitly
            || child_offsets.script_uses_attrs
        {
            continue;
        }
        let Some(parent_offsets) = offsets.get(&usage.parent_file_id) else {
            continue;
        };
        let Some((path, source)) = files.get(parent_index) else {
            continue;
        };
        let source_len = source.as_ref().len() as u32;
        let Some(result) = results.get_mut(parent_index) else {
            continue;
        };
        for attr in &usage.attrs {
            if !attr.fallthrough
                || attr.name_is_dynamic
                || matches!(attr.name.as_str(), "key" | "ref")
            {
                continue;
            }
            let start = parent_offsets
                .template
                .saturating_add(attr.source_start)
                .min(source_len);
            let end = parent_offsets
                .template
                .saturating_add(attr.source_end)
                .max(start.saturating_add(1))
                .min(source_len);
            let message = cstr!(
                "{} passed to <{}> cannot fall through because the component has {}",
                attr.name,
                usage.component_name,
                reason
            );
            let mut diagnostic = LintDiagnostic::warn(RULE, message, start, end);
            if let Some(help) = help_level.process(
                "Pass the value as a declared prop, or bind $attrs explicitly to the intended element in the child.",
            ) {
                diagnostic = diagnostic.with_help(help);
            }
            if let Some(diagnostic) = configure(settings, path, RULE, diagnostic) {
                result.diagnostics.push(diagnostic);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::build_cross_file_lint_output, RULE};
    use std::{fs, path::Path};
    use vize_patina::HelpLevel;

    fn diagnostics(parent: &str, child: &str) -> Vec<(String, u32, u32)> {
        let dir = tempfile::tempdir().unwrap();
        let parent_path = dir.path().join("Parent.vue");
        let child_path = dir.path().join("Dialog.vue");
        fs::write(&parent_path, parent).unwrap();
        fs::write(&child_path, child).unwrap();
        let files = [&parent_path, &child_path]
            .into_iter()
            .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
            .collect::<Vec<_>>();
        let output = build_cross_file_lint_output(&files, HelpLevel::Short, false);
        output
            .results
            .into_iter()
            .filter(|result| {
                Path::new(result.filename.as_str()).file_name().unwrap() == "Parent.vue"
            })
            .flat_map(|result| result.diagnostics)
            .filter(|diagnostic| diagnostic.rule_name == RULE)
            .map(|diagnostic| {
                (
                    diagnostic.message.to_string(),
                    diagnostic.start,
                    diagnostic.end,
                )
            })
            .collect()
    }

    #[test]
    fn warns_at_each_call_site_attr_for_teleport_without_flagging_props_or_emits() {
        let parent = r#"<script setup lang="ts">import Dialog from './Dialog.vue'</script>
<template><Dialog title="Hi" class="overlay" :style="styling" @close="hide" /></template>"#;
        let child = r#"<script setup lang="ts">
defineProps<{ title: string }>()
defineEmits(['close'])
</script>
<template><Teleport to="body"><div /></Teleport></template>"#;
        let found = diagnostics(parent, child);
        assert_eq!(found.len(), 2, "{found:?}");
        assert_eq!(
            found[0].1 as usize,
            parent.find("class=\"overlay\"").unwrap()
        );
        assert_eq!(
            found[1].1 as usize,
            parent.find(":style=\"styling\"").unwrap()
        );
        assert!(
            found
                .iter()
                .all(|(message, _, _)| message.contains("Teleport root"))
        );
    }

    #[test]
    fn covers_fragment_text_and_disabled_inheritance_but_allows_explicit_attrs() {
        let parent = r#"<script setup>import Dialog from './Dialog.vue'</script>
<template>日本語<Dialog class="overlay" /></template>"#;
        for child in [
            "<template><div /><div /></template>",
            "<template>{{ message }}</template>",
            "<script setup>defineOptions({ inheritAttrs: false })</script><template><div /></template>",
            "<script>export default { inheritAttrs: false }</script><template><div /></template>",
        ] {
            let found = diagnostics(parent, child);
            assert_eq!(found.len(), 1, "{child}: {found:?}");
            assert_eq!(
                found[0].1 as usize,
                parent.find("class=\"overlay\"").unwrap()
            );
        }
        for child in [
            "<template><div /></template>",
            "<template><Teleport to=\"body\"><div v-bind=\"$attrs\" /></Teleport></template>",
        ] {
            assert!(diagnostics(parent, child).is_empty(), "{child}");
        }
        let lowercase = diagnostics(
            parent,
            "<template><teleport to=\"body\"><div /></teleport></template>",
        );
        assert_eq!(lowercase.len(), 1, "{lowercase:?}");
        assert_eq!(
            lowercase[0].1 as usize,
            parent.find("class=\"overlay\"").unwrap()
        );
    }

    #[test]
    fn structural_roots_and_script_use_attrs_are_classified_from_syntax() {
        let parent = "<script setup>import Dialog from './Dialog.vue'</script><template><Dialog class=\"x\" :style=\"style\" /></template>";
        for child in [
            "<script setup>// useAttrs()\n</script><template><Teleport to=\"body\"><div /></Teleport></template>",
            "<script setup>const note = 'useAttrs()'</script><template><Teleport to=\"body\"><div /></Teleport></template>",
        ] {
            assert_eq!(diagnostics(parent, child).len(), 2, "{child}");
        }
        assert!(diagnostics(parent, "<script setup>useAttrs()</script><template><Teleport to=\"body\"><div /></Teleport></template>").is_empty());
        assert!(diagnostics(parent, r#"<script setup>\u0075seAttrs()</script><template><Teleport to="body"><div /></Teleport></template>"#).is_empty());
        for call in [
            "import { useAttrs as attrs } from 'vue'; attrs()",
            r"import { useAttrs as attrs } from 'vue'; \u0061ttrs()",
            "attrs(); import { useAttrs as attrs } from 'vue'",
            "import * as Vue from 'vue'; Vue.useAttrs()",
            r"import * as Vue from 'vue'; Vue.\u0075seAttrs()",
            "Vue.useAttrs(); import * as Vue from 'vue'",
        ] {
            let child = format!(
                "<script setup>{call}</script><template><Teleport to=\"body\"><div /></Teleport></template>"
            );
            assert!(diagnostics(parent, &child).is_empty(), "{child}");
        }
        let local = "<script setup>function useAttrs() {} useAttrs()</script><template><Teleport to=\"body\"><div /></Teleport></template>";
        assert_eq!(diagnostics(parent, local).len(), 2);
        let later_local = "<script setup>useAttrs(); function useAttrs() {}</script><template><Teleport to=\"body\"><div /></Teleport></template>";
        assert_eq!(diagnostics(parent, later_local).len(), 2);
        assert!(
            diagnostics(
                parent,
                "<template><div v-if=\"ready\" /><div v-else /></template>"
            )
            .is_empty()
        );
        assert_eq!(
            diagnostics(
                parent,
                "<template><div v-for=\"item in items\" /></template>"
            )
            .len(),
            2
        );
    }
}
