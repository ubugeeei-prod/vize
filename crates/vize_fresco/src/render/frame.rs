//! Measured frame rendering and bounded request coalescing.

use std::{
    io::{self, Write},
    time::Instant,
};

use compact_str::CompactString;
use serde::Serialize;
use thiserror::Error;

use super::{Painter, RenderTree};
use crate::terminal::Backend;

/// Wire version emitted by [`FrameTelemetry`].
pub const FRAME_TELEMETRY_SCHEMA_VERSION: u16 = 1;

/// Requests discarded or merged before one frame began.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameActivityTelemetry {
    dropped_frames: u64,
    coalesced_frames: u64,
}

impl FrameActivityTelemetry {
    /// Return pending frames explicitly discarded before this frame.
    pub const fn dropped_frames(self) -> u64 {
        self.dropped_frames
    }

    /// Return redundant requests merged into the pending frame.
    pub const fn coalesced_frames(self) -> u64 {
        self.coalesced_frames
    }
}

/// Result of requesting a frame from a one-slot coalescer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameRequestOutcome {
    /// No frame was pending, so one was scheduled.
    Scheduled,
    /// A frame was already pending, so the new request was merged into it.
    Coalesced,
}

/// Bounded one-slot frame request queue with loss accounting.
#[derive(Debug, Clone, Default)]
pub struct FrameCoalescer {
    pending: bool,
    dropped_frames: u64,
    coalesced_frames: u64,
}

impl FrameCoalescer {
    /// Create an empty frame queue.
    pub const fn new() -> Self {
        Self {
            pending: false,
            dropped_frames: 0,
            coalesced_frames: 0,
        }
    }

    /// Schedule a frame or merge the request into the existing pending frame.
    pub fn request_frame(&mut self) -> FrameRequestOutcome {
        if self.pending {
            self.coalesced_frames = self.coalesced_frames.saturating_add(1);
            FrameRequestOutcome::Coalesced
        } else {
            self.pending = true;
            FrameRequestOutcome::Scheduled
        }
    }

    /// Discard the pending frame and record the loss.
    pub fn drop_pending_frame(&mut self) -> bool {
        if !self.pending {
            return false;
        }
        self.pending = false;
        self.dropped_frames = self.dropped_frames.saturating_add(1);
        true
    }

    /// Return whether a frame is waiting to render.
    pub const fn has_pending_frame(&self) -> bool {
        self.pending
    }

    /// Snapshot activity accumulated since the prior begun frame.
    ///
    /// This remains observable when the final pending frame is dropped during
    /// shutdown and no later frame exists to consume the counters.
    pub const fn activity(&self) -> FrameActivityTelemetry {
        FrameActivityTelemetry {
            dropped_frames: self.dropped_frames,
            coalesced_frames: self.coalesced_frames,
        }
    }

    /// Begin the pending frame and transfer activity since the prior frame.
    pub fn begin_frame(&mut self) -> Option<FrameActivityTelemetry> {
        if !self.pending {
            return None;
        }
        self.pending = false;
        Some(FrameActivityTelemetry {
            dropped_frames: std::mem::take(&mut self.dropped_frames),
            coalesced_frames: std::mem::take(&mut self.coalesced_frames),
        })
    }
}

/// Complete cost and retention record for one successful frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameTelemetry {
    schema_version: u16,
    layout_time_ns: u64,
    paint_time_ns: u64,
    output_time_ns: u64,
    changed_cells: u64,
    bytes_written: u64,
    retained_nodes: u64,
    dropped_frames: u64,
    coalesced_frames: u64,
}

impl FrameTelemetry {
    /// Return the telemetry wire version.
    pub const fn schema_version(self) -> u16 {
        self.schema_version
    }

    /// Return layout time in nanoseconds.
    pub const fn layout_time_ns(self) -> u64 {
        self.layout_time_ns
    }

    /// Return paint time in nanoseconds.
    pub const fn paint_time_ns(self) -> u64 {
        self.paint_time_ns
    }

    /// Return differential output time in nanoseconds.
    pub const fn output_time_ns(self) -> u64 {
        self.output_time_ns
    }

    /// Return cells changed from the prior successful frame.
    pub const fn changed_cells(self) -> u64 {
        self.changed_cells
    }

    /// Return exact bytes accepted by the terminal writer.
    pub const fn bytes_written(self) -> u64 {
        self.bytes_written
    }

    /// Return render nodes retained by the frame model.
    pub const fn retained_nodes(self) -> u64 {
        self.retained_nodes
    }

