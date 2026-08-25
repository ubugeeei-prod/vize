//! Bounded cell renderer for the interactive Doctor workspace.

mod chrome;
mod detail;
mod tree;

use vize_doctor::FindingSeverity;
use vize_fresco::{
    DiagnosticPresentation, DiagnosticPresentationProfile, DiagnosticTone,
    DiagnosticWorkspaceFocus, DiagnosticWorkspaceMode, DiagnosticWorkspacePane,
    HeadlessSemanticNode, Rect, SemanticRole, SemanticState, TerminalCapabilities,
    terminal::{Cursor, Style},
};
use vize_s0::{String, cstr};

use super::{DoctorSource, DoctorTuiError};
use crate::commands::doctor::tui::model::{DoctorTuiModel, InteractionMode, severity_label};
use chrome::{render_header, render_help};
use tree::{DoctorFrame, DoctorFrameBuilder};

#[derive(Clone)]
pub(super) struct StyledLine {
    pub(super) text: String,
    pub(super) style: Style,
    pub(super) semantic: Option<HeadlessSemanticNode>,
    pub(super) focused: bool,
}

/// Project one bounded, semantic frame from the immutable report and UI state.
///
/// The returned tree is the single source consumed by interactive rendering,
/// headless conformance snapshots, and performance benchmarks. Only visible
/// finding and detail rows are materialized.
pub(super) fn build_frame(
    model: &mut DoctorTuiModel<'_>,
    sources: &[DoctorSource],
    capabilities: TerminalCapabilities,
) -> Result<DoctorFrame, DoctorTuiError> {
    let mut builder = DoctorFrameBuilder::new(HeadlessSemanticNode::new(
        0,
        SemanticRole::Application,
        "Vize Doctor",
    ));
    let layout = model.workspace().layout();
    let viewport = Rect::sized(layout.width(), layout.height());
    if viewport.is_empty() {
        model.set_detail_rows(0);
        let mut cursor = Cursor::new();
        cursor.hide();
        return Ok(builder.finish(cursor));
    }
    render_header(&mut builder, model, capabilities)?;
    if model.mode() == InteractionMode::Help {
        render_help(&mut builder, model, capabilities)?;
        model.set_detail_rows(0);
        let mut cursor = Cursor::new();
        model.place_cursor(&mut cursor);
        return Ok(builder.finish(cursor));
    }

    let layout = model.workspace().layout();
    let active = model.workspace().active_stacked_pane();
    if layout.presents(DiagnosticWorkspacePane::Findings, active) {
        render_findings(&mut builder, model, layout.findings(), capabilities)?;
    }
    if layout.presents(DiagnosticWorkspacePane::Detail, active) {
        let lines = detail::build(model, sources, layout.detail().width, capabilities)?;
        model.set_detail_rows(lines.len());
        render_detail(&mut builder, model, layout.detail(), &lines);
    } else {
        model.set_detail_rows(0);
    }
    if layout.mode() == DiagnosticWorkspaceMode::Split {
        render_divider(&mut builder, layout, capabilities);
    }
    let mut cursor = Cursor::new();
    model.place_cursor(&mut cursor);
    Ok(builder.finish(cursor))
}

