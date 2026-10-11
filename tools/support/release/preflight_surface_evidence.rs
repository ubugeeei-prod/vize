use serde_json::Value;

pub const REQUIRED_SURFACES: &[&str] = &[
    "waiver-audit",
    "typecheck-dependencies",
    "core-tools",
    "lsp",
    "lint-divergence",
    "syntax-highlighter",
    "glyph",
    "typecheck-divergence",
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
            "{artifact_name} surface verdict is not exact enforced release evidence"
        ));
    }
    let surfaces = verdict["surfaces"].as_array().ok_or_else(|| {
        format!("{artifact_name} surface verdict has no complete enforced surface set")
    })?;
    if surfaces.len() != REQUIRED_SURFACES.len()
        || surfaces
            .iter()
            .zip(REQUIRED_SURFACES)
            .any(|(surface, name)| {
                surface["name"].as_str() != Some(name)
                    || surface["budgetMode"].as_str() != Some("enforce")
                    || surface["rawOutcome"].as_str() != Some("success")
                    || surface["outcome"].as_str() != Some("success")
            })
    {
        return Err(format!(
            "{artifact_name} surface verdict has no complete enforced surface set"
        ));
    }
    Ok(())
}

pub fn assert_release_lint_budget(artifact_name: &str, summary: &Value) -> Result<(), String> {
    let count = summary["projectCount"].as_u64();
    let budget = &summary["budget"];
    if budget["status"].as_str() != Some("success")
        || budget["passed"].as_bool() != Some(true)
        || budget["projectCount"].as_u64() != count
        || budget["passedCount"].as_u64() != count
        || ["failedCount", "unusableCount", "breachedCount"]
            .iter()
            .any(|field| budget[field].as_u64() != Some(0))
        || !budget["failedProjects"]
            .as_array()
            .is_some_and(Vec::is_empty)
    {
        return Err(format!(
            "{artifact_name} lint divergence budget is not a complete enforced success"
        ));
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
            "surfaces": REQUIRED_SURFACES.iter().map(|name| json!({
                "name": name, "outcome": "success", "rawOutcome": "success", "budgetMode": "enforce"
            })).collect::<Vec<_>>()
        })
    }

    fn check(verdict: &Value) -> Result<(), String> {
        assert_release_surface_verdict("real-project-matrix-7", &"a".repeat(40), 7, verdict)
    }

    #[test]
    fn complete_original_enforced_report_passes() {
        assert!(check(&valid()).is_ok());
    }

    #[test]
    fn whole_lint_budget_cannot_hide_failed_unusable_or_breached_projects() {
        let valid = json!({ "projectCount": 1, "budget": {
            "status": "success", "passed": true, "projectCount": 1, "passedCount": 1,
            "failedCount": 0, "unusableCount": 0, "breachedCount": 0, "failedProjects": []
        }});
        assert!(assert_release_lint_budget("shard", &valid).is_ok());
        for (field, value) in [
            ("status", json!("failure")),
            ("passed", json!(false)),
            ("projectCount", json!(2)),
            ("passedCount", json!(0)),
            ("failedCount", json!(1)),
            ("unusableCount", json!(1)),
            ("breachedCount", json!(1)),
            ("failedProjects", json!(["fixture"])),
        ] {
            let mut invalid = valid.clone();
            invalid["budget"][field] = value;
            assert!(
                assert_release_lint_budget("shard", &invalid).is_err(),
                "{field}"
            );
        }
    }

    #[test]
    fn every_observational_or_hidden_failure_refuses_success_status() {
        for index in 0..REQUIRED_SURFACES.len() {
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
        for index in 0..REQUIRED_SURFACES.len() {
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
