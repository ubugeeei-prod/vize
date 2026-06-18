use glob::{MatchOptions, Pattern, glob};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

use super::ignores::FmtIgnoreSet;

const NODE_MODULES_DIR: &str = "node_modules";
const VIZE_CACHE_DIR: &str = ".vize";

#[allow(clippy::disallowed_types)]
pub(super) fn collect_files(
    patterns: &[std::string::String],
    ignore_set: Option<&FmtIgnoreSet>,
) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    for pattern in patterns {
        let normalized = normalize_fmt_pattern(pattern);
        if should_walk_with_gitignore(&normalized) {
            if let Some(pattern) = FmtPattern::new(&normalized, &cwd) {
                collect_walked_files(&pattern, ignore_set, &mut files);
            }
        } else if contains_glob_char(&normalized) {
            if let Ok(paths) = glob(&normalized) {
                for path in paths.flatten() {
                    if should_include_format_file(&path, ignore_set) {
                        files.push(path);
                    }
                }
            }
        } else {
            let path = PathBuf::from(&normalized);
            if path.is_file() && should_include_format_file(&path, ignore_set) {
                files.push(path);
            }
        }
    }

    files.sort();
    files.dedup();

    files
}

fn collect_walked_files(
    pattern: &FmtPattern,
    ignore_set: Option<&FmtIgnoreSet>,
    files: &mut Vec<PathBuf>,
) {
    let walker = WalkBuilder::new(".")
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        if pattern.matches(path) && should_include_format_file(path, ignore_set) {
            files.push(path.to_path_buf());
        }
    }
}

fn should_include_format_file(path: &Path, ignore_set: Option<&FmtIgnoreSet>) -> bool {
    is_format_target(path)
        && !is_generated_path(path)
        && !ignore_set.is_some_and(|ignore_set| ignore_set.is_ignored(path))
}

#[inline]
fn should_walk_with_gitignore(pattern: &str) -> bool {
    matches!(
        pattern,
        "**/*"
            | "./**/*"
            | "**/*.vue"
            | "./**/*.vue"
            | "**/*.jsx"
            | "./**/*.jsx"
            | "**/*.tsx"
            | "./**/*.tsx"
    )
}

pub(super) struct FmtPattern {
    pattern: Pattern,
    cwd: PathBuf,
    absolute: bool,
}

impl FmtPattern {
    pub(super) fn new(pattern: &str, cwd: &Path) -> Option<Self> {
        let normalized = normalize_fmt_pattern(pattern);
        let absolute = Path::new(&normalized).is_absolute();
        Pattern::new(&normalized).ok().map(|pattern| Self {
            pattern,
            cwd: cwd.to_path_buf(),
            absolute,
        })
    }

    pub(super) fn matches(&self, path: &Path) -> bool {
        let candidate = if self.absolute {
            let relative = path.strip_prefix(".").unwrap_or(path);
            let absolute = if relative.is_absolute() {
                relative.to_path_buf()
            } else {
                self.cwd.join(relative)
            };
            normalize_path(&absolute)
        } else {
            normalize_path(path.strip_prefix(".").unwrap_or(path))
        };

        self.pattern
            .matches_with(candidate.as_str(), fmt_glob_match_options())
    }
}

fn normalize_fmt_pattern(pattern: &str) -> vize_carton::String {
    let mut normalized: vize_carton::String = pattern.replace('\\', "/").into();
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped.into();
    }

    if normalized.is_empty() || normalized == "." {
        return "**/*".into();
    }

    if !contains_glob_char(&normalized) && Path::new(&normalized).is_dir() {
        if !normalized.ends_with('/') {
            normalized.push('/');
        }
        normalized.push_str("**/*");
    }

    normalized
}

#[inline]
fn is_format_target(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "vue" | "jsx" | "tsx"))
}

#[inline]
fn normalize_path(path: &Path) -> vize_carton::String {
    path.to_string_lossy().replace('\\', "/").into()
}

#[inline]
fn contains_glob_char(pattern: &str) -> bool {
    pattern.contains(['*', '?', '['])
}

fn is_generated_path(path: &Path) -> bool {
    let mut previous = None;
    for component in path.components() {
        let Some(name) = component.as_os_str().to_str() else {
            previous = None;
            continue;
        };
        if previous == Some(NODE_MODULES_DIR) && name == VIZE_CACHE_DIR {
            return true;
        }
        previous = Some(name);
    }
    false
}

