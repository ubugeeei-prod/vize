use std::path::PathBuf;

use vize_carton::String;
use vize_doctor::{
    AnalysisProvenance, DoctorCategory, DoctorFinding, DoctorReport, EvidenceKind,
    FindingAssessment, FindingConfidence, FindingEvidence, FindingImpact, FindingSeverity,
    HealthPenalty, RuleCost, SourceLocation,
};
use vize_fresco::{
    Buffer, ColorPreference, DiagnosticWorkspaceFocus, DiagnosticWorkspaceMode,
    DiagnosticWorkspacePane, FeaturePreference, Key, KeyEvent, TerminalCapabilities,
    TerminalCapabilityProbe, TerminalProfileOptions,
};

use super::{
    DoctorSource, editor_command,
    model::{DoctorTuiModel, InteractionMode, InteractionOutcome},
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
fn navigation_filters_and_incremental_search_preserve_stable_selection() {
    let report = report();
    let mut model = DoctorTuiModel::new(&report, 100, 18);
    let first = model.selected_finding_key();

    assert_eq!(
        model.handle_key(&KeyEvent::key(Key::Down)),
        InteractionOutcome::Changed
    );
    assert_ne!(model.selected_finding_key(), first);

    assert_eq!(
        model.handle_key(&KeyEvent::char('c')),
        InteractionOutcome::Changed
    );
    assert_eq!(model.category_label(), "correctness");
    assert_eq!(model.finding_keys().len(), 1);

    let mut search = DoctorTuiModel::new(&report, 100, 18);
    assert_eq!(
        search.handle_key(&KeyEvent::char('/')),
        InteractionOutcome::Changed
    );
    for character in "accessibility".chars() {
        assert_eq!(
            search.handle_key(&KeyEvent::char(character)),
            InteractionOutcome::Changed
        );
    }
    assert_eq!(search.mode(), InteractionMode::Search);
    assert_eq!(search.finding_keys().len(), 1);
    assert_eq!(
        search.handle_key(&KeyEvent::key(Key::Esc)),
        InteractionOutcome::Changed
    );
    assert_eq!(search.mode(), InteractionMode::Browse);
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
    let _ = model.workspace().active_stacked_pane();
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
fn source_positions_are_unicode_safe_and_one_based() {
    let source = "<template>\n  <p>保存🙂</p>\n</template>";
    let start = source.find("保存🙂").unwrap() as u32;
    let finding = finding(
        "VIZE_UNICODE",
        DoctorCategory::Correctness,
        FindingSeverity::Warning,
        "Unicode source",
        SourceLocation::new("src/App.vue", start, start + 10),
    );
    let report = DoctorReport::new(".", [finding]);
    let sources = [DoctorSource {
        path: PathBuf::from("src/App.vue"),
        source: source.into(),
    }];
    let model = DoctorTuiModel::new(&report, 80, 16);

    assert_eq!(model.source_position(&sources), (2, 6));
}

#[test]
fn virtual_window_materialization_stays_bounded_for_large_reports() {
    let report = large_report();
    let model = DoctorTuiModel::new(&report, 120, 30);

    assert_eq!(model.finding_keys().len(), 10_000);
    assert!(model.workspace().finding_window().materialized_len() <= 31);
}

#[cfg(feature = "profiling")]
#[test]
fn differential_frame_costs_stay_inside_explicit_budgets() {
    let report = large_report();
    let mut tui = super::DoctorTuiBenchmark::new(&report, 120, 30, capabilities(120, 30, true));

    let first_frame = tui.render();
    assert!(first_frame.changed_cells() <= 1_600, "{first_frame:?}");
    assert!(first_frame.bytes_written() <= 8_192, "{first_frame:?}");

    let selection_frame = tui.toggle_selection_and_render();
    assert!(
        selection_frame.changed_cells() <= 128,
        "{selection_frame:?}"
    );
    assert!(
        selection_frame.bytes_written() <= 512,
        "{selection_frame:?}"
    );
}

#[test]
fn source_actions_preserve_editor_flags_and_use_native_location_syntax() {
    let path = std::path::Path::new("/workspace/Unicode view.vue");
    let code = editor_command("code --wait", path, 12, 7).unwrap();
    let code_args: Vec<_> = code.get_args().collect();

    assert_eq!(code.get_program(), "code");
    assert_eq!(code_args[0], "--wait");
    assert_eq!(code_args[1], "--goto");
    assert_eq!(code_args[2], "/workspace/Unicode view.vue:12:7");

    let terminal = editor_command("nvim -f", path, 12, 7).unwrap();
    let terminal_args: Vec<_> = terminal.get_args().collect();
    assert_eq!(terminal.get_program(), "nvim");
    assert_eq!(terminal_args[0], "-f");
    assert_eq!(terminal_args[1], "+12");
    assert_eq!(terminal_args[2], path);
}

fn report() -> DoctorReport {
    let error = finding(
        "VIZE_TEST_ERROR",
        DoctorCategory::Correctness,
        FindingSeverity::Error,
        "Mutable state crosses a contract",
        SourceLocation::new("src/Parent.vue", 20, 25),
    )
    .with_evidence(FindingEvidence::new(
        EvidenceKind::Component,
        "Parent passes mutable state",
    ))
    .with_evidence(FindingEvidence::new(
        EvidenceKind::Reactivity,
        "Child destructures the reactive prop",
    ));
    let warning = finding(
        "VIZE_TEST_A11Y",
        DoctorCategory::Accessibility,
        FindingSeverity::Warning,
        "Accessibility name is missing",
        SourceLocation::new("src/Button.vue", 10, 16),
    );
    DoctorReport::new(".", [warning, error])
}

fn large_report() -> DoctorReport {
    DoctorReport::new(
        ".",
        (0..10_000).map(|index| {
            finding(
                "VIZE_MANY",
                DoctorCategory::Performance,
                FindingSeverity::Notice,
                "Repeated finding",
                SourceLocation::new("src/App.vue", index, index + 1),
            )
        }),
    )
}

fn finding(
    code: &str,
    category: DoctorCategory,
    severity: FindingSeverity,
    title: &str,
    location: SourceLocation,
) -> DoctorFinding {
    DoctorFinding::new(
        code,
        category,
        FindingAssessment::new(
            severity,
            FindingConfidence::High,
            FindingImpact::High,
            HealthPenalty::new(15, "Test penalty"),
        ),
        location,
        title,
        "A detailed explanation with a concrete next action.",
        AnalysisProvenance::new("test-analysis", RuleCost::Low),
    )
}

fn sources() -> Vec<DoctorSource> {
    vec![
        DoctorSource {
            path: PathBuf::from("src/Parent.vue"),
            source: "<template>\n  <Child :item=\"state\" />\n</template>".into(),
        },
        DoctorSource {
            path: PathBuf::from("src/Button.vue"),
            source: "<template><button /></template>".into(),
        },
    ]
}

fn capabilities(width: u16, height: u16, unicode: bool) -> TerminalCapabilities {
    TerminalCapabilities::resolve(
        &TerminalCapabilityProbe::new(width, height, true).with_locale(if unicode {
            "C.UTF-8"
        } else {
            "C"
        }),
        TerminalProfileOptions {
            color: ColorPreference::Never,
            unicode: if unicode {
                FeaturePreference::Always
            } else {
                FeaturePreference::Never
            },
            interactive: FeaturePreference::Always,
            narrow_width: 60,
        },
    )
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
