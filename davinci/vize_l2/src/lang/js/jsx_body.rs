//! Original syntax and completed File facts owned with same-walk JSX structure.

use super::{FileProducer, ProgramInput, ProgramInputError, ProgramScope};
use crate::artifact::ArtifactError;
use crate::file::{FileArtifact, RejectedFile};
use alloc::boxed::Box;
use oxc_parser::ProgramObservation;
use vize_l0::{Allocator, SourceBlock};

mod recorder;
mod records;
use recorder::Recorder;
pub use records::{JsxChildren, JsxNode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsxFileError {
    SyntaxAdmission,
    Input(ProgramInputError),
    Artifact(ArtifactError),
    InvalidProfile,
    AlreadyWalked,
    NotWalked,
    Interrupted,
    IncompleteFile,
    IncompleteBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Pending,
    Interrupted,
    Complete,
}

/// Consumes the actual opaque parser owner, never a caller's AST/File pair.
/// There is no public recorder, seal, observer substitution or mutable syntax.
///
/// ```compile_fail
/// use oxc_ast::ast::Program;
/// use vize_l0::{Allocator, SourceBlock};
/// use vize_l2::lang::js::JsxFileProducer;
/// fn forge<'a>(arena: &'a Allocator, program: Program<'a>, block: SourceBlock<'a>) {
///     let _ = JsxFileProducer::new(arena, program, block, 0);
/// }
/// ```
pub struct JsxFileProducer<'a> {
    observation: ProgramObservation<'a>,
    block: SourceBlock<'a>,
    index: usize,
    producer: FileProducer<'a>,
    recorder: Recorder<'a>,
    phase: Phase,
}

/// A completed source body retains its actual original observation and File.
///
/// ```compile_fail
/// use vize_l2::lang::js::JsxFile;
/// fn forge<'a>() { let _ = JsxFile { observation: (), file: (), records: vec![] }; }
/// ```
pub struct JsxFile<'a> {
    observation: ProgramObservation<'a>,
    file: FileArtifact<'a>,
    records: alloc::vec::Vec<records::Record<'a>>,
}

/// Failed admission, walk or sealing retains original syntax and actual facts.
pub struct RejectedJsxFile<'a> {
    observation: ProgramObservation<'a>,
    error: JsxFileError,
    file: Option<FileArtifact<'a>>,
    rejected_file: Option<Box<RejectedFile<'a>>>,
    records: alloc::vec::Vec<records::Record<'a>>,
}

