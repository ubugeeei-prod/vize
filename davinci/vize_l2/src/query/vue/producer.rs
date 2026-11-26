//! Observe the original scripts in lexical order. Descriptor roles, language,
//! source, static-hoist policy and scope all come from the same SFC.

use super::{Observed, OrdinaryFamilies, VueAnalysis, VueFile};
use crate::artifact::L2TemplateError;
use crate::file::{FileIssueKind, RejectedFile, TemplatePolicy, TemplateRegion, TemplateScope};
use crate::framework::vue::{OptionsObserver, OrdinaryObserver, SetupObserver};
use crate::lang::js::{FileProducer, ProgramInput, ProgramInputError, ProgramScope};
use alloc::boxed::Box;
use oxc_parser::AdmittedProgram;
use vize_l0::Allocator;
use vize_l1::container::vue::{AdmittedDescriptor, ScriptRole, ScriptView};
use vize_l1::embed::syntax::ProgramOptions;

/// Normal-script families to observe in one traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrdinaryDemand {
    Bindings,
    Options,
    Both,
}

/// Which additional facts to retain alongside the File's lexical declarations.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VueDemand {
    pub setup: bool,
    pub ordinary: Option<OrdinaryDemand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VueProgramError {
    Missing(ScriptRole),
    Unexpected(ScriptRole),
    Program(ProgramInputError),
    Artifact(L2TemplateError),
}

/// A File whose present scripts have all been walked in their original roles.
/// The template producer may then attach its typed region before sealing it.
pub struct VueFileProducer<'a> {
    producer: FileProducer<'a>,
    facts: VueAnalysis,
}

impl<'a> VueFileProducer<'a> {
    /// Accept exactly the Programs certified by this descriptor, including any
    /// script for which the caller requests only the usual lexical facts.
    pub fn scripts(
        allocator: &'a Allocator,
        descriptor: AdmittedDescriptor<'_, 'a>,
        ordinary: Option<AdmittedProgram<'_, 'a>>,
        setup: Option<AdmittedProgram<'_, 'a>>,
        demand: VueDemand,
    ) -> Result<Self, VueProgramError> {
        let ordinary = input(descriptor.ordinary(), ordinary, ScriptRole::Ordinary)?;
        let setup = input(descriptor.setup(), setup, ScriptRole::Setup)?;
        let mut this = Self {
            producer: FileProducer::new(allocator, descriptor.source())
                .map_err(VueProgramError::Artifact)?,
            facts: VueAnalysis::default(),
        };
        if let Some(input) = ordinary {
            if let Some(demand) = demand.ordinary {
                this.ordinary(input, demand)
                    .map_err(VueProgramError::Program)?;
            } else {
                this.producer
                    .program(input, ProgramScope::Module)
                    .map_err(VueProgramError::Program)?;
            }
        }
        if let Some(input) = setup {
            if demand.setup {
                let mut observer =
                    SetupObserver::new().with_hoist_static(descriptor.ordinary().is_none());
                let (unit, scope) = this
                    .producer
                    .program_observed(input, ProgramScope::Nested, &mut observer)
                    .map_err(VueProgramError::Program)?;
                this.facts.setup = Some(Box::new(Observed {
                    unit,
                    scope,
                    facts: observer.finish(),
                }));
            } else {
                this.producer
                    .program(input, ProgramScope::Nested)
                    .map_err(VueProgramError::Program)?;
            }
        }
        Ok(this)
    }

    fn ordinary(
        &mut self,
        input: ProgramInput<'_, 'a>,
        demand: OrdinaryDemand,
    ) -> Result<(), ProgramInputError> {
        let scope = ProgramScope::Module;
        let ((unit, scope), bindings, options) = match demand {
            OrdinaryDemand::Bindings => {
                let mut observer = OrdinaryObserver::new();
                let id = self
                    .producer
                    .program_observed(input, scope, &mut observer)?;
                (id, Some(observer.finish()), None)
            }
            OrdinaryDemand::Options => {
                let mut observer = OptionsObserver::new();
                let id = self
                    .producer
                    .program_observed(input, scope, &mut observer)?;
                (id, None, Some(observer.finish()))
            }
            OrdinaryDemand::Both => {
                let mut observers = (OrdinaryObserver::new(), OptionsObserver::new());
                let id = self
                    .producer
                    .program_observed(input, scope, &mut observers)?;
                (id, Some(observers.0.finish()), Some(observers.1.finish()))
            }
        };
        self.facts.ordinary = Some(Box::new(Observed {
            unit,
            scope,
            facts: OrdinaryFamilies { bindings, options },
        }));
        Ok(())
    }

    pub fn template_region<P: TemplatePolicy>(
        &mut self,
        selection: TemplateScope,
        policy: P,
    ) -> Result<TemplateRegion<'_, 'a, P>, FileIssueKind> {
        self.producer.template_region(selection, policy)
    }

    pub fn finish(self) -> Result<VueFile<'a>, RejectedFile<'a>> {
        Ok(VueFile {
            l2ir: self.producer.finish()?,
            facts: self.facts,
        })
    }
}

fn input<'p, 'a>(
    script: Option<ScriptView<'_, 'a>>,
    program: Option<AdmittedProgram<'p, 'a>>,
    role: ScriptRole,
) -> Result<Option<ProgramInput<'p, 'a>>, VueProgramError> {
    match (script, program) {
        (Some(script), Some(program)) => {
            if program.source_type() != ProgramOptions::module(script.lang()).source_type() {
                return Err(VueProgramError::Program(ProgramInputError {
                    span: script.block().span(),
                    kind: FileIssueKind::InvalidProfile,
                }));
            }
            ProgramInput::checked(program, script.block(), script.container_index())
                .map(Some)
                .map_err(VueProgramError::Program)
        }
        (Some(_), None) => Err(VueProgramError::Missing(role)),
        (None, Some(_)) => Err(VueProgramError::Unexpected(role)),
        (None, None) => Ok(None),
    }
}
