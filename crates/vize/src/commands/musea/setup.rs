use std::fs;
use std::path::{Path, PathBuf};
use vize_l0::{String, cstr};

const MUSEA_VITE_PLUGIN: &str = "@vizejs/vite-plugin-musea";
const VITE_CONFIG_FILES: &[&str] = &[
    "vite.config.ts",
    "vite.config.mts",
    "vite.config.js",
    "vite.config.mjs",
    "vite.config.cts",
    "vite.config.cjs",
];

pub(super) fn validate_direct_vite_musea_setup(cwd: &Path) -> Result<(), String> {
    let has_dependency = cwd.ancestors().any(has_musea_vite_plugin_dependency);
    let config_path = find_vite_config(cwd);
    let has_config = config_path
        .as_ref()
        .is_some_and(|path| vite_config_mentions_musea(path));

    if has_dependency && has_config {
        return Ok(());
    }

    let config_hint = config_path
        .as_ref()
        .map(|path| {
            cstr!(
                "found {}, but it does not import or call musea()",
                path.display()
            )
        })
        .unwrap_or_else(|| cstr!("no vite.config.* file found"));

    Err(cstr!(
        "vize musea: Musea is not configured for this Vite project.\n  Install `{}` and add `musea()` from that package to your Vite plugins before running `vize musea serve`.\n  dependency: {}\n  config: {}",
        MUSEA_VITE_PLUGIN,
        if has_dependency { "found" } else { "missing" },
        config_hint
    ))
}

/// Return the nearest package that owns both a Musea Vite dependency and the
/// Vite configuration that enables it. Keeping those two signals on the same
/// ancestor prevents a workspace child from being mistaken for a parent Nuxt
/// application merely because the parent's dependencies are visible.
pub(super) fn find_direct_vite_musea_root(cwd: &Path) -> Option<PathBuf> {
    cwd.ancestors()
        .find(|root| {
            has_musea_vite_plugin_dependency(root)
                && vite_config_at(root).is_some_and(|path| vite_config_mentions_musea(&path))
        })
        .map(Path::to_path_buf)
}

fn has_musea_vite_plugin_dependency(root: &Path) -> bool {
    package_json_has_dependency(root, MUSEA_VITE_PLUGIN)
        || root
            .join("node_modules")
            .join("@vizejs")
            .join("vite-plugin-musea")
            .join("package.json")
            .exists()
}

fn find_vite_config(cwd: &Path) -> Option<PathBuf> {
    cwd.ancestors().find_map(vite_config_at)
}

fn vite_config_at(root: &Path) -> Option<PathBuf> {
    VITE_CONFIG_FILES
        .iter()
        .map(|file_name| root.join(file_name))
        .find(|path| path.is_file())
}

fn vite_config_mentions_musea(path: &Path) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    content.contains(MUSEA_VITE_PLUGIN) || content.contains("musea(")
}

pub(super) fn package_json_has_dependency(root: &Path, dependency: &str) -> bool {
    let Ok(content) = fs::read_to_string(root.join("package.json")) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };

    [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ]
    .into_iter()
    .any(|section| {
        value
            .get(section)
            .and_then(serde_json::Value::as_object)
            .is_some_and(|dependencies| dependencies.contains_key(dependency))
    })
}
