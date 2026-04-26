//! Input collection for the `check` runner.
//!
//! The direct runner and socket runner both use these helpers to normalize
//! explicit paths, globs, and directories into stable file lists.

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use vize_carton::FxHashSet;

#[allow(clippy::disallowed_types)]
pub(super) fn collect_check_files(patterns: &[std::string::String]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut seen = FxHashSet::default();

    for pattern in patterns {
        let candidate = PathBuf::from(pattern);
        if candidate.exists() {
            if candidate.is_file() {
                let candidate = normalize_input_path(&candidate);
                if is_supported_check_file(&candidate) && seen.insert(candidate.clone()) {
                    files.push(candidate);
                }
                continue;
            }
            if candidate.is_dir() {
                collect_from_dir(&candidate, &mut files, &mut seen);
                continue;
            }
        }

        let base_dir = base_dir_from_pattern(pattern);
        collect_from_dir(base_dir.as_path(), &mut files, &mut seen);
    }

    files.sort();
    files
}

#[allow(clippy::disallowed_types)]
pub(super) fn collect_vue_files(patterns: &[std::string::String]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut seen = FxHashSet::default();

    for pattern in patterns {
        let candidate = PathBuf::from(pattern);
        if candidate.exists() {
            if candidate.is_file() {
                let candidate = normalize_input_path(&candidate);
                if candidate
                    .extension()
                    .and_then(|extension| extension.to_str())
                    == Some("vue")
                    && seen.insert(candidate.clone())
                {
                    files.push(candidate);
                }
                continue;
            }
            if candidate.is_dir() {
                collect_from_dir_filtered(&candidate, &mut files, &mut seen, true);
                continue;
            }
        }

        let base_dir = base_dir_from_pattern(pattern);
        collect_from_dir_filtered(&base_dir, &mut files, &mut seen, true);
    }

    files.sort();
    files
}

fn collect_from_dir(dir: &Path, files: &mut Vec<PathBuf>, seen: &mut FxHashSet<PathBuf>) {
    collect_from_dir_filtered(dir, files, seen, false);
}

fn collect_from_dir_filtered(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
    vue_only: bool,
) {
    let skip_generated = should_skip_generated_for_root(dir);
    let walker = WalkBuilder::new(dir)
        .standard_filters(true)
        .hidden(true)
        .build_parallel();

    let collected = std::sync::Mutex::new(Vec::<PathBuf>::new());
    walker.run(|| {
        let collected = &collected;
        Box::new(move |entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file()
                    && is_supported_collect_file(path, vue_only)
                    && (!skip_generated || !is_generated_path(path))
                {
                    if let Ok(mut collected) = collected.lock() {
                        collected.push(path.to_path_buf());
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });

    let Ok(collected) = collected.into_inner() else {
        return;
    };
    for path in collected {
        let path = normalize_input_path(&path);
        if seen.insert(path.clone()) {
            files.push(path);
        }
    }
}

fn base_dir_from_pattern(pattern: &str) -> PathBuf {
    let glob_start = pattern.find(['*', '?', '[', '{']).unwrap_or(pattern.len());
    let prefix = &pattern[..glob_start];
    let base = if prefix.is_empty() {
        "."
    } else if let Some(index) = prefix.rfind('/') {
        &prefix[..index]
    } else {
        prefix
    };
    if base.is_empty() {
        PathBuf::from(".")
    } else {
        PathBuf::from(base)
    }
}

fn normalize_input_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn should_skip_generated_for_root(root: &Path) -> bool {
    !root
        .components()
        .any(|component| component.as_os_str().to_str() == Some("__agent_only"))
}

fn is_generated_path(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|name| matches!(name, "__agent_only" | "target"))
    })
}

fn is_supported_collect_file(path: &Path, vue_only: bool) -> bool {
    if vue_only {
        return path.extension().and_then(|extension| extension.to_str()) == Some("vue");
    }
    is_supported_check_file(path)
}

fn is_supported_check_file(path: &Path) -> bool {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
    {
        return true;
    }

    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "vue" | "ts" | "tsx" | "mts" | "cts"))
}

#[cfg(test)]
mod tests {
    use super::{base_dir_from_pattern, collect_check_files, collect_vue_files};
    use std::fs;
    use std::path::{Path, PathBuf};
    use vize_carton::cstr;

    fn unique_case_dir(name: &str) -> PathBuf {
        static NEXT_CASE_ID: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("__agent_only")
            .join("tests")
            .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
    }

    #[test]
    fn base_dir_from_glob_patterns() {
        assert_eq!(
            base_dir_from_pattern("./src/**/*.vue"),
            PathBuf::from("./src")
        );
        assert_eq!(base_dir_from_pattern("."), PathBuf::from("."));
    }

    #[test]
    fn collect_check_files_includes_ts_and_vue_and_dts() {
        let case_dir = unique_case_dir("collect-check");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join("src")).unwrap();
        fs::write(case_dir.join("src/App.vue"), "").unwrap();
        fs::write(case_dir.join("src/main.ts"), "").unwrap();
        fs::write(case_dir.join("src/env.d.ts"), "").unwrap();
        fs::write(case_dir.join("src/skip.js"), "").unwrap();

        let files = collect_check_files(&vec![case_dir.display().to_string()]);

        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|path| path.ends_with("App.vue")));
        assert!(files.iter().any(|path| path.ends_with("main.ts")));
        assert!(files.iter().any(|path| path.ends_with("env.d.ts")));

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn collect_vue_files_stays_vue_only() {
        let case_dir = unique_case_dir("collect-vue");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join("src")).unwrap();
        fs::write(case_dir.join("src/App.vue"), "").unwrap();
        fs::write(case_dir.join("src/main.ts"), "").unwrap();

        let files = collect_vue_files(&vec![case_dir.display().to_string()]);

        assert_eq!(files, vec![case_dir.join("src/App.vue")]);

        let _ = fs::remove_dir_all(&case_dir);
    }
}
