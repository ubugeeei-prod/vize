//! Original syntax and completed File facts owned with same-walk JSX structure.

use super::{L2IrProducer, ProgramInput, ProgramInputError, ProgramScope};
use crate::artifact::L2TemplateError;
use crate::file::{L2Ir, RejectedL2Ir};
use alloc::boxed::Box;
use oxc_parser::ProgramObservation;
use vize_l0::{Allocator, SourceBlock};

mod markup;
mod recorder;
mod records;
pub use markup::{JsxMarkupIssue, JsxMarkupIssueKind, JsxMarkupL2Ir, RejectedJsxMarkupL2Ir};
use recorder::Recorder;
pub use records::{JsxChildren, JsxNode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsxL2IrError {
    SyntaxAdmission,
    Input(ProgramInputError),
    Artifact(L2TemplateError),
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
/// use vize_l2::lang::js::JsxL2IrProducer;
/// fn forge<'a>(arena: &'a Allocator, program: Program<'a>, block: SourceBlock<'a>) {
///     let _ = JsxL2IrProducer::new(arena, program, block, 0);
/// }
/// ```
pub struct JsxL2IrProducer<'a> {
    observation: ProgramObservation<'a>,
    block: SourceBlock<'a>,
    index: usize,
    producer: L2IrProducer<'a>,
    recorder: Recorder<'a>,
    phase: Phase,
}

/// A completed source body retains its actual original observation and File.
///
/// ```compile_fail
/// use vize_l2::lang::js::JsxL2Ir;
/// fn forge<'a>() { let _ = JsxL2Ir { observation: todo!(), l2ir: todo!(), records: vec![] }; }
/// ```
pub struct JsxL2Ir<'a> {
    observation: ProgramObservation<'a>,
    l2ir: L2Ir<'a>,
    records: alloc::vec::Vec<records::Record<'a>>,
}

/// Failed admission, walk or sealing retains original syntax and actual facts.
pub struct RejectedJsxL2Ir<'a> {
    observation: ProgramObservation<'a>,
    error: JsxL2IrError,
    file: Option<L2Ir<'a>>,
    rejected_file: Option<Box<RejectedL2Ir<'a>>>,
    records: alloc::vec::Vec<records::Record<'a>>,
}

impl<'a> JsxL2IrProducer<'a> {
    pub fn new(
        allocator: &'a Allocator,
        observation: ProgramObservation<'a>,
        block: SourceBlock<'a>,
        container_index: usize,
    ) -> Result<Self, Box<RejectedJsxL2Ir<'a>>> {
        let admission = observation
            .admitted()
            .ok_or(JsxL2IrError::SyntaxAdmission)
            .and_then(|admitted| {
                let input = ProgramInput::checked(admitted, block, container_index)
                    .map_err(JsxL2IrError::Input)?;
                if !input.source_type().is_jsx() || !input.source_type().is_module() {
                    return Err(JsxL2IrError::InvalidProfile);
                }
                Ok(())
            });
        if let Err(error) = admission {
            return Err(Box::new(RejectedJsxL2Ir::new(observation, error)));
        }
        let producer = match L2IrProducer::new(allocator, block.root_source()) {
            Ok(producer) => producer,
            Err(error) => {
                return Err(Box::new(RejectedJsxL2Ir::new(
                    observation,
                    JsxL2IrError::Artifact(error),
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
    pub fn walk(&mut self) -> Result<(), JsxL2IrError> {
        self.walk_with(|producer, input, recorder| {
            producer
                .program_observed(input, ProgramScope::Module, recorder)
                .map(|_| ())
        })
    }

    fn walk_with(
        &mut self,
        visit: impl FnOnce(
            &mut L2IrProducer<'a>,
            ProgramInput<'_, 'a>,
            &mut Recorder<'a>,
        ) -> Result<(), ProgramInputError>,
    ) -> Result<(), JsxL2IrError> {
        if self.phase != Phase::Pending {
            return Err(JsxL2IrError::AlreadyWalked);
        }
        self.phase = Phase::Interrupted;
        let admitted = self
            .observation
            .admitted()
            .ok_or(JsxL2IrError::SyntaxAdmission)?;
        let input =
            ProgramInput::checked(admitted, self.block, self.index).map_err(JsxL2IrError::Input)?;
        visit(&mut self.producer, input, &mut self.recorder).map_err(JsxL2IrError::Input)?;
        self.phase = Phase::Complete;
        Ok(())
    }

    #[must_use]
    pub fn observation(&self) -> &ProgramObservation<'a> {
        &self.observation
    }

    pub fn finish(self) -> Result<JsxL2Ir<'a>, Box<RejectedJsxL2Ir<'a>>> {
        let Self {
            observation,
            producer,
            recorder,
            phase,
            ..
        } = self;
        let body_complete = recorder.frames_complete() && recorder.jsx_count() > 0;
        let mut rejection = RejectedJsxL2Ir::new(
            observation,
            match phase {
                Phase::Pending => JsxL2IrError::NotWalked,
                Phase::Interrupted => JsxL2IrError::Interrupted,
                Phase::Complete => JsxL2IrError::IncompleteBody,
            },
        );
        rejection.records = recorder.records;
        let file = match producer.finish() {
            Ok(file) => file,
            Err(file) => {
                rejection.error = JsxL2IrError::Artifact(file.artifact.error);
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
            rejection.error = JsxL2IrError::IncompleteFile;
            rejection.file = Some(file);
            return Err(Box::new(rejection));
        }
        if !body_complete {
            rejection.file = Some(file);
            return Err(Box::new(rejection));
        }
        Ok(JsxL2Ir {
            observation: rejection.observation,
            l2ir: file,
            records: rejection.records,
        })
    }
}

impl<'a> JsxL2Ir<'a> {
    #[must_use]
    pub fn observation(&self) -> &ProgramObservation<'a> {
        &self.observation
    }
    #[must_use]
    pub fn l2ir(&self) -> &L2Ir<'a> {
        &self.l2ir
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

impl<'a> RejectedJsxL2Ir<'a> {
    fn new(observation: ProgramObservation<'a>, error: JsxL2IrError) -> Self {
        Self {
            observation,
            error,
            file: None,
            rejected_file: None,
            records: alloc::vec::Vec::new(),
        }
    }
    #[must_use]
    pub fn error(&self) -> JsxL2IrError {
        self.error
    }
    #[must_use]
    pub fn observation(&self) -> &ProgramObservation<'a> {
        &self.observation
    }
    #[must_use]
    pub fn file(&self) -> Option<&L2Ir<'a>> {
        self.file.as_ref()
    }
    #[must_use]
    pub fn rejected_file(&self) -> Option<&RejectedL2Ir<'a>> {
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
