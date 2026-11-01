//! Terminal input-root observations, before any optional semantic transform.

use super::{
    CorsaResult, FxHashSet, PathBuf, RuntimeDependencyLink, RuntimeDependencyRoots, VirtualProject,
};

/// These are observations from the two existing materialization producers,
/// not a complete native input set. The terminal witness must still seal all
/// source/config/library and reference closure before admitting any edit.
/// Package links are requests: the old package writer can warn and continue
/// after failure, so the witness must verify every actual target before use.
pub(in crate::batch) struct CliMaterializedInputs {
    pub(in crate::batch) requested_package_links: vize_carton::FxHashMap<PathBuf, PathBuf>,
    pub(in crate::batch) runtime_links: Option<Vec<RuntimeDependencyLink>>,
}

impl VirtualProject {
    /// The caller holds the ordinary MaterializeLock throughout preparation,
    /// witness validation and native checking. Default callers use materialize.
    pub(in crate::batch) fn materialize_cli_inputs(&self) -> CorsaResult<CliMaterializedInputs> {
        let mut roots = RuntimeDependencyRoots::observed();
        let package_links = self.materialize_snapshot(
            &FxHashSet::default(),
            &Default::default(),
            None,
            false,
            Some(&mut roots),
        )?;
        Ok(CliMaterializedInputs {
            requested_package_links: package_links,
            runtime_links: roots.finish(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    #[test]
    fn terminal_preparation_retains_whole_writer_observations_without_membership_claims() {
        let owner = tempfile::tempdir().unwrap();
        let root = owner.path().canonicalize().unwrap();
        let source = root.join("source");
        let storage = root.join("storage");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&storage).unwrap();
        let input = source.join("input.ts");
        fs::write(&input, "export const value = 1;\n").unwrap();
        crate::batch::runtime_deps::with_test_env_overrides(
            &[
                ("VIZE_VUE_PACKAGE", None),
                ("VIZE_VUE_NAMESPACE_PACKAGE", None),
                ("VIZE_VUE_RUNTIME_DOM_PACKAGE", None),
                ("VIZE_VITE_PACKAGE", None),
                ("VIZE_RUNTIME_NODE_MODULES", None),
                (
                    "VIZE_TEST_WORKSPACE_NODE_MODULES",
                    Some(Path::new("__none__")),
                ),
            ],
            || {
                let mut project = VirtualProject::new(&source).unwrap();
                project.scope_cli_diagnostics(&storage);
                project.register_path(&input).unwrap();
                let _lock = crate::batch::materialize_lock::MaterializeLock::acquire(
                    project.virtual_root(),
                )
                .unwrap();
                let inputs: crate::batch::virtual_project::CliMaterializedInputs =
                    project.materialize_cli_inputs().unwrap();
                assert_eq!(
                    inputs.requested_package_links,
                    vize_carton::FxHashMap::default()
                );
                assert_eq!(inputs.runtime_links, Some(Vec::new()));
                let file = project.virtual_files_sorted()[0];
                assert_eq!(
                    fs::read_to_string(&file.virtual_path).unwrap(),
                    file.content
                );
                assert!(project.owned_cli_support_path().is_none());
                assert!(project.private_return_fingerprint().is_none());
                assert!(
                    project
                        .virtual_root()
                        .join("node_modules/vue/index.d.ts")
                        .is_file()
                );
            },
        );
    }
}
