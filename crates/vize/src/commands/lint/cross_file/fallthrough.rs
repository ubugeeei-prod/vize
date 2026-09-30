//! Call-site warning for attrs that a statically resolved child cannot inherit.

use std::path::PathBuf;

use super::sfc::{CrossFileSourceOffsets, FallthroughRoot};
use vize_croquis_cf::{CrossFileAnalyzer, CrossFileResult, FileId};
use vize_l0::{FxHashMap, cstr};
use vize_patina::{HelpLevel, LintDiagnostic, LintResult};

pub(super) const RULE: &str = "vue/cross-file-attrs-fallthrough";

pub(super) fn apply<S: AsRef<str>>(
    files: &[(PathBuf, S)],
    analyzer: &CrossFileAnalyzer,
    analysis: &CrossFileResult,
    indexes: &FxHashMap<FileId, usize>,
    offsets: &FxHashMap<FileId, CrossFileSourceOffsets>,
    results: &mut [LintResult],
    help_level: HelpLevel,
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
        let Some((_, source)) = files.get(parent_index) else {
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
            result.diagnostics.push(diagnostic);
        }
    }
}

#[cfg(test)]
mod tests;
