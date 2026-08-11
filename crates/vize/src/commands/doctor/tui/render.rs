//! Bounded cell renderer for the interactive Doctor workspace.

mod detail;

use vize_carton::{String, cstr};
use vize_doctor::FindingSeverity;
use vize_fresco::{
    DiagnosticPresentation, DiagnosticPresentationProfile, DiagnosticTone,
    DiagnosticWorkspaceFocus, DiagnosticWorkspaceMode, DiagnosticWorkspacePane, Rect,
    TerminalCapabilities, TextWrap,
    terminal::{Buffer, Color, Style},
    text::WrapMode,
};

use super::{DoctorSource, DoctorTuiError};
use crate::commands::doctor::tui::model::{DoctorTuiModel, InteractionMode, severity_label};

#[derive(Clone)]
pub(super) struct StyledLine {
    pub(super) text: String,
    pub(super) style: Style,
}

pub(super) fn render_frame(
    buffer: &mut Buffer,
    model: &mut DoctorTuiModel<'_>,
    sources: &[DoctorSource],
    capabilities: TerminalCapabilities,
) -> Result<(), DoctorTuiError> {
    buffer.clear();
    if buffer.width() == 0 || buffer.height() == 0 {
        return Ok(());
    }
    render_header(buffer, model, capabilities)?;
    if model.mode() == InteractionMode::Help {
        render_help(buffer, model, capabilities)?;
        model.set_detail_rows(0);
        return Ok(());
    }

    let layout = model.workspace().layout();
    let active = model.workspace().active_stacked_pane();
    if layout.presents(DiagnosticWorkspacePane::Findings, active) {
        render_findings(buffer, model, layout.findings(), capabilities)?;
    }
    if layout.presents(DiagnosticWorkspacePane::Detail, active) {
        let lines = detail::build(model, sources, layout.detail().width, capabilities)?;
        model.set_detail_rows(lines.len());
        render_detail(buffer, model, layout.detail(), &lines);
    } else {
        model.set_detail_rows(0);
    }
    if layout.mode() == DiagnosticWorkspaceMode::Split {
        render_divider(buffer, layout, capabilities);
    }
    Ok(())
}

fn render_header(
    buffer: &mut Buffer,
    model: &DoctorTuiModel<'_>,
    capabilities: TerminalCapabilities,
) -> Result<(), DoctorTuiError> {
    let width = buffer.width();
    let title_style = capabilities.adapt_style(Style::new().fg(Color::Cyan).bold());
    write_clipped(buffer, 0, 0, "VIZE DOCTOR", title_style, width);

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
    write_clipped(
        buffer,
        13,
        0,
        &score_text,
        capabilities.adapt_style(presentation.style()),
        width.saturating_sub(13),
    );

    if buffer.height() > 1 {
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
        write_clipped(
            buffer,
            0,
            1,
            &status,
            capabilities.adapt_style(Style::new().fg(Color::Gray)),
            width,
        );
    }
    if buffer.height() > 2 {
        let separator = capabilities.select_symbol("─", "-");
        buffer.fill(
            Rect::new(0, 2, width, 1),
            separator.chars().next().unwrap_or('-'),
            Style::new(),
        );
        let hints = "j/k navigate  Tab focus  c/C category  s/S severity  / search  ? help  q quit";
        write_clipped(
            buffer,
            1,
            2,
            hints,
            capabilities.adapt_style(Style::new().dim()),
            width.saturating_sub(2),
        );
    }
    Ok(())
}

fn render_findings(
    buffer: &mut Buffer,
    model: &DoctorTuiModel<'_>,
    area: Rect,
    capabilities: TerminalCapabilities,
) -> Result<(), DoctorTuiError> {
    if area.is_empty() {
        return Ok(());
    }
    if model.finding_keys().is_empty() {
        write_clipped(
            buffer,
            area.x,
            area.y,
            "No findings match the current filters.",
            capabilities.adapt_style(Style::new().dim()),
            area.width,
        );
        return Ok(());
    }
    let selected = model.selected_finding_key();
    let focus = model.workspace().focus() == DiagnosticWorkspaceFocus::Findings;
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
        write_clipped(
            buffer,
            area.x,
            area.y.saturating_add(row as u16),
            &text,
            style,
            area.width,
        );
    }
    Ok(())
}

fn render_detail(
    buffer: &mut Buffer,
    model: &DoctorTuiModel<'_>,
    area: Rect,
    lines: &[StyledLine],
) {
    let start = model.workspace().detail_scroll();
    for (row, line) in lines
        .iter()
        .skip(start)
        .take(usize::from(area.height))
        .enumerate()
    {
        write_clipped(
            buffer,
            area.x,
            area.y.saturating_add(row as u16),
            &line.text,
            line.style,
            area.width,
        );
    }
}

fn render_divider(
    buffer: &mut Buffer,
    layout: vize_fresco::DiagnosticWorkspaceLayout,
    capabilities: TerminalCapabilities,
) {
    let x = layout.findings().x.saturating_add(layout.findings().width);
    let glyph = capabilities
        .select_symbol("│", "|")
        .chars()
        .next()
        .unwrap_or('|');
    buffer.fill(
        Rect::new(x, layout.content().y, 1, layout.content().height),
        glyph,
        capabilities.adapt_style(Style::new().dim()),
    );
}

fn render_help(
    buffer: &mut Buffer,
    model: &DoctorTuiModel<'_>,
    capabilities: TerminalCapabilities,
) -> Result<(), DoctorTuiError> {
    let area = model.workspace().layout().content();
    write_clipped(
        buffer,
        area.x,
        area.y,
        "Keyboard help — Esc, ? or F1 closes this view",
        capabilities.adapt_style(Style::new().fg(Color::Cyan).bold()),
        area.width,
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
        write_clipped(
            buffer,
            area.x,
            area.y.saturating_add(row as u16 + 1),
            &hint.text(profile(capabilities, false)),
            capabilities.adapt_style(hint.style()),
            area.width,
        );
    }
    Ok(())
}

pub(super) fn write_clipped(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    text: &str,
    style: Style,
    width: u16,
) {
    if width == 0 || y >= buffer.height() {
        return;
    }
    if let Some(line) = TextWrap::wrap(text, usize::from(width), WrapMode::TruncateEnd).first() {
        buffer.set_string(x, y, line, style);
    }
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