impl<'a> JsxFileProducer<'a> {
    pub fn new(
        allocator: &'a Allocator,
        observation: ProgramObservation<'a>,
        block: SourceBlock<'a>,
        container_index: usize,
    ) -> Result<Self, Box<RejectedJsxFile<'a>>> {
        let admission = observation
            .admitted()
            .ok_or(JsxFileError::SyntaxAdmission)
            .and_then(|admitted| {
                let input = ProgramInput::checked(admitted, block, container_index)
                    .map_err(JsxFileError::Input)?;
                if !input.source_type().is_jsx() || !input.source_type().is_module() {
                    return Err(JsxFileError::InvalidProfile);
                }
                Ok(())
            });
        if let Err(error) = admission {
            return Err(Box::new(RejectedJsxFile::new(observation, error)));
        }
        let producer = match FileProducer::new(allocator, block.root_source()) {
            Ok(producer) => producer,
            Err(error) => {
                return Err(Box::new(RejectedJsxFile::new(
                    observation,
                    JsxFileError::Artifact(error),
                )));
            }
        };
        Ok(Self {
            observation,
            block,
            index: container_index,
            producer,
            recorder: Recorder::new(),
            phase: Phase::Pending,
        })
    }

    /// Park the owning state before the sole visit. A caught unwind cannot retry
    /// or turn an interrupted original File/body into a completed view.
    pub fn walk(&mut self) -> Result<(), JsxFileError> {
        self.walk_with(|producer, input, recorder| {
            producer
                .program_observed(input, ProgramScope::Module, recorder)
                .map(|_| ())
        })
    }

    fn walk_with(
        &mut self,
        visit: impl FnOnce(
            &mut FileProducer<'a>,
            ProgramInput<'_, 'a>,
            &mut Recorder<'a>,
        ) -> Result<(), ProgramInputError>,
    ) -> Result<(), JsxFileError> {
        if self.phase != Phase::Pending {
            return Err(JsxFileError::AlreadyWalked);
        }
        self.phase = Phase::Interrupted;
        let admitted = self
            .observation
            .admitted()
            .ok_or(JsxFileError::SyntaxAdmission)?;
        let input =
            ProgramInput::checked(admitted, self.block, self.index).map_err(JsxFileError::Input)?;
        visit(&mut self.producer, input, &mut self.recorder).map_err(JsxFileError::Input)?;
        self.phase = Phase::Complete;
        Ok(())
    }

    #[must_use]
    pub fn observation(&self) -> &ProgramObservation<'a> {
        &self.observation
    }

    pub fn finish(self) -> Result<JsxFile<'a>, Box<RejectedJsxFile<'a>>> {
        let Self {
            observation,
            producer,
            recorder,
            phase,
            ..
        } = self;
        let body_complete = recorder.frames_complete() && recorder.jsx_count() > 0;
        let mut rejection = RejectedJsxFile::new(
            observation,
            match phase {
                Phase::Pending => JsxFileError::NotWalked,
                Phase::Interrupted => JsxFileError::Interrupted,
                Phase::Complete => JsxFileError::IncompleteBody,
            },
        );
        rejection.records = recorder.records;
        let file = match producer.finish() {
            Ok(file) => file,
            Err(file) => {
                rejection.error = JsxFileError::Artifact(file.artifact.error);
                rejection.rejected_file = Some(Box::new(file));
                return Err(Box::new(rejection));
            }
        };
        if phase != Phase::Complete {
            rejection.file = Some(file);
            return Err(Box::new(rejection));
        }
        // Structural File sealing alone does not imply semantic completion.
        if !file.is_complete() {
            rejection.error = JsxFileError::IncompleteFile;
            rejection.file = Some(file);
            return Err(Box::new(rejection));
        }
        if !body_complete {
            rejection.file = Some(file);
            return Err(Box::new(rejection));
        }
        Ok(JsxFile {
            observation: rejection.observation,
            file,
            records: rejection.records,
        })
    }
}

impl<'a> JsxFile<'a> {
    #[must_use]
    pub fn observation(&self) -> &ProgramObservation<'a> {
        &self.observation
    }
    #[must_use]
    pub fn file(&self) -> &FileArtifact<'a> {
        &self.file
    }
    #[must_use]
    pub fn node(&self, index: usize) -> Option<JsxNode<'_, 'a>> {
        self.records.get(index)?;
        Some(JsxNode::new(self, index))
    }
    pub fn nodes(&self) -> impl Iterator<Item = JsxNode<'_, 'a>> {
        (0..self.records.len()).map(|index| JsxNode::new(self, index))
    }
}

impl<'a> RejectedJsxFile<'a> {
    fn new(observation: ProgramObservation<'a>, error: JsxFileError) -> Self {
        Self {
            observation,
            error,
            file: None,
            rejected_file: None,
            records: alloc::vec::Vec::new(),
        }
    }
    #[must_use]
    pub fn error(&self) -> JsxFileError {
        self.error
    }
    #[must_use]
    pub fn observation(&self) -> &ProgramObservation<'a> {
        &self.observation
    }
    #[must_use]
    pub fn file(&self) -> Option<&FileArtifact<'a>> {
        self.file.as_ref()
    }
    #[must_use]
    pub fn rejected_file(&self) -> Option<&RejectedFile<'a>> {
        self.rejected_file.as_deref()
    }
    /// Partial recording is diagnostic evidence, never a completed node view.
    #[must_use]
    pub fn recorded_nodes(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests;
