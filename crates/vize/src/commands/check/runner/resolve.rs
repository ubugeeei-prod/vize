//! Project-root, TypeScript/JavaScript config, declaration-output, and path
//! resolution helpers for the `check` runner.

use std::path::{Path, PathBuf};

use vize_canon::DeclarationEmitOptions;
use vize_l0::{FxHashSet, String};

use crate::commands::check::tsconfig_inputs::{
    TsconfigDeclarationOptions, load_tsconfig_declaration_options,
};

pub(super) fn resolve_declaration_emit_options(
    declaration_dir: Option<&Path>,
    tsconfig_path: Option<&Path>,
    project_root: &Path,
) -> DeclarationEmitOptions {
    let tsconfig_options = tsconfig_path
        .map(load_tsconfig_declaration_options)
        .unwrap_or_default();
    let out_dir = resolve_declaration_dir(declaration_dir, &tsconfig_options, project_root);

    DeclarationEmitOptions::new(out_dir)
        .with_declaration_map(tsconfig_options.declaration_map.unwrap_or(false))
}

pub(super) fn resolve_declaration_dir(
    declaration_dir: Option<&Path>,
    tsconfig_options: &TsconfigDeclarationOptions,
    project_root: &Path,
) -> PathBuf {
    declaration_dir
        .map(|path| {
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                project_root.join(path)
            }
        })
        .or_else(|| tsconfig_options.output_dir().map(Path::to_path_buf))
        .unwrap_or_else(|| project_root.join("dist").join("types"))
}

pub(super) fn resolve_project_root(
    explicit_tsconfig: Option<&Path>,
    cwd: &Path,
    files: &[PathBuf],
) -> PathBuf {
    if let Some(tsconfig) = explicit_tsconfig {
        let tsconfig_path = if tsconfig.is_absolute() {
            tsconfig.to_path_buf()
        } else {
            cwd.join(tsconfig)
        };
        let tsconfig_dir = vize_carton::path::canonicalize_non_verbatim(&tsconfig_path)
            .parent()
            .map(|parent| parent.to_path_buf())
            .unwrap_or_else(|| cwd.to_path_buf());
        if files.is_empty()
            || (project_root_has_package_boundary(&tsconfig_dir)
                && !has_source_input_outside_root(&tsconfig_dir, files))
        {
            return tsconfig_dir;
        }

        return common_project_root(tsconfig_dir, files);
    }

    if let Some(root) = resolve_project_root_from_files(files) {
        return root;
    }

    if let Some(root) = find_nearest_tsconfig_dir(cwd) {
        return root;
    }

    cwd.to_path_buf()
}

pub(super) fn resolve_project_root_with_boundary(
    explicit_tsconfig: Option<&Path>,
    cwd: &Path,
    files: &[PathBuf],
    boundary: Option<&Path>,
) -> PathBuf {
    let Some(boundary) = boundary.filter(|_| explicit_tsconfig.is_none()) else {
        return resolve_project_root(explicit_tsconfig, cwd, files);
    };
    files
        .iter()
        .find_map(|file| find_nearest_tsconfig_path_within(file, boundary))
        .and_then(|config| config.parent().map(Path::to_path_buf))
        .or_else(|| common_file_parent(files))
        .unwrap_or_else(|| boundary.to_path_buf())
}

pub(super) fn resolve_tsconfig_path(
    explicit_tsconfig: Option<&Path>,
    cwd: &Path,
    project_root: &Path,
    files: &[PathBuf],
) -> Option<PathBuf> {
    if let Some(tsconfig) = explicit_tsconfig {
        let tsconfig_path = if tsconfig.is_absolute() {
            tsconfig.to_path_buf()
        } else {
            cwd.join(tsconfig)
        };
        return Some(vize_carton::path::canonicalize_non_verbatim(&tsconfig_path));
    }

    project_config_path(project_root).or_else(|| {
        files
            .iter()
            .find_map(|file| find_nearest_tsconfig_path(file))
    })
}

