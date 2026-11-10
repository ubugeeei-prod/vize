use serde_json::json;
use std::path::Path;
use vize_l0::{String, ToCompactString, cstr};

/// Write a minimal `tsconfig.json` that keeps the native checker in strict mode.
pub(super) fn write_temp_tsconfig(temp_dir_path: &Path) -> Result<(), String> {
    let tsconfig_content = json!({
        "compilerOptions": {
            "target": "ES2022",
            "module": "ESNext",
            "moduleResolution": "bundler",
            "allowImportingTsExtensions": true,
            "lib": ["ES2022", "DOM", "DOM.Iterable"],
            "strict": true,
            "noEmit": true,
            "skipLibCheck": true
        }
    });
    std::fs::write(
        temp_dir_path.join("tsconfig.json"),
        tsconfig_content.to_compact_string(),
    )
    .map_err(|e| cstr!("Failed to write temp tsconfig.json: {e}"))
}

pub(super) fn workspace_config_path(project_root: &Path) -> std::path::PathBuf {
    let tsconfig = project_root.join("tsconfig.json");
    if tsconfig.is_file() {
        return tsconfig;
    }
    let jsconfig = project_root.join("jsconfig.json");
    if jsconfig.is_file() {
        return jsconfig;
    }
    tsconfig
}

pub(super) fn write_materialized_project_tsconfig(
    project_root: &Path,
    overlay_root: &Path,
    selected_config: Option<&Path>,
) -> Result<std::path::PathBuf, String> {
    let config_path = overlay_root.join("tsconfig.json");
    let selected = selected_config
        .map(|config| {
            if config.is_absolute() {
                config.to_path_buf()
            } else {
                project_root.join(config)
            }
        })
        .unwrap_or_else(|| workspace_config_path(project_root));
    let mut include = vec![String::new("**/*")];
    // A generated root (rootless scratch or Canon mirror) owns these exact
    // declarations outside the overlay. This `include` replaces the root's
    // own, so name them again, without globbing unrelated root files.
    for name in crate::batch::virtual_project::ROOT_DECLARATION_FILES {
        let declarations = project_root.join(name);
        if declarations.is_file() {
            include.push(crate::file_uri::path_to_typescript_name(&declarations));
        }
    }
    let content = json!({
        "extends": crate::file_uri::path_to_typescript_name(&selected),
        "compilerOptions": {
            "allowImportingTsExtensions": true,
            "noEmit": true
        },
        "include": include,
        "exclude": []
    });
    super::session_storage::write_owned_file(
        overlay_root,
        &config_path,
        content.to_compact_string().as_bytes(),
    )?;
    Ok(config_path)
}

pub(super) fn write_vue_module_stubs(temp_dir_path: &Path) -> Result<(), String> {
    let content = r#"declare module "*.vue" {
  const component: import("vue").DefineComponent<any, any, any>;
  export default component;
}

declare module "*.vue.ts" {
  const component: import("vue").DefineComponent<any, any, any>;
  export default component;
}
"#;
    std::fs::write(temp_dir_path.join("__vize_vue_modules.d.ts"), content)
        .map_err(|e| cstr!("Failed to write Vue module declarations: {e}"))
}

/// Write the shared ambient helpers into the scratch session so virtual
/// documents generated with the hoisted preamble resolve the program-wide
/// helper declarations. Self-contained (non-hoisted) documents are unaffected:
/// their module-local helpers shadow these globals and the `ImportMeta`
/// interface merges with identical members.
pub(super) fn write_shared_helper_decls(temp_dir_path: &Path) -> Result<(), String> {
    std::fs::write(
        temp_dir_path.join(crate::virtual_ts::SHARED_PREAMBLE_FILE_NAME),
        crate::virtual_ts::SHARED_PREAMBLE_DTS,
    )
    .map_err(|e| cstr!("Failed to write shared helper declarations: {e}"))
}

