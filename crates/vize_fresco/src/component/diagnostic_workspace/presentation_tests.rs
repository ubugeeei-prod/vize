use super::{
    DiagnosticPresentation, DiagnosticPresentationError, DiagnosticPresentationKind,
    DiagnosticPresentationProfile, DiagnosticTone,
};
use crate::{
    component::BoxNode,
    headless::{HeadlessPresentation, HeadlessRenderer, HeadlessSemanticNode, SemanticRole},
    layout::Dimension,
    render::{NodeKind, RenderTree},
    terminal::{Color, TerminalCapabilities, TerminalCapabilityProbe, TerminalProfileOptions},
};

fn presentation(
    kind: DiagnosticPresentationKind,
    value: &str,
    tone: DiagnosticTone,
) -> DiagnosticPresentation {
    DiagnosticPresentation::new(kind, value, tone).unwrap()
}

fn complete_presentations() -> Vec<DiagnosticPresentation> {
    vec![
        presentation(
            DiagnosticPresentationKind::Status,
            "Ready",
            DiagnosticTone::Positive,
        ),
        DiagnosticPresentation::score(92, 100, DiagnosticTone::Positive).unwrap(),
        presentation(
            DiagnosticPresentationKind::Severity,
            "Error",
            DiagnosticTone::Negative,
        ),
        presentation(
            DiagnosticPresentationKind::Confidence,
            "Certain",
            DiagnosticTone::Informational,
        ),
        presentation(
            DiagnosticPresentationKind::Impact,
            "Application",
            DiagnosticTone::Caution,
        ),
        DiagnosticPresentation::code_location("src/App.vue", 12, 7).unwrap(),
        DiagnosticPresentation::evidence("Missing dependency edge", 2, 4).unwrap(),
        presentation(
            DiagnosticPresentationKind::FixSafety,
            "Review required",
            DiagnosticTone::Caution,
        ),
        DiagnosticPresentation::key_hint("j", "Next finding").unwrap(),
    ]
}

#[test]
fn every_required_presentation_has_a_stable_label_and_semantic_role() {
    let expected = [
        (
            DiagnosticPresentationKind::Status,
            "Status",
            SemanticRole::Status,
        ),
        (
            DiagnosticPresentationKind::Score,
            "Score",
            SemanticRole::Progress,
        ),
        (
            DiagnosticPresentationKind::Severity,
            "Severity",
            SemanticRole::Alert,
        ),
        (
            DiagnosticPresentationKind::Confidence,
            "Confidence",
            SemanticRole::Status,
        ),
        (
            DiagnosticPresentationKind::Impact,
            "Impact",
            SemanticRole::Status,
        ),
        (
            DiagnosticPresentationKind::CodeLocation,
            "Location",
            SemanticRole::Code,
        ),
        (
            DiagnosticPresentationKind::Evidence,
            "Evidence",
            SemanticRole::Group,
        ),
        (
            DiagnosticPresentationKind::FixSafety,
            "Fix safety",
            SemanticRole::Status,
        ),
        (
            DiagnosticPresentationKind::KeyHint,
            "Key hint",
            SemanticRole::Code,
        ),
    ];

    for ((presentation, (kind, label, role)), node_id) in
        complete_presentations().iter().zip(expected).zip(1_u64..)
    {
        assert_eq!(presentation.kind(), kind);
        assert_eq!(kind.label(), label);
        let semantic = presentation.semantic_node(node_id);
        assert_eq!(semantic.role, role);
        assert_eq!(semantic.name, label);
        assert_eq!(semantic.state.value.as_deref(), Some(presentation.value()));
    }
}

