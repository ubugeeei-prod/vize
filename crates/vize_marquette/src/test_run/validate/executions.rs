use vize_s0::cstr;

use crate::ContractDiagnostic;
use crate::validate::rules::contract_path;

use super::super::model::{
    TestRunEvidence, TestRunSuiteExecution, TestRunSuiteKind, TestRunSuiteOutcome,
};
use super::rules::{check_digest, check_identifier, check_retained_evidence, check_safe_integer};
use super::{TEST_RUN_MAX_SHARDS, TEST_RUN_MAX_SUITES, TEST_RUN_MAX_TARGETS};

pub(super) fn validate_targets(
    evidence: &TestRunEvidence,
    diagnostics: &mut Vec<ContractDiagnostic>,
) {
    if evidence.targets.is_empty() || evidence.targets.len() > TEST_RUN_MAX_TARGETS {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_131",
            "targets",
            "record must include between 1 and 32 target executions",
        ));
    }

    let mut seen = std::collections::BTreeSet::new();
    for target in &evidence.targets {
        let path = contract_path("targets", &target.id);
        check_identifier(&target.id, &path, diagnostics);
        check_identifier(
            &target.environment,
            &contract_path(&path, "environment"),
            diagnostics,
        );
        if !seen.insert(&target.id) {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_117",
                path.clone(),
                "target execution is recorded more than once",
            ));
        }
        if !evidence.selection.target_ids.contains(&target.id) {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_118",
                path,
                "target execution was not selected for this candidate",
            ));
        }
    }
    for selected in &evidence.selection.target_ids {
        if !evidence.targets.iter().any(|target| target.id == *selected) {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_119",
                contract_path("selection.targetIds", selected),
                "selected target has no recorded execution",
            ));
        }
    }
}

pub(super) fn validate_suites(
    evidence: &TestRunEvidence,
    diagnostics: &mut Vec<ContractDiagnostic>,
) {
    if evidence.suites.is_empty() || evidence.suites.len() > TEST_RUN_MAX_SUITES {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_131",
            "suites",
            "record must include between 1 and 512 suite executions",
        ));
    }

    let mut shards = std::collections::BTreeMap::new();
    let mut checked_ids = std::collections::BTreeSet::new();
    for suite in &evidence.suites {
        let path = contract_path("suites", &suite.id);
        if !seen_shard(&mut shards, suite) {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_120",
                path.clone(),
                "suite shard is recorded more than once",
            ));
        }
        // Suite-id-scoped invariants are shard-independent, so evaluate them
        // once per unique suite id. Running them per shard would emit the same
        // code/path/message diagnostic once for every shard of the suite.
        if checked_ids.insert(suite.id.as_str()) {
            check_identifier(&suite.id, &path, diagnostics);
            if !evidence.selection.suite_ids.contains(&suite.id) {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_MARQUETTE_121",
                    path.clone(),
                    "suite execution was not selected for this candidate",
                ));
            }
            if !evidence
                .targets
                .iter()
                .any(|target| target.id == suite.target_id)
            {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_MARQUETTE_123",
                    contract_path(&path, "targetId"),
                    "suite target has no recorded target execution",
                ));
            }
        }
        if suite.shard_count == 0
            || suite.shard_count > TEST_RUN_MAX_SHARDS
            || suite.shard_index == 0
            || suite.shard_index > suite.shard_count
        {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_124",
                contract_path(&path, "shardIndex"),
                "shard index must fall within a shard count between 1 and 1024",
            ));
        }
        check_safe_integer(
            suite.duration_ms,
            &contract_path(&path, "durationMs"),
            diagnostics,
        );
        check_digest(
            &suite.invocation_fingerprint,
            &contract_path(&path, "invocationFingerprint"),
            diagnostics,
        );
        check_retained_evidence(&suite.report, &contract_path(&path, "report"), diagnostics);
        check_retained_evidence(&suite.log, &contract_path(&path, "log"), diagnostics);

        let executed = u64::from(suite.passed) + u64::from(suite.failed) + u64::from(suite.skipped);
        let consistent = match suite.outcome {
            TestRunSuiteOutcome::Passed => suite.failed == 0 && executed > 0,
            TestRunSuiteOutcome::Failed => suite.failed > 0,
            TestRunSuiteOutcome::Cancelled => true,
        };
        if !consistent {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_130",
                contract_path(&path, "outcome"),
                "suite outcome does not match its recorded counts",
            ));
        }
    }

    for selected in &evidence.selection.suite_ids {
        if !evidence.suites.iter().any(|suite| suite.id == *selected) {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_122",
                contract_path("selection.suiteIds", selected),
                "selected suite has no recorded execution",
            ));
        }
    }

    for (id, group) in &shards {
        let path = contract_path("suites", id);
        let expected = group.shard_count;
        if group.kinds_disagree || group.targets_disagree || group.counts_disagree {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_126",
                path.clone(),
                "every shard of one suite must share its kind, target, and shard count",
            ));
        } else if expected != 0
            && expected <= TEST_RUN_MAX_SHARDS
            && group.indexes.len() != expected as usize
        {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_125",
                path,
                cstr!("suite must record every shard from 1 to {expected}"),
            ));
        }
    }
}

struct ShardGroup<'a> {
    shard_count: u32,
    kind: TestRunSuiteKind,
    target_id: &'a str,
    indexes: std::collections::BTreeSet<u32>,
    kinds_disagree: bool,
    targets_disagree: bool,
    counts_disagree: bool,
}

fn seen_shard<'a>(
    shards: &mut std::collections::BTreeMap<&'a str, ShardGroup<'a>>,
    suite: &'a TestRunSuiteExecution,
) -> bool {
    let group = shards
        .entry(suite.id.as_str())
        .or_insert_with(|| ShardGroup {
            shard_count: suite.shard_count,
            kind: suite.kind,
            target_id: suite.target_id.as_str(),
            indexes: std::collections::BTreeSet::new(),
            kinds_disagree: false,
            targets_disagree: false,
            counts_disagree: false,
        });
    group.kinds_disagree |= group.kind != suite.kind;
    group.targets_disagree |= group.target_id != suite.target_id.as_str();
    group.counts_disagree |= group.shard_count != suite.shard_count;
    group.indexes.insert(suite.shard_index)
}
