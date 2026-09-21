//! The per-component template complexity section: own and rendered scores
//! from the S2 facts, and where each component's complexity comes from.

use vize_croquis_cf::{
    ComplexityContributor, ComponentComplexity,
    TEMPLATE_COGNITIVE_WARN_ABOVE as COGNITIVE_WARN_ABOVE,
    TEMPLATE_CYCLOMATIC_WARN_ABOVE as CYCLOMATIC_WARN_ABOVE,
};
use vize_s0::{String, appendln, appends};

use super::escape_table_cell;

/// Components listed, most complex render tree first.
const MAX_COMPONENTS: usize = 10;
/// Contributors named per component.
const MAX_CONTRIBUTORS: usize = 3;

pub(super) fn append_template_section(out: &mut String, components: &[ComponentComplexity]) {
    appendln!(out);
    appendln!(out, "### Template Complexity");
    appendln!(out);
    if components.is_empty() {
        appendln!(out, "No component templates were analyzed.");
        return;
    }
    appendln!(
        out,
        "Own: the component's template alone. Rendered: own plus every distinct component it \
         renders, recursion counted once. Own scores above cyclomatic ",
        @CYCLOMATIC_WARN_ABOVE,
        " or cognitive ",
        @COGNITIVE_WARN_ABOVE,
        " exceed the corpus p95."
    );
    appendln!(out);
    appendln!(
        out,
        "| Component | Own cyclomatic | Own cognitive | Rendered cyclomatic | Rendered cognitive | Renders | Where it comes from |"
    );
    appendln!(out, "| --- | ---: | ---: | ---: | ---: | ---: | --- |");
    for component in components.iter().take(MAX_COMPONENTS) {
        append_component_row(out, component);
    }
    if components.len() > MAX_COMPONENTS {
        appendln!(
            out,
            "\n",
            @components.len() - MAX_COMPONENTS,
            " more component templates are not listed."
        );
    }
}

fn append_component_row(out: &mut String, component: &ComponentComplexity) {
    let own = component.template.own;
    let mut name = escape_table_cell(component.file_name.as_str());
    if component.template.exceeds_thresholds() {
        name.push_str(" (over threshold)");
    }
    if component.recursive {
        name.push_str(" (recursive)");
    }
    let mut sources = String::default();
    for contributor in component.template.top_contributors(MAX_CONTRIBUTORS) {
        append_contributor(&mut sources, contributor);
    }
    if sources.is_empty() {
        sources.push('-');
    }
    appendln!(
        out,
        "| ",
        name.as_str(),
        " | ",
        @own.cyclomatic,
        " | ",
        @own.cognitive,
        " | ",
        @component.rendered.cyclomatic,
        " | ",
        @component.rendered.cognitive,
        " | ",
        @component.rendered_components,
        " | ",
        sources.as_str(),
        " |"
    );
}

fn append_contributor(out: &mut String, contributor: &ComplexityContributor) {
    if !out.is_empty() {
        out.push_str(", ");
    }
    appends!(
        out,
        "`",
        contributor.kind,
        "` L",
        @contributor.line,
        ":",
        @contributor.column,
        " +",
        @contributor.cognitive
    );
}
