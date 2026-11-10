use super::{ServeArgs, setup};
use setup::validate_direct_vite_musea_setup;
use std::path::{Path, PathBuf};
use vize_l0::{String, ToCompactString, cstr};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ServePlan {
    pub(super) program: PathBuf,
    pub(super) args: Vec<String>,
    pub(super) env: Vec<(String, String)>,
}

pub(super) fn create_serve_plan(args: &ServeArgs, cwd: &Path) -> Result<ServePlan, String> {
    if let Some(stories) = &args.stories {
        return Err(cstr!(
            "vize musea: --stories is not supported by the Vite-backed serve entrypoint yet (got {}). Configure Musea include patterns in vue.musea in vite.config.* instead.",
            stories.display()
        ));
    }

    let project_root = match setup::find_direct_vite_musea_root(cwd) {
        Some(root) => root,
        None if let Some(nuxt_root) = find_nuxt_project_root(cwd) => {
            return Err(nuxt_musea_message(&nuxt_root, args.build));
        }
        None => {
            validate_direct_vite_musea_setup(cwd)?;
            return Err(cstr!(
                "vize musea: could not determine the direct Musea project root"
            ));
        }
    };
    let program = resolve_vite_runner(&project_root).unwrap_or_else(|| {
        PathBuf::from(
            if setup::package_json_has_dependency(&project_root, "vite-plus") {
                "vp"
            } else {
                "vite"
            },
        )
    });
    let mut env = Vec::new();
    if let Some(config_path) = &args.config {
        env.push((
            cstr!("VIZE_CONFIG_FILE"),
            config_path.to_string_lossy().as_ref().to_compact_string(),
        ));
    }

    if args.build {
        env.push((cstr!("VIZE_MUSEA_STATIC_BUILD"), cstr!("1")));
        return Ok(ServePlan {
            program,
            args: vec![cstr!("build")],
            env,
        });
    }

    let mut vite_args = vec![
        cstr!("dev"),
        cstr!("--host"),
        args.host.clone(),
        cstr!("--port"),
        args.port.to_compact_string(),
    ];
    if args.open {
        vite_args.extend([cstr!("--open"), cstr!("/__musea__")]);
    }
    if args.strict_port {
        vite_args.push(cstr!("--strictPort"));
    }

    Ok(ServePlan {
        program,
        args: vite_args,
        env,
    })
}

pub(super) fn resolve_vite_binary(cwd: &Path) -> Option<PathBuf> {
    resolve_binary(cwd, VITE_BIN_NAMES)
}

pub(super) fn resolve_vite_runner(cwd: &Path) -> Option<PathBuf> {
    for ancestor in cwd.ancestors() {
        let bin_dir = ancestor.join("node_modules").join(".bin");
        for name in VITE_BIN_NAMES {
            let candidate = bin_dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        for name in VITE_PLUS_BIN_NAMES {
            let candidate = bin_dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn resolve_binary(cwd: &Path, names: &[&str]) -> Option<PathBuf> {
    cwd.ancestors().find_map(|ancestor| {
        let bin_dir = ancestor.join("node_modules").join(".bin");
        names
            .iter()
            .map(|name| bin_dir.join(name))
            .find(|candidate| candidate.is_file())
    })
}

fn find_nuxt_project_root(cwd: &Path) -> Option<PathBuf> {
    cwd.ancestors()
        .find(|ancestor| is_nuxt_project_root(ancestor))
        .map(Path::to_path_buf)
}

fn is_nuxt_project_root(root: &Path) -> bool {
    ["nuxt.config.ts", "nuxt.config.mts", "nuxt.config.js"]
        .into_iter()
        .any(|file_name| root.join(file_name).exists())
        || setup::package_json_has_dependency(root, "nuxt")
}

fn has_vize_nuxt_integration(root: &Path) -> bool {
    setup::package_json_has_dependency(root, "@vizejs/nuxt")
        || root
            .join("node_modules")
            .join("@vizejs")
            .join("nuxt")
            .join("package.json")
            .exists()
}

fn nuxt_musea_message(root: &Path, build: bool) -> String {
    let command = if build { "nuxi build" } else { "nuxi dev" };
    if has_vize_nuxt_integration(root) {
        return cstr!(
            "vize musea: detected a Nuxt project using `@vizejs/nuxt` at {}.\n  The standalone `vize musea` command only runs direct Vite projects with `vite` and `@vizejs/vite-plugin-musea` configured in Vite.\n  In this Nuxt setup, Musea is provided by the Nuxt module at `/__musea__/`; run `{}` and open that route instead.",
            root.display(),
            command
        );
    }

    cstr!(
        "vize musea: detected a Nuxt project at {}.\n  The standalone `vize musea` command only runs direct Vite projects with `vite` and `@vizejs/vite-plugin-musea` configured in Vite.\n  For Nuxt, enable `@vizejs/nuxt`, run `{}`, and open `/__musea__/` instead.",
        root.display(),
        command
    )
}

#[cfg(windows)]
pub(super) const VITE_BIN_NAMES: &[&str] = &["vite.cmd", "vite.ps1", "vite"];

#[cfg(not(windows))]
pub(super) const VITE_BIN_NAMES: &[&str] = &["vite"];

#[cfg(windows)]
pub(super) const VITE_PLUS_BIN_NAMES: &[&str] = &["vp.cmd", "vp.ps1", "vp"];

#[cfg(not(windows))]
pub(super) const VITE_PLUS_BIN_NAMES: &[&str] = &["vp"];
