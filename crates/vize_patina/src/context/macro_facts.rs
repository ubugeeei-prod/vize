//! Template and macro-type facts shared by SFC rules within one lint pass.

use super::LintContext;
use crate::linter::run_type_sources::RunTypeSources;
use std::cell::OnceCell;
use vize_croquis::types::ResolvedTypeWorld;
use vize_l1::markup::SurfaceTree;

#[derive(Default)]
pub(crate) struct MacroFacts<'a> {
    template_surface: Option<&'a SurfaceTree<'a>>,
    type_sources: Option<&'a RunTypeSources>,
    world: OnceCell<Option<ResolvedTypeWorld>>,
}

impl<'a> LintContext<'a> {
    /// Attach the validated template surface of this template pass.
    #[inline]
    pub(crate) fn set_template_surface(&mut self, surface: &'a SurfaceTree<'a>) {
        self.macro_facts.template_surface = Some(surface);
    }

    /// The validated template surface, in a template pass.
    #[inline]
    pub(crate) fn template_surface(&self) -> Option<&'a SurfaceTree<'a>> {
        self.macro_facts.template_surface
    }

    /// Resolve imported macro types against the linter's shared disk view.
    #[inline]
    pub(crate) fn set_type_sources(&mut self, sources: Option<&'a RunTypeSources>) {
        self.macro_facts.type_sources = sources;
    }

    /// The disk view imported macro types resolve against, when shared.
    #[inline]
    pub(crate) fn type_sources(&self) -> Option<&'a RunTypeSources> {
        self.macro_facts.type_sources
    }

    /// The cross-file world that resolves the SFC's imported macro types.
    ///
    /// The analysis world is reused when present; otherwise the world is
    /// built on the first call, so a rule asks only for a type it cannot
    /// resolve within the SFC.
    pub(crate) fn macro_type_world(&self) -> Option<&ResolvedTypeWorld> {
        if let Some(world) = self
            .analysis()
            .and_then(|analysis| analysis.types.resolved_world())
        {
            return Some(world);
        }
        self.macro_facts
            .world
            .get_or_init(|| {
                let descriptor = self.sfc_descriptor()?;
                match self.macro_facts.type_sources {
                    Some(sources) => {
                        vize_atelier_sfc::croquis::resolve_descriptor_type_world_with_sources(
                            descriptor,
                            &sources.importer(self.filename),
                            &sources.snapshot,
                        )
                    }
                    None => vize_atelier_sfc::croquis::resolve_descriptor_type_world(
                        descriptor,
                        self.filename,
                    ),
                }
            })
            .as_ref()
    }
}
