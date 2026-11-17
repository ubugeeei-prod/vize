//! A run-scoped disk view resolves imported macro types exactly as a fresh
//! view per SFC does, for every SFC that shares the imported modules.

use vize_patina::{LintPreset, Linter};

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

#[test]
fn shared_view_matches_fresh_views_across_sfcs() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("base.ts"),
        "export interface Base {\n  title: string;\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("types.ts"),
        "import type { Base } from './base';\nexport interface CardProps extends Base {\n  count?: number;\n}\n",
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
    let files = [
        (card, dir.path().join("Card.vue"), "missing"),
        (badge, dir.path().join("Badge.vue"), "absent"),
    ];
    let shared = Linter::with_preset(LintPreset::HappyPath).with_run_scoped_type_sources();
    for (source, path, undefined) in &files {
        let fresh = findings(&Linter::with_preset(LintPreset::HappyPath), source, path);
        let reused = findings(&shared, source, path);
        assert_eq!(reused, fresh, "{}", path.display());
        let undefined_refs: Vec<_> = reused
            .iter()
            .filter(|(rule, _, _)| rule.starts_with(UNDEFINED_REFS))
            .map(|(_, start, end)| &source[*start as usize..*end as usize])
            .collect();
        // Props from both modules of the chain resolved; only the unknown name is reported.
        assert_eq!(undefined_refs, [*undefined], "{}", path.display());
    }
}
