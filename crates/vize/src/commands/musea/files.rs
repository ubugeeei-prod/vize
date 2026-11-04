//! Story discovery for `vize musea migrate`.
//!
//! `commands::fmt` is compiled only with the glyph feature. Migrate keeps its
//! own walker so `*.stories.{tsx,ts,jsx,js}` files are found when glyph is off.

use glob::{MatchOptions, Pattern};
use ignore::WalkBuilder;
use std::path::{Component, Path, PathBuf};

pub(super) fn collect_files(patterns: &[impl AsRef<str>]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if vize_carton::path::is_git_metadata_path(&cwd) {
        return files;
    }

    for pattern in patterns {
        collect_pattern(pattern.as_ref(), &cwd, &mut files);
    }
    files.sort();
    files.dedup();
    files
}

pub(super) fn is_story_file(path: &Path) -> bool {
    let name = match path.file_name().and_then(|name| name.to_str()) {
        Some(name) => name,
        None => return false,
    };
    matches!(
        name.rsplit_once('.'),
        Some((stem, "tsx" | "ts" | "jsx" | "js")) if stem.ends_with(".stories")
    )
}

pub(super) fn is_default_ignored_story_path(path: &Path) -> bool {
    path.components().any(|component| {
        component.as_os_str().to_str().is_some_and(|name| {
            matches!(
                name,
                "node_modules" | ".nuxt" | ".output" | "dist" | "storybook-static"
            )
        })
    })
}

fn collect_pattern(input: &str, cwd: &Path, files: &mut Vec<PathBuf>) {
    let normalized = normalize_pattern(input);
    if normalized.is_empty() {
        collect_walked(Path::new("."), None, true, files);
        return;
    }

    let path = PathBuf::from(normalized.as_str());
    if vize_carton::path::is_git_metadata_path(&path) {
        return;
    }
    // Bracket routes such as `[id].stories.tsx` are literal files, not classes.
    if path.is_file() {
        push_story(&path, files);
        return;
    }
    if !contains_glob_char(normalized.as_str()) {
        if path.is_dir() {
            collect_walked(&path, None, respect_ignores(&path), files);
        }
        return;
    }

    let Some(glob) = StoryGlob::compile(normalized.as_str(), cwd) else {
        return;
    };
    let root = walk_root(normalized.as_str());
    collect_walked(&root, Some(&glob), respect_ignores(&root), files);
}

fn collect_walked(
    root: &Path,
    pattern: Option<&StoryGlob>,
    respect_ignores: bool,
    files: &mut Vec<PathBuf>,
) {
    if vize_carton::path::is_git_metadata_path(root) || is_default_ignored_story_path(root) {
        return;
    }

    let walker = WalkBuilder::new(root)
        .hidden(false)
        .ignore(respect_ignores)
        .git_ignore(respect_ignores)
        .git_global(respect_ignores)
        .git_exclude(respect_ignores)
        .parents(respect_ignores)
        .require_git(false)
        .filter_entry(|entry| entry.depth() == 0 || keep_walk_entry(entry.path()))
        .build();

    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if pattern.is_some_and(|glob| !glob.matches(path)) {
            continue;
        }
        push_story(path, files);
    }
}

fn push_story(path: &Path, files: &mut Vec<PathBuf>) {
    if vize_carton::path::is_git_metadata_path(path)
        || is_default_ignored_story_path(path)
        || !is_story_file(path)
    {
        return;
    }
    files.push(normalize_collected_path(path));
}

fn keep_walk_entry(path: &Path) -> bool {
    !vize_carton::path::is_git_metadata_path(path) && !is_default_ignored_story_path(path)
}

/// Project-wide globs honor ignore files. Explicit paths and `..` patterns do not.
fn respect_ignores(root: &Path) -> bool {
    if root
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return false;
    }
    !root.is_absolute() || std::env::current_dir().is_ok_and(|cwd| root.starts_with(cwd))
}

struct StoryGlob {
    pattern: Pattern,
    cwd: PathBuf,
    absolute: bool,
}

impl StoryGlob {
    fn compile(pattern: &str, cwd: &Path) -> Option<Self> {
        let absolute = Path::new(pattern).is_absolute();
        Pattern::new(pattern).ok().map(|pattern| Self {
            pattern,
            cwd: cwd.to_path_buf(),
            absolute,
        })
    }

