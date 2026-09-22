//! P0-13 pilot seeding (`--fixtures` / `--matrix` / `--corpus-shard`).

use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
};

use crate::common;
use crate::davinci_fpfn::{
    CLASS_A, CLASS_A_RULE, CLASS_B, EditRecord, Identifier, Injection, SeedFile, SeedManifest,
    SeedScope, SourceInfo, UNUSED_BINDING_NAME, apply_seed, describe_seeded_span, list_vue_files,
    plan_class_a, plan_class_b, resolve_corpus_sources, resolve_fixture_sources,
};

pub fn seed(
    repo_root: &Path,
    fixtures: Option<&Path>,
    matrix: bool,
    corpus_shard: bool,
    out_dir: &Path,
) -> Result<SeedManifest, String> {
    let source = resolve_sources(repo_root, fixtures, matrix, corpus_shard, out_dir)?;
    let mut files = Vec::new();
    let mut injections = Vec::new();
    let mut edits = BTreeMap::<String, Vec<EditRecord>>::new();
    let mut class_a_eligible = 0usize;
    for root in &source.roots {
        for rel_path in list_vue_files(&root.root)? {
            let seed_path = format!("{}{}", root.prefix, rel_path);
            let original = common::read_text(root.root.join(&rel_path))?;
            let (class_a, class_a_reason) = plan_class_a(&original);
            let (class_b, _) = plan_class_b(&original);
            let applied = apply_seed(&original, class_a.as_ref(), class_b.as_ref());
            files.push(SeedFile {
                path: seed_path.clone(),
                class_a: class_a.is_some(),
                class_b: class_b.is_some(),
                class_a_reason: class_a_reason.clone(),
            });
            if let Some(plan) = &class_a {
                class_a_eligible += 1;
                let ref_start = map_template_ref(
                    &applied.seeded,
                    plan.template_ref[0],
                    &plan.name,
                    &applied.edits,
                )?;
                let starts = crate::davinci_fpfn::line_starts_of(&applied.seeded);
                injections.push(Injection {
                    class_name: CLASS_A.to_string(),
                    path: seed_path.clone(),
                    expected_rule: Some(CLASS_A_RULE.to_string()),
                    identifier: Identifier {
                        original: Some(plan.name.clone()),
                        seeded: plan.seeded_name.clone(),
                    },
                    script_rename_count: Some(plan.rename_spans.len()),
                    created_script_setup_block: None,
                    expected: describe_seeded_span(
                        &applied.seeded,
                        &starts,
                        ref_start,
                        ref_start + plan.name.len(),
                    ),
                    note: None,
                });
            }
            if let Some(plan) = &class_b {
                let id_start = applied.seeded.find(UNUSED_BINDING_NAME).ok_or_else(|| {
                    "seed-defects internal error: unused binding not found".to_string()
                })?;
                let starts = crate::davinci_fpfn::line_starts_of(&applied.seeded);
                injections.push(Injection {
                    class_name: CLASS_B.to_string(),
                    path: seed_path.clone(),
                    expected_rule: None,
                    identifier: Identifier {
                        original: None,
                        seeded: UNUSED_BINDING_NAME.to_string(),
                    },
                    script_rename_count: None,
                    created_script_setup_block: Some(plan.created_block),
                    expected: describe_seeded_span(
                        &applied.seeded,
                        &starts,
                        id_start,
                        id_start + UNUSED_BINDING_NAME.len(),
                    ),
                    note: Some(
                        "vize_croquis unused_bindings has no lint consumer (documented FN, ledger-fn.md)"
                            .to_string(),
                    ),
                });
            }
            if !applied.edits.is_empty() {
                edits.insert(seed_path.clone(), applied.edits);
            }
            common::write_text(out_dir.join("original").join(&seed_path), &original)?;
            common::write_text(out_dir.join("seeded").join(&seed_path), &applied.seeded)?;
        }
    }
    injections.sort_by(|a, b| a.path.cmp(&b.path).then(a.class_name.cmp(&b.class_name)));
    let manifest = SeedManifest {
        schema_version: 1,
        tool: "tools/commands/davinci/seed-defects.rs".to_string(),
        source: SourceInfo {
            kind: source.kind,
            label: source.label,
        },
        scope: SeedScope {
            files_copied: files.len(),
            class_a_eligible,
            class_a_injections: injections
                .iter()
                .filter(|injection| injection.class_name == CLASS_A)
                .count(),
            class_b_injections: injections
                .iter()
                .filter(|injection| injection.class_name == CLASS_B)
                .count(),
        },
        files,
        injections,
        edits,
    };
    common::write_json_pretty(out_dir.join("manifest.json"), &manifest)?;
    println!(
        "seed-defects: source={} -> {}",
        manifest.source.label,
        common::relative_path(
            &env::current_dir().map_err(|error| error.to_string())?,
            out_dir
        )
    );
    println!(
        "scope-proof: files-scanned={} class-a-eligible={} class-a-injections={} class-b-injections={}",
        manifest.scope.files_copied,
        manifest.scope.class_a_eligible,
        manifest.scope.class_a_injections,
        manifest.scope.class_b_injections
    );
    Ok(manifest)
}

pub fn resolve_sources(
    repo_root: &Path,
    fixtures: Option<&Path>,
    matrix: bool,
    corpus_shard: bool,
    out_dir: &Path,
) -> Result<crate::davinci_fpfn::ResolvedSources, String> {
    let picked = usize::from(fixtures.is_some()) + usize::from(matrix) + usize::from(corpus_shard);
    if picked != 1 {
        return Err(format!(
            "exactly one of --fixtures/--matrix/--corpus-shard is required\n\n{}",
            crate::USAGE
        ));
    }
    if let Some(fixtures) = fixtures {
        return resolve_fixture_sources(repo_root, &absolute(fixtures));
    }
    if matrix {
        let matrix_dir = out_dir.join("matrix-src");
        common::run_capture_in(
            "rust-script",
            &[
                "tools/commands/davinci/matrix-gen.rs",
                "--write",
                "--out-dir",
                matrix_dir.to_string_lossy().as_ref(),
            ],
            repo_root,
        )?;
        return Ok(crate::davinci_fpfn::ResolvedSources {
            kind: "matrix".to_string(),
            label: "matrix-gen".to_string(),
            roots: vec![crate::davinci_fpfn::SourceRoot {
                root: matrix_dir,
                prefix: String::new(),
            }],
        });
    }
    resolve_corpus_sources(repo_root)
}

fn map_template_ref(
    seeded: &str,
    original_ref_start: usize,
    name: &str,
    edits: &[EditRecord],
) -> Result<usize, String> {
    let mut ref_start = original_ref_start as isize;
    for edit in edits {
        if edit.span[1] <= original_ref_start {
            ref_start += edit.delta;
        }
    }
    let ref_start = usize::try_from(ref_start).map_err(|_| "negative template ref".to_string())?;
    let found = seeded
        .get(ref_start..ref_start + name.len())
        .ok_or_else(|| {
            "seed-defects internal error: template ref relocation out of range".to_string()
        })?;
    if found != name {
        return Err(format!(
            "seed-defects internal error: template ref relocation failed ({found})"
        ));
    }
    Ok(ref_start)
}

pub fn absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}
