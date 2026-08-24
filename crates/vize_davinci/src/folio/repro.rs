//! The crash-repro page (`[repro]`) - what the ICE policy writes and what
//! `vize repro` reads back (charter #30, P2-13).
//!
//! A `repro.folio` is a self-contained reproducer for a pipeline run that
//! stopped on a panic: the pipeline string (P2-2 grammar), the run
//! configuration a replay needs, the failure the run recorded, and the
//! last-good stage dump. "Last-good" is literal: it is the artifact as of the
//! last stage that completed, and today - before any S2 stage runs on the
//! compile path - that is the authored source itself, carried under
//! `artifact-stage=source`.
//!
//! # Why this page is hand-written
//!
//! Two of its decisions are semantic, which is exactly what P2-4's derive
//! refuses to make:
//!
//! - **The artifact section is verbatim and terminal.** Everything after the
//!   `[repro.artifact]` header line, byte for byte to end of input, is the
//!   embedded artifact - blank lines, `[`-prefixed lines, all of it. That is
//!   the only way to embed an arbitrary dump without an escaping scheme, and
//!   it means the section must come last and is exempt from the
//!   one-blank-line rule (folio-format.md rule 4), an exemption the format
//!   doc records.
//! - **`failed-pass` may be empty.** A panic caught outside a driven pipeline
//!   (the build path's real compile, until P2-12b routes it through the pass
//!   manager) is not attributable to a pass; the field prints with an empty
//!   value rather than inventing one.
//!
//! # Normalization
//!
//! Scalar order and config-entry order normalize by the first print (header
//! fields in declaration order, config sorted by key - rule 1). The artifact
//! is verbatim except for one thing: canonical text is LF-terminated, so a
//! missing final newline is added by the first print, and [`ReproFolio::
//! normalize`] applies the same to hand-built values so the structural
//! round-trip law quantifies over normalized values (the `CroquisFolio::
//! normalize` precedent).
//!
//! Scalar values are line-atomic (the [`FolioValue`](super::value::FolioValue)
//! contract): a writer embedding a panic payload must normalize newlines out
//! of `reason` before constructing the page.

use core::fmt;

use vize_s0::{FxHashMap, String, cstr};

use super::page::{PagePrinter, ParseState};
use super::{Folio, FolioError, FolioMode, page};
use crate::pass::parse_pipelines;

/// Render a failure identity as one line: `{stage}.{pass}: {reason}`, with
/// `?` standing in for an unattributable pass.
///
/// This is the one formatter every surface reporting a repro failure goes
/// through - the build's error report, `vize repro`'s verdict, and the
/// equality both sides assert - so "the same failure" is the same bytes.
#[must_use]
pub fn failure_text(stage: &str, pass: &str, reason: &str) -> String {
    if pass.is_empty() {
        cstr!("{stage}.?: {reason}")
    } else {
        cstr!("{stage}.{pass}: {reason}")
    }
}

/// The `[repro]` page: pipeline string, run config, recorded failure, and
/// the last-good stage dump.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReproFolio {
    /// The pipeline the failed run executed, in the P2-2 pipeline grammar.
    /// Parse validates it against that grammar, so a repro never carries a
    /// pipeline a replay cannot parse.
    pub pipeline: String,
    /// Stage of the pipeline segment the failure landed in.
    pub failed_stage: String,
    /// The pass that failed; empty when the panic is not attributable to one.
    pub failed_pass: String,
    /// The panic payload, newline-normalized by the writer.
    pub reason: String,
    /// How to read `artifact`: `source` means the authored input verbatim;
    /// any other name is the folio stage that parses the dump.
    pub artifact_stage: String,
    /// Replay configuration (`mode`, `inject-panic`, ...), printed sorted.
    pub config: FxHashMap<String, String>,
    /// The last-good stage dump, verbatim.
    pub artifact: String,
}

impl ReproFolio {
    /// The recorded failure as [`failure_text`] renders it.
    #[must_use]
    pub fn failure(&self) -> String {
        failure_text(
            self.failed_stage.as_str(),
            self.failed_pass.as_str(),
            self.reason.as_str(),
        )
    }

    /// Canonicalize a hand-built value: a non-empty artifact gains its
    /// terminal LF. Parsed values are already normalized.
    pub fn normalize(&mut self) {
        if !self.artifact.is_empty() && !self.artifact.ends_with('\n') {
            self.artifact.push('\n');
        }
    }
}

