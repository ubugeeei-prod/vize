//! Terminal-free visual contracts for every supported Doctor TUI profile.

use vize_fresco::{
    CapabilityReason, ColorSupport, DiagnosticWorkspaceFocus, DiagnosticWorkspaceMode,
    DiagnosticWorkspacePane, HeadlessRenderer, HeadlessSnapshot, Key, KeyEvent, SemanticRole,
    TerminalCapabilities, TerminalCapabilityProbe, TerminalProfileOptions,
};

use super::{capabilities, report, sources};
use crate::commands::doctor::tui::{
    model::{DoctorTuiModel, InteractionOutcome},
    render::build_frame,
};

#[test]
fn empty_viewport_keeps_semantics_valid_and_cursor_hidden() {
    let report = report();
    let mut model = DoctorTuiModel::new(&report, 0, 0);

    let snapshot = render_snapshot(&mut model, &sources(), capabilities(0, 0, true), 0, 0);

    assert!(snapshot.viewport().is_empty());
    assert!(snapshot.cells().is_empty());
    assert!(!snapshot.cursor().visible);
    assert_eq!(snapshot.semantics().len(), 1);
    assert!(!snapshot.semantics()[0].presented);
}

#[test]
fn search_focus_exposes_value_and_exact_terminal_cursor() {
    let report = report();
    let mut model = DoctorTuiModel::new(&report, 80, 16);
    assert_eq!(
        model.handle_key(&KeyEvent::char('/')),
        InteractionOutcome::Changed
    );
    assert_eq!(
        model.handle_key(&KeyEvent::char('m')),
        InteractionOutcome::Changed
    );

    let snapshot = render_snapshot(&mut model, &sources(), capabilities(80, 16, true), 80, 16);
    let focused = snapshot.focus().unwrap();
    let focused = snapshot
        .semantics()
        .iter()
        .find(|node| node.node_id == focused)
        .unwrap();

    assert_eq!(focused.role, SemanticRole::SearchBox);
    assert_eq!(focused.name, "Finding search");
    assert_eq!(focused.state.value.as_deref(), Some("m"));
    assert!(snapshot.cursor().visible);
    assert_eq!((snapshot.cursor().x, snapshot.cursor().y), (3, 1));
}

#[test]
fn frame_contains_semantic_summary_list_detail_and_evidence() {
    let report = report();
    let sources = sources();
    let mut model = DoctorTuiModel::new(&report, 100, 18);
    let snapshot = render_snapshot(&mut model, &sources, capabilities(100, 18, true), 100, 18);
    let screen = screen_text(&snapshot);

    assert!(screen.contains("VIZE DOCTOR"));
    assert!(screen.contains("Score: 95 / 100"));
    assert!(screen.contains("VIZE_TEST_ERROR"));
    assert!(screen.contains("Severity: error"));
    assert!(screen.contains("Evidence"));
    assert!(screen.contains("component: Parent passes mutable state"));
    assert!(screen.contains("Fix safety: unavailable"));
    assert_eq!(snapshot.semantics()[0].role, SemanticRole::Application);
    assert!(
        snapshot
            .semantics()
            .iter()
            .any(|node| node.role == SemanticRole::Progress && node.name == "Score")
    );
    let focused = snapshot.focus().unwrap();
    let focused = snapshot
        .semantics()
        .iter()
        .find(|node| node.node_id == focused)
        .unwrap();
    assert_eq!(focused.role, SemanticRole::ListItem);
    assert_eq!(
        focused.name,
        "VIZE_TEST_ERROR — Mutable state crosses a contract"
    );
    assert!(focused.state.selected);
    assert_eq!(focused.state.position, Some(1));
    assert_eq!(focused.state.set_size, Some(2));
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
    let evidence_snapshot =
        render_snapshot(&mut model, &sources(), capabilities(100, 18, true), 100, 18);
    let focused = evidence_snapshot.focus().unwrap();
    let focused = evidence_snapshot
        .semantics()
        .iter()
        .find(|node| node.node_id == focused)
        .unwrap();
    assert_eq!(focused.role, SemanticRole::Group);
    assert_eq!(focused.name, "Evidence");
    assert!(focused.state.selected);
    assert_eq!(focused.state.position, Some(2));
    assert_eq!(focused.state.set_size, Some(2));

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

    let snapshot = render_snapshot(&mut model, &sources(), capabilities(50, 14, true), 50, 14);
    insta::assert_snapshot!("doctor_tui_narrow", screen_text(&snapshot));
}

#[test]
fn ascii_monochrome_profile_retains_every_meaning_without_color() {
    let report = report();
    let sources = sources();
    let mut model = DoctorTuiModel::new(&report, 100, 16);
    let snapshot = render_snapshot(&mut model, &sources, capabilities(100, 16, false), 100, 16);
    let screen = screen_text(&snapshot);

    assert!(screen.contains("x Severity: error"));
    assert!(screen.contains("x Impact: high"));
    assert!(!screen.contains('│'));
    assert!(snapshot.cells().iter().all(|cell| cell.style.fg.is_none()));
    insta::assert_snapshot!("doctor_tui_ascii_monochrome", screen);
}

#[test]
fn redirected_utf8_profile_has_a_stable_terminal_free_snapshot() {
    let report = report();
    let sources = sources();
    let mut model = DoctorTuiModel::new(&report, 100, 16);
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

    let snapshot = render_snapshot(&mut model, &sources, capabilities, 100, 16);
    let screen = screen_text(&snapshot);

    assert!(screen.contains("✕ Severity: error"));
    assert!(screen.contains('│'));
    assert!(snapshot.cells().iter().all(|cell| cell.style.fg.is_none()));
    insta::assert_snapshot!("doctor_tui_redirected_utf8", screen);
}

fn render_snapshot(
    model: &mut DoctorTuiModel<'_>,
    sources: &[crate::commands::doctor::DoctorSource],
    capabilities: TerminalCapabilities,
    width: u16,
    height: u16,
) -> HeadlessSnapshot {
    let mut frame = build_frame(model, sources, capabilities).unwrap();
    let presentation = frame.presentation().clone();
    HeadlessRenderer::new(width, height)
        .unwrap()
        .render(frame.tree_mut(), &presentation)
        .unwrap()
}

fn screen_text(snapshot: &HeadlessSnapshot) -> String {
    let mut screen = String::new();
    for y in 0..snapshot.viewport().height {
        if y > 0 {
            screen.push('\n');
        }
        let line = snapshot.row_text(y).unwrap();
        screen.push_str(line.trim_end());
    }
    screen
}
