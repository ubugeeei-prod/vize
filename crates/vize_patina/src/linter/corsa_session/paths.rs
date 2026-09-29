#[path = "paths_store.rs"]
mod store;
pub(super) use store::remove_finished_process_sessions;
pub(super) use store::resolve_corsa_executable;
pub(super) use store::resolve_project_root;
pub(super) use store::session_tsconfig_contents;
use store::{
    cleanup_stale_session_roots, push_u64, remove_empty_session_parents, session_store_root,
};

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

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::store::is_stale_session_directory;
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
