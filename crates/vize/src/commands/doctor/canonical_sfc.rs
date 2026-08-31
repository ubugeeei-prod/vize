//! Doctor rule for the opt-in public component authoring contract.

use vize_atelier_sfc::{BlockLocation, SfcDescriptor};
use vize_doctor::{
    AnalysisProvenance, DoctorCategory, DoctorFinding, EvidenceKind, FindingAssessment,
    FindingConfidence, FindingEvidence, FindingFix, FindingImpact, FindingSeverity, HealthPenalty,
    RuleCost, SourceLocation, SuppressionPolicy,
};
use vize_s0::cstr;

pub(super) const RULE_CODE: &str = "VIZE_DOCTOR_SFC_EXPLICIT_SECTIONS";

const CONTRACT_CAPABILITY: &str = "public-sfc-contract";
const GATE_RULE: &str = "explicit-sfc";
const UNAVAILABLE_FIX_REASON: &str =
    "Canonical SFC sections require authored source changes that must be reviewed.";

#[derive(Clone)]
struct ContractProblem {
    requirement: &'static str,
    location: SourceLocation,
}

/// Checks the same explicit-section invariant as the UI source-quality
/// `explicit-sfc` authoring gate without rescanning SFC source text.
///
/// The caller must opt a source into the public component contract. Ordinary
/// application SFCs intentionally remain outside this rule.
pub(super) fn finding(path: &str, descriptor: &SfcDescriptor<'_>) -> Option<DoctorFinding> {
    let source_end = bounded_offset(descriptor.source.len());
    let insertion = || SourceLocation::new(path, source_end, source_end);
    let mut problems = Vec::with_capacity(3);

    match descriptor.script_setup.as_ref() {
        Some(script) if script.lang.as_deref() == Some("ts") => {}
        Some(script) => problems.push(problem(
            "<script setup lang=\"ts\">",
            opening_location(path, &script.loc),
        )),
        None => problems.push(problem(
            "<script setup lang=\"ts\">",
            descriptor
                .script
                .as_ref()
                .map(|script| opening_location(path, &script.loc))
                .unwrap_or_else(&insertion),
        )),
    }

    if descriptor.template.is_none() {
        problems.push(problem("<template>", insertion()));
    }

    if !descriptor.styles.iter().any(|style| style.scoped) {
        problems.push(problem(
            "<style scoped>",
            descriptor
                .styles
                .first()
                .map(|style| opening_location(path, &style.loc))
                .unwrap_or_else(&insertion),
        ));
    }

    let primary = problems.first()?.location.clone();
    let problem_names = problems
        .iter()
        .map(|problem| problem.requirement)
        .collect::<Vec<_>>()
        .join(", ");
    let mut finding = DoctorFinding::new(
        RULE_CODE,
        DoctorCategory::Maintainability,
        FindingAssessment::new(
            FindingSeverity::Error,
            FindingConfidence::Certain,
            FindingImpact::Medium,
            HealthPenalty::new(20, "Public component source contract is incomplete"),
        ),
        primary,
        "Public component does not use canonical SFC sections",
        cstr!(
            "Declare every canonical public component section explicitly. Noncanonical sections: \
             {}.",
            problem_names
        ),
        AnalysisProvenance::new(CONTRACT_CAPABILITY, RuleCost::Low)
            .with_invalidation_inputs([path]),
    )
    .with_failure_scenario(
        "A public component bypasses the source-owned SFC contract used by quality gates, \
         editors, and target adapters.",
    )
    .with_documentation("https://github.com/ubugeeei-prod/vize/issues/3134")
    .with_fix(FindingFix::unavailable(UNAVAILABLE_FIX_REASON))
    .with_suppression(SuppressionPolicy::Forbidden);

    for problem in problems {
        finding = finding.with_evidence(
            FindingEvidence::new(
                EvidenceKind::Contract,
                cstr!("Expected explicit {} section", problem.requirement),
            )
            .with_location(problem.location)
            .with_detail("authoringGateRule", GATE_RULE)
            .with_detail("requirement", problem.requirement),
        );
    }
    Some(finding)
}

