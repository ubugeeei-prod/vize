use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use vize_carton::{String, ToCompactString, cstr};

const EXECUTABLE_ENV_VARS: [&str; 2] = ["CORSA_EXECUTABLE", "CORSA_PATH"];
const LEGACY_EXECUTABLE_ENV_VARS: [&str; 2] = ["TSGO_EXECUTABLE", "TSGO_PATH"];
const EXECUTABLE_NAMES: [&str; 2] = ["corsa", "tsgo"];
const SESSION_DIRECTORY_PREFIX: &str = "session-";
pub(super) const VIRTUAL_FILE_NAME: &str = "active.patina.ts";
pub(super) const TSCONFIG_FILE_NAME: &str = "tsconfig.json";
static SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);
pub(super) const TSCONFIG_CONTENTS: &str = r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "files": ["active.patina.ts"]
}
"#;

pub(super) fn path_to_wire(path: &Path) -> String {
    path.to_string_lossy().as_ref().to_compact_string()
}

pub(super) fn allocate_session_root(project_root: &Path) -> PathBuf {
    let session_name = next_session_directory_name();
    cleanup_stale_session_roots(project_root);
    session_store_root(project_root).join(session_name.as_str())
}

pub(super) fn remove_session_root(session_root: &Path) {
    let _ = std::fs::remove_dir_all(session_root);
    remove_empty_session_parents(session_root);
}

pub(super) fn next_session_directory_name() -> String {
    let counter = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id() as u64;
    let mut name = String::with_capacity(32);
    name.push_str("session-");
    push_u64(&mut name, pid);
    name.push('-');
    push_u64(&mut name, counter);
    name
}

fn session_store_root(project_root: &Path) -> PathBuf {
    project_root.join(".vize").join("patina")
}

fn legacy_session_store_root(project_root: &Path) -> PathBuf {
    project_root
        .join("node_modules")
        .join(".vize")
        .join("patina")
}

fn cleanup_stale_session_roots(project_root: &Path) {
    cleanup_stale_sessions_in(&session_store_root(project_root));
    cleanup_stale_sessions_in(&legacy_session_store_root(project_root));
}

fn cleanup_stale_sessions_in(session_store: &Path) {
    let Ok(entries) = std::fs::read_dir(session_store) else {
        return;
    };

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }

        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if is_stale_session_directory(name) {
            remove_session_root(&entry.path());
        }
    }
}

fn is_stale_session_directory(name: &str) -> bool {
    let Some(pid) = session_directory_pid(name) else {
        return false;
    };
    !process_is_running(pid)
}

fn session_directory_pid(name: &str) -> Option<u64> {
    let rest = name.strip_prefix(SESSION_DIRECTORY_PREFIX)?;
    let (pid, _) = rest.split_once('-')?;
    pid.parse().ok()
}