fn render_findings(
    builder: &mut DoctorFrameBuilder,
    model: &DoctorTuiModel<'_>,
    area: Rect,
    capabilities: TerminalCapabilities,
) -> Result<(), DoctorTuiError> {
    if area.is_empty() {
        return Ok(());
    }
    let has_findings = !model.finding_keys().is_empty();
    let pane = builder.container(
        builder.root(),
        area,
        Some(HeadlessSemanticNode::new(
            0,
            SemanticRole::List,
            "Doctor findings",
        )),
        !has_findings
            && model.mode() == InteractionMode::Browse
            && model.workspace().focus() == DiagnosticWorkspaceFocus::Findings,
    );
    if model.finding_keys().is_empty() {
        builder.text(
            pane,
            Rect::new(0, 0, area.width, 1),
            "No findings match the current filters.",
            capabilities.adapt_style(Style::new().dim()),
            Some(HeadlessSemanticNode::new(
                0,
                SemanticRole::Status,
                "No findings match the current filters",
            )),
            false,
        );
        return Ok(());
    }
    let selected = model.selected_finding_key();
    let focus = model.mode() == InteractionMode::Browse
        && model.workspace().focus() == DiagnosticWorkspaceFocus::Findings;
    let set_size = model.finding_keys().len() as u64;
    for (row, position) in model
        .workspace()
        .finding_window()
        .visible_range()
        .enumerate()
    {
        let Some(key) = model.finding_keys().get(position).copied() else {
            continue;
        };
        let finding = &model.report().findings()[key];
        let marker = if selected == Some(key) {
            capabilities.select_symbol("›", ">")
        } else {
            " "
        };
        let severity = DiagnosticPresentation::new(
            vize_fresco::DiagnosticPresentationKind::Severity,
            severity_label(finding.assessment.severity),
            severity_tone(finding.assessment.severity),
        )?;
        let text = cstr!(
            "{marker} {} {} — {}",
            severity.text(profile(capabilities, true)),
            finding.code,
            finding.title,
        );
        let mut style = capabilities.adapt_style(severity.style());
        if selected == Some(key) {
            style.reverse = true;
            style.bold = true;
            style.underline = focus;
        }
        let is_selected = selected == Some(key);
        let semantic = HeadlessSemanticNode::new(
            0,
            SemanticRole::ListItem,
            cstr!("{} — {}", finding.code, finding.title),
        )
        .with_description(cstr!(
            "{} severity; {}",
            severity_label(finding.assessment.severity),
            finding.message,
        ))
        .with_state(
            SemanticState::default()
                .with_set_position(position as u64 + 1, set_size)
                .with_selected(is_selected),
        );
        builder.text(
            pane,
            Rect::new(0, row as u16, area.width, 1),
            text,
            style,
            Some(semantic),
            is_selected && focus,
        );
    }
    Ok(())
}

fn render_detail(
    builder: &mut DoctorFrameBuilder,
    model: &DoctorTuiModel<'_>,
    area: Rect,
    lines: &[StyledLine],
) {
    let description = model
        .selected_finding()
        .map(|finding| finding.message.as_str())
        .unwrap_or("No finding selected");
    let pane_semantic =
        HeadlessSemanticNode::new(0, SemanticRole::Region, "Selected finding details")
            .with_description(description);
    let pane = builder.container(
        builder.root(),
        area,
        Some(pane_semantic),
        model.mode() == InteractionMode::Browse
            && model.workspace().focus() == DiagnosticWorkspaceFocus::Detail,
    );
    builder.detail_lines(
        pane,
        area.width,
        lines,
        model.workspace().detail_scroll(),
        area.height,
    );
}

fn render_divider(
    builder: &mut DoctorFrameBuilder,
    layout: vize_fresco::DiagnosticWorkspaceLayout,
    capabilities: TerminalCapabilities,
) {
    let x = layout.findings().x.saturating_add(layout.findings().width);
    let glyph = capabilities.select_symbol("│", "|");
    builder.vertical_rule(
        builder.root(),
        Rect::new(x, layout.content().y, 1, layout.content().height),
        glyph,
        capabilities.adapt_style(Style::new().dim()),
    );
}

pub(super) const fn severity_tone(severity: FindingSeverity) -> DiagnosticTone {
    match severity {
        FindingSeverity::Error => DiagnosticTone::Negative,
        FindingSeverity::Warning => DiagnosticTone::Caution,
        FindingSeverity::Notice => DiagnosticTone::Informational,
    }
}

pub(super) fn profile(
    capabilities: TerminalCapabilities,
    compact: bool,
) -> DiagnosticPresentationProfile {
    DiagnosticPresentationProfile::from(capabilities).with_compact(compact)
}
