//! Doctor-domain labels and non-color presentation tones.

use vize_doctor::{EvidenceKind, FindingConfidence, FindingImpact, FixSafety, RuleCost};
use vize_fresco::DiagnosticTone;

pub(super) const fn confidence_label(value: FindingConfidence) -> &'static str {
    match value {
        FindingConfidence::Certain => "certain",
        FindingConfidence::High => "high",
        FindingConfidence::Medium => "medium",
        FindingConfidence::Low => "low",
    }
}

pub(super) const fn confidence_tone(value: FindingConfidence) -> DiagnosticTone {
    match value {
        FindingConfidence::Certain | FindingConfidence::High => DiagnosticTone::Positive,
        FindingConfidence::Medium => DiagnosticTone::Caution,
        FindingConfidence::Low => DiagnosticTone::Informational,
    }
}

pub(super) const fn impact_label(value: FindingImpact) -> &'static str {
    match value {
        FindingImpact::Critical => "critical",
        FindingImpact::High => "high",
        FindingImpact::Medium => "medium",
        FindingImpact::Low => "low",
    }
}

pub(super) const fn impact_tone(value: FindingImpact) -> DiagnosticTone {
    match value {
        FindingImpact::Critical | FindingImpact::High => DiagnosticTone::Negative,
        FindingImpact::Medium => DiagnosticTone::Caution,
        FindingImpact::Low => DiagnosticTone::Informational,
    }
}

pub(super) const fn fix_safety_label(value: FixSafety) -> &'static str {
    match value {
        FixSafety::Safe => "safe",
        FixSafety::ReviewRequired => "review required",
        FixSafety::Unavailable => "unavailable",
    }
}

pub(super) const fn evidence_kind_label(value: EvidenceKind) -> &'static str {
    match value {
        EvidenceKind::Source => "source",
        EvidenceKind::Type => "type",
        EvidenceKind::ControlFlow => "control flow",
        EvidenceKind::Reactivity => "reactivity",
        EvidenceKind::Component => "component",
        EvidenceKind::Css => "css",
        EvidenceKind::BuildGraph => "build graph",
        EvidenceKind::Contract => "contract",
        EvidenceKind::Measurement => "measurement",
    }
}

pub(super) const fn rule_cost_label(value: RuleCost) -> &'static str {
    match value {
        RuleCost::Trivial => "trivial",
        RuleCost::Low => "low cost",
        RuleCost::Moderate => "moderate cost",
        RuleCost::High => "high cost",
    }
}
