//! Restricted Program admission and the actual file construction capability.

use crate::artifact::ArtifactError;
use crate::file::{
    FileArtifact, FileBuilder, FileIssueKind, RejectedFile, ScopeId, ScriptProfile, ScriptUnitId,
};
use crate::file::{TemplatePolicy, TemplateRegion, TemplateScope};
use crate::resolution::source::ProgramReferenceSource;
use oxc_parser::{AdmittedProgram, ParseOptions};
use vize_l0::{Allocator, SourceBlock, Span};

mod guard;
pub(crate) mod observer;
mod walk;
use guard::ProgramWalkGuard;
use observer::{FileObserver, NoObserver};

/// Which actual lexical boundary this language unit introduces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramScope {
    Module,
    Nested,
}

/// A rejected source/profile never provides a semantic declaration capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgramInputError {
    pub span: Span,
    pub kind: FileIssueKind,
}

/// Whole original syntax, checked against its complete source block/profile.
///
/// Admission comes from the original parser-owned no-hole observation, not a
/// caller's AST or discarded diagnostic list. Native conversion retains its
/// whole NativeSyntax owner, comments and normally owned diagnostics.
///
/// Raw Programs cannot establish admission:
/// ```compile_fail
/// use oxc_ast::ast::Program;
/// use vize_l0::SourceBlock;
/// use vize_l2::lang::js::ProgramInput;
/// fn forge<'p, 'a>(program: &'p Program<'a>, block: SourceBlock<'a>) {
///     let _ = ProgramInput::checked(program, block, 0);
/// }
/// ```
pub struct ProgramInput<'p, 'a> {
    references: ProgramReferenceSource<'p, 'a>,
    block: SourceBlock<'a>,
    index: u32,
    profile: ScriptProfile,
}

impl<'p, 'a> ProgramInput<'p, 'a> {
    #[must_use]
    pub fn span(&self) -> Span {
        self.block.span()
    }
    /// Original stock observation profile, already checked against this Program.
    #[must_use]
    pub fn source_type(&self) -> oxc_span::SourceType {
        self.references.program().source_type
    }
    pub fn checked(
        admitted: AdmittedProgram<'p, 'a>,
        block: SourceBlock<'a>,
        container_index: usize,
    ) -> Result<Self, ProgramInputError> {
        let reject = |kind| ProgramInputError {
            span: block.span(),
            kind,
        };
        let source_type = admitted.source_type();
        let program = admitted.program();
        if source_type.is_unambiguous()
            || program.source_type != source_type
            || admitted.options() != ParseOptions::default()
        {
            return Err(reject(FileIssueKind::InvalidProfile));
        }
        if !core::ptr::eq(admitted.source(), block.source()) {
            return Err(reject(FileIssueKind::InvalidSource));
        }
        let references = ProgramReferenceSource::checked(
            program,
            block.root_source(),
            block.span(),
            source_type,
        )
        .map_err(|_| reject(FileIssueKind::InvalidSource))?;
        let index =
            u32::try_from(container_index).map_err(|_| reject(FileIssueKind::BindingLimit))?;
        Ok(Self {
            references,
            block,
            index,
            profile: ScriptProfile {
                typescript: source_type.is_typescript(),
                jsx: source_type.is_jsx(),
                module: source_type.is_module(),
            },
        })
    }
}

/// Sole source of file-local declarations from original admitted observations.
pub struct FileProducer<'a> {
    builder: FileBuilder<'a>,
}

impl<'a> FileProducer<'a> {
    pub fn new(allocator: &'a Allocator, source: &'a str) -> Result<Self, ArtifactError> {
        Ok(Self {
            builder: FileBuilder::new(allocator, source)?,
        })
    }

    pub fn program(
        &mut self,
        input: ProgramInput<'_, 'a>,
        scope: ProgramScope,
    ) -> Result<ScriptUnitId, ProgramInputError> {
        self.program_observed(input, scope, &mut NoObserver)
            .map(|(unit, _)| unit)
    }

    pub fn program_observed<O: FileObserver<'a>>(
        &mut self,
        input: ProgramInput<'_, 'a>,
        scope: ProgramScope,
        observer: &mut O,
    ) -> Result<(ScriptUnitId, ScopeId), ProgramInputError> {
        if !core::ptr::eq(input.block.root_source(), self.builder.source) {
            return Err(ProgramInputError {
                span: input.block.span(),
                kind: FileIssueKind::InvalidSource,
            });
        }
        let Some((unit, lexical_scope)) = self.builder.facts.unit(
            input.index,
            input.block.span(),
            input.profile,
            scope == ProgramScope::Nested,
            crate::file::ProgramOrigin::checked(input.references.program()),
        ) else {
            return Err(ProgramInputError {
                span: input.block.span(),
                kind: FileIssueKind::DuplicateUnit,
            });
        };
        let Some(mut guard) = ProgramWalkGuard::new(&mut self.builder.facts) else {
            return Err(ProgramInputError {
                span: input.block.span(),
                kind: FileIssueKind::InvalidSpan,
            });
        };
        if let Some(unit) = guard.facts().units.last() {
            observer.unit(unit);
        }
        if !input.profile.module
            || input
                .references
                .program()
                .source_type
                .is_typescript_definition()
        {
            guard
                .facts()
                .issue(unit, input.block.span(), FileIssueKind::InvalidProfile);
        } else {
            walk::program(&input, guard.facts(), unit, lexical_scope, observer);
        }
        guard.complete();
        Ok((unit, lexical_scope))
    }

    pub fn finish(self) -> Result<FileArtifact<'a>, RejectedFile<'a>> {
        self.builder.finish()
    }

    #[must_use]
    pub fn issues(&self) -> &[crate::file::FileIssue] {
        &self.builder.facts.issues
    }

    pub fn interrupted_programs(&self) -> impl Iterator<Item = crate::file::FileIssue> + '_ {
        self.builder
            .facts
            .units
            .iter()
            .filter_map(crate::file::ScriptUnit::interruption)
    }

    /// A lower diagnostic factory cannot confer native SFC admission.
    pub fn template_region<P: TemplatePolicy>(
        &mut self,
        selection: TemplateScope,
        policy: P,
    ) -> Result<TemplateRegion<'_, 'a, P>, FileIssueKind> {
        let scope = match selection {
            TemplateScope::Root => ScopeId(0),
            TemplateScope::LastUnit => {
                self.builder
                    .facts
                    .units
                    .last()
                    .ok_or(FileIssueKind::InvalidSpan)?
                    .scope
            }
        };
        let builder = &mut self.builder;
        Ok(TemplateRegion::new(crate::file::region::RootRegion::root(
            builder.canonical.region(),
            &mut builder.facts,
            scope,
            policy,
            builder.source,
        )))
    }
}
