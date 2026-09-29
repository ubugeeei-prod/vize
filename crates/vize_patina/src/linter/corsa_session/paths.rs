use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use vize_l0::{
    String, ToCompactString,
    corsa_resolver::{CORSA_EXECUTABLE_NAMES, CorsaResolveError, CorsaResolveRequest},
    cstr,
};

const SESSION_DIRECTORY_PREFIX: &str = "session-";
pub(super) const TSCONFIG_FILE_NAME: &str = "tsconfig.json";
static SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);
pub(super) const TSCONFIG_CONTENTS: &str = r#"{
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
}
"#;

pub(super) fn path_to_wire(path: &Path) -> String {
    path.to_string_lossy().as_ref().to_compact_string()
}

/// Mirror the authored file's package-relative directory below the session
/// root. TypeScript's `rootDirs` then resolves its relative imports against
/// the real package tree without changing Canon's generated source offsets.
pub(super) fn virtual_file_path(
    session_root: &Path,
    project_root: &Path,
    filename: &str,
) -> PathBuf {
    let authored = Path::new(filename);
    let authored = if authored.is_absolute() {
        authored.to_path_buf()
    } else {
        std::env::current_dir().map_or_else(|_| authored.to_path_buf(), |cwd| cwd.join(authored))
    };
    let relative = authored
        .strip_prefix(project_root)
        .ok()
        .filter(|path| {
            path.components()
                .all(|component| matches!(component, std::path::Component::Normal(_)))
        })
        .unwrap_or_else(|| {
            // A source outside the package or with parent traversals cannot be
            // mirrored safely inside the session directory.
            authored
                .file_name()
                .map(Path::new)
                .unwrap_or(authored.as_path())
        });
    let mut output = session_root.join(relative);
    let filename = output.file_name().unwrap_or_default().to_string_lossy();
    output.set_file_name(&*cstr!("{filename}.patina.ts"));
    output
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

    match vize_l0::corsa_resolver::resolve_corsa_executable(request) {
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

pub(super) fn remove_finished_process_sessions(project_root: &Path) {
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

pub(super) fn session_tsconfig_contents(project_root: &Path, filename: &str) -> String {
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
    for key in ["paths", "baseUrl", "types", "typeRoots"] {
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

fn ambient_declaration_files(tsconfig: &Path) -> Vec<String> {
    let Some(base) = tsconfig.parent() else {
        return Vec::new();
    };
    let Ok(content) = std::fs::read_to_string(tsconfig) else {
        return Vec::new();
    };
    let Some(value) = parse_jsonc(&content) else {
        return Vec::new();
    };
    let excludes = exclude_patterns(&value);
    let mut matched = Vec::new();
    if let Some(files) = value.get("files").and_then(Value::as_array) {
        for entry in files.iter().filter_map(Value::as_str) {
            let path = resolve_config_path(base, entry);
            if path.is_file() {
                matched.push(path);
            }
        }
    }
    if let Some(include) = value.get("include").and_then(Value::as_array) {
        for entry in include.iter().filter_map(Value::as_str) {
            collect_include_entry(base, entry, &mut matched);
        }
    }
    if value.get("files").is_none() && value.get("include").is_none() {
        walk_glob(base, &["**", "*.d.ts"], &mut matched);
    }

    let mut declarations = Vec::new();
    for path in matched {
        if !is_declaration_file(&path) {
            continue;
        }
        let relative = path.strip_prefix(base).unwrap_or(path.as_path());
        let relative = slash_path(relative);
        if is_excluded(relative.as_str(), &excludes) {
            continue;
        }
        let absolute = std::fs::canonicalize(&path).unwrap_or(path);
        declarations.push(slash_path(&absolute));
    }
    declarations.sort();
    declarations.dedup();
    declarations
}

fn exclude_patterns(value: &Value) -> Vec<String> {
    match value.get("exclude") {
        Some(Value::Array(entries)) => entries
            .iter()
            .filter_map(Value::as_str)
            .map(ToCompactString::to_compact_string)
            .collect(),
        Some(_) => Vec::new(),
        None => ["node_modules", "bower_components", "jspm_packages"]
            .into_iter()
            .map(ToCompactString::to_compact_string)
            .collect(),
    }
}

fn is_excluded(relative: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    let mut prefix = relative;
    loop {
        if patterns
            .iter()
            .any(|pattern| glob_match(pattern.as_str(), prefix))
        {
            return true;
        }
        let Some((parent, _)) = prefix.rsplit_once('/') else {
            return false;
        };
        prefix = parent;
    }
}

fn collect_include_entry(base: &Path, pattern: &str, out: &mut Vec<PathBuf>) {
    if pattern.contains('*') || pattern.contains('?') {
        collect_glob(base, pattern, out);
        return;
    }
    let path = resolve_config_path(base, pattern);
    if path.is_file() {
        out.push(path);
    } else if path.is_dir() {
        walk_glob(&path, &["**", "*.d.ts"], out);
    }
}

fn resolve_config_path(base: &Path, pattern: &str) -> PathBuf {
    let pattern = pattern.trim_start_matches("./");
    let path = Path::new(pattern);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(pattern)
    }
}

fn collect_glob(base: &Path, pattern: &str, out: &mut Vec<PathBuf>) {
    let pattern = pattern.trim_start_matches("./");
    let absolute = Path::new(pattern).is_absolute();
    let mut root = if absolute {
        PathBuf::from("/")
    } else {
        base.to_path_buf()
    };
    let raw_parts: Vec<&str> = pattern.split('/').filter(|part| !part.is_empty()).collect();
    let mut index = 0usize;
    if absolute {
        while raw_parts
            .get(index)
            .is_some_and(|part| !part.contains('*') && !part.contains('?'))
        {
            let Some(part) = raw_parts.get(index) else {
                break;
            };
            root.push(part);
            index += 1;
        }
    }
    let Some(parts) = raw_parts.get(index..) else {
        return;
    };
    if parts.is_empty() {
        if root.is_file() {
            out.push(root);
        }
        return;
    }
    walk_glob(&root, parts, out);
}

fn walk_glob(dir: &Path, parts: &[&str], out: &mut Vec<PathBuf>) {
    let Some((head, rest)) = parts.split_first() else {
        return;
    };
    if *head == "**" {
        walk_glob(dir, rest, out);
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            if is_walkable_dir(&entry) {
                walk_glob(&entry.path(), parts, out);
            }
        }
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if !segment_match(head, &name) {
            continue;
        }
        let path = entry.path();
        if rest.is_empty() {
            if path.is_file() {
                out.push(path);
            }
            continue;
        }
        if is_walkable_dir(&entry) {
            walk_glob(&path, rest, out);
        }
    }
}

fn is_walkable_dir(entry: &std::fs::DirEntry) -> bool {
    let Ok(file_type) = entry.file_type() else {
        return false;
    };
    if !file_type.is_dir() {
        return false;
    }
    let Ok(name) = entry.file_name().into_string() else {
        return false;
    };
    !matches!(
        name.as_str(),
        "node_modules"
            | "bower_components"
            | "jspm_packages"
            | ".vize"
            | ".git"
            | "target"
            | "dist"
    )
}

fn is_declaration_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
}

fn segment_match(pattern: &str, name: &str) -> bool {
    segment_match_bytes(pattern.as_bytes(), name.as_bytes())
}

fn segment_match_bytes(pattern: &[u8], name: &[u8]) -> bool {
    let mut pattern_index = 0usize;
    let mut name_index = 0usize;
    let mut star_pattern = None;
    let mut star_name = 0usize;
    while name_index < name.len() {
        let Some(name_byte) = name.get(name_index).copied() else {
            break;
        };
        if pattern
            .get(pattern_index)
            .is_some_and(|pattern_byte| *pattern_byte == b'?' || *pattern_byte == name_byte)
        {
            pattern_index += 1;
            name_index += 1;
            continue;
        }
        if pattern
            .get(pattern_index)
            .is_some_and(|pattern_byte| *pattern_byte == b'*')
        {
            star_pattern = Some(pattern_index);
            star_name = name_index;
            pattern_index += 1;
            continue;
        }
        if let Some(star) = star_pattern {
            pattern_index = star + 1;
            star_name += 1;
            name_index = star_name;
            continue;
        }
        return false;
    }
    while pattern.get(pattern_index).is_some_and(|byte| *byte == b'*') {
        pattern_index += 1;
    }
    pattern_index == pattern.len()
}

fn glob_match(pattern: &str, relative: &str) -> bool {
    let pattern = pattern.trim_start_matches("./");
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|part| !part.is_empty()).collect();
    let name_parts: Vec<&str> = relative
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    glob_parts(&pattern_parts, &name_parts)
}

fn glob_parts(pattern: &[&str], name: &[&str]) -> bool {
    let Some((head, pattern_rest)) = pattern.split_first() else {
        return name.is_empty();
    };
    if *head == "**" {
        if glob_parts(pattern_rest, name) {
            return true;
        }
        return match name.split_first() {
            Some((_, name_rest)) => glob_parts(pattern, name_rest),
            None => false,
        };
    }
    let Some((name_head, name_rest)) = name.split_first() else {
        return false;
    };
    segment_match(head, name_head) && glob_parts(pattern_rest, name_rest)
}

fn slash_path(path: &Path) -> String {
    let mut rendered = String::new("");
    for component in path.components() {
        match component {
            std::path::Component::RootDir => rendered.push('/'),
            std::path::Component::Normal(part) => {
                if !rendered.is_empty() && !rendered.ends_with('/') {
                    rendered.push('/');
                }
                rendered.push_str(&part.to_string_lossy());
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => rendered.push_str("/.."),
            std::path::Component::Prefix(prefix) => {
                rendered.push_str(&prefix.as_os_str().to_string_lossy());
            }
        }
    }
    rendered
}

fn parse_jsonc(content: &str) -> Option<Value> {
    let stripped = strip_json_comments(content);
    let normalized = strip_trailing_commas(stripped.as_str());
    serde_json::from_str(normalized.as_str()).ok()
}

fn strip_json_comments(content: &str) -> String {
    let mut output = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment = false;
    while let Some(ch) = chars.next() {
        if line_comment {
            if ch == '\n' {
                line_comment = false;
                output.push('\n');
            }
            continue;
        }
        if block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                block_comment = false;
            } else if ch == '\n' {
                output.push('\n');
            }
            continue;
        }
        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
            output.push(ch);
            continue;
        }
        if ch == '/' && chars.peek() == Some(&'/') {
            let _ = chars.next();
            line_comment = true;
            continue;
        }
        if ch == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            block_comment = true;
            continue;
        }
        output.push(ch);
    }
    output
}

