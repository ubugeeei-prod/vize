#![expect(
    clippy::disallowed_macros,
    reason = "the identical public API probe crosses dependency alias migrations"
)]

use vize_doctor::{
    AnalysisProvenance, DoctorCategory, DoctorFilterSpec, DoctorFinding, FindingAssessment,
    FindingConfidence, FindingImpact, FindingSeverity, HealthPenalty, RuleCost, SourceLocation,
};

pub fn scenario(kind: &str, count: usize) -> (DoctorFilterSpec, Vec<DoctorFinding>, Vec<bool>) {
    let patterns: Vec<_> = (0..count)
        .map(|index| match kind {
            "rule-literal" => format!("VIZE_DOCTOR_RULE_{index:03}"),
            "rule-regex" => format!("VIZE_DOCTOR_RULE_{index:03}_[AB]"),
            "path-prefix" => format!("packages/app-{index:03}/**"),
            "path-suffix" => format!("**/app-{index:03}/components/Panel.vue"),
            _ => format!("packages/app-{index:03}/src/[AB]*.vue"),
        })
        .map(Into::into)
        .collect();
    let spec = if kind.starts_with("rule-") {
        DoctorFilterSpec {
            rules: patterns,
            ..DoctorFilterSpec::default()
        }
    } else {
        DoctorFilterSpec {
            paths: patterns,
            ..DoctorFilterSpec::default()
        }
    };
    let mut findings = Vec::new();
    let mut expected = Vec::new();
    for candidate in 0..64 {
        let index = match candidate % 4 {
            0 => 0,
            1 => count.saturating_sub(1),
            2 => count / 2,
            _ => count + candidate,
        };
        let code = match kind {
            "rule-literal" => format!("VIZE_DOCTOR_RULE_{index:03}"),
            "rule-regex" => format!("VIZE_DOCTOR_RULE_{index:03}_A"),
            _ => "VIZE_DOCTOR_PERFORMANCE_001".to_owned(),
        };
        let path = match kind {
            "path-prefix" => format!("packages/app-{index:03}/src/组件/Panel.vue"),
            "path-suffix" => format!("packages/日本語/app-{index:03}/components/Panel.vue"),
            "path-regex" => format!("packages/app-{index:03}/src/BPanel.vue"),
            _ => "packages/account/src/Login.vue".to_owned(),
        };
        findings.push(DoctorFinding::new(
            code,
            DoctorCategory::Performance,
            FindingAssessment::new(
                FindingSeverity::Warning,
                FindingConfidence::High,
                FindingImpact::Medium,
                HealthPenalty::new(1, "Measured filter cost"),
            ),
            SourceLocation::new(path, candidate as u32, candidate as u32 + 1),
            "Planted filter finding",
            "Repeated rule-code and source-path filtering",
            AnalysisProvenance::new("filter-benchmark", RuleCost::Low),
        ));
        expected.push(candidate % 4 != 3);
    }
    (spec, findings, expected)
}