/// Declaration-order section indexes for [`ParseState`].
const SECTION_CONFIG: usize = 0;
const SECTION_ARTIFACT: usize = 1;

impl Folio for ReproFolio {
    /// Both modes print the same canonical text: nothing on this page is a
    /// span or a default, so there is nothing for `Display` to elide.
    fn print<W: fmt::Write>(&self, w: &mut W, _mode: FolioMode) -> fmt::Result {
        {
            let mut printer = PagePrinter::new(w, "repro");
            printer.open()?;
            printer.scalar("pipeline", &self.pipeline)?;
            printer.scalar("failed-stage", &self.failed_stage)?;
            printer.scalar("failed-pass", &self.failed_pass)?;
            printer.scalar("reason", &self.reason)?;
            printer.scalar("artifact-stage", &self.artifact_stage)?;
            printer.close_header()?;
            printer.map("config", &self.config)?;
        }
        if !self.artifact.is_empty() {
            writeln!(w, "[repro.artifact]")?;
            w.write_str(self.artifact.as_str())?;
            if !self.artifact.ends_with('\n') {
                w.write_char('\n')?;
            }
        }
        Ok(())
    }

    fn parse(input: &str) -> Result<Self, FolioError> {
        let mut state = ParseState::new("repro");
        let mut pipeline = None;
        let mut failed_stage = None;
        let mut failed_pass = None;
        let mut reason = None;
        let mut artifact_stage = None;
        let mut config: FxHashMap<String, String> = FxHashMap::default();
        let mut artifact = String::default();

        let mut rest = input;
        let mut line_no = 0usize;
        while !rest.is_empty() {
            line_no += 1;
            let (line, advance) = match rest.split_once('\n') {
                Some((line, _)) => (line, line.len() + 1),
                None => (rest, rest.len()),
            };
            match state.classify(line, line_no)? {
                page::LineEvent::Skip => {}
                page::LineEvent::Field => {
                    let (name, value) = page::split_field(line, line_no)?;
                    match name {
                        "pipeline" => page::set_scalar(&mut pipeline, name, value, line_no)?,
                        "failed-stage" => {
                            page::set_scalar(&mut failed_stage, name, value, line_no)?;
                        }
                        "failed-pass" => page::set_scalar(&mut failed_pass, name, value, line_no)?,
                        "reason" => page::set_scalar(&mut reason, name, value, line_no)?,
                        "artifact-stage" => {
                            page::set_scalar(&mut artifact_stage, name, value, line_no)?;
                        }
                        _ => return Err(page::unknown_field(name, line_no)),
                    }
                }
                page::LineEvent::Section("config") => {
                    state.enter_section(SECTION_CONFIG, "config", line_no)?;
                }
                page::LineEvent::Section("artifact") => {
                    state.enter_section(SECTION_ARTIFACT, "artifact", line_no)?;
                    // Terminal by design: the rest of the input is the
                    // artifact, verbatim - nothing after it is classified.
                    artifact = String::from(&rest[advance..]);
                    break;
                }
                page::LineEvent::Section(other) => {
                    return Err(state.unknown_section(other, line_no));
                }
                page::LineEvent::Entry(SECTION_CONFIG) => {
                    page::map_insert(&mut config, line, line_no)?;
                }
                page::LineEvent::Entry(_) => {
                    unreachable!("the artifact section never yields entry lines")
                }
            }
            rest = &rest[advance..];
        }
        state.require_header()?;

        let pipeline: String = page::require_scalar(pipeline, "pipeline")?;
        if let Err(error) = parse_pipelines(pipeline.as_str()) {
            return Err(FolioError::new(
                0,
                cstr!("invalid pipeline `{pipeline}`: {error}"),
            ));
        }
        let mut folio = Self {
            pipeline,
            failed_stage: page::require_scalar(failed_stage, "failed-stage")?,
            failed_pass: page::require_scalar(failed_pass, "failed-pass")?,
            reason: page::require_scalar(reason, "reason")?,
            artifact_stage: page::require_scalar(artifact_stage, "artifact-stage")?,
            config,
            artifact,
        };
        folio.normalize();
        Ok(folio)
    }
}
