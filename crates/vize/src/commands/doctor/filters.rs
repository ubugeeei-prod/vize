//! CLI adapters for the reporter-neutral filter contract.

use vize_doctor::{
    DoctorCategory, DoctorFilter, DoctorFilterError, DoctorFilterSpec, FindingConfidence,
    FindingSeverity,
};

use super::DoctorArgs;

pub(super) fn compile(args: &DoctorArgs) -> Result<DoctorFilter, DoctorFilterError> {
    DoctorFilterSpec {
        categories: args.categories.clone(),
        severities: args.severities.clone(),
        confidences: args.confidences.clone(),
        targets: args.targets.clone(),
        rules: args.rules.clone(),
        paths: args.path_filters.clone(),
        routes: args.routes.clone(),
        environments: args.environments.clone(),
        packages: args.packages.clone(),
        changed_files: args.changed_files.clone(),
    }
    .compile()
}

pub(super) fn parse_category(value: &str) -> Result<DoctorCategory, &'static str> {
    match value {
        "correctness" => Ok(DoctorCategory::Correctness),
        "accessibility" => Ok(DoctorCategory::Accessibility),
        "performance" => Ok(DoctorCategory::Performance),
        "maintainability" => Ok(DoctorCategory::Maintainability),
        "security" => Ok(DoctorCategory::Security),
        "production-readiness" => Ok(DoctorCategory::ProductionReadiness),
        _ => Err(
            "expected correctness, accessibility, performance, maintainability, security, or production-readiness",
        ),
    }
}

pub(super) fn parse_severity(value: &str) -> Result<FindingSeverity, &'static str> {
    match value {
        "error" => Ok(FindingSeverity::Error),
        "warning" => Ok(FindingSeverity::Warning),
        "notice" => Ok(FindingSeverity::Notice),
        _ => Err("expected error, warning, or notice"),
    }
}

pub(super) fn parse_confidence(value: &str) -> Result<FindingConfidence, &'static str> {
    match value {
        "certain" => Ok(FindingConfidence::Certain),
        "high" => Ok(FindingConfidence::High),
        "medium" => Ok(FindingConfidence::Medium),
        "low" => Ok(FindingConfidence::Low),
        _ => Err("expected certain, high, medium, or low"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsers_accept_only_wire_contract_spellings() {
        assert_eq!(
            parse_category("production-readiness").unwrap(),
            DoctorCategory::ProductionReadiness
        );
        assert_eq!(parse_severity("warning").unwrap(), FindingSeverity::Warning);
        assert_eq!(parse_confidence("high").unwrap(), FindingConfidence::High);
        assert!(parse_category("ProductionReadiness").is_err());
        assert!(parse_severity("warn").is_err());
        assert!(parse_confidence("unknown").is_err());
    }
}
