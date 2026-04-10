use std::path::{Path, PathBuf};
use vize_carton::{cstr, String};

const EXECUTABLE_NAMES: [&str; 2] = ["corsa", "tsgo"];

#[derive(Clone, Copy, Eq, PartialEq)]
enum CorsaCandidateKind {
    Wrapper,
    Native,
}

struct CorsaCandidate {
    path: String,
    kind: CorsaCandidateKind,
}

/// Walk upward until a `node_modules/vue` anchor is found.
pub(super) fn find_node_modules_with_vue(start: &Path) -> Option<PathBuf> {
    let mut dir = start;
    loop {
        let node_modules = dir.join("node_modules");
        if node_modules.join("vue").is_dir() {
            return Some(node_modules);
        }
        dir = dir.parent()?;
    }
}

/// Resolve the base directory used for per-client scratch state.
pub(super) fn resolve_temp_dir_base(project_root: Option<&Path>) -> PathBuf {
    let fallback_root = project_root
        .map(Path::to_path_buf)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    find_node_modules_with_vue(&fallback_root)
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or(fallback_root)
        .join("__agent_only")
        .join("vize-corsa")
}

/// Resolve the directories we should search when looking for a Corsa executable.
pub(crate) fn corsa_search_roots(working_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(working_dir) = working_dir {
        push_unique_root(&mut roots, working_dir.to_path_buf());
    }

    if let Ok(current_dir) = std::env::current_dir() {
        push_unique_root(&mut roots, current_dir);
    }

    if let Some(workspace_root) = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
    {
        push_unique_root(&mut roots, workspace_root.to_path_buf());
    }

    roots
}

/// Resolve a Corsa executable from a list of candidate project roots.
pub(crate) fn find_corsa_in_search_roots(search_roots: &[PathBuf]) -> Option<String> {
    let mut fallback = None;

    for root in search_roots {
        let root = root.to_string_lossy();
        let Some(candidate) = find_corsa_candidate(Some(root.as_ref())) else {
            continue;
        };

        if candidate.kind == CorsaCandidateKind::Native {
            return Some(candidate.path);
        }

        if fallback.is_none() {
            fallback = Some(candidate.path);
        }
    }

    fallback
}

/// Resolve a project-local Corsa executable from the current directory or ancestors.
#[cfg(test)]
pub(crate) fn find_corsa_in_local_node_modules(working_dir: Option<&str>) -> Option<String> {
    find_corsa_candidate(working_dir).map(|candidate| candidate.path)
}

pub(crate) fn normalize_corsa_path(path: &str) -> Option<String> {
    let path = PathBuf::from(path);
    let Some(bin_dir) = path.parent() else {
        return Some(path.to_string_lossy().into());
    };
    if bin_dir.file_name().and_then(|name| name.to_str()) != Some(".bin") {
        return Some(path.to_string_lossy().into());
    }

    let Some(node_modules_dir) = bin_dir.parent() else {
        return Some(path.to_string_lossy().into());
    };
    if node_modules_dir.file_name().and_then(|name| name.to_str()) != Some("node_modules") {
        return Some(path.to_string_lossy().into());
    }

    let Some(project_root) = node_modules_dir.parent() else {
        return Some(path.to_string_lossy().into());
    };

    let original = path.to_string_lossy().into_owned();
    find_corsa_in_search_roots(&corsa_search_roots(Some(project_root)))
        .filter(|resolved| resolved.as_str() != original.as_str())
        .or(Some(original.into()))
}

fn find_corsa_candidate(working_dir: Option<&str>) -> Option<CorsaCandidate> {
    let base_dir = working_dir
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())?;

    let mut fallback = None;
    let mut current = Some(base_dir.as_path());
    while let Some(dir) = current {
        if let Some(path) = search_project_cache(dir) {
            return Some(CorsaCandidate {
                path,
                kind: CorsaCandidateKind::Native,
            });
        }
        if let Some(parent) = dir.parent() {
            if let Some(path) = search_project_cache(&parent.join("corsa-bind")) {
                return Some(CorsaCandidate {
                    path,
                    kind: CorsaCandidateKind::Native,
                });
            }
        }
        if let Some(candidate) = search_local_install(dir) {
            if should_replace_candidate(fallback.as_ref(), &candidate) {
                fallback = Some(candidate);
            }
        }
        current = dir.parent();
    }

    fallback
}

