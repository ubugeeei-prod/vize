//! Whether the program's `#imports` module re-exports Vue's `useTemplateRef`.

use std::path::{Path, PathBuf};

use super::PathAliasResolver;

impl PathAliasResolver {
    /// The file the first resolvable `paths` target of `specifier` names, as
    /// TypeScript tries them in order. `baseUrl` lookup is not consulted: an
    /// alias such as `#imports` names no file below the base.
    fn alias_target_file(&self, specifier: &str) -> Option<PathBuf> {
        let (alias, matched) = self.selected_alias(specifier)?;
        alias.targets.iter().find_map(|target| {
            let target = if target.contains('*') {
                alias.base_dir.join(target.replace('*', matched))
            } else {
                alias.base_dir.join(target.as_str())
            };
            declaration_candidates(&target).find(|candidate| candidate.is_file())
        })
    }
}

/// TypeScript's extension and directory-index probes for a `paths` target.
fn declaration_candidates(target: &Path) -> impl Iterator<Item = PathBuf> + '_ {
    let with_suffix = |suffix: &str| {
        let mut path = target.as_os_str().to_owned();
        path.push(suffix);
        PathBuf::from(path)
    };
    [
        target.to_path_buf(),
        with_suffix(".ts"),
        with_suffix(".tsx"),
        with_suffix(".d.ts"),
        target.join("index.ts"),
        target.join("index.tsx"),
        target.join("index.d.ts"),
    ]
    .into_iter()
}

/// The file `#imports` resolves to through the tsconfig `paths` (Nuxt
/// generates that mapping), and whether it re-exports Vue's own
/// `useTemplateRef`. An explicit import from the alias is then Vue's
/// composable by origin. Hosts that cache the answer watch the file.
pub(in crate::commands::check) fn imports_alias_vue_template_ref(
    tsconfig_path: &Path,
) -> Option<(PathBuf, bool)> {
    let path = PathAliasResolver::from_tsconfig(Some(tsconfig_path))
        .alias_target_file(vize_canon::virtual_ts::TemplateRefOwnership::IMPORTS_ALIAS)?;
    let reexported = std::fs::read_to_string(&path)
        .is_ok_and(|source| vize_canon::batch::reexports_vue_template_ref(&source));
    Some((path, reexported))
}

/// Whether the program's `#imports` module re-exports Vue's `useTemplateRef`.
pub(in crate::commands::check) fn imports_alias_reexports_vue_template_ref(
    tsconfig_path: &Path,
) -> bool {
    imports_alias_vue_template_ref(tsconfig_path).is_some_and(|(_, reexported)| reexported)
}

#[cfg(test)]
mod tests {
    use super::imports_alias_reexports_vue_template_ref;

    fn project(tsconfig: &str, files: &[(&str, &str)]) -> (tempfile::TempDir, std::path::PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        for (name, content) in files {
            let path = root.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        }
        let config = root.join("tsconfig.json");
        std::fs::write(&config, tsconfig).unwrap();
        (directory, config)
    }

    #[test]
    fn nuxt_style_alias_to_a_vue_reexport_is_recognized() {
        for (tsconfig, file) in [
            (
                r##"{"compilerOptions":{"paths":{"#imports":["./imports.d.ts"]}}}"##,
                "imports.d.ts",
            ),
            (
                r##"{"compilerOptions":{"paths":{"#imports":["./.nuxt/imports"]}}}"##,
                ".nuxt/imports.d.ts",
            ),
        ] {
            let (_directory, config) = project(
                tsconfig,
                &[(file, "export { ref, useTemplateRef } from 'vue'\n")],
            );
            assert!(
                imports_alias_reexports_vue_template_ref(&config),
                "{tsconfig}"
            );
        }
    }

    #[test]
    fn missing_alias_or_foreign_export_is_not() {
        let (_directory, config) = project(
            r##"{"compilerOptions":{"paths":{"#imports":["./imports.d.ts"]}}}"##,
            &[(
                "imports.d.ts",
                "export { useTemplateRef } from './composables'\n",
            )],
        );
        assert!(!imports_alias_reexports_vue_template_ref(&config));
        let (_directory, config) = project(
            r#"{"compilerOptions":{}}"#,
            &[("imports.d.ts", "export { useTemplateRef } from 'vue'\n")],
        );
        assert!(!imports_alias_reexports_vue_template_ref(&config));
    }
}
