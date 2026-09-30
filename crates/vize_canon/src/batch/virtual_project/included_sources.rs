//! Sources the diagnostic pass must mirror even when the initial file list
//! omitted them.

use std::path::{Path, PathBuf};

use vize_carton::FxHashSet;

use super::VirtualProject;
use crate::batch::error::CorsaResult;

impl VirtualProject {
    pub(crate) fn ensure_included_sources(&mut self) -> CorsaResult<()> {
        let Some(tsconfig) = self.resolved_tsconfig_path() else {
            return Ok(());
        };
        let included =
            super::tsconfig_gen::references::included_sources(&tsconfig, &self.project_root);
        self.register_unregistered_sources(included)
    }

    pub(crate) fn adopt_diagnostic_sources(
        &mut self,
        paths: impl IntoIterator<Item = PathBuf>,
    ) -> CorsaResult<()> {
        let accepted: Vec<PathBuf> = paths
            .into_iter()
            .filter(|path| {
                path.is_file()
                    && self.source_file_policy().accepts_project_source(path)
                    && !path_contains_node_modules(path)
            })
            .collect();
        self.register_unregistered_sources(accepted)
    }

    fn register_unregistered_sources(
        &mut self,
        paths: impl IntoIterator<Item = PathBuf>,
    ) -> CorsaResult<()> {
        let known: FxHashSet<PathBuf> = self
            .original_index
            .keys()
            .map(|path| vize_carton::path::canonicalize_non_verbatim(path))
            .collect();
        let missing: Vec<PathBuf> = paths
            .into_iter()
            .map(|path| vize_carton::path::canonicalize_non_verbatim(&path))
            .filter(|path| path.is_file() && !known.contains(path))
            .filter(|path| self.source_file_policy().accepts_project_source(path))
            .filter(|path| !path_contains_node_modules(path))
            .collect::<FxHashSet<_>>()
            .into_iter()
            .collect();
        if missing.is_empty() {
            return Ok(());
        }
        let mut roots = self.declaration_roots.clone().unwrap_or_else(|| {
            self.registered_original_paths_sorted()
                .into_iter()
                .map(|path| vize_carton::path::canonicalize_non_verbatim(&path))
                .collect()
        });
        for path in &missing {
            roots.insert(path.clone());
        }
        let listed: Vec<PathBuf> = roots.into_iter().collect();
        self.set_declaration_roots(&listed);
        let mut registered_any = false;
        for path in &missing {
            if self.register_path(path).is_ok() {
                registered_any = true;
            }
        }
        if registered_any {
            self.register_reachable_dependencies()?;
            self.finalize_package_routes()?;
        }
        Ok(())
    }
}

