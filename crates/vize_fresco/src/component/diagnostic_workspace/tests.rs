use std::mem::size_of;

use super::{
    DiagnosticWorkspaceFocus, DiagnosticWorkspaceMode, DiagnosticWorkspaceOptions,
    DiagnosticWorkspacePane, DiagnosticWorkspaceState,
};
use crate::component::VirtualListNavigation;

#[test]
fn documented_workspace_defaults_are_stable() {
    let options = DiagnosticWorkspaceOptions::default();
    let state = DiagnosticWorkspaceState::<u64, u64>::new(80, 6);

    assert_eq!(options.split_width, 80);
    assert_eq!(options.list_percent, 40);
    assert_eq!(options.chrome_rows, 3);
    assert_eq!(options.overscan, 2);
    assert_eq!(state.options(), options);
    assert_eq!(state.layout().mode(), DiagnosticWorkspaceMode::Split);
    assert_eq!(state.layout().content().height, 3);
    assert_eq!(state.findings().overscan(), 2);
    assert_eq!(state.evidence().overscan(), 2);
    assert_eq!(state.focus(), DiagnosticWorkspaceFocus::Findings);
}

#[test]
fn default_split_layout_reserves_chrome_and_a_divider() {
    let state = DiagnosticWorkspaceState::<u64, u64>::new(120, 30);
    let layout = state.layout();

    assert_eq!(layout.mode(), DiagnosticWorkspaceMode::Split);
    assert_eq!(layout.content(), crate::layout::Rect::new(0, 3, 120, 27));
    assert_eq!(layout.findings(), crate::layout::Rect::new(0, 3, 48, 27));
    assert_eq!(layout.detail(), crate::layout::Rect::new(49, 3, 71, 27));
    assert!(layout.presents(
        DiagnosticWorkspacePane::Findings,
        state.active_stacked_pane()
    ));
    assert!(layout.presents(DiagnosticWorkspacePane::Detail, state.active_stacked_pane()));
}

#[test]
fn narrow_layout_presents_only_the_semantically_focused_pane() {
    let mut state = DiagnosticWorkspaceState::<u64, u64>::new(79, 20);
    let layout = state.layout();

    assert_eq!(layout.mode(), DiagnosticWorkspaceMode::Stacked);
    assert_eq!(layout.findings(), layout.content());
    assert_eq!(layout.detail(), layout.content());
    assert!(layout.presents(
        DiagnosticWorkspacePane::Findings,
        state.active_stacked_pane()
    ));
    assert!(!layout.presents(DiagnosticWorkspacePane::Detail, state.active_stacked_pane()));

    assert!(state.set_focus(DiagnosticWorkspaceFocus::Detail));
    assert_eq!(state.active_stacked_pane(), DiagnosticWorkspacePane::Detail);
}

#[test]
fn ten_thousand_findings_materialize_only_viewport_plus_overscan() {
    let keys = (0_u64..10_000).collect::<Vec<_>>();
    let mut state = DiagnosticWorkspaceState::<u64, u64>::new(120, 27);

    assert!(state.reconcile_findings(&keys));
    assert!(state.select_finding(&keys, 9_000));
    let window = state.finding_window();

    assert_eq!(state.findings().selected_key(), Some(&9_000));
    assert!(window.is_visible(9_000));
    assert_eq!(window.visible_len(), 24);
    assert!(window.materialized_len() <= 28);
    assert!(size_of::<DiagnosticWorkspaceState<u64, u64>>() < 256);
}

#[test]
fn filtering_preserves_stable_selection_and_resets_only_changed_details() {
    let keys = (0_u64..10).collect::<Vec<_>>();
    let filtered = vec![0, 2, 4, 6, 8];
    let replaced = vec![0, 2, 4, 8];
    let evidence = vec![100, 101];
    let mut state = DiagnosticWorkspaceState::new(100, 20);

    let _ = state.reconcile_findings(&keys);
    let _ = state.select_finding(&keys, 6);
    let _ = state.reconcile_evidence(&evidence);
    let _ = state.navigate_evidence(&evidence, VirtualListNavigation::Next);
    let _ = state.set_detail_content_rows(100);
    let _ = state.scroll_detail(15);

    assert!(!state.reconcile_findings(&filtered));
    assert_eq!(state.findings().selected_key(), Some(&6));
    assert_eq!(state.evidence().selected_key(), Some(&101));
    assert_eq!(state.detail_scroll(), 15);

    assert!(state.reconcile_findings(&replaced));
    assert_eq!(state.findings().selected_key(), Some(&8));
    assert_eq!(state.evidence().selected_key(), None);
    assert_eq!(state.detail_scroll(), 0);
}

