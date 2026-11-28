//! Restricted Program admission and the actual file construction capability.

use crate::ir::{
    L2Ir, L2IrBuilder, L2IrIssueKind, RejectedL2Ir, ScopeId, ScriptProfile, ScriptUnitId,
};
use crate::ir::{TemplatePolicy, TemplateRegion, TemplateScope};
use crate::resolution::source::ProgramReferenceSource;
use crate::template::L2TemplateError;
use oxc_parser::{AdmittedProgram, ParseOptions};
use vize_l0::{Allocator, SourceBlock, Span};

mod guard;
mod module_source;
pub use module_source::{
    ModuleSourceError, ModuleSourceErrorKind, ModuleSourceKind, OriginalModuleSource,
    OriginalModuleSources,
};
pub mod native;
pub(crate) mod observer;
pub(crate) mod ordinary;
pub(crate) mod setup;
mod walk;
use guard::ProgramWalkGuard;
use observer::{L2IrObserver, NoObserver};

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
    pub kind: L2IrIssueKind,
}

/// Whole original syntax, checked against its complete source block/profile.
///
/// Admission comes from the original parser-owned no-hole observation, not a
/// caller's AST or discarded diagnostic list. Native conversion retains its
/// whole EmbedSyntax owner, comments and normally owned diagnostics.
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
    #[cfg(feature = "lexical-arrow-initializers")]
    lexical_arrow_initializers: bool,
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
            return Err(reject(L2IrIssueKind::InvalidProfile));
        }
        if !core::ptr::eq(admitted.source(), block.source()) {
            return Err(reject(L2IrIssueKind::InvalidSource));
        }
        let references = ProgramReferenceSource::checked(
            admitted,
            block.root_source(),
            block.span(),
            source_type,
        )
        .map_err(|_| reject(L2IrIssueKind::InvalidSource))?;
        let index =
            u32::try_from(container_index).map_err(|_| reject(L2IrIssueKind::BindingLimit))?;
        Ok(Self {
            references,
            #[cfg(feature = "lexical-arrow-initializers")]
            lexical_arrow_initializers: false,
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
pub struct L2IrProducer<'a> {
    #[cfg(feature = "lexical-arrow-initializers")]
    lexical_arrow_initializers: bool,
    builder: L2IrBuilder<'a>,
}

impl<'a> L2IrProducer<'a> {
    pub fn new(allocator: &'a Allocator, source: &'a str) -> Result<Self, L2TemplateError> {
        Ok(Self {
            builder: L2IrBuilder::new(allocator, source)?,
            #[cfg(feature = "lexical-arrow-initializers")]
            lexical_arrow_initializers: false,
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

    pub fn program_observed<O: L2IrObserver<'a>>(
        &mut self,
        input: ProgramInput<'_, 'a>,
        scope: ProgramScope,
        observer: &mut O,
    ) -> Result<(ScriptUnitId, ScopeId), ProgramInputError> {
        if !core::ptr::eq(input.block.root_source(), self.builder.source) {
            return Err(ProgramInputError {
                span: input.block.span(),
                kind: L2IrIssueKind::InvalidSource,
            });
        }
        #[cfg(feature = "lexical-arrow-initializers")]
        let input = if self.lexical_arrow_initializers {
            input.with_lexical_arrow_initializers()
        } else {
            input
        };
        let Some((unit, lexical_scope)) = self.builder.facts.unit(
            input.index,
            input.block.span(),
            input.profile,
            scope == ProgramScope::Nested,
            crate::ir::ProgramOrigin::checked(input.references.admitted()),
        ) else {
            return Err(ProgramInputError {
                span: input.block.span(),
                kind: L2IrIssueKind::DuplicateUnit,
            });
        };
        let Some(mut guard) = ProgramWalkGuard::new(&mut self.builder.facts) else {
            return Err(ProgramInputError {
                span: input.block.span(),
                kind: L2IrIssueKind::InvalidSpan,
            });
        };
        #[cfg(feature = "lexical-arrow-initializers")]
        if input.lexical_arrow_initializers {
            guard.facts().expressions.in_arena = true;
        }
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
                .issue(unit, input.block.span(), L2IrIssueKind::InvalidProfile);
        } else {
            walk::program(&input, guard.facts(), unit, lexical_scope, observer);
        }
        guard.complete();
        Ok((unit, lexical_scope))
    }

    pub fn finish(self) -> Result<L2Ir<'a>, RejectedL2Ir<'a>> {
        self.builder.finish()
    }

    /// Only the original-owning JSX producer may populate this File's markup.
    pub(super) fn jsx_region(&mut self) -> crate::template::RegionBuilder<'_, 'a> {
        self.builder.canonical.region()
    }

    /// Original sole completed ordinary script facts, before template custody.
    /// This borrows the producer and grants no finished File or product authority.
    #[must_use]
    pub fn ordinary_empty_script(&self) -> Option<ordinary::OrdinaryEmptyScript<'_>> {
        ordinary::OrdinaryEmptyScript::from_facts(&self.builder.facts)
    }

    #[must_use]
    pub fn issues(&self) -> &[crate::ir::L2IrIssue] {
        &self.builder.facts.issues
    }

    pub fn interrupted_programs(&self) -> impl Iterator<Item = crate::ir::L2IrIssue> + '_ {
        self.builder
            .facts
            .units
            .iter()
            .filter_map(crate::ir::ScriptUnit::interruption)
    }

    /// A lower diagnostic factory cannot confer native SFC admission.
    pub fn template_region<P: TemplatePolicy>(
        &mut self,
        selection: TemplateScope,
        policy: P,
    ) -> Result<TemplateRegion<'_, 'a, P>, L2IrIssueKind> {
        let scope = match selection {
            TemplateScope::Root => ScopeId(0),
            TemplateScope::LastUnit => {
                self.builder
                    .facts
                    .units
                    .last()
                    .ok_or(L2IrIssueKind::InvalidSpan)?
                    .scope
            }
        };
        let builder = &mut self.builder;
        Ok(TemplateRegion::new(crate::ir::region::RootRegion::root(
            builder.canonical.region(),
            &mut builder.facts,
            scope,
            policy,
            builder.source,
        )))
    }
}

#[cfg(feature = "lexical-arrow-initializers")]
impl ProgramInput<'_, '_> {
    /// Request lexical arrow-initializer analysis during this original Program walk.
    #[must_use]
    pub fn with_lexical_arrow_initializers(mut self) -> Self {
        self.lexical_arrow_initializers = true;
        self
    }
}

#[cfg(feature = "lexical-arrow-initializers")]
impl L2IrProducer<'_> {
    /// Enable lexical arrow initializers in each admitted original Program of this file.
    pub fn enable_lexical_arrow_initializers(&mut self) {
        self.lexical_arrow_initializers = true;
    }

    /// Bounded capacity hints from the original surface. No nodes, issues or
    /// resolution facts are created by this operation.
    pub fn reserve_template_storage(&mut self, roots: usize, expressions: usize) {
        let roots = roots.min(32);
        let expressions = expressions.min(32);
        self.builder
            .canonical
            .reserve_template_storage(roots, expressions);
        self.builder.facts.expressions.reserve_if_empty(expressions);
    }
}
