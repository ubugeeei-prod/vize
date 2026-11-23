//! The complexity rule over an existing parsed and analyzed template.
use super::{
    COGNITIVE_WARN_ABOVE, CYCLOMATIC_WARN_ABOVE, Contribution, MAX_LABELS, META,
    MaxTemplateComplexity, label,
};
use crate::LintDiagnostic;
use vize_l1_to_l2::pass::template::OriginalTemplate;
mod window;

impl MaxTemplateComplexity {
    /// Compute this rule's diagnostic from a shared template analysis.
    /// All ranges address the original SFC. The caller owns suppression
    /// and severity policy when publishing the diagnostic.
    #[must_use]
    pub fn original_diagnostic(template: &OriginalTemplate<'_>) -> Option<LintDiagnostic> {
        let complexity = &template.analysis().complexity;
        if !complexity.exceeds_default_thresholds() {
            return None;
        }
        let opening = template.open_tag();
        let mut diagnostic = LintDiagnostic::warn(
            META.name,
            vize_l0::cstr!(
                "Template complexity is too high: cyclomatic {} (limit {CYCLOMATIC_WARN_ABOVE}), cognitive {} (limit {COGNITIVE_WARN_ABOVE})",
                complexity.cyclomatic, complexity.cognitive,
            ),
            opening.start,
            opening.end,
        )
        .with_help(
            "Move the deepest branches or loops into child components: a child's complexity \
             never counts toward this component's own score",
        );
        for row in ranked_labels(&complexity.contributions) {
            diagnostic = diagnostic.with_label(label(row), row.span.start, row.span.end);
        }
        Some(diagnostic)
    }
}

fn ranked_labels(rows: &[Contribution]) -> impl Iterator<Item = &Contribution> {
    let mut best: [Option<&Contribution>; MAX_LABELS] = [None; MAX_LABELS];
    for row in rows
        .iter()
        .filter(|r| r.cognitive != 0 || r.cyclomatic != 0)
    {
        window::insert(&mut best, row, |row, old| {
            row.cognitive
                .cmp(&old.cognitive)
                .then(row.cyclomatic.cmp(&old.cyclomatic))
                .then_with(|| old.span.start.cmp(&row.span.start))
                .is_gt()
        });
    }
    best.sort_by_key(|slot| slot.map(Contribution::sort_key));
    best.into_iter().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::facts::max_template_complexity::top_contributors;
    use vize_l0::{Span, id::NodeId};
    use vize_l1_to_l2::pass::cfg::DecisionKind;

    #[test]
    fn filtering_ties_and_displacement_keep_the_same_label_identities() {
        // Seven rows cross the five-label limit. Each input independently
        // chooses an ignored row, a tied score, another source position or a
        // stronger score. Equal source keys deliberately keep distinct ids.
        for mut choices in 0u32..4u32.pow(7) {
            let rows: [Contribution; 7] = core::array::from_fn(|index| {
                let variant = choices % 4;
                choices /= 4;
                let (cognitive, cyclomatic, start) =
                    [(0, 0, 1), (1, 1, 5), (1, 1, 2), (2, 0, 5)][variant as usize];
                Contribution {
                    span: Span::new(start, start + (index % 2) as u32),
                    kind: if index % 2 == 0 {
                        DecisionKind::If
                    } else {
                        DecisionKind::Logical
                    },
                    op: NodeId::from_index(index as u32),
                    nesting: index as u32,
                    cognitive,
                    cyclomatic,
                }
            });
            let actual: Vec<_> = ranked_labels(&rows).collect();
            let expected: Vec<_> = top_contributors(&rows).collect();
            assert_eq!(actual, expected);
        }
    }
}
