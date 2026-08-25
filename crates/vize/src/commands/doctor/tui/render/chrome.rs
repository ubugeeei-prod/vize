//! Semantic title, status, shortcut, and help chrome.

use vize_fresco::{
    DiagnosticPresentation, DiagnosticTone, HeadlessSemanticNode, Rect, SemanticRole,
    SemanticState, TerminalCapabilities,
    terminal::{Color, Style},
};
use vize_s0::cstr;

use super::{profile, tree::DoctorFrameBuilder};
use crate::commands::doctor::tui::{
    DoctorTuiError,
    model::{DoctorTuiModel, InteractionMode},
};

pub(super) fn render_header(
    builder: &mut DoctorFrameBuilder,
    model: &DoctorTuiModel<'_>,
    capabilities: TerminalCapabilities,
) -> Result<(), DoctorTuiError> {
    let layout = model.workspace().layout();
    let width = layout.width();
    let root = builder.root();
    let title_style = capabilities.adapt_style(Style::new().fg(Color::Cyan).bold());
    builder.text(
        root,
        Rect::new(0, 0, width, 1),
        "VIZE DOCTOR",
        title_style,
        Some(
            HeadlessSemanticNode::new(0, SemanticRole::Heading, "Vize Doctor")
                .with_state(SemanticState::default().with_level(1)),
        ),
        false,
    );

    let score = model.report().summary().overall_score;
    let score_tone = if score >= 90 {
        DiagnosticTone::Positive
    } else if score >= 70 {
        DiagnosticTone::Caution
    } else {
        DiagnosticTone::Negative
    };
    let presentation = DiagnosticPresentation::score(u64::from(score), 100, score_tone)?;
    let score_text = presentation.text(profile(capabilities, false));
    builder.text(
        root,
        Rect::new(13, 0, width.saturating_sub(13), 1),
        score_text,
        capabilities.adapt_style(presentation.style()),
        Some(presentation.semantic_node(0)),
        false,
    );

    render_status(builder, model, capabilities, width, layout.height());
    Ok(())
}

fn render_status(
    builder: &mut DoctorFrameBuilder,
    model: &DoctorTuiModel<'_>,
    capabilities: TerminalCapabilities,
    width: u16,
    height: u16,
) {
    let root = builder.root();
    if height > 1 {
        let status = if model.mode() == InteractionMode::Search {
            cstr!("/ {}", model.search())
        } else {
            cstr!(
                "category={}  severity={}  visible={}/{}  {}",
                model.category_label(),
                model.severity_label(),
                model.finding_keys().len(),
                model.report().findings().len(),
                model.status(),
            )
        };
        let semantic = if model.mode() == InteractionMode::Search {
            HeadlessSemanticNode::new(0, SemanticRole::SearchBox, "Finding search")
                .with_state(SemanticState::default().with_value(model.search()))
        } else {
            HeadlessSemanticNode::new(0, SemanticRole::Status, "Doctor filter status")
                .with_state(SemanticState::default().with_value(status.clone()))
        };
        builder.text(
            root,
            Rect::new(0, 1, width, 1),
            status,
            capabilities.adapt_style(Style::new().fg(Color::Gray)),
            Some(semantic),
            model.mode() == InteractionMode::Search,
        );
    }
    if height > 2 {
        let separator = capabilities.select_symbol("─", "-");
        builder.text(
            root,
            Rect::new(0, 2, width, 1),
            separator.repeat(usize::from(width)),
            Style::new(),
            None,
            false,
        );
        builder.text(
            root,
            Rect::new(1, 2, width.saturating_sub(2), 1),
            "j/k navigate  Tab focus  c/C category  s/S severity  / search  ? help  q quit",
            capabilities.adapt_style(Style::new().dim()),
            Some(HeadlessSemanticNode::new(
                0,
                SemanticRole::Text,
                "Keyboard shortcuts",
            )),
            false,
        );
    }
}

pub(super) fn render_help(
    builder: &mut DoctorFrameBuilder,
    model: &DoctorTuiModel<'_>,
    capabilities: TerminalCapabilities,
) -> Result<(), DoctorTuiError> {
    let area = model.workspace().layout().content();
    let pane = builder.container(
        builder.root(),
        area,
        Some(HeadlessSemanticNode::new(
            0,
            SemanticRole::Region,
            "Keyboard help",
        )),
        true,
    );
    builder.text(
        pane,
        Rect::new(0, 0, area.width, 1),
        "Keyboard help — Esc, ? or F1 closes this view",
        capabilities.adapt_style(Style::new().fg(Color::Cyan).bold()),
        Some(
            HeadlessSemanticNode::new(0, SemanticRole::Heading, "Keyboard help")
                .with_state(SemanticState::default().with_level(2)),
        ),
        false,
    );
    for (row, binding) in model
        .keymap()
        .bindings()
        .iter()
        .take(usize::from(area.height.saturating_sub(1)))
        .enumerate()
    {
        let hint =
            DiagnosticPresentation::key_hint(binding.chord.label(), binding.command.description())?;
        builder.text(
            pane,
            Rect::new(0, row as u16 + 1, area.width, 1),
            hint.text(profile(capabilities, false)),
            capabilities.adapt_style(hint.style()),
            Some(hint.semantic_node(0)),
            false,
        );
    }
    Ok(())
}
