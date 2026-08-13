use std::io;

use super::{
    FRAME_TELEMETRY_SCHEMA_VERSION, FrameCoalescer, FrameRenderer, FrameRequestOutcome,
    frame_test_support::{AlwaysFailWriter, text_tree},
};
use crate::terminal::Backend;

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
