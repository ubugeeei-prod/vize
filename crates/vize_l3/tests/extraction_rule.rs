//! The pinned multi-metric commit rule, judged directly.

use vize_impeto::extract::{
    Metric, Metrics, OptTier, OptimizationBudget, Rejection, TiePolicy, epsilon_pct, judge,
};

const BASE: Metrics = Metrics {
    emitted_size: 200,
    reactive_edges: 10,
    update_path: 20,
};

fn metrics(emitted_size: u64, reactive_edges: u64, update_path: u64) -> Metrics {
    Metrics {
        emitted_size,
        reactive_edges,
        update_path,
    }
}

fn loosened(size: u32, edges: u32, path: u32, required: u32) -> OptimizationBudget {
    OptimizationBudget {
        tier: OptTier::O3,
        candidate_budget: 128,
        emitted_size_epsilon_pct: size,
        reactive_edge_epsilon_pct: edges,
        update_path_epsilon_pct: path,
        required_improvements_min: required,
        tie_policy: TiePolicy::Reject,
    }
}

#[test]
fn pinned_tiers_commit_only_pareto_improvements() {
    for tier in [OptTier::O1, OptTier::O2, OptTier::O3] {
        let budget = tier.budget();
        assert_eq!(judge(BASE, metrics(199, 10, 20), &budget), Ok(()));
        assert_eq!(judge(BASE, metrics(200, 9, 20), &budget), Ok(()));
        assert_eq!(judge(BASE, metrics(200, 10, 19), &budget), Ok(()));
        assert_eq!(judge(BASE, BASE, &budget), Err(Rejection::NoImprovement));
        assert_eq!(
            judge(BASE, metrics(201, 9, 19), &budget),
            Err(Rejection::Regressed(Metric::EmittedSize))
        );
        assert_eq!(
            judge(BASE, metrics(150, 11, 19), &budget),
            Err(Rejection::Regressed(Metric::ReactiveEdge))
        );
        assert_eq!(
            judge(BASE, metrics(150, 9, 21), &budget),
            Err(Rejection::Regressed(Metric::UpdatePath))
        );
    }
}

#[test]
fn constraints_are_judged_before_the_objective() {
    let budget = OptTier::O2.budget();
    assert_eq!(
        judge(BASE, metrics(300, 11, 30), &budget),
        Err(Rejection::Regressed(Metric::ReactiveEdge))
    );
    assert_eq!(
        judge(BASE, metrics(300, 10, 30), &budget),
        Err(Rejection::Regressed(Metric::UpdatePath))
    );
}

#[test]
fn o0_still_rejects_ties() {
    let budget = OptTier::O0.budget();
    assert_eq!(judge(BASE, BASE, &budget), Err(Rejection::NoImprovement));
    assert_eq!(judge(BASE, metrics(199, 10, 20), &budget), Ok(()));
}

#[test]
fn an_epsilon_admits_a_regression_up_to_its_floor_percentage() {
    let budget = loosened(10, 0, 0, 1);
    assert_eq!(judge(BASE, metrics(220, 9, 20), &budget), Ok(()));
    assert_eq!(
        judge(BASE, metrics(221, 9, 20), &budget),
        Err(Rejection::Regressed(Metric::EmittedSize))
    );
    let budget = loosened(0, 15, 0, 1);
    assert_eq!(judge(BASE, metrics(199, 11, 20), &budget), Ok(()));
    assert_eq!(
        judge(BASE, metrics(199, 12, 20), &budget),
        Err(Rejection::Regressed(Metric::ReactiveEdge))
    );
}

#[test]
fn required_improvements_count_strict_gains_only() {
    let budget = loosened(0, 0, 0, 2);
    assert_eq!(
        judge(BASE, metrics(199, 10, 20), &budget),
        Err(Rejection::NoImprovement)
    );
    assert_eq!(judge(BASE, metrics(199, 9, 20), &budget), Ok(()));
}

#[test]
fn epsilons_are_read_per_metric() {
    let budget = loosened(1, 2, 3, 1);
    assert_eq!(
        Metric::CHECK_ORDER.map(|metric| (metric, epsilon_pct(&budget, metric))),
        [
            (Metric::ReactiveEdge, 2),
            (Metric::UpdatePath, 3),
            (Metric::EmittedSize, 1),
        ]
    );
}
