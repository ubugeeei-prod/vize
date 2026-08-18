//! The folio-printing observer: a stage dump after every pass.
//!
//! `architecture.md`'s folio contract is that every stage dumps to stable,
//! readable text, and the reason it is worth having after *each* pass is
//! bisection: given a wrong output, the pass that first made it wrong is found
//! by diffing consecutive dumps rather than by reasoning.
//!
//! # What this lands, and what P2-4 adds
//!
//! The observer takes any [`Folio`] artifact and a sink to print into, so it
//! works with the hand-written [`Folio`] impls that exist today (P0-10's
//! `CroquisFolio`). What P2-4 adds is `#[derive(Folio)]`, so every S2 type
//! gets its page for free, and `davinci-opt --pipeline`, which is this
//! observer wired to a CLI. Neither changes this type's shape.

use core::fmt::Write;

use super::{PassEvent, PassObserver, Pipeline};
use crate::folio::{Folio, FolioMode};

/// Prints `artifact` after every pass into a caller-supplied sink.
///
/// The artifact is borrowed per call rather than held, because the observer
/// outlives any one stage value and holding it would tie the observer's
/// lifetime to the artifact's for no benefit.
#[derive(Debug)]
pub struct FolioObserver<W> {
    /// Where pages go.
    pub sink: W,
    /// Which form to print. `Full` is the parseable one and the only one
    /// carrying a round-trip law; `Display` is for reading.
    pub mode: FolioMode,
    /// Pages written.
    pub pages: u32,
}

impl<W: Write> FolioObserver<W> {
    /// A folio observer printing `Full`-mode pages into `sink`.
    #[must_use]
    pub const fn new(sink: W) -> Self {
        Self {
            sink,
            mode: FolioMode::Full,
            pages: 0,
        }
    }

    /// A folio observer printing `mode` pages into `sink`.
    #[must_use]
    pub const fn with_mode(sink: W, mode: FolioMode) -> Self {
        Self {
            sink,
            mode,
            pages: 0,
        }
    }

    /// Print `artifact`'s page, headed by the pass that produced it.
    ///
    /// The header is what makes a sequence of pages bisectable: without the
    /// pass name a diff says the output changed, not who changed it.
    ///
    /// # Errors
    ///
    /// Propagates the sink's write failure.
    pub fn page<A: Folio>(
        &mut self,
        event: &PassEvent<'_>,
        artifact: &A,
    ) -> Result<(), core::fmt::Error> {
        writeln!(
            self.sink,
            "; after {}.{} (walk {})",
            event.pipeline.stage,
            event.desc().name,
            event.group_index
        )?;
        artifact.print(&mut self.sink, self.mode)?;
        self.pages += 1;
        Ok(())
    }
}

/// The observer half is deliberately empty: printing needs the artifact, which
/// the hooks do not carry, so [`FolioObserver::page`] is called by whoever has
/// it. Implementing the trait anyway means a folio observer composes with the
/// others through [`Pair`](super::Pair) instead of being a special case.
impl<W> PassObserver for FolioObserver<W> {
    fn before_pipeline(&mut self, _pipeline: &Pipeline) {}
}
