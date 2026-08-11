//! Terminal-free visual contracts for every supported Doctor TUI profile.

use vize_carton::String;
use vize_fresco::{
    Buffer, CapabilityReason, ColorSupport, DiagnosticWorkspaceFocus, DiagnosticWorkspaceMode,
    DiagnosticWorkspacePane, Key, KeyEvent, TerminalCapabilities, TerminalCapabilityProbe,
    TerminalProfileOptions,
};

use super::{capabilities, report, sources};
use crate::commands::doctor::tui::{
    model::{DoctorTuiModel, InteractionOutcome},
    render::render_frame,
};

#[test]
fn frame_contains_semantic_summary_list_detail_and_evidence() {
    let report = report();
    let sources = sources();
    let mut model = DoctorTuiModel::new(&report, 100, 18);
    let mut buffer = Buffer::new(100, 18);

    render_frame(
        &mut buffer,
        &mut model,
        &sources,
        capabilities(100, 18, true),
    )
    .unwrap();
    let screen = screen_text(&buffer);

    assert!(screen.contains("VIZE DOCTOR"));
    assert!(screen.contains("Score: 95 / 100"));
    assert!(screen.contains("VIZE_TEST_ERROR"));
    assert!(screen.contains("Severity: error"));
    assert!(screen.contains("Evidence"));
    assert!(screen.contains("component: Parent passes mutable state"));
    assert!(screen.contains("Fix safety: unavailable"));
    insta::assert_snapshot!("doctor_tui_wide", screen);
}

#[test]
fn evidence_focus_and_narrow_stacked_navigation_use_fresco_state() {
    let report = report();
    let mut model = DoctorTuiModel::new(&report, 100, 18);

    assert_eq!(
        model.handle_key(&KeyEvent::char(']')),
        InteractionOutcome::Changed
    );
    assert_eq!(model.selected_evidence_key(), Some(1));
    assert_eq!(
        model.workspace().focus(),
        DiagnosticWorkspaceFocus::Evidence
    );

    model.resize(50, 14);
    assert_eq!(
        model.workspace().layout().mode(),
        DiagnosticWorkspaceMode::Stacked
    );
    assert_eq!(
        model.workspace().active_stacked_pane(),
        DiagnosticWorkspacePane::Detail
    );
    assert_eq!(
        model.handle_key(&KeyEvent::key(Key::Tab)),
        InteractionOutcome::Changed
    );
    assert_eq!(
        model.workspace().active_stacked_pane(),
        DiagnosticWorkspacePane::Findings
    );

    let mut buffer = Buffer::new(50, 14);
    render_frame(
        &mut buffer,
        &mut model,
        &sources(),
        capabilities(50, 14, true),
    )
    .unwrap();
    insta::assert_snapshot!("doctor_tui_narrow", screen_text(&buffer));
}

#[test]
fn ascii_monochrome_profile_retains_every_meaning_without_color() {
    let report = report();
    let sources = sources();
    let mut model = DoctorTuiModel::new(&report, 100, 16);
    let mut buffer = Buffer::new(100, 16);

    render_frame(
        &mut buffer,
        &mut model,
        &sources,
        capabilities(100, 16, false),
    )
    .unwrap();
    let screen = screen_text(&buffer);

    assert!(screen.contains("x Severity: error"));
    assert!(screen.contains("x Impact: high"));
    assert!(!screen.contains('│'));
    assert!(buffer.iter().all(|(_, _, cell)| cell.style.fg.is_none()));
    insta::assert_snapshot!("doctor_tui_ascii_monochrome", screen);
}

#[test]
fn redirected_utf8_profile_has_a_stable_terminal_free_snapshot() {
    let report = report();
    let sources = sources();
    let mut model = DoctorTuiModel::new(&report, 100, 16);
    let mut buffer = Buffer::new(100, 16);
    let capabilities = TerminalCapabilities::resolve(
        &TerminalCapabilityProbe::new(100, 16, false).with_locale("ja_JP.UTF-8"),
        TerminalProfileOptions::default(),
    );

    assert!(capabilities.is_redirected());
    assert_eq!(capabilities.color().value(), ColorSupport::Monochrome);
    assert_eq!(
        capabilities.color().reason(),
        CapabilityReason::RedirectedOutput
    );
    assert!(capabilities.unicode().value());
    assert_eq!(
        capabilities.unicode().reason(),
        CapabilityReason::Utf8Locale
    );
    assert!(!capabilities.interactive().value());
    assert_eq!(
        capabilities.interactive().reason(),
        CapabilityReason::RedirectedOutput
    );

    render_frame(&mut buffer, &mut model, &sources, capabilities).unwrap();
    let screen = screen_text(&buffer);

    assert!(screen.contains("✕ Severity: error"));
    assert!(screen.contains('│'));
    assert!(buffer.iter().all(|(_, _, cell)| cell.style.fg.is_none()));
    insta::assert_snapshot!("doctor_tui_redirected_utf8", screen);
}

fn screen_text(buffer: &Buffer) -> String {
    let mut screen = String::new("");
    for y in 0..buffer.height() {
        if y > 0 {
            screen.push('\n');
        }
        let mut line = String::new("");
        for x in 0..buffer.width() {
            let cell = buffer.get(x, y).unwrap();
            if !cell.is_continuation {
                line.push_str(&cell.symbol);
            }
        }
        screen.push_str(line.trim_end());
    }
    screen
}
