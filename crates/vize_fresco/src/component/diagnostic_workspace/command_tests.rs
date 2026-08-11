use super::{
    DiagnosticWorkspaceAction, DiagnosticWorkspaceCommand as Command,
    DiagnosticWorkspaceCommandOutcome as Outcome, DiagnosticWorkspaceFocus,
    DiagnosticWorkspaceState,
};

#[test]
fn finding_commands_are_stable_bounded_and_page_aware() {
    let findings = (0_u64..100).collect::<Vec<_>>();
    let mut state = DiagnosticWorkspaceState::<u64, u64>::new(120, 10);

    assert_eq!(
        state.apply_command(Command::NextFinding, &findings, &[]),
        Outcome::Changed
    );
    assert_eq!(state.findings().selected_key(), Some(&1));
    assert_eq!(
        state.apply_command(Command::PageDownFindings, &findings, &[]),
        Outcome::Changed
    );
    assert_eq!(state.findings().selected_key(), Some(&7));
    assert_eq!(
        state.apply_command(Command::LastFinding, &findings, &[]),
        Outcome::Changed
    );
    assert_eq!(state.findings().selected_key(), Some(&99));
    assert_eq!(
        state.apply_command(Command::NextFinding, &findings, &[]),
        Outcome::Boundary
    );
    assert_eq!(state.findings().selected_key(), Some(&99));
}

#[test]
fn evidence_commands_focus_available_evidence_and_fail_closed_when_empty() {
    let findings = [1_u64];
    let evidence = [10_u64, 11, 12];
    let mut state = DiagnosticWorkspaceState::new(100, 20);
    let _ = state.reconcile_findings(&findings);

    assert_eq!(
        state.apply_command(Command::NextEvidence, &findings, &[]),
        Outcome::Boundary
    );
    assert_eq!(state.focus(), DiagnosticWorkspaceFocus::Findings);

    let _ = state.reconcile_evidence(&evidence);
    assert_eq!(
        state.apply_command(Command::NextEvidence, &findings, &evidence),
        Outcome::Changed
    );
    assert_eq!(state.evidence().selected_key(), Some(&11));
    assert_eq!(state.focus(), DiagnosticWorkspaceFocus::Evidence);
    assert_eq!(
        state.apply_command(Command::PreviousEvidence, &findings, &evidence),
        Outcome::Changed
    );
    assert_eq!(state.evidence().selected_key(), Some(&10));
}

#[test]
fn detail_scroll_and_focus_commands_use_explicit_non_wrapping_boundaries() {
    let mut state = DiagnosticWorkspaceState::<u64, u64>::new(100, 20);
    let _ = state.set_detail_content_rows(100);

    assert_eq!(
        state.apply_command(Command::PageDownDetail, &[], &[]),
        Outcome::Changed
    );
    assert_eq!(state.detail_scroll(), 16);
    assert_eq!(
        state.apply_command(Command::ScrollDetailUp, &[], &[]),
        Outcome::Changed
    );
    assert_eq!(state.detail_scroll(), 15);
    assert_eq!(
        state.apply_command(Command::FocusNext, &[], &[]),
        Outcome::Changed
    );
    assert_eq!(state.focus(), DiagnosticWorkspaceFocus::Detail);

    let _ = state.scroll_detail(isize::MIN);
    assert_eq!(
        state.apply_command(Command::ScrollDetailUp, &[], &[]),
        Outcome::Boundary
    );
}

#[test]
fn application_actions_dispatch_without_mutating_workspace_state() {
    let findings = [7_u64];
    let mut state = DiagnosticWorkspaceState::<u64, u64>::new(100, 20);
    let cases = [
        (
            Command::NextCategory,
            DiagnosticWorkspaceAction::NextCategory,
        ),
        (
            Command::PreviousCategory,
            DiagnosticWorkspaceAction::PreviousCategory,
        ),
        (
            Command::NextSeverity,
            DiagnosticWorkspaceAction::NextSeverity,
        ),
        (
            Command::PreviousSeverity,
            DiagnosticWorkspaceAction::PreviousSeverity,
        ),
        (Command::Search, DiagnosticWorkspaceAction::Search),
        (Command::Help, DiagnosticWorkspaceAction::Help),
        (Command::Exit, DiagnosticWorkspaceAction::Exit),
    ];

    for (command, action) in cases {
        assert_eq!(
            state.apply_command(command, &findings, &[]),
            Outcome::Dispatch(action)
        );
    }
    assert_eq!(state.focus(), DiagnosticWorkspaceFocus::Findings);
    assert_eq!(state.findings().selected_key(), None);
}

#[test]
fn open_source_requires_a_selected_finding() {
    let findings = [7_u64];
    let mut state = DiagnosticWorkspaceState::<u64, u64>::new(100, 20);

    assert_eq!(
        state.apply_command(Command::OpenSource, &findings, &[]),
        Outcome::Boundary
    );
    let _ = state.reconcile_findings(&findings);
    assert_eq!(
        state.apply_command(Command::OpenSource, &findings, &[]),
        Outcome::Dispatch(DiagnosticWorkspaceAction::OpenSource)
    );
}

#[test]
fn command_names_descriptions_and_wire_values_are_stable() {
    let command = Command::PageDownFindings;
    assert_eq!(command.as_str(), "page-down-findings");
    assert_eq!(command.description(), "Next page of findings");
    assert_eq!(
        serde_json::to_string(&command).unwrap(),
        "\"page-down-findings\""
    );
    assert_eq!(
        serde_json::from_str::<Command>("\"page-down-findings\"").unwrap(),
        command
    );
}
