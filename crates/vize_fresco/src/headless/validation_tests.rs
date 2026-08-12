use super::{
    AnnouncementPoliteness, HeadlessAnnouncement, HeadlessPresentation, HeadlessRenderError,
    HeadlessRenderer, HeadlessSemanticNode, SemanticRole, SemanticState,
};
use crate::{
    component::{BoxNode, TextNode},
    render::RenderTree,
};

fn tree() -> RenderTree {
    let mut tree = RenderTree::new();
    let root = tree.next_id();
    tree.insert_root(BoxNode::new().column().build(root));
    let child = tree.next_id();
    tree.insert(TextNode::new("Finding").build(child));
    tree.add_child(root, child);
    tree
}

fn render_error(semantic: HeadlessSemanticNode) -> HeadlessRenderError {
    HeadlessRenderer::new(20, 4)
        .unwrap()
        .render(
            &mut tree(),
            &HeadlessPresentation::new().with_semantics([semantic]),
        )
        .unwrap_err()
}

#[test]
fn semantic_node_references_and_accessible_text_fail_closed() {
    assert_eq!(
        render_error(HeadlessSemanticNode::new(
            99,
            SemanticRole::Status,
            "Missing"
        )),
        HeadlessRenderError::UnknownSemanticNode(99)
    );
    assert_eq!(
        render_error(HeadlessSemanticNode::new(1, SemanticRole::Status, "  \n")),
        HeadlessRenderError::EmptySemanticName(1)
    );
    assert_eq!(
        render_error(
            HeadlessSemanticNode::new(1, SemanticRole::Status, "Finding").with_description(" \t")
        ),
        HeadlessRenderError::EmptySemanticDescription(1)
    );
}

#[test]
fn role_specific_state_rejects_every_invalid_boundary() {
    assert_eq!(
        render_error(
            HeadlessSemanticNode::new(1, SemanticRole::Heading, "Finding")
                .with_state(SemanticState::default().with_level(0))
        ),
        HeadlessRenderError::InvalidHeadingLevel {
            node_id: 1,
            level: 0
        }
    );
    assert_eq!(
        render_error(
            HeadlessSemanticNode::new(1, SemanticRole::Status, "Finding")
                .with_state(SemanticState::default().with_level(2))
        ),
        HeadlessRenderError::UnexpectedHeadingLevel(1)
    );
    assert_eq!(
        render_error(
            HeadlessSemanticNode::new(1, SemanticRole::ListItem, "Finding").with_state(
                SemanticState {
                    position: Some(2),
                    set_size: Some(1),
                    ..SemanticState::default()
                }
            )
        ),
        HeadlessRenderError::InvalidSetPosition {
            node_id: 1,
            position: 2,
            set_size: 1
        }
    );
}

#[test]
fn duplicate_identity_focus_and_announcement_contracts_fail_closed() {
    let semantic = HeadlessSemanticNode::new(0, SemanticRole::Application, "Doctor");
    let error = HeadlessRenderer::new(20, 4)
        .unwrap()
        .render(
            &mut tree(),
            &HeadlessPresentation::new().with_semantics([semantic.clone(), semantic]),
        )
        .unwrap_err();
    assert_eq!(error, HeadlessRenderError::DuplicateSemanticNode(0));

    let error = HeadlessRenderer::new(20, 4)
        .unwrap()
        .render(
            &mut tree(),
            &HeadlessPresentation::new()
                .with_semantics([HeadlessSemanticNode::new(
                    0,
                    SemanticRole::Application,
                    "Doctor",
                )])
                .with_focus(1),
        )
        .unwrap_err();
    assert_eq!(error, HeadlessRenderError::UnknownFocus(1));

    let error = HeadlessRenderer::new(20, 4)
        .unwrap()
        .render(
            &mut tree(),
            &HeadlessPresentation::new().with_announcements([HeadlessAnnouncement::new(
                AnnouncementPoliteness::Polite,
                " \n",
            )]),
        )
        .unwrap_err();
    assert_eq!(error, HeadlessRenderError::EmptyAnnouncement(0));

    let error = HeadlessRenderer::new(20, 4)
        .unwrap()
        .render(
            &mut tree(),
            &HeadlessPresentation::new()
                .with_semantics([HeadlessSemanticNode::new(
                    0,
                    SemanticRole::Application,
                    "Doctor",
                )])
                .with_announcements([HeadlessAnnouncement::new(
                    AnnouncementPoliteness::Assertive,
                    "Failure",
                )
                .with_source(1)]),
        )
        .unwrap_err();
    assert_eq!(
        error,
        HeadlessRenderError::UnknownAnnouncementSource {
            index: 0,
            node_id: 1
        }
    );
}
