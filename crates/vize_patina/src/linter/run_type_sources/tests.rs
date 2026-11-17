use super::RunTypeSources;
use crate::{LintPreset, Linter};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

#[test]
fn relative_filenames_become_absolute_importers_under_the_run_root() {
    let sources = RunTypeSources::with_root(Some(PathBuf::from("/work/app")));
    assert_eq!(sources.importer("src/Card.vue"), "/work/app/src/Card.vue");
    assert_eq!(sources.importer("./Card.vue"), "/work/app/Card.vue");
    assert_eq!(sources.importer("src/./Card.vue"), "/work/app/src/Card.vue");
    // `..` is left for the snapshot's full canonicalization.
    assert_eq!(
        sources.importer("../lib/Card.vue"),
        "/work/app/../lib/Card.vue"
    );
    assert!(matches!(
        sources.importer("/abs/Card.vue"),
        Cow::Borrowed("/abs/Card.vue")
    ));
    assert!(matches!(sources.importer(""), Cow::Borrowed("")));
    let rootless = RunTypeSources::with_root(None);
    assert!(matches!(
        rootless.importer("src/Card.vue"),
        Cow::Borrowed("src/Card.vue")
    ));
}

const CARD: &str = r#"<script setup lang="ts">
import type { CardProps } from './types';
defineProps<CardProps>();
</script>

<template>
  <p>{{ title }} {{ count }} {{ missing }}</p>
</template>
"#;

fn undefined_refs<'s>(linter: &Linter, filename: &str, source: &'s str) -> Vec<&'s str> {
    linter
        .lint_sfc(source, filename)
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/no-undefined-refs")
        .filter_map(|diagnostic| source.get(diagnostic.start as usize..diagnostic.end as usize))
        .collect()
}

fn run_linter(root: Option<&Path>) -> Linter {
    let mut linter = Linter::with_preset(LintPreset::HappyPath);
    linter.type_sources = Some(RunTypeSources::with_root(root.map(Path::to_path_buf)));
    linter
}

#[test]
fn a_relative_filename_reads_the_types_beside_it_under_the_run_root() {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("temp dir");
    };
    let written = std::fs::write(
        dir.path().join("base.ts"),
        "export interface Base {\n  title: string;\n}\n",
    )
    .and_then(|()| {
        std::fs::write(
            dir.path().join("types.ts"),
            "import type { Base } from './base';\nexport interface CardProps extends Base {\n  count?: number;\n}\n",
        )
    });
    assert!(written.is_ok());
    let Some(absolute) = dir.path().join("Card.vue").to_str().map(str::to_owned) else {
        panic!("utf-8 temp path");
    };

    // The absolute spelling, with or without a run root, is the reference.
    let fresh = undefined_refs(&Linter::with_preset(LintPreset::HappyPath), &absolute, CARD);
    assert_eq!(fresh, ["missing"]);
    assert_eq!(undefined_refs(&run_linter(None), &absolute, CARD), fresh);
    // Under the root, the relative spelling resolves both modules of the chain.
    assert_eq!(
        undefined_refs(&run_linter(Some(dir.path())), "Card.vue", CARD),
        fresh
    );
    assert_eq!(
        undefined_refs(&run_linter(Some(dir.path())), "./Card.vue", CARD),
        fresh
    );
}