    /// Return pending frames discarded before this frame.
    pub const fn dropped_frames(self) -> u64 {
        self.dropped_frames
    }

    /// Return requests merged into this frame.
    pub const fn coalesced_frames(self) -> u64 {
        self.coalesced_frames
    }

    /// Return total measured layout, paint, and output time in nanoseconds.
    pub const fn total_time_ns(self) -> u64 {
        self.layout_time_ns
            .saturating_add(self.paint_time_ns)
            .saturating_add(self.output_time_ns)
    }
}

/// Failed output after layout and paint completed.
#[derive(Debug, Error)]
#[error("cannot write measured Fresco frame: {source}")]
pub struct FrameRenderError {
    #[source]
    source: io::Error,
    layout_time_ns: u64,
    paint_time_ns: u64,
    retained_nodes: u64,
    activity: FrameActivityTelemetry,
}

impl FrameRenderError {
    /// Return the terminal writer failure.
    pub const fn source_error(&self) -> &io::Error {
        &self.source
    }

    /// Return completed layout time in nanoseconds.
    pub const fn layout_time_ns(&self) -> u64 {
        self.layout_time_ns
    }

    /// Return completed paint time in nanoseconds.
    pub const fn paint_time_ns(&self) -> u64 {
        self.paint_time_ns
    }

    /// Return nodes retained by the failed frame.
    pub const fn retained_nodes(&self) -> u64 {
        self.retained_nodes
    }

    /// Return queue activity assigned to the failed frame.
    pub const fn activity(&self) -> FrameActivityTelemetry {
        self.activity
    }
}

/// Reusable production renderer with retained text-wrap scratch storage.
#[derive(Debug, Default)]
pub struct FrameRenderer {
    wrap_scratch: Vec<CompactString>,
}

impl FrameRenderer {
    /// Create a renderer with no allocated text-wrap scratch storage.
    pub const fn new() -> Self {
        Self {
            wrap_scratch: Vec::new(),
        }
    }

    /// Render a pending frame, returning `None` without work when the queue is empty.
    pub fn render_pending<W: Write>(
        &mut self,
        tree: &mut RenderTree,
        backend: &mut Backend<W>,
        coalescer: &mut FrameCoalescer,
    ) -> Result<Option<FrameTelemetry>, FrameRenderError> {
        let Some(activity) = coalescer.begin_frame() else {
            return Ok(None);
        };
        self.render(tree, backend, activity).map(Some)
    }

    /// Compute layout, paint, flush, and measure one complete frame.
    ///
    /// Paint time includes clearing a frame retained after failed output. The
    /// successful-frame path performs only a blank-state check before painting.
    pub fn render<W: Write>(
        &mut self,
        tree: &mut RenderTree,
        backend: &mut Backend<W>,
        activity: FrameActivityTelemetry,
    ) -> Result<FrameTelemetry, FrameRenderError> {
        let retained_nodes = count_nodes(tree);
        let layout_started = Instant::now();
        tree.compute_layout(backend.width(), backend.height());
        let layout_time_ns = elapsed_ns(layout_started);

        let paint_started = Instant::now();
        backend.prepare_retained_frame();
        let mut painter = Painter::with_wrap_scratch(
            backend.buffer_mut(),
            std::mem::take(&mut self.wrap_scratch),
        );
        painter.paint_tree(tree);
        self.wrap_scratch = painter.into_wrap_scratch();
        let paint_time_ns = elapsed_ns(paint_started);

        let output_started = Instant::now();
        let output = backend
            .flush_measured()
            .map_err(|source| FrameRenderError {
                source,
                layout_time_ns,
                paint_time_ns,
                retained_nodes,
                activity,
            })?;
        let output_time_ns = elapsed_ns(output_started);

        Ok(FrameTelemetry {
            schema_version: FRAME_TELEMETRY_SCHEMA_VERSION,
            layout_time_ns,
            paint_time_ns,
            output_time_ns,
            changed_cells: output.changed_cells(),
            bytes_written: output.bytes_written(),
            retained_nodes,
            dropped_frames: activity.dropped_frames,
            coalesced_frames: activity.coalesced_frames,
        })
    }
}

fn elapsed_ns(started: Instant) -> u64 {
    started.elapsed().as_nanos().try_into().unwrap_or(u64::MAX)
}

fn count_nodes(tree: &RenderTree) -> u64 {
    tree.node_count().try_into().unwrap_or(u64::MAX)
}
