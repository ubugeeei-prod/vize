//! Producer-owned diagnostic completeness, outside the public virtual-file DTO.

use super::VirtualProject;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::batch) enum DiagnosticRoute {
    #[default]
    NativeCliAllowed,
    /// This projection deliberately retains known-invalid syntax. The native
    /// CLI cannot complete its semantic phase; use the same-config session.
    ProjectSessionRequired,
}

impl VirtualProject {
    pub(in crate::batch) fn requires_project_session(&self) -> bool {
        // Source ownership already handles replacement/removal and canonical
        // aliases. Derive the route from that live metadata instead of keeping
        // a second invalidation-sensitive cache or reparsing generated text.
        self.source_artifacts
            .values()
            .any(|artifacts| artifacts.diagnostic_route == DiagnosticRoute::ProjectSessionRequired)
    }
}

#[cfg(test)]
mod tests;
