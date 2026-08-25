//! Allocation-conscious filter labels and precomputed search documents.

use vize_doctor::{DoctorCategory, DoctorFinding, FindingSeverity};
use vize_s0::String;

pub(super) fn cycle(current: usize, length: usize, forward: bool) -> usize {
    if forward {
        (current + 1) % length
    } else {
        current.checked_sub(1).unwrap_or(length - 1)
    }
}

pub(super) fn search_document(finding: &DoctorFinding) -> String {
    let mut document = String::new("");
    for value in [
        finding.code.as_str(),
        finding.title.as_str(),
        finding.message.as_str(),
        finding.primary.path.as_str(),
        category_label(finding.category),
        severity_label(finding.assessment.severity),
    ] {
        document.push_str(value);
        document.push(' ');
    }
    document.to_lowercase()
}

pub(in crate::commands::doctor::tui) const fn category_label(
    category: DoctorCategory,
) -> &'static str {
    match category {
        DoctorCategory::Correctness => "correctness",
        DoctorCategory::Accessibility => "accessibility",
        DoctorCategory::Performance => "performance",
        DoctorCategory::Maintainability => "maintainability",
        DoctorCategory::Security => "security",
        DoctorCategory::ProductionReadiness => "production readiness",
    }
}

pub(in crate::commands::doctor::tui) const fn severity_label(
    severity: FindingSeverity,
) -> &'static str {
    match severity {
        FindingSeverity::Error => "error",
        FindingSeverity::Warning => "warning",
        FindingSeverity::Notice => "notice",
    }
}