fn strip_trailing_commas(content: &str) -> String {
    let mut output = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    while let Some(ch) = chars.next() {
        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
            output.push(ch);
            continue;
        }
        if ch == ',' {
            let mut lookahead = chars.clone();
            let next = loop {
                match lookahead.next() {
                    Some(next) if next.is_whitespace() => continue,
                    other => break other,
                }
            };
            if matches!(next, Some('}' | ']')) {
                continue;
            }
        }
        output.push(ch);
    }
    output
}

fn push_u64(buffer: &mut String, value: u64) {
    let rendered = value.to_compact_string();
    buffer.push_str(rendered.as_str());
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::is_stale_session_directory;
    use super::{
        cleanup_stale_session_roots, remove_finished_process_sessions, resolve_corsa_executable,
        session_store_root, session_tsconfig_contents, virtual_file_path,
    };
    use std::{
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };
    use vize_l0::cstr;

    static NEXT_CASE_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn virtual_file_mirrors_the_authored_package_directory() {
        let package = Path::new("/workspace/ui");
        let session = package.join(".vize/patina/session-1-0");
        assert_eq!(
            virtual_file_path(&session, package, "/workspace/ui/src/components/Button.vue",),
            session.join("src/components/Button.vue.patina.ts")
        );
    }

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
                vize_l0::corsa_resolver::platform_suffix()
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

    #[test]
    fn removes_finished_process_session_directories() {
        let root = case_dir("finished-session");
        let _ = std::fs::remove_dir_all(&root);
        let store = session_store_root(&root);
        let live = store.join(format!("session-{}-7", std::process::id()));
        let foreign = store.join("session-9999999999-7");
        let legacy = root
            .join("node_modules")
            .join(".vize")
            .join("patina")
            .join(format!("session-{}-8", std::process::id()));

        std::fs::create_dir_all(&live).unwrap();
        std::fs::create_dir_all(&foreign).unwrap();
        std::fs::create_dir_all(&legacy).unwrap();

        remove_finished_process_sessions(&root);

        assert!(!live.exists());
        assert!(!legacy.exists());
        assert!(foreign.exists());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn session_tsconfig_honors_extended_paths() {
        let root = case_dir("tsconfig-paths");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("base.json"),
            r#"{ "compilerOptions": { "paths": { "@app/*": ["src/*"] } } }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("tsconfig.json"),
            r#"{ "extends": "./base.json", "compilerOptions": { "baseUrl": "." } }"#,
        )
        .unwrap();

        let value: serde_json::Value = serde_json::from_str(&session_tsconfig_contents(
            &root,
            &root.join("src/App.vue").to_string_lossy(),
        ))
        .unwrap();
        let canonical = std::fs::canonicalize(&root).unwrap();
        let options = &value["compilerOptions"];
        assert_eq!(
            Path::new(options["baseUrl"].as_str().unwrap()),
            canonical.as_path()
        );
        assert_eq!(
            Path::new(options["paths"]["@app/*"][0].as_str().unwrap()),
            canonical.join("src/*")
        );
        assert_eq!(value["include"][0], "**/*.patina.ts");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn session_tsconfig_includes_ambient_declarations_from_include() {
        let root = case_dir("tsconfig-ambient");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("src/env.d.ts"),
            "declare module \"vue\" { interface ComponentCustomProperties { $t: (key: string) => string } }\n",
        )
        .unwrap();
        std::fs::write(root.join("src/skip.ts"), "export const skip = 1\n").unwrap();
        std::fs::write(
            root.join("src/hidden.d.ts"),
            "declare module \"hidden\" { export const hidden: string }\n",
        )
        .unwrap();
        std::fs::write(
            root.join("tsconfig.json"),
            r#"{
              // ambient augmentations only
              "compilerOptions": { "strict": true },
              "include": ["src/**/*.d.ts", "src/**/*.ts"],
              "exclude": ["src/hidden.d.ts"]
            }"#,
        )
        .unwrap();

        let value: serde_json::Value = serde_json::from_str(&session_tsconfig_contents(
            &root,
            &root.join("src/App.vue").to_string_lossy(),
        ))
        .unwrap();
        let files: Vec<_> = value["files"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|entry| entry.as_str())
            .collect();
        let env = slash_display(&std::fs::canonicalize(root.join("src/env.d.ts")).unwrap());
        let hidden = slash_display(&std::fs::canonicalize(root.join("src/hidden.d.ts")).unwrap());
        assert!(files.contains(&env.as_str()), "{files:?}");
        assert!(
            !files.iter().any(|file| file.ends_with("skip.ts")),
            "{files:?}"
        );
        assert!(!files.contains(&hidden.as_str()), "{files:?}");

        let _ = std::fs::remove_dir_all(&root);
    }

    fn slash_display(path: &Path) -> std::string::String {
        path.to_string_lossy().replace('\\', "/")
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
