//! Sources another program of the same check owns enter this mirror as that
//! program's emitted declarations, the way `tsc -b` feeds an importer the
//! referenced project's output instead of checking its sources again.

use std::path::{Path, PathBuf};

use super::{VirtualProject, build::mirrored_canonical_virtual_path};
use crate::batch::CorsaResult;

/// The declaration file TypeScript emits for one mirrored source: an SFC
/// projection `X.vue.ts` becomes `X.vue.d.ts`, a script `X.ts` becomes
/// `X.d.ts`. Declaration inputs have no emitted twin.
fn emitted_declaration_name(name: &str) -> Option<vize_carton::String> {
    if name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts") {
        return None;
    }
    let stem = name
        .strip_suffix(".tsx")
        .or_else(|| name.strip_suffix(".ts"))?;
    Some(vize_carton::cstr!("{stem}.d.ts"))
}

/// The mirror path an importer gives one foreign source's declaration.
fn foreign_declaration_path(
    project_root: &Path,
    virtual_root: &Path,
    canonical_source: &Path,
) -> CorsaResult<Option<PathBuf>> {
    let mirrored = mirrored_canonical_virtual_path(project_root, virtual_root, canonical_source)?;
    let Some(name) = mirrored.file_name().and_then(|name| name.to_str()) else {
        return Ok(None);
    };
    let declaration = if name.ends_with(".vue") {
        vize_carton::cstr!("{name}.d.ts")
    } else {
        match emitted_declaration_name(name) {
            Some(declaration) => declaration,
            None => return Ok(None),
        }
    };
    Ok(Some(mirrored.with_file_name(declaration.as_str())))
}

impl VirtualProject {
    /// Mirror `declarations` (source path, emitted declaration path) in place
    /// of the sources themselves. Registered before the scan, so the sources
    /// never project, their importers rewrite into the mirror, and the
    /// reachability walk leaves them to their owning program.
    pub fn register_foreign_declarations(
        &mut self,
        declarations: &[(PathBuf, PathBuf)],
    ) -> CorsaResult<()> {
        for (source, declaration) in declarations {
            // One `realpath` serves both the mirror path and the ownership set.
            let canonical = vize_carton::path::canonicalize_non_verbatim(source);
            let Some(virtual_path) =
                foreign_declaration_path(&self.project_root, &self.virtual_root, &canonical)?
            else {
                continue;
            };
            self.foreign_sources.insert(canonical);
            self.foreign_declaration_files.insert(virtual_path.clone());
            self.passthrough_files
                .insert(virtual_path, declaration.clone());
        }
        Ok(())
    }

    /// Whether `canonical` is owned by another program and mirrored as its
    /// declaration here.
    pub(super) fn is_foreign_source(&self, canonical: &Path) -> bool {
        !self.foreign_sources.is_empty() && self.foreign_sources.contains(canonical)
    }

    /// The declarations a mirror-layout emit into `out_dir` produced for this
    /// program's own sources, keyed by the authored source path, for the
    /// programs that import them.
    pub fn emitted_mirror_declarations(&self, out_dir: &Path) -> Vec<(PathBuf, PathBuf)> {
        let mut declarations = Vec::new();
        for file in self.virtual_files_sorted() {
            let Ok(relative) = file.virtual_path.strip_prefix(&self.virtual_root) else {
                continue;
            };
            if relative
                .components()
                .any(|component| component.as_os_str() == "node_modules")
            {
                continue;
            }
            let Some(name) = relative.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let Some(declaration) = emitted_declaration_name(name) else {
                continue;
            };
            let emitted = out_dir.join(relative).with_file_name(declaration.as_str());
            if emitted.is_file() {
                declarations.push((file.original_path.clone(), emitted));
            }
        }
        declarations
    }
}

#[cfg(test)]
mod tests {
    use super::emitted_declaration_name;
    use crate::batch::VirtualProject;

    /// A foreign SFC mirrors as its owner's `.vue.d.ts`, joins the program,
    /// and is known as foreign so the reachability walk leaves it alone.
    #[test]
    fn foreign_declarations_join_the_mirror_as_declaration_inputs() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let kit = root.join("packages/kit/src");
        let emitted = root.join("emitted/packages/kit/src");
        std::fs::create_dir_all(&kit).unwrap();
        std::fs::create_dir_all(&emitted).unwrap();
        std::fs::write(kit.join("Button.vue"), "<template><button /></template>").unwrap();
        std::fs::write(kit.join("model.ts"), "export const model = 1;").unwrap();
        std::fs::write(emitted.join("Button.vue.d.ts"), "export default {};").unwrap();
        std::fs::write(emitted.join("model.d.ts"), "export declare const model: 1;").unwrap();
        std::fs::write(root.join("tsconfig.json"), "{}").unwrap();
        let mut project = VirtualProject::new(&root).unwrap();
        project
            .register_foreign_declarations(&[
                (kit.join("Button.vue"), emitted.join("Button.vue.d.ts")),
                (kit.join("model.ts"), emitted.join("model.d.ts")),
                (kit.join("styles.css"), emitted.join("styles.css")),
            ])
            .unwrap();
        let virtual_root = project.virtual_root().to_path_buf();
        let button = virtual_root.join("packages/kit/src/Button.vue.d.ts");
        let model = virtual_root.join("packages/kit/src/model.d.ts");
        let mut materialized: Vec<_> = project
            .expected_materialized_files()
            .into_iter()
            .filter(|path| path.starts_with(virtual_root.join("packages")))
            .collect();
        materialized.sort();
        assert_eq!(materialized, vec![button, model]);
        let foreign_includes: Vec<_> = project
            .include_paths(None, false)
            .into_iter()
            .filter(|path| path.starts_with("packages/"))
            .collect();
        assert_eq!(
            foreign_includes,
            [
                "packages/kit/src/Button.vue.d.ts",
                "packages/kit/src/model.d.ts"
            ]
        );
        assert_eq!(
            [
                project.is_foreign_source(&kit.join("Button.vue")),
                project.is_foreign_source(&kit.join("styles.css")),
            ],
            [true, false]
        );
        assert_eq!(
            project.emitted_mirror_declarations(&root.join("emitted")),
            Vec::new()
        );
    }

    #[test]
    fn emitted_names_follow_typescript() {
        assert_eq!(
            emitted_declaration_name("Button.vue.ts").as_deref(),
            Some("Button.vue.d.ts")
        );
        assert_eq!(
            emitted_declaration_name("Page.vue.tsx").as_deref(),
            Some("Page.vue.d.ts")
        );
        assert_eq!(
            emitted_declaration_name("model.ts").as_deref(),
            Some("model.d.ts")
        );
        assert_eq!(emitted_declaration_name("components.d.ts"), None);
        assert_eq!(emitted_declaration_name("styles.css"), None);
    }
}
