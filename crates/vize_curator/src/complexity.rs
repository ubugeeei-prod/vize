//! Cross-file complexity report rendering.

#[cfg(test)]
mod tests;

use vize_croquis_cf::{
    ComplexityBand, ComplexityDimension, ComplexityDimensionBreakdown, ComplexityHotspot,
    ComplexityInput, ComplexityReport,
};
use vize_s0::{String, appendln, appends};

pub fn render_complexity_markdown(
    report: &ComplexityReport,
    hotspots: &[ComplexityHotspot],
) -> String {
    let mut out = String::default();

    appendln!(out, "## Cross-file Complexity");
    appendln!(out);
    appendln!(out, "| Metric | Value |");
    appendln!(out, "| --- | ---: |");
    appendln!(out, "| Band | ", band_label(report.band), " |");
    appendln!(out, "| Total score | ", @report.total_score, " |");
    appendln!(out, "| Cyclomatic score | ", @report.cyclomatic_score, " |");
    appendln!(out, "| Cognitive score | ", @report.cognitive_score, " |");

    appendln!(out);
    appendln!(out, "### Dimensions");
    appendln!(out);
    appendln!(out, "| Dimension | Score |");
    appendln!(out, "| --- | ---: |");
    for dimension in report.dimensions.breakdown() {
        append_dimension_row(&mut out, dimension);
    }

    appendln!(out);
    appendln!(out, "### Top Hotspots");
    appendln!(out);
    if hotspots.is_empty() {
        appendln!(out, "No component hotspots were identified.");
        return out;
    }

    appendln!(out, "| Rank | File | Component | Score | Reason |");
    appendln!(out, "| ---: | --- | --- | ---: | --- |");
    for (index, hotspot) in hotspots.iter().take(5).enumerate() {
        append_hotspot_row(&mut out, index + 1, hotspot);
    }

    out
}

fn append_dimension_row(out: &mut String, dimension: ComplexityDimensionBreakdown) {
    appendln!(
        out,
        "| ",
        dimension_label(dimension.dimension),
        " | ",
        @dimension.score,
        " |"
    );
}

fn append_hotspot_row(out: &mut String, rank: usize, hotspot: &ComplexityHotspot) {
    let file_name = escape_table_cell(hotspot.file_name.as_str());
    let component_name = hotspot
        .component_name
        .as_ref()
        .map(|name| escape_table_cell(name.as_str()))
        .unwrap_or_else(|| "-".into());
    let reason = escape_table_cell(hotspot_reason(hotspot).as_str());

    appendln!(
        out,
        "| ",
        @rank,
        " | ",
        file_name.as_str(),
        " | ",
        component_name.as_str(),
        " | ",
        @hotspot.total_score,
        " | ",
        reason.as_str(),
        " |"
    );
}

fn hotspot_reason(hotspot: &ComplexityHotspot) -> String {
    let mut reason = String::default();
    if let Some(dominant) = hotspot.dominant_dimension {
        appends!(
            reason,
            dimension_label(dominant.dimension),
            " (",
            @dominant.score,
            ")"
        );
    } else {
        reason.push_str("balanced dimensions");
    }

    let drivers = input_drivers(hotspot.input);
    if !drivers.is_empty() {
        appends!(reason, " via ", drivers.as_str());
    }

    reason
}

fn input_drivers(input: ComplexityInput) -> String {
    let mut drivers = String::default();
    push_driver(&mut drivers, "v-if", input.template_if_count);
    push_driver(&mut drivers, "v-for", input.template_for_count);
    push_driver(
        &mut drivers,
        "logical ops",
        input.template_logical_operator_count,
    );
    push_driver(
        &mut drivers,
        "tree if depth",
        input.component_tree_v_if_max_depth,
    );
    push_driver(
        &mut drivers,
        "tree for depth",
        input.component_tree_v_for_max_depth,
    );
    push_driver(
        &mut drivers,
        "scoped slot depth",
        input.component_tree_scoped_slot_max_depth,
    );
    push_driver(&mut drivers, "slots", input.slot_count);
    push_driver(&mut drivers, "prop edges", input.prop_drilling_edge_count);
    push_driver(
        &mut drivers,
        "global refs",
        input.global_state_reference_count,
    );
    push_driver(
        &mut drivers,
        "provide depth",
        input.provide_inject_max_depth,
    );
    push_driver(
        &mut drivers,
        "provide refs",
        input.provide_inject_reference_count,
    );
    push_driver(
        &mut drivers,
        "provide fanout",
        input.provide_inject_fanout_count,
    );
    push_driver(
        &mut drivers,
        "fallthrough risks",
        input.fallthrough_risk_count,
    );
    push_driver(&mut drivers, "reactive nodes", input.reactive_node_count);
    push_driver(&mut drivers, "reactive edges", input.reactive_edge_count);
    push_driver(&mut drivers, "reactive cycles", input.reactive_cycle_count);
    drivers
}

fn push_driver(drivers: &mut String, label: &'static str, value: usize) {
    if value == 0 {
        return;
    }
    if !drivers.is_empty() {
        drivers.push_str(", ");
    }
    appends!(drivers, label, "=", @value);
}

fn escape_table_cell(value: &str) -> String {
    let mut escaped = String::default();
    for ch in value.chars() {
        match ch {
            '|' => escaped.push_str("\\|"),
            '\n' | '\r' => escaped.push(' '),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn band_label(band: ComplexityBand) -> &'static str {
    match band {
        ComplexityBand::Low => "low",
        ComplexityBand::Moderate => "moderate",
        ComplexityBand::High => "high",
        ComplexityBand::Extreme => "extreme",
    }
}

fn dimension_label(dimension: ComplexityDimension) -> &'static str {
    dimension.as_str()
}
