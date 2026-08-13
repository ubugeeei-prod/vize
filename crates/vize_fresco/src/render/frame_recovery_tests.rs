//! Recovery behavior after a failed frame retains an already-painted buffer.

use super::{
    FrameRenderer,
    frame_test_support::{FailOnceWriter, set_root_text, styled_child_tree, text_tree},
};
use crate::terminal::{Backend, Cell, Color, Style};

#[test]
fn retained_frame_preparation_clears_exactly_once() {
    let mut backend = Backend::with_writer(4, 1, Vec::new());
    assert!(!backend.prepare_retained_frame());

    backend
        .buffer_mut()
        .set_string(0, 0, "old", Style::new().fg(Color::Red));
    assert!(backend.prepare_retained_frame());
    assert_eq!(backend.buffer().get(0, 0), Some(&Cell::EMPTY));
    assert_eq!(backend.buffer().get(1, 0), Some(&Cell::EMPTY));
    assert_eq!(backend.buffer().get(2, 0), Some(&Cell::EMPTY));
    assert!(!backend.prepare_retained_frame());
}

#[test]
fn direct_backend_retry_preserves_the_already_painted_buffer() {
    let mut backend = Backend::with_writer(5, 1, FailOnceWriter::armed());
    backend.buffer_mut().set_string(0, 0, "retry", Style::new());

    backend.flush_measured().unwrap_err();
    assert_eq!(
        backend.buffer().get(0, 0).map(|cell| cell.symbol.as_str()),
        Some("r")
    );

    let retried = backend.flush_measured().unwrap();
    assert_eq!(retried.changed_cells(), 5);
}

#[test]
fn changed_tree_after_output_failure_erases_shortened_wide_content() {
    let mut tree = text_tree("AB🙂");
    let mut backend = Backend::with_writer(6, 1, FailOnceWriter::armed());
    let mut renderer = FrameRenderer::new();

    renderer
        .render(&mut tree, &mut backend, Default::default())
        .unwrap_err();
    set_root_text(&mut tree, "Z");

    let recovered = renderer
        .render(&mut tree, &mut backend, Default::default())
        .unwrap();
    assert_eq!(recovered.changed_cells(), 1);

    let unchanged = renderer
        .render(&mut tree, &mut backend, Default::default())
        .unwrap();
    assert_eq!(unchanged.changed_cells(), 0);
}

#[test]
fn recovery_diffs_shortened_wide_content_against_previous_successful_frame() {
    let mut tree = text_tree("AB🙂");
    let mut backend = Backend::with_writer(6, 1, FailOnceWriter::default());
    let mut renderer = FrameRenderer::new();

    let first = renderer
        .render(&mut tree, &mut backend, Default::default())
        .unwrap();
    assert_eq!(first.changed_cells(), 4);

    backend.writer_mut().armed = true;
    renderer
        .render(&mut tree, &mut backend, Default::default())
        .unwrap_err();
    set_root_text(&mut tree, "Z");

    let recovered = renderer
        .render(&mut tree, &mut backend, Default::default())
        .unwrap();
    assert_eq!(recovered.changed_cells(), 4);

    let unchanged = renderer
        .render(&mut tree, &mut backend, Default::default())
        .unwrap();
    assert_eq!(unchanged.changed_cells(), 0);
}

#[test]
fn unchanged_tree_after_output_failure_repaints_the_complete_frame() {
    let mut tree = text_tree("same");
    let mut backend = Backend::with_writer(4, 1, FailOnceWriter::armed());
    let mut renderer = FrameRenderer::new();

    renderer
        .render(&mut tree, &mut backend, Default::default())
        .unwrap_err();
    let recovered = renderer
        .render(&mut tree, &mut backend, Default::default())
        .unwrap();

    assert_eq!(recovered.changed_cells(), 4);
    assert!(!backend.writer().output.is_empty());
}

#[test]
fn removed_styled_child_after_failure_diffs_against_previous_successful_frame() {
    let mut tree = styled_child_tree("界x");
    let root = tree.root().unwrap();
    let child = tree.get(root).unwrap().children.first().copied().unwrap();
    let mut backend = Backend::with_writer(4, 1, FailOnceWriter::default());
    let mut renderer = FrameRenderer::new();

    let first = renderer
        .render(&mut tree, &mut backend, Default::default())
        .unwrap();
    assert_eq!(first.changed_cells(), 4);

    backend.writer_mut().armed = true;
    renderer
        .render(&mut tree, &mut backend, Default::default())
        .unwrap_err();
    tree.remove_child(root, child);
    tree.remove(child);

    let recovered = renderer
        .render(&mut tree, &mut backend, Default::default())
        .unwrap();
    assert_eq!(recovered.changed_cells(), 4);

    let unchanged = renderer
        .render(&mut tree, &mut backend, Default::default())
        .unwrap();
    assert_eq!(unchanged.changed_cells(), 0);
}

#[test]
fn removed_styled_child_after_output_failure_leaves_no_stale_cells() {
    let mut tree = styled_child_tree("stale");
    let root = tree.root().unwrap();
    let child = tree.get(root).unwrap().children.first().copied().unwrap();
    let mut backend = Backend::with_writer(8, 1, FailOnceWriter::armed());
    let mut renderer = FrameRenderer::new();

    renderer
        .render(&mut tree, &mut backend, Default::default())
        .unwrap_err();
    tree.remove_child(root, child);
    tree.remove(child);

    let recovered = renderer
        .render(&mut tree, &mut backend, Default::default())
        .unwrap();
    assert_eq!(recovered.changed_cells(), 0);
}