#[test]
fn visual_text_never_relies_on_color_and_has_exact_ascii_fallbacks() {
    let passing = presentation(
        DiagnosticPresentationKind::Status,
        "Healthy",
        DiagnosticTone::Positive,
    );
    assert_eq!(
        passing.text(DiagnosticPresentationProfile::unicode()),
        "✓ Status: Healthy"
    );
    assert_eq!(
        passing.text(DiagnosticPresentationProfile::ascii()),
        "+ Status: Healthy"
    );
    assert_eq!(
        passing.text(DiagnosticPresentationProfile::ascii().with_compact(true)),
        "+ Healthy"
    );

    let failing = presentation(
        DiagnosticPresentationKind::Severity,
        "Critical",
        DiagnosticTone::Negative,
    );
    assert_eq!(
        failing.text(DiagnosticPresentationProfile::unicode()),
        "✕ Severity: Critical"
    );
    assert_eq!(
        failing.text(DiagnosticPresentationProfile::ascii()),
        "x Severity: Critical"
    );
}

#[test]
fn text_inputs_are_trimmed_and_empty_values_fail_closed() {
    let normalized = DiagnosticPresentation::new(
        DiagnosticPresentationKind::Status,
        "  Ready  ",
        DiagnosticTone::Positive,
    )
    .unwrap()
    .with_description("  Analysis is complete.  ")
    .unwrap();
    assert_eq!(normalized.value(), "Ready");
    assert_eq!(normalized.description(), Some("Analysis is complete."));

    assert_eq!(
        DiagnosticPresentation::new(
            DiagnosticPresentationKind::Status,
            " \n ",
            DiagnosticTone::Neutral,
        ),
        Err(DiagnosticPresentationError::EmptyText { field: "value" })
    );
    assert_eq!(
        DiagnosticPresentation::key_hint(" ", "Next"),
        Err(DiagnosticPresentationError::EmptyText { field: "key" })
    );
    assert_eq!(
        normalized.clone().with_description("\t"),
        Err(DiagnosticPresentationError::EmptyText {
            field: "description"
        })
    );
}

#[test]
fn score_location_and_evidence_boundaries_are_explicit() {
    let score = DiagnosticPresentation::score(92, 100, DiagnosticTone::Positive).unwrap();
    assert_eq!(score.value(), "92 / 100");
    assert_eq!(score.score_bounds(), Some((92, 100)));
    assert_eq!(
        DiagnosticPresentation::score(1, 0, DiagnosticTone::Negative),
        Err(DiagnosticPresentationError::InvalidScore {
            value: 1,
            maximum: 0
        })
    );
    assert_eq!(
        DiagnosticPresentation::score(101, 100, DiagnosticTone::Negative),
        Err(DiagnosticPresentationError::InvalidScore {
            value: 101,
            maximum: 100
        })
    );

    let location = DiagnosticPresentation::code_location("src/日本語.vue", 12, 7).unwrap();
    assert_eq!(location.value(), "src/日本語.vue:12:7");
    assert_eq!(
        DiagnosticPresentation::code_location("src/App.vue", 0, 1),
        Err(DiagnosticPresentationError::InvalidCodeLocation)
    );

    let evidence = DiagnosticPresentation::evidence("Related component", 2, 4).unwrap();
    assert_eq!(evidence.evidence_position(), Some((2, 4)));
    let semantic = evidence.semantic_node(5);
    assert_eq!(semantic.state.position, Some(2));
    assert_eq!(semantic.state.set_size, Some(4));
    assert_eq!(
        DiagnosticPresentation::evidence("Related", 5, 4),
        Err(DiagnosticPresentationError::InvalidEvidencePosition {
            position: 5,
            set_size: 4
        })
    );
}

#[test]
fn render_nodes_preserve_text_and_tone_attributes() {
    let warning = presentation(
        DiagnosticPresentationKind::Impact,
        "Production",
        DiagnosticTone::Caution,
    );
    let node = warning.render_node(7, DiagnosticPresentationProfile::ascii());
    assert_eq!(node.appearance.fg, Some(Color::Yellow));
    assert!(node.appearance.bold);
    match node.kind {
        NodeKind::Text(content) => assert_eq!(content.text, "! Impact: Production"),
        other => panic!("expected presentation text node, got {other:?}"),
    }

    let neutral = DiagnosticPresentation::key_hint("?", "Help").unwrap();
    let node = neutral.render_node(8, DiagnosticPresentationProfile::unicode());
    assert_eq!(node.appearance.fg, None);
    assert!(!node.appearance.bold);
}