fn path_contains_node_modules(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "node_modules")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use vize_carton::FxHashSet;

    use super::super::VirtualProject;

    fn case_dir(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("vize-tests")
            .join("tests")
            .join(vize_carton::cstr!("{name}-{}", std::process::id()).as_str())
    }

    fn source_extension(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("ts" | "tsx" | "mts" | "cts" | "vue" | "js" | "jsx" | "mjs" | "cjs")
        )
    }

    /// Every non-declaration file the diagnostic pass sees is a program member,
    /// and every source `include` covers is registered. The assertion walks the
    /// include pattern itself; it does not name one omitted file.
    #[test]
    fn included_sources_join_the_virtual_project() {
        let root = case_dir("included-sources");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src/generated")).unwrap();
        fs::write(root.join(".gitignore"), "src/generated/\n").unwrap();
        fs::write(
            root.join("tsconfig.json"),
            r#"{
  "compilerOptions": { "strict": true, "moduleResolution": "bundler" },
  "include": ["src/**/*"]
}
"#,
        )
        .unwrap();
        for index in 0..4 {
            let body = match index {
                0 => {
                    "import { value as other } from \"./member_1\";\nexport const value = other;\n"
                }
                1 => "export const value = 1;\n",
                2 => "export const value = 2;\n",
                _ => "export const value = 3;\n",
            };
            let source = vize_carton::cstr!("src/member_{index}.ts");
            let generated = vize_carton::cstr!("src/generated/member_{index}.ts");
            fs::write(root.join(source.as_str()), body).unwrap();
            fs::write(root.join(generated.as_str()), body).unwrap();
        }

        let member_0 = root.join("src/member_0.ts");
        let mut project = VirtualProject::new(&root).unwrap();
        project.set_tsconfig_path(Some(root.join("tsconfig.json")));
        project.set_declaration_roots(std::slice::from_ref(&member_0));
        project.register_path(&member_0).unwrap();
        project.register_reachable_dependencies().unwrap();

        let program_before = project.topology_program_files();
        let mut diagnosed = 0;
        for file in project.virtual_files_sorted() {
            let original = vize_carton::path::canonicalize_non_verbatim(&file.original_path);
            if crate::batch::declaration_path::is_declaration_file(&original) {
                continue;
            }
            diagnosed += 1;
            let name = file.virtual_path.file_name().and_then(|name| name.to_str());
            let Some(name) = name else {
                continue;
            };
            assert!(
                program_before.iter().any(|entry| {
                    std::path::Path::new(entry.as_str())
                        .file_name()
                        .and_then(|file_name| file_name.to_str())
                        == Some(name)
                }),
                "diagnosed {name} is missing from the virtual program: {program_before:?}"
            );
        }
        assert!(
            diagnosed >= 2,
            "the imported sibling never joined the diagnosed set: {program_before:?}"
        );

        project.ensure_included_sources().unwrap();

        let pattern = glob::Pattern::new("src/**/*").unwrap();
        let options = glob::MatchOptions {
            case_sensitive: true,
            require_literal_separator: true,
            require_literal_leading_dot: false,
        };
        let registered: FxHashSet<PathBuf> = project
            .registered_original_paths_sorted()
            .into_iter()
            .map(|path| vize_carton::path::canonicalize_non_verbatim(&path))
            .collect();
        let program_after = project.topology_program_files();
        let mut covered = 0;
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_entry(|entry| {
                entry.depth() == 0 || !entry.file_name().to_string_lossy().starts_with('.')
            })
        {
            let Ok(entry) = entry else {
                continue;
            };
            let path = entry.path();
            if !path.is_file() || !source_extension(path) {
                continue;
            }
            let relative = path
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if !pattern.matches_with(&relative, options) {
                continue;
            }
            covered += 1;
            let canonical = vize_carton::path::canonicalize_non_verbatim(path);
            assert!(
                registered
                    .iter()
                    .any(|registered_path| registered_path == &canonical),
                "include omitted {relative}"
            );
            let name = path.file_name().and_then(|name| name.to_str()).unwrap();
            assert!(
                program_after.iter().any(|entry| {
                    std::path::Path::new(entry.as_str())
                        .file_name()
                        .and_then(|file_name| file_name.to_str())
                        == Some(name)
                }),
                "included {relative} is not a program member: {program_after:?}"
            );
        }
        assert!(
            covered >= 8,
            "the include walk did not see the authored sources ({covered})"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// Generated declarations stay on the authored path (#2047). Mirroring
    /// `types/codegen/schema.d.ts` gives that module two identities.
    #[test]
    fn included_declarations_stay_on_their_authored_path() {
        let root = case_dir("included-declarations");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("types/codegen")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("tsconfig.json"),
            r#"{
  "compilerOptions": { "strict": true },
  "include": ["src/**/*.ts", "types/**/*.d.ts"]
}
"#,
        )
        .unwrap();
        let entry = root.join("src/entry.ts");
        fs::write(&entry, "export const ready = true;\n").unwrap();
        fs::write(
            root.join("types/codegen/schema.d.ts"),
            "export type Row = {}\n",
        )
        .unwrap();

        let mut project = VirtualProject::new(&root).unwrap();
        project.set_tsconfig_path(Some(root.join("tsconfig.json")));
        project.set_declaration_roots(std::slice::from_ref(&entry));
        project.register_path(&entry).unwrap();
        project.ensure_included_sources().unwrap();

        assert_eq!(
            project.registered_original_paths_sorted(),
            vec![entry.clone()],
            "generated declarations stay on their authored path"
        );
        assert_eq!(
            project.topology_program_files(),
            [
                "__vize_helpers.d.ts",
                "__vize_vue_modules.d.ts",
                "src/entry.ts",
            ],
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// A workspace script outside this project stays registered for the
    /// import, but it is not a second program root (`TS2451`).
    #[test]
    fn out_of_root_scripts_stay_out_of_the_virtual_program() {
        let root = case_dir("out-of-root-script");
        let _ = fs::remove_dir_all(&root);
        let app = root.join("app");
        let package = root.join("packages/source");
        fs::create_dir_all(app.join("src")).unwrap();
        fs::create_dir_all(package.join("src")).unwrap();
        fs::write(
            app.join("tsconfig.json"),
            r#"{ "compilerOptions": { "strict": true }, "include": ["src/entry.ts"] }"#,
        )
        .unwrap();
        let entry = app.join("src/entry.ts");
        fs::write(&entry, "export const ready = true;\n").unwrap();
        let outside = package.join("src/index.ts");
        fs::write(&outside, "const invalid: string = 42\nvoid invalid\n").unwrap();

        let mut project = VirtualProject::new(&app).unwrap();
        project.set_tsconfig_path(Some(app.join("tsconfig.json")));
        project.set_declaration_roots(&[entry.clone(), outside.clone()]);
        project.register_path(&entry).unwrap();
        project.register_path(&outside).unwrap();

        assert_eq!(
            project.topology_program_files(),
            [
                "__vize_helpers.d.ts",
                "__vize_vue_modules.d.ts",
                "src/entry.ts",
            ],
        );
        let mut registered = vec![entry, outside];
        registered.sort();
        assert_eq!(project.registered_original_paths_sorted(), registered);
        let _ = fs::remove_dir_all(&root);
    }
}
