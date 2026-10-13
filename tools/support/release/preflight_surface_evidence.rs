use serde_json::Value;

/// How release preflight treats one Real Project Matrix surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReleaseSurfacePolicy {
    /// The surface must run in `enforce` mode and succeed without normalization.
    Enforced,
    /// The surface runs in `record-only` mode. Its raw outcome is retained as
    /// evidence, and a raw failure does not refuse the release.
    Recorded,
    /// The surface is dispatched as `skip` and must report that it did not run.
    Skipped,
}

/// Every original surface in report order with its release policy. The three
/// parity surfaces do not gate a release until per-project budgets gate them
/// again: lint and LSP record their raw outcomes, and typecheck divergence is
/// skipped because one fixture exhausts its 100-minute step timeout.
pub const RELEASE_SURFACES: &[(&str, ReleaseSurfacePolicy)] = &[
    ("waiver-audit", ReleaseSurfacePolicy::Enforced),
    ("typecheck-dependencies", ReleaseSurfacePolicy::Enforced),
    ("core-tools", ReleaseSurfacePolicy::Enforced),
    ("lsp", ReleaseSurfacePolicy::Recorded),
    ("lint-divergence", ReleaseSurfacePolicy::Recorded),
    ("syntax-highlighter", ReleaseSurfacePolicy::Enforced),
    ("glyph", ReleaseSurfacePolicy::Enforced),
    ("typecheck-divergence", ReleaseSurfacePolicy::Skipped),
];

pub fn assert_release_surface_verdict(
    artifact_name: &str,
    head_sha: &str,
    shard: usize,
    verdict: &Value,
) -> Result<(), String> {
    if verdict["status"].as_str() != Some("success") {
        return Err(format!(
            "{artifact_name} surface verdict is {}",
            verdict["status"]
        ));
    }
    if verdict["schema"].as_str() != Some("vize.realProjectSurfaceVerdict")
        || verdict["version"].as_u64() != Some(2)
        || verdict["sourceCommit"].as_str() != Some(head_sha)
        || verdict["shardIndex"].as_str() != Some(shard.to_string().as_str())
        || !verdict["failedSurfaceNames"]
            .as_array()
            .is_some_and(Vec::is_empty)
    {
        return Err(format!(
            "{artifact_name} surface verdict is not exact release-gated evidence"
        ));
    }
    let surfaces = verdict["surfaces"].as_array().ok_or_else(|| {
        format!("{artifact_name} surface verdict has no complete release-gated surface set")
    })?;
    if surfaces.len() != RELEASE_SURFACES.len()
        || surfaces
            .iter()
            .zip(RELEASE_SURFACES)
            .any(|(surface, (name, policy))| {
                surface["name"].as_str() != Some(name)
                    || surface["outcome"].as_str() != Some("success")
                    || !surface_matches_policy(surface, *policy)
            })
    {
        return Err(format!(
            "{artifact_name} surface verdict has no complete release-gated surface set"
        ));
    }
    Ok(())
}

fn surface_matches_policy(surface: &Value, policy: ReleaseSurfacePolicy) -> bool {
    let mode = surface["budgetMode"].as_str();
    let raw_outcome = surface["rawOutcome"].as_str();
    match policy {
        ReleaseSurfacePolicy::Enforced => mode == Some("enforce") && raw_outcome == Some("success"),
        ReleaseSurfacePolicy::Recorded => {
            mode == Some("record-only") && matches!(raw_outcome, Some("success" | "failure"))
        }
        ReleaseSurfacePolicy::Skipped => mode == Some("skip") && raw_outcome == Some("skipped"),
    }
}

