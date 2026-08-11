use super::{
    AnnouncementPoliteness, HeadlessAnnouncement, HeadlessPresentation, HeadlessRenderError,
    HeadlessRenderer, HeadlessSemanticNode, SemanticRole, SemanticState,
};
use crate::{
    component::{BoxNode, TextNode},
    layout::Dimension,
    render::RenderTree,
    terminal::{Color, Cursor, Style},
};

fn sized_text(id: u64, text: &str) -> crate::render::RenderNode {
    let mut node = TextNode::new(text).build(id);
    node.style.width = Dimension::Percent(100.0);
    node.style.height = Dimension::Points(1.0);
    node.style.flex_shrink = 0.0;
    node
}

fn diagnostic_tree() -> RenderTree {
    let mut tree = RenderTree::new();
    let root_id = tree.next_id();
    let root = BoxNode::new()
        .column()
        .width_percent(100.0)
        .height_percent(100.0)
        .build(root_id);
    tree.insert_root(root);

    let heading_id = tree.next_id();
    let mut heading = sized_text(heading_id, "診断🧭e\u{301}");
    heading.appearance.fg = Some(Color::LightCyan);
    heading.appearance.bold = true;
    tree.insert(heading);
    tree.add_child(root_id, heading_id);

    let decoration_id = tree.next_id();
    let decoration = BoxNode::new()
        .column()
        .height(2.0)
        .shrink(0.0)
        .build(decoration_id);
    tree.insert(decoration);

    let score_id = tree.next_id();
    tree.insert(sized_text(score_id, "92 / 100"));
    tree.add_child(root_id, score_id);
    tree.add_child(root_id, decoration_id);
    tree
}

fn presentation(order: [u64; 3]) -> HeadlessPresentation {
    let semantic = |node_id| match node_id {
        0 => HeadlessSemanticNode::new(0, SemanticRole::Application, "Doctor"),
        1 => HeadlessSemanticNode::new(1, SemanticRole::Heading, "Diagnostics")
            .with_state(SemanticState::default().with_level(1)),
        3 => HeadlessSemanticNode::new(3, SemanticRole::Progress, "Overall health")
            .with_description("Application health score")
            .with_state(
                SemanticState::default()
                    .with_value("92 / 100")
                    .with_selected(true),
            ),
        _ => unreachable!(),
    };
    HeadlessPresentation::new()
        .with_semantics(order.map(semantic))
        .with_focus(3)
        .with_announcements([HeadlessAnnouncement::new(
            AnnouncementPoliteness::Polite,
            "1 finding selected",
        )
        .with_source(3)])
}

#[test]
fn snapshot_is_stable_across_semantic_input_order() {
    let mut tree = diagnostic_tree();
    let mut renderer = HeadlessRenderer::new(16, 4).unwrap();

    let first = renderer
        .render(&mut tree, &presentation([3, 0, 1]))
        .unwrap();
    let second = renderer
        .render(&mut tree, &presentation([1, 3, 0]))
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(
        first
            .semantics()
            .iter()
            .map(|node| node.node_id)
            .collect::<Vec<_>>(),
        vec![0, 1, 3]
    );
    assert_eq!(first.semantics()[2].parent, Some(0));
    assert_eq!(first.semantics()[2].depth, 1);
    assert!(first.semantics()[2].presented);
    assert_eq!(first.focus(), Some(3));
    assert_eq!(first.announcements()[0].source, Some(3));
    assert_eq!(first.viewport(), crate::layout::Rect::sized(16, 4));
    assert_eq!(first.cells().len(), 64);
    assert_eq!(serde_json::to_value(&first).unwrap()["focus"], 3);
}

#[test]
fn cell_snapshot_preserves_wide_cells_style_and_visual_rows() {
    let mut tree = diagnostic_tree();
    let mut renderer = HeadlessRenderer::new(16, 4).unwrap();
    let snapshot = renderer
        .render(&mut tree, &presentation([0, 1, 3]))
        .unwrap();

    assert_eq!(snapshot.cell(0, 0).unwrap().symbol, "診");
    assert!(snapshot.cell(1, 0).unwrap().continuation);
    assert_eq!(snapshot.cell(2, 0).unwrap().symbol, "断");
    assert!(snapshot.cell(3, 0).unwrap().continuation);
    assert_eq!(snapshot.cell(4, 0).unwrap().symbol, "🧭");
    assert_eq!(
        snapshot.cell(0, 0).unwrap().style,
        Style::new().fg(Color::LightCyan).bold()
    );
    assert!(snapshot.row_text(0).unwrap().starts_with("診断🧭e"));
    assert!(snapshot.row_text(1).unwrap().starts_with("92 / 100"));
    assert_eq!(snapshot.row_text(4), None);
    assert_eq!(snapshot.screen_text().lines().count(), 4);
}