    fn matches(&self, path: &Path) -> bool {
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
            .matches_with(candidate.as_str(), story_glob_options())
    }
}

fn walk_root(pattern: &str) -> PathBuf {
    let mut root = PathBuf::new();
    for component in Path::new(pattern).components() {
        let text = component.as_os_str().to_string_lossy();
        if contains_glob_char(&text) {
            break;
        }
        root.push(component);
    }
    if root.as_os_str().is_empty() {
        root.push(".");
    }
    root
}

fn normalize_pattern(pattern: &str) -> vize_l0::String {
    let mut normalized: vize_l0::String = pattern.replace('\\', "/").into();
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped.into();
    }
    normalized
}

fn normalize_path(path: &Path) -> vize_l0::String {
    path.to_string_lossy().replace('\\', "/").into()
}

fn normalize_collected_path(path: &Path) -> PathBuf {
    path.strip_prefix(".").unwrap_or(path).to_path_buf()
}

fn contains_glob_char(pattern: &str) -> bool {
    pattern.contains(['*', '?', '['])
}

fn story_glob_options() -> MatchOptions {
    MatchOptions {
        case_sensitive: !cfg!(windows),
        require_literal_separator: true,
        require_literal_leading_dot: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{StoryGlob, collect_files};
    use std::fs;
    use std::path::{Path, PathBuf};

    fn write_file(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, "export default {}\n").unwrap();
    }

    fn prefixed(root: &Path, suffix: &str) -> vize_l0::String {
        let mut pattern = vize_l0::String::new("");
        pattern.push_str(&root.to_string_lossy());
        pattern.push_str(suffix);
        pattern
    }

    #[test]
    fn collects_story_globs_directories_and_literal_cli_paths() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let kept = [
            "src/Button.stories.tsx",
            "src/Button.stories.ts",
            "src/Button.stories.jsx",
            "nested/Card.stories.js",
            ".storybook/Widget.stories.tsx",
        ];
        let skipped = [
            "src/Button.tsx",
            "src/notes.ts",
            "src/Button.vue",
            "node_modules/pkg/Hidden.stories.tsx",
            ".nuxt/Gen.stories.ts",
            ".output/Out.stories.jsx",
            "dist/Dist.stories.js",
            "storybook-static/Static.stories.tsx",
            ".git/Git.stories.ts",
        ];
        for relative in kept.iter().chain(skipped.iter()) {
            write_file(&root.join(relative));
        }
        let bracket = root.join("src/[id].stories.tsx");
        write_file(&bracket);

        let patterns = [
            prefixed(root, "/**/*.stories.tsx"),
            prefixed(root, "/**/*.stories.ts"),
            prefixed(root, "/**/*.stories.jsx"),
            prefixed(root, "/**/*.stories.js"),
        ];
        let mut expected: Vec<PathBuf> = kept.iter().map(|relative| root.join(relative)).collect();
        expected.push(bracket.clone());
        expected.sort();

        assert_eq!(collect_files(&patterns), expected);
        assert_eq!(collect_files(&[prefixed(root, "")]), expected);
        assert_eq!(
            collect_files(&[prefixed(root, "/**/*.stories.ts")]),
            vec![root.join("src/Button.stories.ts")]
        );
        assert_eq!(collect_files(&[prefixed(&bracket, "")]), vec![bracket]);
    }

    #[test]
    fn relative_globs_match_only_story_suffixes() {
        let cwd = PathBuf::from("/work");
        let stories = StoryGlob::compile("**/*.stories.tsx", &cwd).unwrap();
        assert!(stories.matches(Path::new("Button.stories.tsx")));
        assert!(stories.matches(Path::new("src/Button.stories.tsx")));
        assert!(stories.matches(Path::new("./nested/Button.stories.tsx")));
        assert!(!stories.matches(Path::new("Button.stories.ts")));
        assert!(!stories.matches(Path::new("src/Button.tsx")));
        assert!(!stories.matches(Path::new("src/Button.stories.tsx.bak")));

        let typescript = StoryGlob::compile("/work/**/*.stories.ts", &cwd).unwrap();
        assert!(typescript.matches(Path::new("/work/src/Card.stories.ts")));
        assert!(typescript.matches(Path::new("src/Card.stories.ts")));
        assert!(!typescript.matches(Path::new("/work/src/Card.stories.tsx")));
    }
}