#[test]
fn terminal_capabilities_select_text_density_symbols_and_safe_color() {
    let presentation = presentation(
        DiagnosticPresentationKind::Severity,
        "Blocking",
        DiagnosticTone::Negative,
    );
    let redirected_ascii = TerminalCapabilities::resolve(
        &TerminalCapabilityProbe::new(42, 18, false).with_locale("C"),
        TerminalProfileOptions::default(),
    );
    let profile = DiagnosticPresentationProfile::from(redirected_ascii);
    assert!(!profile.uses_unicode());
    assert!(profile.is_compact());

    let node = presentation.render_node_for_capabilities(9, redirected_ascii);
    assert_eq!(node.appearance.fg, None);
    assert!(node.appearance.bold);
    match node.kind {
        NodeKind::Text(content) => assert_eq!(content.text, "x Blocking"),
        other => panic!("expected capability-adapted text node, got {other:?}"),
    }

    let wide_unicode = TerminalCapabilities::resolve(
        &TerminalCapabilityProbe::new(120, 40, true)
            .with_term("xterm-256color")
            .with_locale("ja_JP.UTF-8"),
        TerminalProfileOptions::default(),
    );
    let node = presentation.render_node_for_capabilities(10, wide_unicode);
    assert_eq!(node.appearance.fg, Some(Color::Red));
    match node.kind {
        NodeKind::Text(content) => assert_eq!(content.text, "✕ Severity: Blocking"),
        other => panic!("expected capability-adapted text node, got {other:?}"),
    }
}

#[test]
fn headless_snapshot_covers_all_visual_and_accessible_presentations() {
    let presentations = complete_presentations();
    let mut tree = RenderTree::new();
    let root = tree.next_id();
    tree.insert_root(
        BoxNode::new()
            .column()
            .width_percent(100.0)
            .height_percent(100.0)
            .build(root),
    );
    let mut semantics = vec![HeadlessSemanticNode::new(
        root,
        SemanticRole::Application,
        "Diagnostics",
    )];

    for presentation in &presentations {
        let node_id = tree.next_id();
        let mut node = presentation.render_node(node_id, DiagnosticPresentationProfile::ascii());
        node.style.width = Dimension::Percent(100.0);
        node.style.height = Dimension::Points(1.0);
        node.style.flex_shrink = 0.0;
        tree.insert(node);
        tree.add_child(root, node_id);
        semantics.push(presentation.semantic_node(node_id));
    }

    let snapshot = HeadlessRenderer::new(80, 10)
        .unwrap()
        .render(
            &mut tree,
            &HeadlessPresentation::new().with_semantics(semantics),
        )
        .unwrap();

    assert_eq!(snapshot.semantics().len(), presentations.len() + 1);
    assert_eq!(snapshot.semantics()[1].name, "Status");
    assert_eq!(snapshot.semantics()[2].role, SemanticRole::Progress);
    assert_eq!(snapshot.semantics()[7].state.position, Some(2));
    let screen = snapshot.screen_text();
    assert!(screen.contains("+ Status: Ready"), "{screen:?}");
    assert!(screen.contains("- Key hint: j: Next finding"), "{screen:?}");
}

#[test]
fn presentation_contract_round_trips_without_losing_semantics() {
    let presentation = DiagnosticPresentation::evidence("型の証拠 🧭", 3, 9)
        .unwrap()
        .with_description("Combining mark: e\u{301}")
        .unwrap();
    let json = serde_json::to_string(&presentation).unwrap();
    let decoded: DiagnosticPresentation = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, presentation);
    assert_eq!(decoded.semantic_node(9), presentation.semantic_node(9));
}