pub(super) fn resolve_tsconfig_path_with_boundary(
    explicit_tsconfig: Option<&Path>,
    cwd: &Path,
    project_root: &Path,
    files: &[PathBuf],
    boundary: Option<&Path>,
) -> Option<PathBuf> {
    let Some(boundary) = boundary.filter(|_| explicit_tsconfig.is_none()) else {
        return resolve_tsconfig_path(explicit_tsconfig, cwd, project_root, files);
    };
    files
        .iter()
        .find_map(|file| find_nearest_tsconfig_path_within(file, boundary))
}

pub(super) fn external_input_tsconfig_boundary(
    patterns: &[std::string::String],
    cwd: &Path,
) -> Option<PathBuf> {
    let canonicalize = vize_carton::path::canonicalize_non_verbatim;
    let cwd = canonicalize(cwd);
    let mut boundary = cwd.clone();
    let mut found_external = false;
    for pattern in patterns {
        let path = Path::new(pattern);
        if !path.is_absolute() {
            continue;
        }
        let input_root = if path.exists() {
            path.to_path_buf()
        } else {
            super::collect::base_dir_from_pattern(pattern)
        };
        if !input_root.exists() {
            continue;
        }
        let path = canonicalize(&input_root);
        if path.starts_with(&cwd) {
            continue;
        }
        found_external = true;
        if !retain_common_ancestor(&mut boundary, &path) {
            // Different Windows volume prefixes have no shared ancestor. Keep
            // the cwd as a non-matching ceiling so implicit source discovery
            // stays disabled for that input.
            return Some(cwd);
        }
    }
    found_external.then_some(boundary)
}

fn retain_common_ancestor(boundary: &mut PathBuf, path: &Path) -> bool {
    while !path.starts_with(&*boundary) {
        if !boundary.pop() || boundary.as_os_str().is_empty() {
            return false;
        }
    }
    true
}

pub(super) fn explicit_input_root(project_root: &Path, cwd: &Path) -> PathBuf {
    let cwd = vize_carton::path::canonicalize_non_verbatim(cwd);
    if project_root.starts_with(&cwd) {
        cwd
    } else {
        project_root.to_path_buf()
    }
}

pub(super) fn project_root_has_package_boundary(project_root: &Path) -> bool {
    project_root.join("package.json").is_file()
}

pub(super) fn retain_project_files(
    files: &mut Vec<PathBuf>,
    inputs: &[PathBuf],
    program_declarations: &[PathBuf],
    project_root: &Path,
) {
    // Ambient `compilerOptions.types` packages can resolve from an ancestor
    // `node_modules` outside the source root. The virtual project extends the
    // real tsconfig and lets TypeScript load those packages normally. Explicit
    // tsconfig `files` entries are program roots even when they live there:
    // default runs carry them in `inputs`, explicit runs (`vize check src`)
    // register them separately as `program_declarations`.
    let input_set: FxHashSet<_> = inputs
        .iter()
        .chain(program_declarations)
        .map(PathBuf::as_path)
        .collect();
    files.retain(|path| {
        input_set.contains(path.as_path())
            || path.starts_with(project_root)
            || !path_has_component(path, "node_modules")
    });
}

fn has_source_input_outside_root(root: &Path, files: &[PathBuf]) -> bool {
    files.iter().any(|file| {
        !path_has_component(file, "node_modules")
            && !is_declaration_file(file)
            && !file.starts_with(root)
    })
}

fn is_declaration_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
}

pub(super) fn validate_inputs_in_root(
    root: &Path,
    files: &[PathBuf],
    enabled: bool,
) -> Result<(), String> {
    if !enabled {
        return Ok(());
    }
    validate_explicit_inputs_in_root(root, files)
}

fn validate_explicit_inputs_in_root(root: &Path, files: &[PathBuf]) -> Result<(), String> {
    let root = vize_carton::path::canonicalize_non_verbatim(root);
    for file in files {
        let path = vize_carton::path::canonicalize_non_verbatim(file);
        if !path.starts_with(&root) {
            return Err(vize_l0::cstr!(
                "explicit check input `{}` is outside project root `{}`.",
                path.display(),
                root.display()
            ));
        }
    }
    Ok(())
}

mod config;

pub(super) use config::{
    find_nearest_tsconfig_dir, find_nearest_tsconfig_path, find_nearest_tsconfig_path_within,
    project_config_path,
};

