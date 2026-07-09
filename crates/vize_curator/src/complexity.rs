use serde::Serialize;

/// Raw cross-file signals used to score Vue component complexity.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ComplexityInput {
    pub template_if_count: usize,
    pub template_for_count: usize,
    pub slot_count: usize,
    pub prop_drilling_edge_count: usize,
    pub global_state_reference_count: usize,
    pub provide_inject_max_depth: usize,
    pub provide_inject_reference_count: usize,
    pub fallthrough_risk_count: usize,
    pub reactive_node_count: usize,
    pub reactive_edge_count: usize,
    pub reactive_cycle_count: usize,
}

/// Per-dimension complexity score.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ComplexityDimensionScores {
    pub template_control_flow: u32,
    pub slot_usage: u32,
    pub prop_drilling: u32,
    pub global_state: u32,
    pub provide_inject: u32,
    pub fallthrough_attrs: u32,
    pub reactive_graph: u32,
}

impl ComplexityDimensionScores {
    pub fn total(self) -> u32 {
        self.template_control_flow
            .saturating_add(self.slot_usage)
            .saturating_add(self.prop_drilling)
            .saturating_add(self.global_state)
            .saturating_add(self.provide_inject)
            .saturating_add(self.fallthrough_attrs)
            .saturating_add(self.reactive_graph)
    }
}

/// Human-facing band for the total complexity score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ComplexityBand {
    Low,
    Moderate,
    High,
    Extreme,
}

/// Explainable complexity score for one cross-file graph or component slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ComplexityReport {
    pub input: ComplexityInput,
    pub dimensions: ComplexityDimensionScores,
    pub total_score: u32,
    pub band: ComplexityBand,
}

impl ComplexityReport {
    pub fn from_input(input: ComplexityInput) -> Self {
        let dimensions = ComplexityDimensionScores {
            template_control_flow: weighted(input.template_if_count, 2)
                .saturating_add(weighted(input.template_for_count, 3)),
            slot_usage: weighted(input.slot_count, 2),
            prop_drilling: weighted(input.prop_drilling_edge_count, 3),
            global_state: weighted(input.global_state_reference_count, 2),
            provide_inject: weighted(input.provide_inject_max_depth.saturating_sub(1), 2)
                .saturating_add(weighted(input.provide_inject_reference_count, 1)),
            fallthrough_attrs: weighted(input.fallthrough_risk_count, 4),
            reactive_graph: weighted(input.reactive_node_count, 1)
                .saturating_add(weighted(input.reactive_edge_count, 2))
                .saturating_add(weighted(input.reactive_cycle_count, 10)),
        };
        let total_score = dimensions.total();

        Self {
            input,
            dimensions,
            total_score,
            band: band_for_score(total_score),
        }
    }
}

pub fn band_for_score(score: u32) -> ComplexityBand {
    match score {
        0..=14 => ComplexityBand::Low,
        15..=34 => ComplexityBand::Moderate,
        35..=69 => ComplexityBand::High,
        _ => ComplexityBand::Extreme,
    }
}

fn weighted(count: usize, weight: u32) -> u32 {
    u32::try_from(count)
        .unwrap_or(u32::MAX)
        .saturating_mul(weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_each_complexity_dimension() {
        let report = ComplexityReport::from_input(ComplexityInput {
            template_if_count: 2,
            template_for_count: 1,
            slot_count: 3,
            prop_drilling_edge_count: 2,
            global_state_reference_count: 4,
            provide_inject_max_depth: 3,
            provide_inject_reference_count: 5,
            fallthrough_risk_count: 2,
            reactive_node_count: 6,
            reactive_edge_count: 7,
            reactive_cycle_count: 1,
        });

        assert_eq!(report.dimensions.template_control_flow, 7);
        assert_eq!(report.dimensions.slot_usage, 6);
        assert_eq!(report.dimensions.prop_drilling, 6);
        assert_eq!(report.dimensions.global_state, 8);
        assert_eq!(report.dimensions.provide_inject, 9);
        assert_eq!(report.dimensions.fallthrough_attrs, 8);
        assert_eq!(report.dimensions.reactive_graph, 30);
        assert_eq!(report.total_score, 74);
        assert_eq!(report.band, ComplexityBand::Extreme);
    }

    #[test]
    fn shallow_empty_graph_is_low_complexity() {
        let report = ComplexityReport::from_input(ComplexityInput::default());

        assert_eq!(report.total_score, 0);
        assert_eq!(report.band, ComplexityBand::Low);
    }

    #[test]
    fn provide_depth_starts_after_root() {
        let report = ComplexityReport::from_input(ComplexityInput {
            provide_inject_max_depth: 1,
            provide_inject_reference_count: 1,
            ..ComplexityInput::default()
        });

        assert_eq!(report.dimensions.provide_inject, 1);
        assert_eq!(report.total_score, 1);
    }

    #[test]
    fn score_saturates_instead_of_overflowing() {
        let report = ComplexityReport::from_input(ComplexityInput {
            reactive_cycle_count: usize::MAX,
            ..ComplexityInput::default()
        });

        assert_eq!(report.dimensions.reactive_graph, u32::MAX);
        assert_eq!(report.total_score, u32::MAX);
        assert_eq!(report.band, ComplexityBand::Extreme);
    }
}
