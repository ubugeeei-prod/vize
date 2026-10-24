#![expect(clippy::expect_used, reason = "tests assert by panicking")]
#![expect(clippy::panic, reason = "tests assert by panicking")]
use super::artifact::BatchIncrementalBudget;
use vize_canon::IncrementalCheckMetrics;
const BUDGET_SCALE_ENV: &str = "VIZE_TIER_L_BUDGET_SCALE";

pub(super) fn budget_scale() -> f64 {
    match std::env::var(BUDGET_SCALE_ENV) {
        Ok(raw) if raw.is_empty() => 1.0,
        Ok(raw) => {
            let scale = raw.parse::<f64>().unwrap_or_else(|_| {
                panic!(
                    "{BUDGET_SCALE_ENV} must be a finite number greater than 0 and at most 100, got {raw:?}"
                )
            });
            assert!(
                scale.is_finite() && scale > 0.0 && scale <= 100.0,
                "{BUDGET_SCALE_ENV} must be a finite number greater than 0 and at most 100, got {raw:?}"
            );
            scale
        }
        Err(std::env::VarError::NotPresent) => 1.0,
        Err(error) => panic!("{BUDGET_SCALE_ENV} must be valid UTF-8: {error}"),
    }
}

pub(super) fn assert_no_injected_diagnostics(result: &vize_canon::BatchTypeCheckResult) {
    assert!(
        result.diagnostics.iter().all(|diagnostic| {
            diagnostic.file.file_name().and_then(|name| name.to_str())
                != Some("__vize_batch_incremental_oracle__.vue")
        }),
        "clean source retained injected-file diagnostics"
    );
}

pub(super) fn assert_cold_metrics(
    metrics: IncrementalCheckMetrics,
    budget: &BatchIncrementalBudget,
) {
    assert_eq!((metrics.checks, metrics.session_starts), (1, 1));
    assert_eq!((metrics.session_reuses, metrics.session_refreshes), (0, 0));
    assert_eq!(metrics.session_to_cli_fallbacks, 0);
    assert!(metrics.last_session_started);
    assert!(!metrics.last_session_to_cli_fallback);
    assert_eq!(
        (
            metrics.last_changed_files,
            metrics.last_created_files,
            metrics.last_deleted_files
        ),
        (0, 0, 0)
    );
    assert_requested_budget(metrics, budget);
}

pub(super) fn assert_warm_metrics(
    metrics: IncrementalCheckMetrics,
    checks: usize,
    reuses: usize,
    budget: &BatchIncrementalBudget,
) {
    assert_eq!((metrics.checks, metrics.session_starts), (checks, 1));
    assert_eq!(
        (metrics.session_reuses, metrics.session_refreshes),
        (reuses, reuses)
    );
    assert_eq!(metrics.session_to_cli_fallbacks, 0);
    assert!(metrics.last_session_reused && metrics.last_session_refreshed);
    assert!(!metrics.last_session_to_cli_fallback);
    assert_eq!(metrics.last_changed_files, budget.max_changed_files);
    assert_eq!(
        (metrics.last_created_files, metrics.last_deleted_files),
        (0, 0)
    );
    assert_requested_budget(metrics, budget);
}

fn assert_requested_budget(metrics: IncrementalCheckMetrics, budget: &BatchIncrementalBudget) {
    assert!(metrics.last_requested_files > 0);
    assert_eq!(
        metrics.last_requested_files, budget.max_requested_files,
        "the pinned Tier-L corpus must request every registered source, including dependencies"
    );
    assert!(
        metrics.last_requested_files <= budget.max_requested_files,
        "requested {} files, budget is {}",
        metrics.last_requested_files,
        budget.max_requested_files
    );
}

pub(super) fn assert_within_budget(
    lane: &str,
    elapsed_ms: u128,
    budget_ms: u64,
    budget_scale: f64,
) {
    let scaled_budget_ms = (budget_ms as f64 * budget_scale).ceil() as u128;
    assert!(
        elapsed_ms < scaled_budget_ms,
        "{lane} took {elapsed_ms}ms, budget is {budget_ms}ms at scale {budget_scale} ({scaled_budget_ms}ms)"
    );
}