/// Resolve a globally installed Corsa executable from common package-manager paths.
pub(super) fn find_corsa_in_common_locations() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    for executable in EXECUTABLE_NAMES {
        let candidates: [String; 10] = [
            cstr!("{home}/.npm-global/bin/{executable}"),
            cstr!("{home}/.npm/bin/{executable}"),
            cstr!("{home}/.local/share/pnpm/{executable}"),
            cstr!("{home}/.volta/bin/{executable}"),
            cstr!("{home}/.local/share/mise/shims/{executable}"),
            cstr!("{home}/.asdf/shims/{executable}"),
            cstr!("{home}/.local/share/fnm/node-versions/current/bin/{executable}"),
            cstr!("{home}/.nvm/versions/node/current/bin/{executable}"),
            cstr!("/opt/homebrew/bin/{executable}"),
            cstr!("/usr/local/bin/{executable}"),
        ];

        for path in candidates {
            if Path::new(path.as_str()).exists() {
                return Some(path);
            }
        }
    }

    if let Ok(output) = std::process::Command::new("npm")
        .args(["root", "-g"])
        .output()
    {
        if output.status.success() {
            #[allow(clippy::disallowed_types)]
            let npm_root = std::string::String::from_utf8_lossy(&output.stdout);
            let npm_root = npm_root.trim();
            if let Some(lib_parent) = Path::new(npm_root).parent() {
                for executable in EXECUTABLE_NAMES {
                    let corsa_path = lib_parent.join("bin").join(executable);
                    if corsa_path.exists() {
                        return Some(corsa_path.to_string_lossy().into());
                    }
                }
            }
        }
    }

    None
}

fn push_unique_root(roots: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !roots.iter().any(|existing| existing == &candidate) {
        roots.push(candidate);
    }
}

/// Resolve a `corsa` executable from `PATH`, with `tsgo` as a legacy fallback.
pub(super) fn find_corsa_in_path() -> Option<String> {
    for executable in EXECUTABLE_NAMES {
        if let Ok(path) = which::which(executable) {
            return Some(path.to_string_lossy().into());
        }
    }

    None
}

fn should_replace_candidate(existing: Option<&CorsaCandidate>, candidate: &CorsaCandidate) -> bool {
    match existing {
        None => true,
        Some(existing) => {
            existing.kind == CorsaCandidateKind::Wrapper
                && candidate.kind == CorsaCandidateKind::Native
        }
    }
}

fn search_local_install(dir: &Path) -> Option<CorsaCandidate> {
    let platform_suffix = platform_suffix();
    let pnpm_pattern = dir.join("node_modules/.pnpm");
    if pnpm_pattern.exists() {
        if let Ok(entries) = std::fs::read_dir(&pnpm_pattern) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("@typescript+native-preview-")
                    && name_str.contains(platform_suffix)
                {
                    for executable in EXECUTABLE_NAMES {
                        let native_path = entry.path().join(&*cstr!(
                            "node_modules/@typescript/native-preview-{}/lib/{}",
                            platform_suffix,
                            executable
                        ));
                        if native_path.exists() {
                            return Some(CorsaCandidate {
                                path: native_path.to_string_lossy().into(),
                                kind: CorsaCandidateKind::Native,
                            });
                        }
                    }
                }
            }
        }
    }

    for executable in EXECUTABLE_NAMES {
        let native_candidates = [
            dir.join(&*cstr!(
                "node_modules/@typescript/native-preview-{}/lib/{}",
                platform_suffix,
                executable
            )),
            dir.join(&*cstr!(
                "node_modules/@typescript/native-preview/lib/{executable}"
            )),
        ];
        for candidate in &native_candidates {
            if candidate.exists() {
                return Some(CorsaCandidate {
                    path: candidate.to_string_lossy().into(),
                    kind: CorsaCandidateKind::Native,
                });
            }
        }
    }

    for executable in EXECUTABLE_NAMES {
        let fallback_candidates = [
            dir.join("node_modules/.bin").join(executable),
            dir.join("node_modules/@typescript/native-preview/bin")
                .join(executable),
        ];
        for candidate in &fallback_candidates {
            if candidate.exists() {
                return Some(CorsaCandidate {
                    path: candidate.to_string_lossy().into(),
                    kind: CorsaCandidateKind::Wrapper,
                });
            }
        }
    }

    None
}

fn search_project_cache(dir: &Path) -> Option<String> {
    for executable in EXECUTABLE_NAMES {
        let cache_candidates = [
            dir.join(".cache").join(executable),
            dir.join(".cache").join(cstr!("{executable}.exe").as_str()),
            dir.join("ref")
                .join("typescript-go")
                .join(".cache")
                .join(executable),
            dir.join("ref")
                .join("typescript-go")
                .join(".cache")
                .join(cstr!("{executable}.exe").as_str()),
            dir.join("ref")
                .join("typescript-go")
                .join("built")
                .join("local")
                .join(executable),
            dir.join("ref")
                .join("typescript-go")
                .join("built")
                .join("local")
                .join(cstr!("{executable}.exe").as_str()),
        ];
        for candidate in &cache_candidates {
            if candidate.exists() {
                return Some(candidate.to_string_lossy().into());
            }
        }
    }

    None
}

fn platform_suffix() -> &'static str {
    if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "darwin-arm64"
        } else {
            "darwin-x64"
        }
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "aarch64") {
            "linux-arm64"
        } else {
            "linux-x64"
        }
    } else if cfg!(target_os = "windows") {
        "win32-x64"
    } else {
        ""
    }
}
