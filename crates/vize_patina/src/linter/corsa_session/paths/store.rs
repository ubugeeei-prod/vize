use super::{
    CORSA_EXECUTABLE_NAMES, CorsaResolveError, CorsaResolveRequest, Path, PathBuf,
    SESSION_DIRECTORY_PREFIX, String, TSCONFIG_CONTENTS, ToCompactString, Value, cstr,
    remove_session_root,
};

mod files;
mod walk;

use files::ambient_declaration_files;

pub(super) fn session_store_root(project_root: &Path) -> PathBuf {
    project_root.join(".vize").join("patina")
}

fn legacy_session_store_root(project_root: &Path) -> PathBuf {
    project_root
        .join("node_modules")
        .join(".vize")
        .join("patina")
}

pub(super) fn cleanup_stale_session_roots(project_root: &Path) {
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

pub(super) fn is_stale_session_directory(name: &str) -> bool {
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

pub(super) fn remove_empty_session_parents(session_root: &Path) {
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

pub(in super::super) fn resolve_project_root(filename: &str) -> PathBuf {
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

pub(in super::super) fn resolve_corsa_executable(
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

pub(in super::super) fn remove_finished_process_sessions(project_root: &Path) {
    let pid = u64::from(std::process::id());
    remove_pid_sessions(&session_store_root(project_root), pid);
    remove_pid_sessions(&legacy_session_store_root(project_root), pid);
}

fn remove_pid_sessions(session_store: &Path, pid: u64) {
    let Ok(entries) = std::fs::read_dir(session_store) else {
        return;
    };
    let prefix = cstr!("session-{pid}-");
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
        if name.starts_with(prefix.as_str()) {
            remove_session_root(&entry.path());
        }
    }
}

pub(in super::super) fn session_tsconfig_contents(project_root: &Path, filename: &str) -> String {
    let mut value = stub_tsconfig_value();
    if let Some(tsconfig) = nearest_tsconfig(filename) {
        overlay_user_compiler_options(&mut value, project_root, &tsconfig);
        let declarations = ambient_declaration_files(&tsconfig);
        if !declarations.is_empty() {
            insert_files(&mut value, &declarations);
        }
    }
    match serde_json::to_string_pretty(&value) {
        Ok(text) => {
            let mut rendered = text.to_compact_string();
            rendered.push('\n');
            rendered
        }
        Err(_) => TSCONFIG_CONTENTS.to_compact_string(),
    }
}

fn stub_tsconfig_value() -> Value {
    serde_json::json!({
        "compilerOptions": {
            "target": "ES2022",
            "module": "ESNext",
            "moduleResolution": "bundler",
            "allowImportingTsExtensions": true,
            "lib": ["ES2022", "DOM", "DOM.Iterable"],
            "rootDirs": [".", "../../.."],
            "strict": true,
            "noEmit": true,
            "skipLibCheck": true
        },
        "include": ["**/*.patina.ts"]
    })
}

fn nearest_tsconfig(filename: &str) -> Option<PathBuf> {
    let mut current = source_directory(filename);
    loop {
        let candidate = current.join("tsconfig.json");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn overlay_user_compiler_options(value: &mut Value, project_root: &Path, tsconfig: &Path) {
    let Ok(snapshot) = vize_canon::snapshot_tsconfig_compiler_options(project_root, tsconfig)
    else {
        return;
    };
    let Some(options) = value
        .get_mut("compilerOptions")
        .and_then(Value::as_object_mut)
    else {
        return;
    };
    for key in [
        "paths",
        "baseUrl",
        "types",
        "typeRoots",
        "noUncheckedIndexedAccess",
    ] {
        if let Some(option) = snapshot.get(key) {
            options.insert(json_string(key), option.clone());
        }
    }
}

fn insert_files(value: &mut Value, declarations: &[String]) {
    let Some(root) = value.as_object_mut() else {
        return;
    };
    let files = declarations
        .iter()
        .map(|path| Value::String(json_string(path.as_str())))
        .collect();
    root.insert(json_string("files"), Value::Array(files));
}

#[expect(
    clippy::disallowed_types,
    reason = "serde_json object keys and string values are std String"
)]
fn json_string(text: &str) -> std::string::String {
    std::string::String::from(text)
}

pub(super) fn push_u64(buffer: &mut String, value: u64) {
    let rendered = value.to_compact_string();
    buffer.push_str(rendered.as_str());
}

#[cfg(test)]
mod compiler_options_tests;