fn problem(requirement: &'static str, location: SourceLocation) -> ContractProblem {
    ContractProblem {
        requirement,
        location,
    }
}

fn opening_location(path: &str, location: &BlockLocation) -> SourceLocation {
    SourceLocation::new(
        path,
        bounded_offset(location.tag_start),
        bounded_offset(location.start),
    )
}

fn bounded_offset(offset: usize) -> u32 {
    u32::try_from(offset).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
    use vize_s0::String;

    #[test]
    fn accepts_semantic_attribute_order_and_companion_script() {
        let source = r#"<script>export const version = 1</script>
<script lang="ts" setup>const value = 1</script>
<template><output>{{ value }}</output></template>
<style lang="css" scoped>output { display: block }</style>
"#;
        let descriptor = parse(source);

        assert!(finding("src/Output.vue", &descriptor).is_none());
    }

    #[test]
    fn reports_exact_noncanonical_opening_tags() {
        let source = r#"<script setup>const value = 1</script>
<template><output>{{ value }}</output></template>
<style>output { display: block }</style>
"#;
        let descriptor = parse(source);
        let finding = finding("src/Output.vue", &descriptor).unwrap();
        let script_end = source.find('>').unwrap() + 1;
        let style_start = source.find("<style>").unwrap();

        assert_eq!(finding.code, RULE_CODE);
        assert_eq!(finding.primary.start, 0);
        assert_eq!(finding.primary.end as usize, script_end);
        assert_eq!(
            &source[finding.primary.start as usize..finding.primary.end as usize],
            "<script setup>"
        );
        assert_eq!(finding.evidence.len(), 2);
        let fix = finding.fix.as_ref().unwrap();
        assert_eq!(fix.safety, vize_doctor::FixSafety::Unavailable);
        assert_eq!(fix.title, UNAVAILABLE_FIX_REASON);
        assert!(fix.edits.is_empty());
        assert!(fix.verification.is_empty());
        assert_eq!(
            finding.evidence[1].location.as_ref().unwrap().start as usize,
            style_start
        );
        assert_eq!(
            &source[style_start..finding.evidence[1].location.as_ref().unwrap().end as usize],
            "<style>"
        );
    }

    #[test]
    fn missing_sections_use_the_exact_eof_insertion_point() {
        let source = "<template><p>Static</p></template>\n";
        let descriptor = parse(source);
        let finding = finding("src/Static.vue", &descriptor).unwrap();

        assert_eq!(finding.primary.start as usize, source.len());
        assert_eq!(finding.primary.end as usize, source.len());
        assert_eq!(finding.evidence.len(), 2);
        assert!(finding.evidence.iter().all(|evidence| {
            evidence
                .location
                .as_ref()
                .is_some_and(|location| location.start as usize == source.len())
        }));
    }

    #[test]
    fn large_source_keeps_diagnostic_output_bounded() {
        let body = "const value = 1;\n".repeat(65_536);
        let mut source = String::from("<script>");
        source.push_str(&body);
        source.push_str("</script>\n");
        let descriptor = parse(&source);
        let finding = finding("src/Large.vue", &descriptor).unwrap();
        let serialized = serde_json::to_vec(&finding).unwrap();

        assert!(source.len() > 1_000_000);
        assert!(
            serialized.len() < 3_072,
            "diagnostic output grew to {} bytes",
            serialized.len()
        );
        assert_eq!(finding.provenance.cost, RuleCost::Low);
        assert_eq!(finding.evidence.len(), 3);
    }

    fn parse(source: &str) -> SfcDescriptor<'_> {
        parse_sfc(
            source,
            SfcParseOptions {
                filename: "fixture.vue".into(),
                ..Default::default()
            },
        )
        .unwrap()
    }
}
