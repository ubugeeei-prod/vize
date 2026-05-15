use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use vize_carton::{String, ToCompactString};

const SESSION_DIR: &str = "vize-patina";
const EXECUTABLE_ENV_VARS: [&str; 2] = ["CORSA_EXECUTABLE", "CORSA_PATH"];
const LEGACY_EXECUTABLE_ENV_VARS: [&str; 2] = ["TSGO_EXECUTABLE", "TSGO_PATH"];
const EXECUTABLE_NAMES: [&str; 2] = ["corsa", "tsgo"];
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
    project_root
        .join("__agent_only")
        .join(SESSION_DIR)
        .join(session_name.as_str())
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

pub(super) fn resolve_corsa_executable(project_root: &Path) -> PathBuf {
    if let Some(path) = resolve_executable_from_env(&EXECUTABLE_ENV_VARS) {
        return path;
    }
    if let Some(path) = resolve_executable_from_env(&LEGACY_EXECUTABLE_ENV_VARS) {
        return path;
    }

    for current in project_root.ancestors() {
        for candidate in corsa_executable_candidates(current) {
            if candidate.exists() {
                return candidate;
            }
        }
        if let Some(parent) = current.parent() {
            for candidate in corsa_executable_candidates(&parent.join("corsa-bind")) {
                if candidate.exists() {
                    return candidate;
                }
            }
        }
        if let Some(path) = resolve_node_modules_executable(current) {
            return path;
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        if let Some(path) = resolve_home_executable(&home) {
            return path;
        }
    }

    resolve_path_executable().unwrap_or_else(|| PathBuf::from(EXECUTABLE_NAMES[0]))
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

fn resolve_home_executable(home: &Path) -> Option<PathBuf> {
    const HOME_PREFIXES: [&str; 5] = [
        ".local/share/mise/shims",
        ".asdf/shims",
        ".volta/bin",
        ".npm-global/bin",
        ".npm/bin",
    ];

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

fn push_u64(buffer: &mut String, value: u64) {
    let rendered = value.to_compact_string();
    buffer.push_str(rendered.as_str());
}
