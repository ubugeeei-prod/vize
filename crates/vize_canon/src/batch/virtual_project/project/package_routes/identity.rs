//! Workspace identity lookup bounded by the existing authoritative importer index.
use std::path::Path;

use super::VirtualProject;

#[cfg(test)]
mod tests;

impl VirtualProject {
    pub(crate) fn workspace_package_route_identity(
        &self,
        importer: &Path,
        specifier: &str,
    ) -> Option<&Path> {
        let keys = self.package_route_importers.get(importer)?;
        let mut identity: Option<&Path> = None;
        for key in keys {
            if key.specifier != specifier {
                continue;
            }
            let Some(binding) = self.package_routes.get(key) else {
                continue;
            };
            let Some(route) = binding
                .route
                .as_ref()
                .filter(|route| route.workspace_source)
            else {
                continue;
            };
            let candidate = route.package_root.as_path();
            if identity.is_some_and(|current| current.as_os_str() != candidate.as_os_str()) {
                // The caller formats this path, so even component-equivalent raw
                // spellings must retain the original exact choice.
                // Import and require occurrences can select different routes.
                // Preserve the previous exact whole-map choice for that ambiguity.
                return self.workspace_package_route_identity_scan(importer, specifier);
            }
            identity = Some(candidate);
        }
        identity
    }

    fn workspace_package_route_identity_scan(
        &self,
        importer: &Path,
        specifier: &str,
    ) -> Option<&Path> {
        self.package_routes
            .values()
            .find(|binding| {
                binding.importer_path == importer
                    && binding.specifier == specifier
                    && binding
                        .route
                        .as_ref()
                        .is_some_and(|route| route.workspace_source)
            })
            .and_then(|binding| binding.route.as_ref())
            .map(|route| route.package_root.as_path())
    }
}
