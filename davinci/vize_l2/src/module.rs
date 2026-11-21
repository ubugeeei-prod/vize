//! An authored host module with component render edits in source order.
//! Position, formatting and emission consumers can visit the same partition.

pub use crate::component::ComponentDef;
use crate::file::ScriptProfile;
use alloc::vec::Vec;
use vize_l0::{Lang, Span};
use vize_l1::embed::{
    regions::{ModuleRegions, ModuleSurface},
    syntax::ProgramGoal,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleError {
    /// The lowered definitions do not cover the original outer regions in order.
    Roots,
    /// A definition's operations index another source buffer.
    ForeignSource,
}

/// Complete render edits of a host module. The parsed regions establish which
/// bytes belong to the host and the L2 definitions own the render operations.
#[derive(Debug)]
pub struct ModuleDef<'a> {
    surface: ModuleSurface<'a>,
    components: Vec<ComponentDef<'a>>,
}

/// One original run. Its span is absolute even at non-ASCII boundaries.
#[derive(Clone, Copy, Debug)]
pub enum ModulePart<'m, 'a> {
    Host { span: Span, text: &'a str },
    Render(&'m ComponentDef<'a>),
}

impl<'a> ModuleDef<'a> {
    pub fn checked(
        regions: ModuleRegions<'a>,
        components: Vec<ComponentDef<'a>>,
    ) -> Result<Self, ModuleError> {
        if regions.roots().len() != components.len()
            || regions
                .roots()
                .iter()
                .zip(&components)
                .any(|(span, component)| *span != component.span)
        {
            return Err(ModuleError::Roots);
        }
        if components
            .iter()
            .any(|component| !core::ptr::eq(component.body.source, regions.surface().source()))
        {
            return Err(ModuleError::ForeignSource);
        }
        Ok(Self {
            surface: regions.into_surface(),
            components,
        })
    }

    #[must_use]
    pub fn source(&self) -> &'a str {
        self.surface.source()
    }
    #[must_use]
    pub fn host(&self) -> ScriptProfile {
        let host = self.surface.host();
        ScriptProfile {
            typescript: host.lang == Lang::Ts,
            jsx: host.jsx,
            module: host.goal == ProgramGoal::Module,
        }
    }
    #[must_use]
    pub fn components(&self) -> &[ComponentDef<'a>] {
        &self.components
    }
    /// Original surface starts that must never appear in a verbatim expression.
    #[must_use]
    pub fn protected_starts(&self) -> &[u32] {
        self.surface.protected_starts()
    }

    /// Visit each host byte exactly once, in order. Adjacent edits and file
    /// boundaries do not introduce empty host runs. The first refusal stops the walk.
    pub fn visit<'m, E>(
        &'m self,
        mut part: impl FnMut(ModulePart<'m, 'a>) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut cursor = 0;
        for component in &self.components {
            if cursor < component.span.start {
                let span = Span::new(cursor, component.span.start);
                part(ModulePart::Host {
                    span,
                    text: span.slice(self.source()),
                })?;
            }
            part(ModulePart::Render(component))?;
            cursor = component.span.end;
        }
        if cursor < self.surface.end() {
            let span = Span::new(cursor, self.surface.end());
            part(ModulePart::Host {
                span,
                text: span.slice(self.source()),
            })?;
        }
        Ok(())
    }
}

#[cfg(feature = "component-sites")]
pub use vize_l1::embed::regions::ComponentSites;

#[cfg(feature = "component-sites")]
impl ModuleDef<'_> {
    /// Source sites captured with this module by the original frontend.
    #[must_use]
    pub fn component_sites(&self) -> Option<&ComponentSites> {
        self.surface.component_sites()
    }
}
