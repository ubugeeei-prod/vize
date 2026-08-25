//! Structured selected-finding rows with semantic Fresco presentations.

mod labels;

use vize_doctor::{DoctorFinding, FixSafety};
use vize_fresco::{
    DiagnosticPresentation, DiagnosticPresentationKind, DiagnosticTone, HeadlessSemanticNode,
    SemanticRole, SemanticState, TerminalCapabilities, TextWrap,
    terminal::{Color, Style},
    text::WrapMode,
};
use vize_s0::{String, cstr};

use super::{StyledLine, profile, severity_tone};
use crate::commands::doctor::{
    DoctorSource,
    tui::{
        DoctorTuiError,
        model::{DoctorTuiModel, InteractionMode},
    },
};
use labels::{
    confidence_label, confidence_tone, evidence_kind_label, fix_safety_label, impact_label,
    impact_tone, rule_cost_label,
};

pub(super) fn build(
    model: &DoctorTuiModel<'_>,
    sources: &[DoctorSource],
    width: u16,
    capabilities: TerminalCapabilities,
) -> Result<Vec<StyledLine>, DoctorTuiError> {
    let Some(finding) = model.selected_finding() else {
        return Ok(vec![line(
            "No finding selected",
            capabilities.adapt_style(Style::new().dim()),
        )]);
    };
    let width = usize::from(width.max(1));
    let mut lines = Vec::new();
    lines.push(
        line(
            cstr!("{} — {}", finding.code, finding.title),
            capabilities.adapt_style(Style::new().fg(Color::Cyan).bold()),
        )
        .with_semantic(
            HeadlessSemanticNode::new(0, SemanticRole::Heading, finding.title.clone())
                .with_description(finding.code.clone())
                .with_state(SemanticState::default().with_level(2)),
        ),
    );
    presentation(
        &mut lines,
        DiagnosticPresentation::new(
            DiagnosticPresentationKind::Severity,
            super::super::model::severity_label(finding.assessment.severity),
            severity_tone(finding.assessment.severity),
        )?,
        capabilities,
    );
    presentation(
        &mut lines,
        DiagnosticPresentation::new(
            DiagnosticPresentationKind::Confidence,
            confidence_label(finding.assessment.confidence),
            confidence_tone(finding.assessment.confidence),
        )?,
        capabilities,
    );
    presentation(
        &mut lines,
        DiagnosticPresentation::new(
            DiagnosticPresentationKind::Impact,
            impact_label(finding.assessment.impact),
            impact_tone(finding.assessment.impact),
        )?,
        capabilities,
    );
    let (source_line, source_column) = model.source_position(sources);
    presentation(
        &mut lines,
        DiagnosticPresentation::code_location(
            finding.primary.path.as_str(),
            source_line,
            source_column,
        )?,
        capabilities,
    );
    lines.push(line(
        cstr!(
            "Category: {}  Penalty: -{}",
            super::super::model::category_label(finding.category),
            finding.assessment.penalty.points,
        ),
        capabilities.adapt_style(Style::new().dim()),
    ));
    push_wrapped(
        &mut lines,
        "",
        &finding.message,
        width,
        Style::new(),
        capabilities,
    );
    if let Some(scenario) = &finding.failure_scenario {
        push_wrapped(
            &mut lines,
            "Failure: ",
            scenario,
            width,
            Style::new().fg(Color::Yellow),
            capabilities,
        );
    }
    evidence_rows(&mut lines, model, finding, width, capabilities)?;
    related_rows(&mut lines, finding, width, capabilities);
    fix_rows(&mut lines, finding, width, capabilities)?;
    provenance_rows(&mut lines, finding, width, capabilities);
    if let Some(documentation) = &finding.documentation {
        push_wrapped(
            &mut lines,
            "Docs: ",
            documentation,
            width,
            Style::new().fg(Color::Blue),
            capabilities,
        );
    }
    Ok(lines)
}

