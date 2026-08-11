//! Feature-gated, process-state-free performance harness for the Doctor TUI.

use std::io;

use vize_doctor::DoctorReport;
use vize_fresco::render::Painter;
use vize_fresco::{Backend, FrameOutputTelemetry, Key, KeyEvent, TerminalCapabilities};

use super::{DoctorTuiModel, build_frame};

/// Headless Doctor workspace used by Criterion and downstream performance gates.
///
/// The harness owns an injected sink backend, so measurements include semantic
/// projection, bounded cell painting, differential output generation, and
/// exact changed-cell/byte accounting without reading or mutating process
/// terminal state. It deliberately borrows the immutable report just like the
/// interactive command.
#[doc(hidden)]
pub struct DoctorTuiBenchmark<'report> {
    model: DoctorTuiModel<'report>,
    backend: Backend<io::Sink>,
    capabilities: TerminalCapabilities,
    selection_forward: bool,
    search_has_query: bool,
    retained_nodes: usize,
}

impl<'report> DoctorTuiBenchmark<'report> {
    /// Create a headless workspace for an immutable report and explicit profile.
    pub fn new(
        report: &'report DoctorReport,
        width: u16,
        height: u16,
        capabilities: TerminalCapabilities,
    ) -> Self {
        Self {
            model: DoctorTuiModel::new(report, width, height),
            backend: Backend::with_writer(width, height, io::sink()),
            capabilities,
            selection_forward: true,
            search_has_query: false,
            retained_nodes: 0,
        }
    }

    /// Paint and differentially flush one complete frame.
    pub fn render(&mut self) -> FrameOutputTelemetry {
        let mut frame = build_frame(&mut self.model, &[], self.capabilities)
            .expect("benchmark fixture must produce valid semantic presentations");
        self.model.place_cursor(self.backend.cursor_mut());
        self.retained_nodes = frame.tree_mut().node_count();
        frame
            .tree_mut()
            .compute_layout(self.backend.width(), self.backend.height());
        self.backend.buffer_mut().clear();
        Painter::new(self.backend.buffer_mut()).paint_tree(frame.tree_mut());
        self.backend
            .flush_measured()
            .expect("injected sink must accept a complete frame")
    }

    /// Alternate between adjacent findings and render the resulting diff frame.
    pub fn toggle_selection_and_render(&mut self) -> FrameOutputTelemetry {
        let key = if self.selection_forward {
            Key::Down
        } else {
            Key::Up
        };
        self.selection_forward = !self.selection_forward;
        let _ = self.model.handle_key(&KeyEvent::key(key));
        self.render()
    }

    /// Alternate an unmatched one-character query and render the result.
    pub fn toggle_search_and_render(&mut self) -> FrameOutputTelemetry {
        if self.model.mode() != super::model::InteractionMode::Search {
            let _ = self.model.handle_key(&KeyEvent::char('/'));
        }
        let key = if self.search_has_query {
            Key::Backspace
        } else {
            Key::Char('\u{10ffff}')
        };
        self.search_has_query = !self.search_has_query;
        let _ = self.model.handle_key(&KeyEvent::key(key));
        self.render()
    }

    /// Return the number of finding rows retained for the current viewport.
    pub fn materialized_findings(&self) -> usize {
        self.model.workspace().finding_window().materialized_len()
    }

    /// Return render nodes retained by the most recently completed frame.
    pub const fn retained_nodes(&self) -> usize {
        self.retained_nodes
    }
}