#[test]
fn evidence_navigation_is_stable_bounded_and_focus_aware() {
    let findings = vec![1_u64];
    let evidence = vec!["型", "🧭", "é"];
    let reordered = vec!["é", "🧭", "型"];
    let mut state = DiagnosticWorkspaceState::new(100, 20);
    let _ = state.reconcile_findings(&findings);

    assert!(!state.set_focus(DiagnosticWorkspaceFocus::Evidence));
    assert!(state.focus_next());
    assert_eq!(state.focus(), DiagnosticWorkspaceFocus::Detail);
    assert!(state.focus_next());
    assert_eq!(state.focus(), DiagnosticWorkspaceFocus::Findings);

    assert!(state.reconcile_evidence(&evidence));
    assert!(state.navigate_evidence(&evidence, VirtualListNavigation::Next));
    assert_eq!(state.evidence().selected_key(), Some(&"🧭"));
    assert!(!state.reconcile_evidence(&reordered));
    assert_eq!(state.evidence().selected_key(), Some(&"🧭"));

    assert!(state.focus_previous());
    assert_eq!(state.focus(), DiagnosticWorkspaceFocus::Evidence);
    assert!(state.reconcile_evidence(&[]));
    assert_eq!(state.focus(), DiagnosticWorkspaceFocus::Detail);
}

#[test]
fn resize_preserves_selection_and_normalizes_both_virtual_windows() {
    let findings = (0_u64..100).collect::<Vec<_>>();
    let evidence = (1_000_u64..1_100).collect::<Vec<_>>();
    let mut state = DiagnosticWorkspaceState::new(120, 30);
    let _ = state.reconcile_findings(&findings);
    let _ = state.select_finding(&findings, 99);
    let _ = state.reconcile_evidence(&evidence);
    let _ = state.select_finding(&findings, 99);

    assert!(state.resize(40, 8));
    assert_eq!(state.layout().mode(), DiagnosticWorkspaceMode::Stacked);
    assert_eq!(state.findings().selected_key(), Some(&99));
    assert_eq!(state.evidence().selected_key(), Some(&1_000));
    assert!(state.finding_window().is_visible(99));
    assert_eq!(state.finding_window().visible_len(), 5);

    assert!(state.resize(0, 0));
    assert_eq!(state.finding_window().visible_len(), 0);
    assert_eq!(state.findings().selected_key(), Some(&99));
    assert!(!state.resize(0, 0));
}

#[test]
fn detail_overflow_is_explicit_and_saturating_across_resize() {
    let mut state = DiagnosticWorkspaceState::<u64, u64>::new(100, 20);
    assert!(state.set_detail_content_rows(100));
    assert!(state.scroll_detail(isize::MAX));
    assert_eq!(state.detail_scroll(), 83);
    assert_eq!(state.detail_rows_below(), 0);

    assert!(state.resize(100, 10));
    assert_eq!(state.detail_scroll(), 83);
    assert_eq!(state.detail_rows_below(), 10);
    assert!(state.scroll_detail(isize::MIN));
    assert_eq!(state.detail_scroll(), 0);
}

#[test]
fn options_are_normalized_without_invalid_geometry() {
    let state = DiagnosticWorkspaceState::<u64, u64>::with_options(
        5,
        2,
        DiagnosticWorkspaceOptions {
            split_width: 0,
            list_percent: 100,
            chrome_rows: 50,
            overscan: 9,
        },
    );

    assert_eq!(state.options().split_width, 3);
    assert_eq!(state.options().list_percent, 90);
    assert_eq!(state.layout().content().height, 0);
    assert_eq!(state.layout().detail().width, 1);
    assert_eq!(state.findings().overscan(), 9);
}