fn resolve_project_root_from_files(files: &[PathBuf]) -> Option<PathBuf> {
    let common = common_file_parent(files)?;
    Some(find_nearest_tsconfig_dir(&common).unwrap_or(common))
}

fn common_file_parent(files: &[PathBuf]) -> Option<PathBuf> {
    let mut common = files
        .first()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)?;

    for file in files.iter().skip(1) {
        let parent = file.parent().unwrap_or(file.as_path());
        while !parent.starts_with(&common) {
            if !common.pop() {
                return None;
            }
        }
    }

    Some(common)
}

fn common_project_root(mut common: PathBuf, files: &[PathBuf]) -> PathBuf {
    for file in files {
        if path_has_component(file, "node_modules") {
            continue;
        }
        let parent = file.parent().unwrap_or(file.as_path());
        while !parent.starts_with(&common) {
            if !common.pop() {
                return common;
            }
        }
    }

    common
}

fn path_has_component(path: &Path, component: &str) -> bool {
    path.components()
        .any(|part| part.as_os_str().to_str() == Some(component))
}

pub(super) fn display_path(base: &Path, path: &Path) -> vize_l0::String {
    use vize_l0::cstr;

    path.strip_prefix(base)
        .map(|relative| cstr!("{}", relative.display()))
        .unwrap_or_else(|_| cstr!("{}", path.display()))
}

pub(super) fn resolve_from_config_dir(config_dir: &Path, candidate: &str) -> PathBuf {
    let path = Path::new(candidate);
    if path.is_absolute() {
        return path.to_path_buf();
    }

    config_dir.join(path)
}

/// Largest accepted `typeChecker.servers` value. Each server is a Corsa CLI
/// process with its own program; beyond this the duplicated parse/bind work
/// outweighs any remaining parallelism.
const MAX_CORSA_SERVERS: usize = 32;

pub(super) fn validate_corsa_server_count(servers: Option<usize>) -> Result<(), String> {
    let Some(servers) = servers else {
        return Ok(());
    };
    if servers == 0 {
        return Err("typeChecker.servers must be at least 1.".into());
    }
    if servers > MAX_CORSA_SERVERS {
        return Err(vize_l0::cstr!(
            "typeChecker.servers={servers} exceeds the supported maximum of {MAX_CORSA_SERVERS}."
        ));
    }
    Ok(())
}

#[expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#[cfg(test)]
mod tests {
    use super::{
        external_input_tsconfig_boundary, resolve_project_root, resolve_project_root_with_boundary,
        resolve_tsconfig_path_with_boundary,
    };
    use std::path::{Path, PathBuf};

    fn unique_case_dir(name: &str) -> PathBuf {
        static NEXT_CASE_ID: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root should exist")
            .join("target")
            .join("vize-tests")
            .join("resolve")
            .join(name)
            .join(std::process::id().to_string())
            .join(case_id.to_string())
    }

    #[test]
    fn explicit_tsconfig_with_package_boundary_does_not_widen_to_workspace_root() {
        let workspace = unique_case_dir("package-boundary");
        let _ = std::fs::remove_dir_all(&workspace);
        let package_root = workspace.join("test/e2e/onepass/sign-in");
        std::fs::create_dir_all(package_root.join("src")).unwrap();
        std::fs::create_dir_all(workspace.join("types")).unwrap();
        std::fs::write(package_root.join("package.json"), "{}").unwrap();
        std::fs::write(package_root.join("tsconfig.json"), "{}").unwrap();
        let app = package_root.join("src/main.ts");
        let ambient = workspace.join("types/root.d.ts");
        std::fs::write(&app, "").unwrap();
        std::fs::write(&ambient, "declare const rootOnly: string;\n").unwrap();

        let resolved = resolve_project_root(
            Some(Path::new("tsconfig.json")),
            &package_root,
            &[app, ambient],
        );

        assert_eq!(resolved, package_root);
        let _ = std::fs::remove_dir_all(&workspace);
    }

