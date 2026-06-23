use super::super::paths::find_node_modules_with_vue;
use serde_json::json;
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use vize_carton::{String, ToCompactString, cstr};

const SESSION_META_FILE: &str = "meta.json";
const SESSION_SCHEMA_VERSION: u32 = 1;
pub(super) const STALE_SESSION_SECONDS: u64 = 24 * 60 * 60;

pub(super) fn install_node_modules_link(project_root: Option<&Path>, temp_dir_path: &Path) {
    let node_modules_path = project_root.and_then(find_node_modules_with_vue);
    if let Some(ref node_modules_path) = node_modules_path {
        let symlink_target = temp_dir_path.join("node_modules");
        let _ = std::fs::remove_file(&symlink_target);
        #[cfg(unix)]
        {
            let _ = std::os::unix::fs::symlink(node_modules_path, &symlink_target);
        }
        #[cfg(windows)]
        {
            let _ = std::os::windows::fs::symlink_dir(node_modules_path, &symlink_target);
        }
    }
}

pub(super) fn write_session_meta(temp_dir_path: &Path) -> Result<(), String> {
    let created_at = now_unix_seconds();
    let content = json!({
        "schemaVersion": SESSION_SCHEMA_VERSION,
        "tool": "vize-corsa",
        "pid": std::process::id(),
        "createdAtUnix": created_at
    });
    std::fs::write(
        temp_dir_path.join(SESSION_META_FILE),
        serde_json::to_string_pretty(&content)
            .map_err(|e| cstr!("Failed to serialize Corsa session metadata: {e}"))?,
    )
    .map_err(|e| cstr!("Failed to write Corsa session metadata: {e}"))
}

pub(super) fn cleanup_stale_sessions(base_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(base_dir) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() || !should_remove_session_dir(&path) {
            continue;
        }
        let _ = std::fs::remove_dir_all(path);
    }
}

fn should_remove_session_dir(path: &Path) -> bool {
    let meta_path = path.join(SESSION_META_FILE);
    let Ok(content) = std::fs::read_to_string(meta_path) else {
        return session_dir_is_stale(path);
    };
    let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) else {
        return true;
    };

    if meta
        .get("schemaVersion")
        .and_then(serde_json::Value::as_u64)
        != Some(SESSION_SCHEMA_VERSION as u64)
    {
        return true;
    }

    let Some(pid) = meta
        .get("pid")
        .and_then(serde_json::Value::as_u64)
        .and_then(|pid| u32::try_from(pid).ok())
    else {
        return true;
    };

    if !process_is_alive(pid) {
        return true;
    }

    meta.get("createdAtUnix")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|created_at| {
            now_unix_seconds().saturating_sub(created_at) > STALE_SESSION_SECONDS
        })
}

fn session_dir_is_stale(path: &Path) -> bool {
    std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age.as_secs() > STALE_SESSION_SECONDS)
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(unix)]
fn process_is_alive(pid: u32) -> bool {
    if pid == std::process::id() {
        return true;
    }
    // SAFETY: `kill(pid, 0)` does not send a signal; it only checks whether the
    // process exists and is visible to the current user.
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

#[cfg(not(unix))]
fn process_is_alive(_pid: u32) -> bool {
    true
}

/// Write a minimal `tsconfig.json` that keeps the native checker in strict mode.
pub(super) fn write_temp_tsconfig(temp_dir_path: &Path) -> Result<(), String> {
    let tsconfig_content = json!({
        "compilerOptions": {
            "target": "ES2022",
            "module": "ESNext",
            "moduleResolution": "bundler",
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