fn evidence_rows(
    lines: &mut Vec<StyledLine>,
    model: &DoctorTuiModel<'_>,
    finding: &DoctorFinding,
    width: usize,
    capabilities: TerminalCapabilities,
) -> Result<(), DoctorTuiError> {
    if finding.evidence.is_empty() {
        return Ok(());
    }
    lines.push(section("Evidence", capabilities));
    let selected = model.selected_evidence_key();
    for (index, evidence) in finding.evidence.iter().enumerate() {
        let summary = cstr!(
            "{}: {}",
            evidence_kind_label(evidence.kind),
            evidence.summary
        );
        let presentation = DiagnosticPresentation::evidence(
            summary.as_str(),
            index as u64 + 1,
            finding.evidence.len() as u64,
        )?;
        let mut style = capabilities.adapt_style(presentation.style());
        style.reverse = selected == Some(index);
        let line_start = lines.len();
        push_wrapped(
            lines,
            "",
            &presentation.text(profile(capabilities, false)),
            width,
            style,
            capabilities,
        );
        if let Some(line) = lines.get_mut(line_start) {
            let mut semantic = presentation.semantic_node(0);
            semantic.state.selected = selected == Some(index);
            line.semantic = Some(semantic);
            line.focused = selected == Some(index)
                && model.mode() == InteractionMode::Browse
                && model.workspace().focus() == vize_fresco::DiagnosticWorkspaceFocus::Evidence;
        }
        if selected == Some(index) {
            for (key, value) in &evidence.details {
                push_wrapped(
                    lines,
                    "  ",
                    &cstr!("{key}={value}"),
                    width,
                    Style::new().dim(),
                    capabilities,
                );
            }
        }
    }
    Ok(())
}

fn related_rows(
    lines: &mut Vec<StyledLine>,
    finding: &DoctorFinding,
    width: usize,
    capabilities: TerminalCapabilities,
) {
    if finding.related.is_empty() {
        return;
    }
    lines.push(section("Related locations", capabilities));
    for related in &finding.related {
        push_wrapped(
            lines,
            "",
            &cstr!(
                "{}:{}..{} — {}",
                related.location.path,
                related.location.start,
                related.location.end,
                related.message,
            ),
            width,
            Style::new().dim(),
            capabilities,
        );
    }
}

fn fix_rows(
    lines: &mut Vec<StyledLine>,
    finding: &DoctorFinding,
    width: usize,
    capabilities: TerminalCapabilities,
) -> Result<(), DoctorTuiError> {
    let Some(fix) = &finding.fix else {
        return Ok(());
    };
    lines.push(section("Fix", capabilities));
    let tone = match fix.safety {
        FixSafety::Safe => DiagnosticTone::Positive,
        FixSafety::ReviewRequired => DiagnosticTone::Caution,
        FixSafety::Unavailable => DiagnosticTone::Neutral,
    };
    presentation(
        lines,
        DiagnosticPresentation::new(
            DiagnosticPresentationKind::FixSafety,
            fix_safety_label(fix.safety),
            tone,
        )?,
        capabilities,
    );
    push_wrapped(lines, "", &fix.title, width, Style::new(), capabilities);
    for verification in &fix.verification {
        push_wrapped(
            lines,
            "Verify: ",
            verification,
            width,
            Style::new().dim(),
            capabilities,
        );
    }
    if !fix.edits.is_empty() {
        lines.push(line(
            cstr!("{} source edit(s)", fix.edits.len()),
            capabilities.adapt_style(Style::new().dim()),
        ));
    }
    Ok(())
}

fn provenance_rows(
    lines: &mut Vec<StyledLine>,
    finding: &DoctorFinding,
    width: usize,
    capabilities: TerminalCapabilities,
) {
    lines.push(section("Analysis", capabilities));
    push_wrapped(
        lines,
        "",
        &cstr!(
            "{} ({})",
            finding.provenance.capability,
            rule_cost_label(finding.provenance.cost),
        ),
        width,
        Style::new().dim(),
        capabilities,
    );
}

fn presentation(
    lines: &mut Vec<StyledLine>,
    presentation: DiagnosticPresentation,
    capabilities: TerminalCapabilities,
) {
    let text = presentation.text(profile(capabilities, false));
    let semantic = presentation.semantic_node(0);
    lines.push(
        line(
            text.as_str(),
            capabilities.adapt_style(presentation.style()),
        )
        .with_semantic(semantic),
    );
}

fn push_wrapped(
    lines: &mut Vec<StyledLine>,
    prefix: &str,
    text: &str,
    width: usize,
    style: Style,
    capabilities: TerminalCapabilities,
) {
    let mut value = String::from(prefix);
    value.push_str(text);
    for wrapped in TextWrap::wrap(&value, width, WrapMode::Word) {
        lines.push(line(wrapped.as_str(), capabilities.adapt_style(style)));
    }
}

fn section(label: &str, capabilities: TerminalCapabilities) -> StyledLine {
    line(
        cstr!("— {label} —"),
        capabilities.adapt_style(Style::new().fg(Color::Cyan).bold()),
    )
    .with_semantic(
        HeadlessSemanticNode::new(0, SemanticRole::Heading, label)
            .with_state(SemanticState::default().with_level(3)),
    )
}

fn line(text: impl Into<String>, style: Style) -> StyledLine {
    StyledLine {
        text: text.into(),
        style,
        semantic: None,
        focused: false,
    }
}

impl StyledLine {
    fn with_semantic(mut self, semantic: HeadlessSemanticNode) -> Self {
        self.semantic = Some(semantic);
        self
    }
}
