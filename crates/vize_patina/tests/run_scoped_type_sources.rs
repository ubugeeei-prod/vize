//! A run-scoped disk view resolves imported macro types exactly as a fresh
//! view per SFC does, for every SFC that shares the imported modules.

use std::path::PathBuf;
use std::sync::OnceLock;

use vize_patina::{LintPreset, Linter, SharedTypeSources};

const UNDEFINED_REFS: &str = "vue/no-undefined-refs";

fn findings(linter: &Linter, source: &str, path: &std::path::Path) -> Vec<(String, u32, u32)> {
    linter
        .lint_sfc(source, path.to_str().unwrap())
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                format!("{} {}", diagnostic.rule_name, diagnostic.message),
                diagnostic.start,
                diagnostic.end,
            )
        })
        .collect()
}

/// Two SFCs that read their props through a chain of two imported modules,
/// with the one name each template reads that neither module declares.
fn imported_props_sfcs(dir: &std::path::Path) -> [(&'static str, PathBuf, &'static str); 2] {
    std::fs::write(
        dir.join("base.ts"),
        "export interface Base {
  title: string;
}
",
    )
    .unwrap();
    std::fs::write(
        dir.join("types.ts"),
        "import type { Base } from './base';
export interface CardProps extends Base {
  count?: number;
}
",
    )
    .unwrap();
    let card = r#"<script setup lang="ts">
import type { CardProps } from './types';
defineProps<CardProps>();
</script>

<template>
  <p>{{ title }} {{ count }} {{ missing }}</p>
</template>
"#;
    let badge = r#"<script setup lang="ts">
import type { CardProps } from './types';
defineProps<CardProps>();
</script>

<template>
  <button @click="title = 'next'">{{ count }} {{ absent }}</button>
</template>
"#;
    [
        (card, dir.join("Card.vue"), "missing"),
        (badge, dir.join("Badge.vue"), "absent"),
    ]
}

fn assert_matches_fresh(linter: &Linter, source: &str, path: &std::path::Path, undefined: &str) {
    let fresh = findings(&Linter::with_preset(LintPreset::HappyPath), source, path);
    let reused = findings(linter, source, path);
    assert_eq!(reused, fresh, "{}", path.display());
    let undefined_refs: Vec<_> = reused
        .iter()
        .filter(|(rule, _, _)| rule.starts_with(UNDEFINED_REFS))
        .map(|(_, start, end)| &source[*start as usize..*end as usize])
        .collect();
    // Props from both modules of the chain resolved; only the unknown name is reported.
    assert_eq!(undefined_refs, [undefined], "{}", path.display());
}

#[test]
fn shared_view_matches_fresh_views_across_sfcs() {
    let dir = tempfile::tempdir().unwrap();
    let shared = Linter::with_preset(LintPreset::HappyPath).with_run_scoped_type_sources();
    for (source, path, undefined) in &imported_props_sfcs(dir.path()) {
        assert_matches_fresh(&shared, source, path, undefined);
    }
}

#[test]
fn linters_sharing_a_run_view_match_fresh_views_across_sfcs() {
    // A host that creates a linter per file, like the Oxlint plugin, keeps the run's view in a static.
    static SOURCES: OnceLock<SharedTypeSources> = OnceLock::new();
    let dir = tempfile::tempdir().unwrap();
    for (source, path, undefined) in &imported_props_sfcs(dir.path()) {
        let linter = Linter::with_preset(LintPreset::HappyPath)
            .with_shared_type_sources(SOURCES.get_or_init(SharedTypeSources::for_current_dir));
        assert_matches_fresh(&linter, source, path, undefined);
    }
}