#[cfg(unix)]
fn process_is_running(pid: u64) -> bool {
    if pid == 0 || pid > i32::MAX as u64 {
        return false;
    }

    let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
    result == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

#[cfg(not(unix))]
fn process_is_running(_pid: u64) -> bool {
    true
}

fn remove_empty_session_parents(session_root: &Path) {
    let Some(session_store) = session_root.parent() else {
        return;
    };
    let _ = std::fs::remove_dir(session_store);

    let Some(vize_dir) = session_store.parent() else {
        return;
    };
    if vize_dir.file_name().and_then(|name| name.to_str()) == Some(".vize") {
        let _ = std::fs::remove_dir(vize_dir);
    }
}

pub(super) fn resolve_project_root(filename: &str) -> PathBuf {
    let start_dir = source_directory(filename);
    let mut current = start_dir.as_path();
    let mut package_root = None;

    loop {
        if current.join("node_modules").join("vue").is_dir() {
            return current.to_path_buf();
        }
        if package_root.is_none() && current.join("package.json").is_file() {
            package_root = Some(current.to_path_buf());
        }
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }

    package_root.unwrap_or(start_dir)
}

pub(super) fn resolve_corsa_executable(
    project_root: &Path,
    configured_path: Option<&Path>,
) -> Result<PathBuf, String> {
    if let Some(path) = configured_path {
        if !path.exists() {
            return Err(cstr!(
                "Configured Corsa executable does not exist: {}",
                path.display()
            ));
        }
        return Ok(path.to_path_buf());
    }

    if let Some(path) = resolve_executable_from_env(&EXECUTABLE_ENV_VARS) {
        return Ok(path);
    }
    if let Some(path) = resolve_executable_from_env(&LEGACY_EXECUTABLE_ENV_VARS) {
        return Ok(path);
    }

    for current in project_root.ancestors() {
        for candidate in corsa_executable_candidates(current) {
            if candidate.exists() {
                return Ok(candidate);
            }
        }
        if let Some(parent) = current.parent() {
            for candidate in corsa_executable_candidates(&parent.join("corsa-bind")) {
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }
        if let Some(path) = resolve_node_modules_executable(current) {
            return Ok(path);
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        if let Some(path) = resolve_home_executable(&home) {
            return Ok(path);
        }
    }

    Ok(resolve_path_executable().unwrap_or_else(|| PathBuf::from(EXECUTABLE_NAMES[0])))
}

fn source_directory(filename: &str) -> PathBuf {
    let path = Path::new(filename);
    if path.is_absolute() {
        return path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.to_path_buf());
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let joined = cwd.join(path);
    joined.parent().map(Path::to_path_buf).unwrap_or(cwd)
}

fn resolve_executable_from_env(env_names: &[&str]) -> Option<PathBuf> {
    for env_name in env_names {
        let Some(path) = std::env::var_os(env_name).map(PathBuf::from) else {
            continue;
        };
        if path.exists() {
            return Some(path);
        }
    }
    None
}

fn corsa_executable_candidates(root: &Path) -> [PathBuf; 12] {
    [
        root.join(".cache").join("corsa"),
        root.join(".cache").join("corsa.exe"),
        root.join(".cache").join("tsgo"),
        root.join(".cache").join("tsgo.exe"),
        root.join("ref")
            .join("typescript-go")
            .join(".cache")
            .join("corsa"),
        root.join("ref")
            .join("typescript-go")
            .join(".cache")
            .join("corsa.exe"),
        root.join("ref")
            .join("typescript-go")
            .join(".cache")
            .join("tsgo"),
        root.join("ref")
            .join("typescript-go")
            .join(".cache")
            .join("tsgo.exe"),
        root.join("ref")
            .join("typescript-go")
            .join("built")
            .join("local")
            .join("corsa"),
        root.join("ref")
            .join("typescript-go")
            .join("built")
            .join("local")
            .join("corsa.exe"),
        root.join("ref")
            .join("typescript-go")
            .join("built")
            .join("local")
            .join("tsgo"),
        root.join("ref")
            .join("typescript-go")
            .join("built")
            .join("local")
            .join("tsgo.exe"),
    ]
}

fn resolve_node_modules_executable(root: &Path) -> Option<PathBuf> {
    if let Some(path) = resolve_native_preview_executable(root) {
        return Some(path);
    }

    for executable in EXECUTABLE_NAMES {
        let direct = root.join("node_modules").join(".bin").join(executable);
        if direct.exists() {
            return Some(direct);
        }
    }

    for executable in EXECUTABLE_NAMES {
        let native_preview = root
            .join("node_modules")
            .join("@typescript")
            .join("native-preview")
            .join("bin")
            .join(executable);
        if native_preview.exists() {
            return Some(native_preview);
        }
    }

    None
}

fn resolve_native_preview_executable(root: &Path) -> Option<PathBuf> {
    let platform_suffix = platform_suffix();
    if platform_suffix.is_empty() {
        return None;
    }

    let pnpm_root = root.join("node_modules/.pnpm");
    if pnpm_root.exists()
        && let Ok(entries) = std::fs::read_dir(&pnpm_root)
    {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("@typescript+native-preview-") || !name.contains(platform_suffix) {
                continue;
            }

            for executable in EXECUTABLE_NAMES {
                let candidate = entry.path().join(&*cstr!(
                    "node_modules/@typescript/native-preview-{}/lib/{}",
                    platform_suffix,
                    executable
                ));
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }

    for executable in EXECUTABLE_NAMES {
        let native_candidates = [
            root.join(&*cstr!(
                "node_modules/@typescript/native-preview-{}/lib/{}",
                platform_suffix,
                executable
            )),
            root.join(&*cstr!(
                "node_modules/@typescript/native-preview/lib/{executable}"
            )),
        ];
        for candidate in native_candidates {
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    None
}

fn resolve_home_executable(home: &Path) -> Option<PathBuf> {
    const HOME_PREFIXES: [&str; 4] = [".asdf/shims", ".volta/bin", ".npm-global/bin", ".npm/bin"];

    for prefix in HOME_PREFIXES {
        for executable in EXECUTABLE_NAMES {
            let location = home.join(prefix).join(executable);
            if location.exists() {
                return Some(location);
            }
        }
    }

    None
}

fn resolve_path_executable() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;

    for directory in std::env::split_paths(&path) {
        for executable in EXECUTABLE_NAMES {
            let location = directory.join(executable);
            if location.exists() {
                return Some(location);
            }
        }
    }

    None
}

pub(super) fn platform_suffix() -> &'static str {
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

fn push_u64(buffer: &mut String, value: u64) {
    let rendered = value.to_compact_string();
    buffer.push_str(rendered.as_str());
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::is_stale_session_directory;
    use super::{cleanup_stale_session_roots, resolve_corsa_executable, session_store_root};
    use std::{
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };
    use vize_carton::cstr;

    static NEXT_CASE_ID: AtomicU64 = AtomicU64::new(0);

    fn case_dir(name: &str) -> PathBuf {
        let id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("vize-tests")
            .join(&*cstr!(
                "patina-corsa-paths-{name}-{}-{id}",
                std::process::id()
            ))
    }

    #[test]
    fn prefers_native_preview_binary_over_node_modules_bin_wrapper() {
        let root = case_dir("native-over-wrapper");
        let _ = std::fs::remove_dir_all(&root);
        let wrapper = root.join("node_modules/.bin/tsgo");
        let native = root
            .join("node_modules")
            .join("@typescript")
            .join(&*cstr!("native-preview-{}", super::platform_suffix()))
            .join("lib")
            .join("tsgo");

        write_file(&wrapper);
        write_file(&native);

        assert_eq!(resolve_corsa_executable(&root, None).unwrap(), native);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn resolves_node_modules_bin_wrapper_when_native_binary_is_absent() {
        let root = case_dir("wrapper-fallback");
        let _ = std::fs::remove_dir_all(&root);
        let wrapper = root.join("node_modules/.bin/tsgo");

        write_file(&wrapper);

        assert_eq!(resolve_corsa_executable(&root, None).unwrap(), wrapper);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn configured_corsa_path_must_exist() {
        let root = case_dir("configured-missing");
        let missing = root.join("missing-corsa");
        let error = resolve_corsa_executable(&root, Some(missing.as_path())).unwrap_err();

        assert!(error.contains("Configured Corsa executable does not exist"));
        assert!(error.contains("missing-corsa"));
    }

    #[cfg(unix)]
    #[test]
    fn removes_dead_session_directories() {
        let root = case_dir("dead-session-cleanup");
        let _ = std::fs::remove_dir_all(&root);
        let store = session_store_root(&root);
        let stale = store.join("session-9999999999-0");
        let legacy_stale = root
            .join("node_modules")
            .join(".vize")
            .join("patina")
            .join("session-9999999999-1");
        let unrelated = store.join("cache");

        std::fs::create_dir_all(&stale).unwrap();
        std::fs::create_dir_all(&legacy_stale).unwrap();
        std::fs::create_dir_all(&unrelated).unwrap();

        cleanup_stale_session_roots(&root);

        assert!(!stale.exists());
        assert!(!legacy_stale.exists());
        assert!(unrelated.exists());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn keeps_live_session_directories() {
        let root = case_dir("live-session-cleanup");
        let _ = std::fs::remove_dir_all(&root);
        let store = session_store_root(&root);
        let live = store.join(&*cstr!("session-{}-0", std::process::id()));

        std::fs::create_dir_all(&live).unwrap();

        cleanup_stale_session_roots(&root);

        assert!(live.exists());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn identifies_session_directories_by_pid() {
        assert!(is_stale_session_directory("session-9999999999-0"));
        assert!(!is_stale_session_directory("session-not-a-pid-0"));
        assert!(!is_stale_session_directory("cache"));
    }

    fn write_file(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }
}