#[test]
fn resize_clears_cells_and_recomputes_presentation() {
    let mut tree = diagnostic_tree();
    let mut renderer = HeadlessRenderer::new(16, 4).unwrap();
    let _ = renderer
        .render(&mut tree, &presentation([0, 1, 3]))
        .unwrap();

    assert!(renderer.resize(8, 2).unwrap());
    assert!(!renderer.resize(8, 2).unwrap());
    let snapshot = renderer
        .render(&mut tree, &presentation([0, 1, 3]))
        .unwrap();

    assert_eq!(snapshot.viewport(), crate::layout::Rect::sized(8, 2));
    assert_eq!(snapshot.cells().len(), 16);
    assert!(snapshot.semantics().iter().all(|node| node.presented));
    assert_eq!(snapshot.cell(8, 0), None);
}

#[test]
fn detached_nodes_and_non_presented_focus_fail_closed() {
    let mut tree = diagnostic_tree();
    let detached_id = tree.next_id();
    tree.insert(sized_text(detached_id, "Detached"));
    let mut renderer = HeadlessRenderer::new(16, 4).unwrap();
    let error = renderer
        .render(
            &mut tree,
            &HeadlessPresentation::new().with_semantics([HeadlessSemanticNode::new(
                detached_id,
                SemanticRole::Status,
                "Detached",
            )]),
        )
        .unwrap_err();
    assert_eq!(
        error,
        HeadlessRenderError::DetachedSemanticNode(detached_id)
    );

    let mut zero = HeadlessRenderer::new(0, 0).unwrap();
    let error = zero
        .render(
            &mut tree,
            &HeadlessPresentation::new()
                .with_semantics([HeadlessSemanticNode::new(
                    0,
                    SemanticRole::Application,
                    "Doctor",
                )])
                .with_focus(0),
        )
        .unwrap_err();
    assert_eq!(error, HeadlessRenderError::FocusNotPresented(0));
}

#[test]
fn cursor_and_cell_budgets_are_explicit_before_allocation() {
    assert_eq!(
        HeadlessRenderer::with_cell_budget(11, 10, 100)
            .err()
            .unwrap(),
        HeadlessRenderError::CellBudgetExceeded {
            width: 11,
            height: 10,
            required: 110,
            budget: 100
        }
    );

    let mut tree = diagnostic_tree();
    let mut renderer = HeadlessRenderer::new(10, 2).unwrap();
    let error = renderer
        .render(
            &mut tree,
            &HeadlessPresentation::new().with_cursor(Cursor::at(10, 1)),
        )
        .unwrap_err();
    assert_eq!(
        error,
        HeadlessRenderError::CursorOutsideViewport {
            x: 10,
            y: 1,
            width: 10,
            height: 2
        }
    );
}

#[test]
fn empty_tree_and_zero_viewport_have_a_stable_empty_snapshot() {
    let mut tree = RenderTree::new();
    let mut renderer = HeadlessRenderer::new(0, 0).unwrap();
    let snapshot = renderer
        .render(&mut tree, &HeadlessPresentation::new())
        .unwrap();

    assert!(snapshot.semantics().is_empty());
    assert!(snapshot.cells().is_empty());
    assert_eq!(snapshot.screen_text(), "");
    assert!(!snapshot.cursor().visible);
}

#[test]
fn semantic_state_supports_virtualized_set_positions() {
    let state = SemanticState::default().with_set_position(9_001, 10_000);
    assert_eq!(state.position, Some(9_001));
    assert_eq!(state.set_size, Some(10_000));
}

#[test]
fn invalid_heading_and_logical_set_state_fail_closed() {
    let mut tree = diagnostic_tree();
    let mut renderer = HeadlessRenderer::new(16, 4).unwrap();
    let error = renderer
        .render(
            &mut tree,
            &HeadlessPresentation::new().with_semantics([HeadlessSemanticNode::new(
                1,
                SemanticRole::Heading,
                "Diagnostics",
            )
            .with_state(SemanticState::default().with_level(7))]),
        )
        .unwrap_err();
    assert_eq!(
        error,
        HeadlessRenderError::InvalidHeadingLevel {
            node_id: 1,
            level: 7
        }
    );

    let error = renderer
        .render(
            &mut tree,
            &HeadlessPresentation::new().with_semantics([HeadlessSemanticNode::new(
                3,
                SemanticRole::ListItem,
                "Finding",
            )
            .with_state(SemanticState {
                position: Some(1),
                ..SemanticState::default()
            })]),
        )
        .unwrap_err();
    assert_eq!(error, HeadlessRenderError::IncompleteSetPosition(3));
}
