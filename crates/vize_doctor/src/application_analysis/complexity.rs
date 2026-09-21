//! Template-complexity hotspots (Davinci P4-9b): components whose
//! **rendered** template complexity (own plus every distinct component they
//! render, recursion counted once) is above the full-corpus p95.
//!
//! The finding is a notice, not a warning: a large render tree is where to
//! look first, not a defect. The lint rule `vue/max-template-complexity`
//! judges own complexity; this finding is the cross-file view.

use vize_croquis_cf::{ComponentComplexity, CrossFileAnalyzer, CrossFileResult};
use vize_s0::cstr;

use super::{normalize_source_path, relativize_within};
use crate::{
    AnalysisProvenance, DoctorCategory, DoctorFinding, EvidenceKind, FindingAssessment,
    FindingConfidence, FindingEvidence, FindingImpact, FindingSeverity, HealthPenalty, RuleCost,
    SourceLocation,
};

/// Rendered cyclomatic complexity above which a component is a hotspot:
/// the p95 over 40,724 components of the full corpus
/// (`davinci-road/plan/complexity-metrics.md`).
pub const RENDERED_CYCLOMATIC_HOTSPOT_ABOVE: u32 = 106;
/// Rendered cognitive complexity above which a component is a hotspot
/// (same measurement).
pub const RENDERED_COGNITIVE_HOTSPOT_ABOVE: u32 = 139;

/// The stable finding code.
pub const TEMPLATE_COMPLEXITY_HOTSPOT: &str = "VIZE_DOCTOR_TEMPLATE_COMPLEXITY_HOTSPOT";

/// Contributors named in a finding's evidence.
const MAX_EVIDENCE: usize = 3;

/// One finding per component above the rendered thresholds, in the report's
/// order (most complex render tree first). Components whose path cannot be
/// expressed inside the workspace are skipped: a hotspot is advisory and
/// never fabricates a location.
pub(super) fn complexity_findings(
    analyzer: &CrossFileAnalyzer,
    result: &CrossFileResult,
) -> Vec<DoctorFinding> {
    result
        .template_complexity
        .iter()
        .filter(|component| is_hotspot(component))
        .filter_map(|component| finding(analyzer, component))
        .collect()
}

fn is_hotspot(component: &ComponentComplexity) -> bool {
    component.rendered.cyclomatic > RENDERED_CYCLOMATIC_HOTSPOT_ABOVE
        || component.rendered.cognitive > RENDERED_COGNITIVE_HOTSPOT_ABOVE
}

fn finding(analyzer: &CrossFileAnalyzer, component: &ComponentComplexity) -> Option<DoctorFinding> {
    let path = analyzer.get_file_path(component.file_id)?;
    let relative = match analyzer.registry().project_root() {
        Some(root) => relativize_within(path, root),
        None => normalize_source_path(path),
    }?;
    let (start, end) = component
        .template
        .contributors
        .first()
        .map_or((0, 0), |first| (first.start, first.end));
    let primary = SourceLocation::new(relative.clone(), start, end);
    let own = component.template.own;
    let rendered = component.rendered;
    let message = cstr!(
        "Rendering {} pulls in {} other components: rendered template complexity is cyclomatic {} \
         and cognitive {} (own {} and {}); the hotspot thresholds are {} and {}.",
        component.file_name,
        component.rendered_components,
        rendered.cyclomatic,
        rendered.cognitive,
        own.cyclomatic,
        own.cognitive,
        RENDERED_CYCLOMATIC_HOTSPOT_ABOVE,
        RENDERED_COGNITIVE_HOTSPOT_ABOVE,
    );
    let mut evidence = FindingEvidence::new(EvidenceKind::ControlFlow, message.clone())
        .with_location(primary.clone())
        .with_detail("renderedCyclomatic", cstr!("{}", rendered.cyclomatic))
        .with_detail("renderedCognitive", cstr!("{}", rendered.cognitive))
        .with_detail("ownCyclomatic", cstr!("{}", own.cyclomatic))
        .with_detail("ownCognitive", cstr!("{}", own.cognitive))
        .with_detail(
            "renderedComponents",
            cstr!("{}", component.rendered_components),
        )
        .with_detail(
            "recursive",
            if component.recursive { "true" } else { "false" },
        );
    for (index, contributor) in component
        .template
        .top_contributors(MAX_EVIDENCE)
        .into_iter()
        .enumerate()
    {
        evidence = evidence.with_detail(
            cstr!("contributor{}", index + 1),
            cstr!(
                "{} at {}:{} (+{} cognitive)",
                contributor.kind,
                contributor.line,
                contributor.column,
                contributor.cognitive
            ),
        );
    }
    let assessment = FindingAssessment::new(
        FindingSeverity::Notice,
        FindingConfidence::Certain,
        FindingImpact::Medium,
        HealthPenalty::new(5, "Whole-project improvement opportunity"),
    );
    let mut finding = DoctorFinding::new(
        TEMPLATE_COMPLEXITY_HOTSPOT,
        DoctorCategory::Maintainability,
        assessment,
        primary,
        "Template Complexity Hotspot",
        message,
        AnalysisProvenance::new("template-complexity-render-tree", RuleCost::Moderate),
    )
    .with_failure_scenario(
        "A change inside this render tree has to be understood against every branch the tree renders.",
    )
    .with_evidence(evidence);
    // The rendered score depends on every file in the render tree.
    let mut inputs = vec![relative];
    inputs.extend(
        component
            .render_tree
            .iter()
            .filter_map(|id| analyzer.get_file_path(*id))
            .filter_map(|path| match analyzer.registry().project_root() {
                Some(root) => relativize_within(path, root),
                None => normalize_source_path(path),
            }),
    );
    finding.provenance = finding.provenance.with_invalidation_inputs(inputs);
    Some(finding)
}
