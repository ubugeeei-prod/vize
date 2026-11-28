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
use vize_carton::{FxHashMap, FxHashSet};

impl VirtualProject {
    /// Project-local declarations the editor references by absolute path that
    /// import a `.vue` module, read from their open buffer or the disk. They
    /// are mirrored like the host's imports, so a `GlobalComponents` entry
    /// resolves to the generated module as it does in the batch checker. Other
    /// declarations already resolve from their authored path and keep it, as
    /// does an unreadable reference.
    pub(crate) fn referenced_project_declarations(
        &self,
        options: &VirtualTsOptions,
        overlays: &FxHashMap<PathBuf, &str>,
        requested: &[(PathBuf, &str)],
    ) -> Vec<(PathBuf, vize_carton::String)> {
        let mut declarations = Vec::new();
        for reference in &options.reference_paths {
            let path = Path::new(reference.as_str());
            if !path.is_absolute() || !is_declaration_file(path) {
                continue;
            }
            let physical = vize_l0::path::physical_path(path);
            if !physical.starts_with(self.project_root())
                || requested
                    .iter()
                    .any(|(requested, _)| requested == path || *requested == physical)
                || declarations
                    .iter()
                    .any(|(known, _): &(PathBuf, _)| *known == physical)
            {
                continue;
            }
            let text = match overlays.get(path).or_else(|| overlays.get(&physical)) {
                Some(open) => vize_carton::String::from(*open),
                None => match std::fs::read(&physical)
                    .ok()
                    .and_then(|bytes| vize_carton::String::from_utf8(bytes).ok())
                {
                    Some(text) => text,
                    None => continue,
                },
            };
            if imports_vue_module(&physical, &text) {
                declarations.push((physical, text));
            }
        }
        declarations
    }

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

/// Whether a declaration names a `.vue` module in an import, export or type query.
pub(crate) fn imports_vue_module(path: &Path, text: &str) -> bool {
    is_declaration_file(path)
        && text.contains(".vue")
        && crate::batch::ModuleSpecifierFactsCache::default()
            .collect_all_specifiers(text, oxc_span::SourceType::d_ts())
            .iter()
            .any(|specifier| specifier.ends_with(".vue"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> std::string::String {
        std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/_fixtures/differential/typechecker/editor-referenced-global-components")
                .join(name),
        )
        .unwrap()
    }

    #[test]
    fn referenced_local_declarations_are_read_from_open_buffers_or_disk() {
        let local = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let storage = tempfile::tempdir().unwrap();
        let root = vize_l0::path::physical_path(local.path());
        let mut project = VirtualProject::new(&root).unwrap();
        project.scope_editor_namespace(storage.path(), 1);
        let provider = fixture("components.d.ts.txt");
        let closed = root.join("src/components.d.ts");
        let open = root.join("src/open.d.ts");
        let routes = root.join("typed-router.d.ts");
        let requested = root.join("src/requested.d.ts");
        let missing = root.join("src/missing.d.ts");
        let outside = external.path().join("outside.d.ts");
        std::fs::create_dir_all(closed.parent().unwrap()).unwrap();
        std::fs::write(&closed, &provider).unwrap();
        std::fs::write(&open, "export {}\n").unwrap();
        std::fs::write(&requested, "export {}\n").unwrap();
        std::fs::write(&outside, &provider).unwrap();
        // Route keys name `.vue` files without importing them.
        std::fs::write(
            &routes,
            "declare module 'vue-router' { interface Routes { 'src/pages/index.vue': {} } }\nexport {}\n",
        )
        .unwrap();
        let mut options = VirtualTsOptions::default();
        for path in [
            &closed, &open, &requested, &missing, &outside, &routes, &closed,
        ] {
            options.reference_paths.push(path.to_str().unwrap().into());
        }
        let unsaved = "import type Open from './Open.vue'\nexport {}\n";
        let overlays = FxHashMap::from_iter([(open.clone(), unsaved)]);
        let requested_sources = [(requested.clone(), "export {}\n")];
        let declarations =
            project.referenced_project_declarations(&options, &overlays, &requested_sources);
        assert_eq!(
            declarations
                .iter()
                .map(|(path, text)| (path.as_path(), text.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (closed.as_path(), provider.as_str()),
                (open.as_path(), unsaved),
            ]
        );
    }

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
        let inferred = project_root.join("inferred.d.ts");
        let module = project_root.join("nested/auto.d.mts");
        let other = external.path().join("package.d.ts");
        let ordinary = project_root.join("source.ts");
        let relative = Path::new("./local.d.ts");
        let names = [
            &component, &other, &unsaved, &module, &inferred, &component, &ordinary,
        ];
        let mut options = VirtualTsOptions::default();
        options
            .reference_paths
            .extend(names.map(|p| p.to_str().unwrap().into()));
        options
            .reference_paths
            .push(relative.to_str().unwrap().into());
        let inputs =
            [&component, &other, &unsaved, &module, &ordinary].map(|p| (p.clone(), "export {}"));
        project
            .project_declaration_references(&mut options, &inputs)
            .unwrap();
        let mirror = project.virtual_root();
        let expected = [
            mirror.join("nested/auto.d.ts"),
            other,
            mirror.join("unsaved.d.cts"),
            mirror.join("nested/auto.d.mts"),
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
    #[test]
    fn validate_authored_pragma_spelling_before_replacing_the_path() {
        let storage = tempfile::tempdir().unwrap();
        let project = VirtualProject::new(storage.path()).unwrap();
        let invalid = project.project_root().join("both'\".d.ts");
        let reference = invalid.to_str().unwrap();
        let mut options = VirtualTsOptions::default();
        options.reference_paths.push(reference.into());
        let result =
            project.project_declaration_references(&mut options, &[(invalid.clone(), "export {}")]);
        let Err(CorsaError::Configuration(message)) = result else {
            panic!("expected original reference rejection");
        };
        assert_eq!(
            message.as_str(),
            format!(
                "Invalid declaration reference path {reference:?}: a TypeScript reference path cannot contain both quote delimiters"
            )
        );
        assert_eq!(options.reference_paths.as_slice(), &[reference]);
    }
}
