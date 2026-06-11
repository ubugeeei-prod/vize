use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use vize_carton::{
    String, ToCompactString,
    corsa_resolver::{CORSA_EXECUTABLE_NAMES, CorsaResolveError, CorsaResolveRequest},
    cstr,
};

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
    let request = CorsaResolveRequest {
        explicit_path: configured_path,
        project_root: Some(project_root),
    };

    match vize_carton::corsa_resolver::resolve_corsa_executable(request) {
        Ok(path) => Ok(path),
        // Preserve the historical lenient fallback: a bare `corsa` lets the
        // spawn-time `PATH` lookup have the final word.
        Err(CorsaResolveError::NotFound) => Ok(PathBuf::from(CORSA_EXECUTABLE_NAMES[0])),
        Err(error @ CorsaResolveError::ExplicitNotFound { .. }) => Err(cstr!("{error}")),
    }
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
            .join(&*cstr!(
                "native-preview-{}",
                vize_carton::corsa_resolver::platform_suffix()
            ))
            .join("lib")
            .join("tsgo");

        write_file(&wrapper);
        write_file(&native);

        assert_eq!(resolve_corsa_executable(&root, None).unwrap(), native);

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
