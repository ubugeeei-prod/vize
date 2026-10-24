//! Pilot assertion for independently seeded defect classes.

use super::{
    BaselineShiftReport, CLASS_A, CLASS_A_RULE, CLASS_B, CLASS_B_RULE, ClassAMiss, ClassAReport,
    ClassBReport, DiagnosticRow, LintCounts, SeedAssertReport, SeedManifest, VizeCli, apply_seed,
    common, count_by_key, describe_mapped_span, diagnostic_key, flatten_lint_json,
    line_col_to_index, line_starts_of, load_lint_rows, multiset_difference, plan_class_b,
    sort_diagnostics, span_overlaps_edits,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    process::Command,
};

pub fn assert_seeded_tree(
    manifest: &SeedManifest,
    out_dir: &Path,
    cli: Option<&VizeCli>,
    baseline_hook: Option<&Path>,
    seeded_hook: Option<&Path>,
) -> Result<SeedAssertReport, String> {
    let files = manifest
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let baseline_rows = load_lint_rows(baseline_hook, cli, &out_dir.join("original"), &files)?;
    let seeded_rows = load_lint_rows(seeded_hook, cli, &out_dir.join("seeded"), &files)?;
    // Opt-in class (b) runs on a copy with only its own mutation. A class-(a)
    // rename can itself make a declaration unread, and is not a class-(b) defect.
    let baseline_rows: Vec<_> = baseline_rows
        .into_iter()
        .filter(|row| row.rule_id != CLASS_B_RULE)
        .collect();
    let seeded_rows: Vec<_> = seeded_rows
        .into_iter()
        .filter(|row| row.rule_id != CLASS_B_RULE)
        .collect();
    let (class_b, b_baseline, b_seeded, b_shifted, b_misses, b_unmappable, b_unexpected) =
        assert_unused(manifest, out_dir, cli, baseline_hook, seeded_hook)?;
    let (mut shifted, mut unmappable) = shift_baseline(&baseline_rows, manifest, out_dir)?;
    let class_a_rows = expected_class_a_rows(manifest);
    let expected_rows = sort_diagnostics(
        shifted
            .iter()
            .cloned()
            .chain(class_a_rows.iter().map(|(row, _)| row.clone()))
            .collect(),
    );
    let expected_counts = count_by_key(&expected_rows);
    let actual_counts = count_by_key(&seeded_rows);
    let missing_rows = multiset_difference(&expected_counts, &actual_counts);
    let mut unexpected = multiset_difference(&actual_counts, &expected_counts);
    let class_a_keys = class_a_rows
        .iter()
        .map(|(row, identifier)| (diagnostic_key(row), identifier.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut class_a_misses = Vec::new();
    let mut baseline_misses = Vec::new();
    for row in missing_rows {
        if let Some(identifier) = class_a_keys.get(&diagnostic_key(&row)) {
            class_a_misses.push(ClassAMiss::from((&row, identifier.clone())));
        } else {
            baseline_misses.push(row);
        }
    }
    baseline_misses.extend(b_misses);
    shifted.extend(b_shifted);
    unmappable.extend(b_unmappable);
    unexpected.extend(b_unexpected);
    let pass = class_a_misses.is_empty()
        && class_b.misses.is_empty()
        && baseline_misses.is_empty()
        && unmappable.is_empty()
        && unexpected.is_empty();
    Ok(SeedAssertReport {
        schema_version: 1,
        tool: "tools/commands/davinci/seed-defects.rs --assert".to_string(),
        source: manifest.source.clone(),
        scope: manifest.scope.clone(),
        lint: LintCounts {
            baseline_diagnostics: baseline_rows.len() + b_baseline,
            seeded_diagnostics: seeded_rows.len() + b_seeded,
        },
        class_a: ClassAReport {
            expected: class_a_rows.len(),
            detected: class_a_rows.len() - class_a_misses.len(),
            misses: class_a_misses,
        },
        class_b,
        baseline_shift: BaselineShiftReport {
            mapped: shifted.len(),
            misses: baseline_misses,
            unmappable,
        },
        unexpected,
        verdict: if pass { "pass" } else { "fail" }.to_string(),
    })
}

fn shift_baseline(
    rows: &[DiagnosticRow],
    manifest: &SeedManifest,
    out_dir: &Path,
) -> Result<(Vec<DiagnosticRow>, Vec<DiagnosticRow>), String> {
    let mut shifted = Vec::new();
    let mut unmappable = Vec::new();
    for row in rows {
        let edits = manifest.edits.get(&row.path).cloned().unwrap_or_default();
        if edits.is_empty() {
            shifted.push(row.clone());
            continue;
        }
        let original_text = common::read_text(out_dir.join("original").join(&row.path))?;
        let original_starts = line_starts_of(&original_text);
        let start = line_col_to_index(&original_text, &original_starts, row.line, row.column);
        let end = line_col_to_index(
            &original_text,
            &original_starts,
            row.end_line,
            row.end_column,
        );
        if start.is_none()
            || end.is_none()
            || span_overlaps_edits(start.unwrap(), end.unwrap(), &edits)
        {
            unmappable.push(row.clone());
            continue;
        }
        let seeded_text = common::read_text(out_dir.join("seeded").join(&row.path))?;
        if let Some(described) =
            describe_mapped_span(&seeded_text, &edits, start.unwrap(), end.unwrap())
        {
            shifted.push(DiagnosticRow {
                path: row.path.clone(),
                rule_id: row.rule_id.clone(),
                severity: row.severity,
                line: described.line,
                column: described.column,
                end_line: described.end_line,
                end_column: described.end_column,
            });
        } else {
            unmappable.push(row.clone());
        }
    }
    Ok((sort_diagnostics(shifted), sort_diagnostics(unmappable)))
}

fn expected_class_a_rows(manifest: &SeedManifest) -> Vec<(DiagnosticRow, String)> {
    manifest
        .injections
        .iter()
        .filter(|injection| injection.class_name == CLASS_A)
        .map(|injection| {
            (
                DiagnosticRow {
                    path: injection.path.clone(),
                    rule_id: injection.expected_rule.clone().unwrap_or_default(),
                    severity: 1,
                    line: injection.expected.line,
                    column: injection.expected.column,
                    end_line: injection.expected.end_line,
                    end_column: injection.expected.end_column,
                },
                injection.identifier.original.clone().unwrap_or_default(),
            )
        })
        .collect()
}

// Separate mutation plane and full identity multiset for the opt-in consumer.
#[allow(clippy::type_complexity)]
fn assert_unused(
    manifest: &SeedManifest,
    out: &Path,
    cli: Option<&VizeCli>,
    baseline_hook: Option<&Path>,
    seeded_hook: Option<&Path>,
) -> Result<
    (
        ClassBReport,
        usize,
        usize,
        Vec<DiagnosticRow>,
        Vec<DiagnosticRow>,
        Vec<DiagnosticRow>,
        Vec<DiagnosticRow>,
    ),
    String,
> {
    let plane = out.join("class-b");
    let mut b_manifest = manifest.clone();
    b_manifest.edits.clear();
    for file in &manifest.files {
        let original = common::read_text(out.join("original").join(&file.path))?;
        let (plan, _) = plan_class_b(&original);
        let applied = apply_seed(&original, None, plan.as_ref());
        b_manifest.edits.insert(file.path.clone(), applied.edits);
        common::write_text(plane.join("original").join(&file.path), &original)?;
        common::write_text(plane.join("seeded").join(&file.path), &applied.seeded)?;
    }
    let files: Vec<_> = manifest
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect();
    let load = |hook: Option<&Path>, tree: &str| -> Result<Vec<DiagnosticRow>, String> {
        let rows = if let Some(hook) = hook {
            load_lint_rows(Some(hook), cli, &plane.join(tree), &files)?
                .into_iter()
                .filter(|row| row.rule_id != CLASS_A_RULE)
                .collect()
        } else {
            let cwd = plane.join(tree);
            let config = cwd.join("vite.config.mjs");
            common::write_text(
                &config,
                &format!(
                    "export default {};\n",
                    serde_json::json!({"vue": {"linter": {"rules": {CLASS_B_RULE: "warn"}}}})
                ),
            )?;
            let cli = cli.ok_or("vize cli is required")?;
            let output = Command::new(&cli.command)
                .args(&cli.prefix)
                .args(["lint", "--config"])
                .arg(&config)
                .args(["--format", "json"])
                .args(&files)
                .current_dir(&cwd)
                .output()
                .map_err(|error| format!("unused lint: {error}"))?;
            if !matches!(output.status.code(), Some(0 | 1)) {
                return Err(format!(
                    "unused lint failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            flatten_lint_json(
                &serde_json::from_slice(&output.stdout)
                    .map_err(|error| format!("unused lint JSON: {error}"))?,
            )?
        };
        Ok(rows)
    };
    let baseline = load(baseline_hook, "original")?;
    let seeded = load(seeded_hook, "seeded")?;
    let (shifted, unmappable) = shift_baseline(&baseline, &b_manifest, &plane)?;
    let expected: Vec<_> = manifest
        .injections
        .iter()
        .filter(|injection| injection.class_name == CLASS_B)
        .map(|injection| DiagnosticRow {
            path: injection.path.clone(),
            rule_id: CLASS_B_RULE.to_string(),
            severity: 1,
            line: injection.expected.line,
            column: injection.expected.column,
            end_line: injection.expected.end_line,
            end_column: injection.expected.end_column,
        })
        .collect();
    let seed_keys: BTreeSet<_> = expected.iter().map(diagnostic_key).collect();
    let expected_counts = count_by_key(
        &shifted
            .iter()
            .cloned()
            .chain(expected.iter().cloned())
            .collect::<Vec<_>>(),
    );
    let actual_counts = count_by_key(&seeded);
    let mut seed_misses = Vec::new();
    let mut baseline_misses = Vec::new();
    for row in multiset_difference(&expected_counts, &actual_counts) {
        if seed_keys.contains(&diagnostic_key(&row)) {
            seed_misses.push(row);
        } else {
            baseline_misses.push(row);
        }
    }
    let unexpected = multiset_difference(&actual_counts, &expected_counts);
    let report = ClassBReport {
        expected: expected.len(), detected: expected.len() - seed_misses.len(), misses: seed_misses,
        note: "gated by exact rule, severity and span on the independent class-b mutation plane (opt-in consumer)".into(),
    };
    Ok((
        report,
        baseline.len(),
        seeded.len(),
        shifted,
        baseline_misses,
        unmappable,
        unexpected,
    ))
}
