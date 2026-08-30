//! Build-profile accounting for the selected Davinci plan.

use vize_davinci::pass::{BudgetObserver, Pipeline, run_pipeline};

/// Measure the selected build plan through the P2-3 budget observer.
///
/// This is intentionally plan-level accounting: today the real build stages
/// are still the legacy compile path, so the observer reports planned fusion
/// groups for the selected backend. Once builds route through the pass manager,
/// this remains the profile-counter adapter.
pub(crate) fn plan_budget(plan: &Pipeline) -> BudgetObserver {
    let mut budget = BudgetObserver::new();
    run_pipeline(plan, &mut budget, |_| Ok(())).expect("compile plans use no-op pass bodies");
    budget
}

#[cfg(test)]
mod tests {
    use super::plan_budget;
    use crate::commands::davinci_ice::compile_plan;
    use vize_davinci::pass::{Fusability, PassDesc, PassKind, Pipeline, Preserved};

    #[test]
    fn compile_plan_budget_matches_selected_backend_groups() {
        for (ssr, vapor, expected_mode) in [
            (false, false, "dom"),
            (true, false, "ssr"),
            (false, true, "vapor"),
        ] {
            let (plan, mode) = compile_plan(ssr, vapor);
            let budget = plan_budget(plan);

            assert_eq!(mode, expected_mode);
            assert_eq!(budget.pipelines, 1);
            assert_eq!(budget.walks as usize, plan.group_count());
            assert_eq!(budget.passes as usize, plan.passes.len());
            assert_eq!(budget.failures, 0);
        }
    }

    #[test]
    fn plan_budget_counts_a_fused_build_group_once() {
        const PASSES: &[PassDesc] = &[
            PassDesc::new(
                "facts",
                PassKind::Optional,
                Fusability::Fusable,
                Preserved::ALL,
            ),
            PassDesc::new(
                "emit",
                PassKind::Optional,
                Fusability::Fusable,
                Preserved::ALL,
            ),
        ];
        const PLAN: Pipeline = Pipeline::new("s2", PASSES);

        let budget = plan_budget(&PLAN);

        assert_eq!(budget.walks, 1);
        assert_eq!(budget.passes, 2);
        assert_eq!(budget.fusion_ratio_hundredths(), Some(200));
    }
}