/// A Canon root resolves packages through its own install view. The mirror
/// standing in for it must ascend into that view, not into the authored
/// install its storage linked at creation.
pub(super) fn link_canon_dependencies(
    project_root: &Path,
    overlay_root: &Path,
) -> Result<(), String> {
    let source = project_root.join("node_modules");
    if !project_root.join("__vize_vue_modules.d.ts").is_file() || !source.join("vue").is_dir() {
        return Ok(());
    }
    let target = overlay_root
        .parent()
        .ok_or_else(|| cstr!("Private Corsa storage has no owner directory"))?
        .join("node_modules");
    match std::fs::read_link(&target) {
        Ok(current) if current == source => return Ok(()),
        // Only the storage's own dependency link is ever replaced.
        Ok(_) => unlink_dependencies(&target)
            .map_err(|error| cstr!("Failed to retire private Corsa dependencies: {error}"))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(cstr!(
                "Private Corsa dependencies are not an owned link: {error}"
            ));
        }
    }
    link_dependencies(&source, &target)
        .map_err(|error| cstr!("Failed to link private Corsa dependencies: {error}"))
}

fn unlink_dependencies(target: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::fs::remove_file(target)
    }
    #[cfg(windows)]
    {
        std::fs::remove_dir(target)
    }
}

fn link_dependencies(source: &Path, target: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
            .or_else(|_| junction::create(source, target))
    }
}

#[cfg(test)]
mod canon_mirror_tests;

#[cfg(test)]
mod tests {
    use super::{workspace_config_path, write_materialized_project_tsconfig};

    #[test]
    fn materialized_project_config_extends_workspace_options_and_includes_overlays() {
        let project = tempfile::TempDir::new().unwrap();
        let root = project.path();
        let workspace_config = root.join("tsconfig.json");
        std::fs::write(&workspace_config, r#"{"compilerOptions":{"strict":false}}"#).unwrap();

        let storage = crate::lsp_client::session_storage::SessionStorage::new(root).unwrap();
        let generated = write_materialized_project_tsconfig(root, storage.root(), None).unwrap();
        let parsed: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&generated).unwrap()).unwrap();

        assert_eq!(workspace_config_path(root), workspace_config);
        assert_eq!(generated, storage.root().join("tsconfig.json"));
        assert_eq!(
            parsed["extends"],
            crate::file_uri::path_to_typescript_name(&workspace_config).as_str()
        );
        assert_eq!(parsed["include"], serde_json::json!(["**/*"]));
        assert_eq!(parsed["exclude"], serde_json::json!([]));
        assert_eq!(parsed["compilerOptions"]["noEmit"], true);
        assert_eq!(
            parsed["compilerOptions"]["allowImportingTsExtensions"],
            true
        );
        assert_eq!(
            std::fs::read_to_string(workspace_config).unwrap(),
            r#"{"compilerOptions":{"strict":false}}"#
        );
    }

    #[test]
    fn workspace_config_falls_back_to_jsconfig() {
        let project = tempfile::TempDir::new().unwrap();
        let jsconfig = project.path().join("jsconfig.json");
        std::fs::write(&jsconfig, "{}").unwrap();

        assert_eq!(workspace_config_path(project.path()), jsconfig);
    }

    #[test]
    fn materialized_config_preserves_explicit_relative_selection_and_all_authored_bytes() {
        let project = tempfile::tempdir().unwrap();
        let root = project.path();
        std::fs::create_dir(root.join("config")).unwrap();
        let selected = root.join("config/selected.json");
        let authored = br#"{"compilerOptions":{"rootDir":"../src","baseUrl":"..","paths":{"@/*":["src/*"]},"typeRoots":["../types"],"composite":true}}"#;
        std::fs::write(&selected, authored).unwrap();
        std::fs::write(
            root.join("tsconfig.json"),
            "{\"compilerOptions\":{\"strict\":false}}",
        )
        .unwrap();
        let storage = crate::lsp_client::session_storage::SessionStorage::new(root).unwrap();
        let generated = write_materialized_project_tsconfig(
            root,
            storage.root(),
            Some(std::path::Path::new("config/selected.json")),
        )
        .unwrap();
        let parsed: serde_json::Value =
            serde_json::from_slice(&std::fs::read(generated).unwrap()).unwrap();
        assert_eq!(
            parsed["extends"],
            crate::file_uri::path_to_typescript_name(&selected).as_str()
        );
        assert_eq!(
            parsed["compilerOptions"],
            serde_json::json!({"allowImportingTsExtensions":true,"noEmit":true})
        );
        assert_eq!(std::fs::read(selected).unwrap(), authored);
        assert_eq!(
            std::fs::read(root.join("tsconfig.json")).unwrap(),
            b"{\"compilerOptions\":{\"strict\":false}}"
        );
    }
}