#[inline]
fn fmt_glob_match_options() -> MatchOptions {
    MatchOptions {
        case_sensitive: !cfg!(windows),
        require_literal_separator: true,
        require_literal_leading_dot: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{FmtPattern, collect_files};
    use crate::commands::fmt::ignores::FmtIgnoreSet;
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };
    use vize_carton::{String, ToCompactString};

    #[test]
    fn absolute_glob_only_matches_requested_directory() {
        let root = unique_case_dir("absolute-glob");
        let input_dir = root.join("bench-input");
        let other_dir = root.join("other");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&other_dir).unwrap();
        fs::write(input_dir.join("A.vue"), "<template><div/></template>").unwrap();
        fs::write(other_dir.join("B.vue"), "<template><div/></template>").unwrap();

        let pattern = input_dir.join("*.vue").to_string_lossy().into_owned();
        let files = collect_files(&[pattern], None);
        let _ = fs::remove_dir_all(&root);

        assert_eq!(files, vec![input_dir.join("A.vue")]);
    }

    #[test]
    fn collect_files_skips_generated_vize_workspace() {
        let root = unique_case_dir("generated-vize");
        let src = root.join("src");
        let generated = root.join("node_modules/.vize/corsa-overlay/src");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&generated).unwrap();
        fs::write(src.join("App.vue"), "<template><div/></template>").unwrap();
        fs::write(generated.join("App.vue"), "<template><div/></template>").unwrap();

        let pattern = root.join("**/*.vue").to_string_lossy().into_owned();
        let files = collect_files(&[pattern], None);
        let _ = fs::remove_dir_all(&root);

        assert_eq!(files, vec![src.join("App.vue")]);
    }

    #[test]
    fn collect_files_matches_vue_jsx_and_tsx() {
        let root = unique_case_dir("format-targets");
        let src = root.join("src");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("App.vue"), "<template><div/></template>").unwrap();
        fs::write(src.join("Panel.jsx"), "const Panel=()=> <div />").unwrap();
        fs::write(src.join("Widget.tsx"), "const Widget=()=> <div />").unwrap();
        fs::write(src.join("types.ts"), "export type Widget = {}").unwrap();

        let pattern = root.to_string_lossy().into_owned();
        let files = collect_files(&[pattern], None);
        let _ = fs::remove_dir_all(&root);

        assert_eq!(
            files,
            vec![
                src.join("App.vue"),
                src.join("Panel.jsx"),
                src.join("Widget.tsx")
            ]
        );
    }

    #[test]
    fn collect_files_applies_entry_ignores() {
        let root = unique_case_dir("entry-ignores");
        let src = root.join("src");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("App.vue"), "<template><div /></template>").unwrap();
        fs::write(src.join("Ignored.vue"), "<template><div /></template>").unwrap();

        let ignore_set = FmtIgnoreSet::new(
            &[crate::config::ConfigEntryIgnore {
                base_path: None,
                pattern: "src/Ignored.vue".into(),
            }],
            &root,
        );
        let pattern = root.to_string_lossy().into_owned();
        let files = collect_files(&[pattern], ignore_set.as_ref());
        let explicit = collect_files(
            &[src.join("Ignored.vue").to_string_lossy().into_owned()],
            ignore_set.as_ref(),
        );
        let _ = fs::remove_dir_all(&root);

        assert_eq!(files, vec![src.join("App.vue")]);
        assert!(explicit.is_empty());
    }

    #[test]
    fn relative_glob_does_not_match_every_vue_file() {
        let cwd = std::env::current_dir().unwrap();
        let pattern = FmtPattern::new("bench/__in__/*.vue", &cwd).unwrap();

        assert!(pattern.matches(Path::new("./bench/__in__/Component0000.vue")));
        assert!(!pattern.matches(Path::new("./examples/cli/src/App.vue")));
    }

    fn unique_case_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut dir_name = String::from(name);
        dir_name.push('-');
        let pid = std::process::id().to_compact_string();
        dir_name.push_str(pid.as_str());
        dir_name.push('-');
        let nanos = nanos.to_compact_string();
        dir_name.push_str(nanos.as_str());
        std::env::current_dir()
            .unwrap()
            .join("target")
            .join("vize-tests")
            .join("fmt")
            .join(dir_name.as_str())
    }
}
