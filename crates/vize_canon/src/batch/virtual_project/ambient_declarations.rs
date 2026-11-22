//! Project-local references must resolve to the same native modules as rewritten Vue imports.
use super::{VirtualProject, build::mirrored_canonical_virtual_path};
use crate::{
    batch::{
        declaration_path::is_declaration_file,
        error::{CorsaError, CorsaResult},
    },
    virtual_ts::VirtualTsOptions,
};
use std::path::{Path, PathBuf};
use vize_carton::FxHashSet;

impl VirtualProject {
    /// Root references are explicit revision inputs. Every dependency uses
    /// the same absolute spelling, including Vue hosts in nested packages.
    pub(crate) fn project_declaration_references(
        &self,
        options: &mut VirtualTsOptions,
        requested: &[(PathBuf, &str)],
    ) -> CorsaResult<()> {
        if requested.is_empty() || options.reference_paths.is_empty() {
            return Ok(());
        }
        let supplied: FxHashSet<_> = requested
            .iter()
            .map(|(path, _)| path)
            .filter(|path| path.is_absolute() && is_declaration_file(path))
            .map(|path| vize_l0::path::physical_path(path))
            .filter(|path| path.starts_with(self.project_root()))
            .collect();
        if supplied.is_empty() {
            return Ok(());
        }
        for reference in &mut options.reference_paths {
            let path = Path::new(reference.as_str());
            if !path.is_absolute() || !is_declaration_file(path) {
                continue;
            }
            let Some(original) = supplied
                .get(path)
                .or_else(|| supplied.get(&vize_l0::path::physical_path(path)))
            else {
                continue;
            };
            crate::virtual_ts::validate_reference_paths(std::slice::from_ref(reference)).map_err(
                |error| {
                    CorsaError::Configuration(vize_carton::cstr!(
                        "Invalid declaration reference path {:?}: {}",
                        error.path,
                        error.reason
                    ))
                },
            )?;
            // Session scripts preserve .d.ts, .d.mts and .d.cts spellings.
            let mirror = mirrored_canonical_virtual_path(
                self.project_root(),
                self.virtual_root(),
                original,
            )?;
            *reference = mirror
                .to_str()
                .ok_or_else(|| CorsaError::PathError {
                    path: mirror.clone(),
                })?
                .into();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_explicit_original_local_declarations_move_to_the_private_project() {
        let local = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let storage = tempfile::tempdir().unwrap();
        let project_root = vize_l0::path::physical_path(local.path());
        let mut project = VirtualProject::new(&project_root).unwrap();
        project.scope_editor_namespace(storage.path(), 1);
        let component = project_root.join("nested/auto.d.ts");
        let unsaved = project_root.join("unsaved.d.cts");
        let inferred = project_root.join("inferred.d.mts");
        let other = external.path().join("package.d.ts");
        let ordinary = project_root.join("source.ts");
        let relative = Path::new("./local.d.ts");
        let names = [
            &component, &other, &unsaved, &inferred, &component, &ordinary,
        ];
        let mut options = VirtualTsOptions::default();
        options
            .reference_paths
            .extend(names.map(|p| p.to_str().unwrap().into()));
        options
            .reference_paths
            .push(relative.to_str().unwrap().into());
        let inputs = [&component, &other, &unsaved, &ordinary].map(|p| (p.clone(), "export {}"));
        project
            .project_declaration_references(&mut options, &inputs)
            .unwrap();
        let mirror = project.virtual_root();
        let expected = [
            mirror.join("nested/auto.d.ts"),
            other,
            mirror.join("unsaved.d.cts"),
            inferred,
            mirror.join("nested/auto.d.ts"),
            ordinary,
            relative.into(),
        ];
        assert!(
            options
                .reference_paths
                .iter()
                .map(|p| Path::new(p.as_str()))
                .eq(expected.iter().map(PathBuf::as_path))
        );
    }
}