/// Lint divergence is a recorded surface: breached and unusable projects stay
/// visible in the release log, but the retained budget must account for every
/// measured project so a truncated summary cannot pass as evidence.
pub fn assert_release_lint_budget(artifact_name: &str, summary: &Value) -> Result<(), String> {
    let budget = &summary["budget"];
    let count = |name: &str| budget[name].as_u64();
    let failed_projects = budget["failedProjects"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let passed = budget["passed"].as_bool();
    let accounted = match (
        summary["projectCount"].as_u64(),
        count("passedCount"),
        count("failedCount"),
        count("unusableCount"),
        count("breachedCount"),
        passed,
    ) {
        (
            Some(project_count),
            Some(passed_count),
            Some(failed_count),
            Some(unusable_count),
            Some(breached_count),
            Some(passed),
        ) => {
            count("projectCount") == Some(project_count)
                && passed_count.checked_add(failed_count) == Some(project_count)
                && unusable_count.checked_add(breached_count) == Some(failed_count)
                && budget["failedProjects"].is_array()
                && failed_projects.len() as u64 == failed_count
                && failed_projects.iter().all(Value::is_string)
                && passed == (failed_count == 0)
                && budget["status"].as_str() == Some(if passed { "success" } else { "failure" })
        }
        _ => false,
    };
    if !accounted {
        return Err(format!(
            "{artifact_name} lint divergence budget does not account for every measured project"
        ));
    }
    if passed != Some(true) {
        println!(
            "::warning title=Release lint parity recorded::{artifact_name} retains {} breached and {} unusable lint divergence project(s): {}",
            budget["breachedCount"],
            budget["unusableCount"],
            failed_projects
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid() -> Value {
        json!({
            "schema": "vize.realProjectSurfaceVerdict", "version": 2,
            "sourceCommit": "a".repeat(40), "shardIndex": "7", "status": "success",
            "failedSurfaceNames": [],
            "surfaces": RELEASE_SURFACES.iter().map(|(name, policy)| {
                let (mode, raw_outcome) = match policy {
                    ReleaseSurfacePolicy::Enforced => ("enforce", "success"),
                    ReleaseSurfacePolicy::Recorded => ("record-only", "success"),
                    ReleaseSurfacePolicy::Skipped => ("skip", "skipped"),
                };
                json!({
                    "name": name, "outcome": "success", "rawOutcome": raw_outcome,
                    "budgetMode": mode
                })
            }).collect::<Vec<_>>()
        })
    }

    fn check(verdict: &Value) -> Result<(), String> {
        assert_release_surface_verdict("real-project-matrix-7", &"a".repeat(40), 7, verdict)
    }

    fn indices(policy: ReleaseSurfacePolicy) -> Vec<usize> {
        RELEASE_SURFACES
            .iter()
            .enumerate()
            .filter(|(_, (_, candidate))| *candidate == policy)
            .map(|(index, _)| index)
            .collect()
    }

    #[test]
    fn complete_release_gated_report_passes() {
        assert!(check(&valid()).is_ok());
        assert_eq!(indices(ReleaseSurfacePolicy::Enforced).len(), 5);
        assert_eq!(
            indices(ReleaseSurfacePolicy::Recorded)
                .into_iter()
                .map(|index| RELEASE_SURFACES[index].0)
                .collect::<Vec<_>>(),
            ["lsp", "lint-divergence"]
        );
        assert_eq!(
            indices(ReleaseSurfacePolicy::Skipped)
                .into_iter()
                .map(|index| RELEASE_SURFACES[index].0)
                .collect::<Vec<_>>(),
            ["typecheck-divergence"]
        );
    }

    #[test]
    fn skipped_surface_refuses_anything_but_an_explicit_skip() {
        for index in indices(ReleaseSurfacePolicy::Skipped) {
            for (field, value) in [
                ("budgetMode", "enforce"),
                ("budgetMode", "record-only"),
                ("rawOutcome", "success"),
                ("rawOutcome", "failure"),
                ("rawOutcome", "cancelled"),
                ("outcome", "skipped"),
                ("outcome", "failure"),
            ] {
                let mut verdict = valid();
                verdict["surfaces"][index][field] = json!(value);
                assert!(check(&verdict).is_err(), "{index}: {field}={value}");
            }
        }
    }

    #[test]
    fn recorded_parity_surfaces_retain_a_raw_failure_without_refusing() {
        for index in indices(ReleaseSurfacePolicy::Recorded) {
            let mut verdict = valid();
            verdict["surfaces"][index]["rawOutcome"] = json!("failure");
            assert!(check(&verdict).is_ok(), "{index}");
        }
    }

    #[test]
    fn recorded_parity_surfaces_refuse_unrecorded_or_foreign_evidence() {
        for index in indices(ReleaseSurfacePolicy::Recorded) {
            for (field, value) in [
                ("budgetMode", "enforce"),
                ("budgetMode", "skip"),
                ("rawOutcome", "cancelled"),
                ("rawOutcome", "skipped"),
                ("outcome", "failure"),
            ] {
                let mut verdict = valid();
                verdict["surfaces"][index][field] = json!(value);
                assert!(check(&verdict).is_err(), "{index}: {field}={value}");
            }
        }
    }

    #[test]
    fn enforced_surfaces_refuse_every_observational_or_hidden_failure() {
        for index in indices(ReleaseSurfacePolicy::Enforced) {
            for (field, value) in [
                ("budgetMode", "record-only"),
                ("budgetMode", "skip"),
                ("rawOutcome", "failure"),
                ("rawOutcome", "cancelled"),
                ("rawOutcome", "skipped"),
                ("outcome", "failure"),
            ] {
                let mut verdict = valid();
                verdict["surfaces"][index][field] = json!(value);
                assert!(check(&verdict).is_err(), "{index}: {field}={value}");
            }
        }
    }

    #[test]
    fn lint_budget_accepts_recorded_breaches_but_refuses_unaccounted_projects() {
        let passed = json!({ "projectCount": 2, "budget": {
            "status": "success", "passed": true, "projectCount": 2, "passedCount": 2,
            "failedCount": 0, "unusableCount": 0, "breachedCount": 0, "failedProjects": []
        }});
        assert!(assert_release_lint_budget("shard", &passed).is_ok());
        let breached = json!({ "projectCount": 2, "budget": {
            "status": "failure", "passed": false, "projectCount": 2, "passedCount": 0,
            "failedCount": 2, "unusableCount": 1, "breachedCount": 1,
            "failedProjects": ["fixture", "other"]
        }});
        assert!(assert_release_lint_budget("shard", &breached).is_ok());
        for (field, value) in [
            ("status", json!("success")),
            ("passed", json!(true)),
            ("projectCount", json!(3)),
            ("passedCount", json!(1)),
            ("failedCount", json!(1)),
            ("unusableCount", json!(0)),
            ("breachedCount", json!(2)),
            ("failedProjects", json!(["fixture"])),
            ("failedProjects", json!(["fixture", 1])),
            ("failedProjects", Value::Null),
        ] {
            let mut invalid = breached.clone();
            invalid["budget"][field] = value;
            assert!(
                assert_release_lint_budget("shard", &invalid).is_err(),
                "{field}"
            );
        }
        let mut missing = breached.clone();
        missing.as_object_mut().unwrap().remove("budget");
        assert!(assert_release_lint_budget("shard", &missing).is_err());
    }

    #[test]
    fn stale_foreign_missing_or_reordered_authority_refuses() {
        for (field, value) in [
            ("version", json!(1)),
            ("schema", json!("foreign")),
            ("sourceCommit", json!("b".repeat(40))),
            ("shardIndex", json!("8")),
            ("failedSurfaceNames", json!(["core-tools"])),
        ] {
            let mut verdict = valid();
            verdict[field] = value;
            assert!(check(&verdict).is_err(), "{field}");
        }
        for index in 0..RELEASE_SURFACES.len() {
            let mut missing = valid();
            missing["surfaces"].as_array_mut().unwrap().remove(index);
            assert!(check(&missing).is_err());
            let mut duplicate = valid();
            duplicate["surfaces"][index]["name"] = json!("unknown");
            assert!(check(&duplicate).is_err());
            let mut reversed = valid();
            reversed["surfaces"].as_array_mut().unwrap().reverse();
            assert!(check(&reversed).is_err());
        }
    }
}