    #[test]
    fn explicit_tsconfig_with_package_boundary_widens_for_source_inputs_outside_package() {
        let workspace = unique_case_dir("package-boundary-source-outside");
        let _ = std::fs::remove_dir_all(&workspace);
        let package_root = workspace.join("packages/primevue");
        let sibling_root = workspace.join("packages/icons");
        std::fs::create_dir_all(package_root.join("src")).unwrap();
        std::fs::create_dir_all(sibling_root.join("src")).unwrap();
        std::fs::write(package_root.join("package.json"), "{}").unwrap();
        std::fs::write(package_root.join("tsconfig.json"), "{}").unwrap();
        let app = package_root.join("src/Button.vue");
        let sibling = sibling_root.join("src/Icon.vue");
        std::fs::write(&app, "").unwrap();
        std::fs::write(&sibling, "").unwrap();

        let resolved = resolve_project_root(
            Some(Path::new("packages/primevue/tsconfig.json")),
            &workspace,
            &[app, sibling],
        );

        assert_eq!(resolved, workspace.join("packages"));
        let _ = std::fs::remove_dir_all(&workspace);
    }

    #[test]
    fn external_absolute_input_only_discovers_configs_inside_its_shared_boundary() {
        let workspace = unique_case_dir("external-input-config-boundary");
        let _ = std::fs::remove_dir_all(&workspace);
        let fixture = workspace.join("fixture");
        let cwd = fixture.join("cwd");
        let source_root = fixture.join("source");
        let source = source_root.join("src/main.ts");
        std::fs::create_dir_all(&cwd).unwrap();
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, "export const value = 1;\n").unwrap();
        let ancestor_config = workspace.join("tsconfig.json");
        std::fs::write(&ancestor_config, "{}").unwrap();

        let patterns = vec![source.display().to_string()];
        let boundary = external_input_tsconfig_boundary(&patterns, &cwd).unwrap();
        assert_eq!(boundary, fixture);
        let project_root = resolve_project_root_with_boundary(
            None,
            &cwd,
            std::slice::from_ref(&source),
            Some(&boundary),
        );
        assert_eq!(project_root, source.parent().unwrap());
        assert_eq!(
            resolve_tsconfig_path_with_boundary(
                None,
                &cwd,
                &project_root,
                std::slice::from_ref(&source),
                Some(&boundary),
            ),
            None
        );

        let local_config = source_root.join("tsconfig.json");
        std::fs::write(&local_config, "{}").unwrap();
        let project_root = resolve_project_root_with_boundary(
            None,
            &cwd,
            std::slice::from_ref(&source),
            Some(&boundary),
        );
        assert_eq!(project_root, source_root);
        assert_eq!(
            resolve_tsconfig_path_with_boundary(
                None,
                &cwd,
                &project_root,
                std::slice::from_ref(&source),
                Some(&boundary),
            ),
            Some(local_config)
        );
        assert_eq!(
            resolve_tsconfig_path_with_boundary(
                Some(&ancestor_config),
                &cwd,
                &project_root,
                std::slice::from_ref(&source),
                Some(&boundary),
            ),
            Some(ancestor_config)
        );
        let _ = std::fs::remove_dir_all(&workspace);
    }

    #[test]
    fn unrelated_path_prefixes_refuse_an_unbounded_common_ancestor() {
        let mut boundary = PathBuf::from("volume-a/work");
        assert!(!super::retain_common_ancestor(
            &mut boundary,
            Path::new("volume-b/source.ts")
        ));
        assert!(boundary.as_os_str().is_empty());
    }

    #[test]
    fn absolute_glob_uses_its_existing_static_prefix_for_the_boundary() {
        let workspace = unique_case_dir("external-input-glob-boundary");
        let _ = std::fs::remove_dir_all(&workspace);
        let fixture = workspace.join("fixture");
        let cwd = fixture.join("cwd");
        let source = fixture.join("source/src");
        std::fs::create_dir_all(&cwd).unwrap();
        std::fs::create_dir_all(&source).unwrap();
        let pattern = format!("{}/**/*.ts", source.display());
        assert!(!Path::new(&pattern).exists());

        assert_eq!(
            external_input_tsconfig_boundary(&[pattern], &cwd),
            Some(fixture)
        );
        let _ = std::fs::remove_dir_all(&workspace);
    }
}
