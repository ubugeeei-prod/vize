//! Resolved lint-plan inspection shared by CLI and development integrations.

pub(crate) mod matcher;

use matcher::{LintPlanScope, absolute_path, normalize_path};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
use vize_s0::{String, config::LintRuleSeverity};

/// Serializable ordered plan accepted by the inspector surface.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct InspectorLintPlan {
    pub items: Vec<InspectorLintPlanItem>,
}

/// One named rule block and its file scope.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct InspectorLintPlanItem {
    pub name: String,
    pub base_path: Option<String>,
    pub files: Option<Vec<String>>,
    pub ignores: Vec<String>,
    pub rules: BTreeMap<String, LintRuleSeverity>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorLintPlanPayload {
    pub schema: &'static str,
    pub version: u8,
    pub root: String,
    pub items: Vec<InspectorLintPlanItemSummary>,
    pub files: Vec<InspectorLintPlanFile>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorLintPlanItemSummary {
    pub name: String,
    pub base_path: Option<String>,
    pub files: Option<Vec<String>>,
    pub ignores: Vec<String>,
    pub rules: BTreeMap<String, LintRuleSeverity>,
    pub global_ignore: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorLintPlanFile {
    pub path: String,
    pub ignored: bool,
    pub ignored_by: Vec<String>,
    pub matched_items: Vec<String>,
    pub rules: Vec<InspectorLintPlanRule>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorLintPlanRule {
    pub name: String,
    pub severity: LintRuleSeverity,
    pub set_by: String,
}

struct CompiledItem<'a> {
    item: &'a InspectorLintPlanItem,
    scope: LintPlanScope,
    global_ignore: bool,
}

/// Resolve ordered rule provenance for every requested file.
pub fn inspect_lint_plan(
    plan: &InspectorLintPlan,
    root: &Path,
    files: &[String],
) -> InspectorLintPlanPayload {
    let root = absolute_path(root, Path::new("."));
    let compiled = plan
        .items
        .iter()
        .map(|item| CompiledItem {
            global_ignore: item.files.is_none()
                && item.rules.is_empty()
                && !item.ignores.is_empty(),
            scope: LintPlanScope::new(
                item.base_path.as_deref(),
                item.files.as_deref(),
                &item.ignores,
                &root,
                Path::new("."),
            ),
            item,
        })
        .collect::<Vec<_>>();
    let files = files
        .iter()
        .map(|file| inspect_file(&compiled, &root, file.as_str()))
        .collect();
    let items = compiled
        .iter()
        .map(|entry| InspectorLintPlanItemSummary {
            name: entry.item.name.clone(),
            base_path: entry.item.base_path.clone(),
            files: entry.item.files.clone(),
            ignores: entry.item.ignores.clone(),
            rules: entry.item.rules.clone(),
            global_ignore: entry.global_ignore,
        })
        .collect();
    InspectorLintPlanPayload {
        schema: "vize.inspector.lint-plan",
        version: 1,
        root: normalize_path(&root),
        items,
        files,
    }
}

fn inspect_file(compiled: &[CompiledItem<'_>], root: &Path, file: &str) -> InspectorLintPlanFile {
    let absolute = absolute_path(Path::new(file), root);
    let path = absolute
        .strip_prefix(root)
        .map_or_else(|_| normalize_path(&absolute), normalize_path);
    let ignored_by = compiled
        .iter()
        .filter(|entry| entry.global_ignore && entry.scope.ignores(&absolute))
        .map(|entry| entry.item.name.clone())
        .collect::<Vec<_>>();
    if !ignored_by.is_empty() {
        return InspectorLintPlanFile {
            path,
            ignored: true,
            ignored_by,
            matched_items: Vec::new(),
            rules: Vec::new(),
        };
    }

    let mut matched_items = Vec::new();
    let mut rules = BTreeMap::new();
    for entry in compiled {
        if entry.global_ignore || !entry.scope.matches(&absolute) {
            continue;
        }
        matched_items.push(entry.item.name.clone());
        for (name, severity) in &entry.item.rules {
            rules.insert(name.clone(), (*severity, entry.item.name.clone()));
        }
    }
    InspectorLintPlanFile {
        path,
        ignored: false,
        ignored_by,
        matched_items,
        rules: rules
            .into_iter()
            .map(|(name, (severity, set_by))| InspectorLintPlanRule {
                name,
                severity,
                set_by,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests;
