use std::io::{self, Write};

use super::{
    FRAME_TELEMETRY_SCHEMA_VERSION, FrameCoalescer, FrameRenderer, FrameRequestOutcome, RenderNode,
    RenderTree,
};
use crate::{layout::Dimension, terminal::Backend};

fn text_tree(value: &str) -> RenderTree {
    let mut tree = RenderTree::new();
    let id = tree.next_id();
    let mut node = RenderNode::text_node(id, value);
    node.style.width = Dimension::Points(4.0);
    node.style.height = Dimension::Points(1.0);
    tree.insert_root(node);
    tree
}

#[test]
fn coalescer_has_bounded_state_and_exact_loss_accounting() {
    let mut coalescer = FrameCoalescer::new();
    assert!(!coalescer.has_pending_frame());
    assert_eq!(coalescer.request_frame(), FrameRequestOutcome::Scheduled);
    assert_eq!(coalescer.request_frame(), FrameRequestOutcome::Coalesced);
    assert_eq!(coalescer.request_frame(), FrameRequestOutcome::Coalesced);
    assert!(coalescer.drop_pending_frame());
    assert!(!coalescer.drop_pending_frame());
    assert_eq!(coalescer.activity().dropped_frames(), 1);
    assert_eq!(coalescer.activity().coalesced_frames(), 2);
    assert_eq!(coalescer.request_frame(), FrameRequestOutcome::Scheduled);

    let activity = coalescer.begin_frame().unwrap();
    assert_eq!(activity.dropped_frames(), 1);
    assert_eq!(activity.coalesced_frames(), 2);
    assert!(coalescer.begin_frame().is_none());

    coalescer.request_frame();
    let next = coalescer.begin_frame().unwrap();
    assert_eq!(next.dropped_frames(), 0);
    assert_eq!(next.coalesced_frames(), 0);
}

#[test]
fn measured_renderer_reports_every_frame_budget_dimension() {
    let mut tree = text_tree("A");
    let mut backend = Backend::with_writer(4, 1, Vec::new());
    let mut coalescer = FrameCoalescer::new();
    let mut renderer = FrameRenderer::new();
    coalescer.request_frame();
    coalescer.request_frame();

    let first = renderer
        .render_pending(&mut tree, &mut backend, &mut coalescer)
        .unwrap()
        .unwrap();
    assert_eq!(first.schema_version(), FRAME_TELEMETRY_SCHEMA_VERSION);
    assert_eq!(first.changed_cells(), 1);
    assert_eq!(first.bytes_written(), backend.writer().len() as u64);
    assert_eq!(first.retained_nodes(), 1);
    assert_eq!(first.dropped_frames(), 0);
    assert_eq!(first.coalesced_frames(), 1);
    assert_eq!(
        first.total_time_ns(),
        first
            .layout_time_ns()
            .saturating_add(first.paint_time_ns())
            .saturating_add(first.output_time_ns())
    );

    let bytes_before = backend.writer().len();
    coalescer.request_frame();
    let unchanged = renderer
        .render_pending(&mut tree, &mut backend, &mut coalescer)
        .unwrap()
        .unwrap();
    assert_eq!(unchanged.changed_cells(), 0);
    assert_eq!(
        unchanged.bytes_written(),
        (backend.writer().len() - bytes_before) as u64
    );
}

#[test]
fn empty_queue_performs_no_layout_paint_or_output_work() {
    let mut tree = text_tree("A");
    let mut backend = Backend::with_writer(4, 1, Vec::new());
    let mut coalescer = FrameCoalescer::new();
    let mut renderer = FrameRenderer::new();

    assert!(
        renderer
            .render_pending(&mut tree, &mut backend, &mut coalescer)
            .unwrap()
            .is_none()
    );
    assert!(backend.writer().is_empty());
    assert!(tree.get(tree.root().unwrap()).unwrap().layout.is_none());
}

#[test]
fn output_failure_preserves_partial_timing_activity_and_retry_buffer() {
    let mut tree = text_tree("A");
    let mut backend = Backend::with_writer(4, 1, AlwaysFailWriter);
    let mut coalescer = FrameCoalescer::new();
    let mut renderer = FrameRenderer::new();
    coalescer.request_frame();
    coalescer.request_frame();

    let error = renderer
        .render_pending(&mut tree, &mut backend, &mut coalescer)
        .unwrap_err();
    assert_eq!(error.source_error().kind(), io::ErrorKind::Other);
    assert_eq!(error.retained_nodes(), 1);
    assert_eq!(error.activity().dropped_frames(), 0);
    assert_eq!(error.activity().coalesced_frames(), 1);
    assert_eq!(
        backend.buffer().get(0, 0).map(|cell| cell.symbol.as_str()),
        Some("A")
    );
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
fn removed_styled_child_after_output_failure_leaves_no_stale_cells() {
    let mut tree = RenderTree::new();
    let root = tree.next_id();
    tree.insert_root(RenderNode::box_node(root));
    let child = tree.next_id();
    let mut node = RenderNode::text_node(child, "stale");
    node.style.width = Dimension::Points(5.0);
    node.style.height = Dimension::Points(1.0);
    node.appearance.fg = Some(crate::terminal::Color::Red);
    node.appearance.bg = Some(crate::terminal::Color::Blue);
    node.appearance.inverse = true;
    node.appearance.underline = true;
    tree.insert(node);
    tree.add_child(root, child);
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

#[test]
fn telemetry_serialization_is_versioned_and_uses_portable_units() {
    let mut tree = text_tree("A");
    let mut backend = Backend::with_writer(4, 1, io::sink());
    let mut coalescer = FrameCoalescer::new();
    let mut renderer = FrameRenderer::new();
    coalescer.request_frame();
    let telemetry = renderer
        .render_pending(&mut tree, &mut backend, &mut coalescer)
        .unwrap()
        .unwrap();

    let value = serde_json::to_value(telemetry).unwrap();
    assert_eq!(value["schemaVersion"], FRAME_TELEMETRY_SCHEMA_VERSION);
    assert!(value["layoutTimeNs"].is_u64());
    assert!(value["paintTimeNs"].is_u64());
    assert!(value["outputTimeNs"].is_u64());
    assert_eq!(value["retainedNodes"], 1);
    assert_eq!(value["droppedFrames"], 0);
    assert_eq!(value["coalescedFrames"], 0);
}

#[derive(Debug)]
struct AlwaysFailWriter;

impl Write for AlwaysFailWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("injected frame failure"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("injected frame failure"))
    }
}

#[derive(Debug, Default)]
struct FailOnceWriter {
    armed: bool,
    output: Vec<u8>,
}

impl FailOnceWriter {
    fn armed() -> Self {
        Self {
            armed: true,
            output: Vec::new(),
        }
    }
}

impl Write for FailOnceWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if std::mem::take(&mut self.armed) {
            return Err(io::Error::other("injected first-frame failure"));
        }
        self.output.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn set_root_text(tree: &mut RenderTree, text: &str) {
    let root = tree.root().unwrap();
    let node = tree.get_mut(root).unwrap();
    let super::NodeKind::Text(content) = &mut node.kind else {
        panic!("test tree root must be text");
    };
    content.text = text.into();
}
