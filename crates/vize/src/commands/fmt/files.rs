use glob::{MatchOptions, Pattern};
use ignore::WalkBuilder;
use std::path::{Component, Path, PathBuf};

use super::ignores::FmtIgnoreSet;
use super::patterns::is_format_extension;

const NODE_MODULES_DIR: &str = "node_modules";

pub(crate) fn collect_files(
    patterns: &[impl AsRef<str>],
    ignore_set: Option<&FmtIgnoreSet>,
) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if vize_carton::path::is_git_metadata_path(&cwd) {
        return files;
    }

    for pattern in patterns {
        let normalized = normalize_fmt_pattern(pattern.as_ref());
        // File-based routes (`pages/[id].vue`) contain `[` but are literal files.
        // A glob would treat the brackets as a character class and skip them.
        let literal = PathBuf::from(normalized.as_str());
        if vize_carton::path::is_git_metadata_path(&literal) {
            continue;
        }
        if literal.is_file() {
            if should_include_format_file(&literal, ignore_set) {
                files.push(literal);
            }
            continue;
        }
        if contains_glob_char(&normalized) {
            if let Some(pattern) = FmtPattern::new(&normalized, &cwd) {
                collect_walked_files(&pattern, &normalized, ignore_set, &mut files);
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
    input: &str,
    ignore_set: Option<&FmtIgnoreSet>,
    files: &mut Vec<PathBuf>,
) {
    let mut root = PathBuf::new();
    for component in Path::new(input).components() {
        if contains_glob_char(&component.as_os_str().to_string_lossy()) {
            break;
        }
        root.push(component);
    }
    if root.as_os_str().is_empty() {
        root.push(".");
    }
    let respect_ignores = should_walk_with_gitignore(input);
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .ignore(respect_ignores)
        .git_ignore(respect_ignores)
        .git_global(respect_ignores)
        .git_exclude(respect_ignores)
        .require_git(false)
        .filter_entry(|entry| {
            !vize_carton::path::is_git_metadata_path(entry.path())
                && !is_dependency_path(entry.path())
        })
        .build();

    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        if pattern.matches(path) && should_include_format_file(path, ignore_set) {
            files.push(path.strip_prefix(".").unwrap_or(path).to_path_buf());
        }
    }
}

fn should_include_format_file(path: &Path, ignore_set: Option<&FmtIgnoreSet>) -> bool {
    !vize_carton::path::is_git_metadata_path(path)
        && path.is_file()
        && is_format_target(path)
        && !is_dependency_path(path)
        && !ignore_set.is_some_and(|ignore_set| ignore_set.is_ignored(path))
}

#[inline]
fn should_walk_with_gitignore(pattern: &str) -> bool {
    // `normalize_fmt_pattern` strips leading `./`, but accept it defensively.
    let bare = pattern.strip_prefix("./").unwrap_or(pattern);
    let path = Path::new(bare);
    // The default globs are rooted at the Vite project root, so an absolute
    // glob inside the working directory honors ignore files like its relative
    // form; build output and generated `.void`/`.nuxt` stay out.
    contains_glob_char(bare)
        && (!path.is_absolute() || std::env::current_dir().is_ok_and(|cwd| path.starts_with(cwd)))
        && !path
            .components()
            .any(|component| component == Component::ParentDir)
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

fn normalize_fmt_pattern(pattern: &str) -> vize_l0::String {
    let mut normalized: vize_l0::String = pattern.replace('\\', "/").into();
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
        .is_some_and(is_format_extension)
}

#[inline]
fn normalize_path(path: &Path) -> vize_l0::String {
    path.to_string_lossy().replace('\\', "/").into()
}

#[inline]
fn contains_glob_char(pattern: &str) -> bool {
    pattern.contains(['*', '?', '['])
}

fn is_dependency_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == NODE_MODULES_DIR)
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
#[path = "files/contract_tests.rs"]
mod tests;

#[cfg(test)]
mod files_tests {
    use super::collect_files;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };
    use vize_l0::{String, ToCompactString};

    #[test]
    fn formatting_never_selects_git_metadata_even_when_explicitly_named() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let source = root.join(".github/App.vue");
        let metadata = root.join(".git/worktrees/cache/App.vue");
        for file in [&source, &metadata] {
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, "<template><div/></template>").unwrap();
        }
        for input in [
            root.display().to_string(),
            root.join("**/*.vue").display().to_string(),
        ] {
            assert_eq!(collect_files(&[input], None), vec![source.clone()]);
        }
        for input in [
            metadata.display().to_string(),
            root.join(".git").display().to_string(),
            root.join("**/.git/**/*.vue").display().to_string(),
        ] {
            assert!(collect_files(&[input], None).is_empty());
        }
    }

    #[test]
    fn collect_files_ignores_supported_extension_directories() {
        let root = unique_case_dir("format-extension-directories");
        let src = root.join("src");
        let component_dir = src.join("Directory.vue");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&component_dir).unwrap();
        fs::write(src.join("App.vue"), "<template><div/></template>").unwrap();
        fs::write(
            component_dir.join("Nested.vue"),
            "<template><div/></template>",
        )
        .unwrap();

        let pattern = root.to_string_lossy().into_owned();
        let glob_pattern = root.join("**/*.vue").to_string_lossy().into_owned();
        let files_from_dir = collect_files(&[pattern], None);
        let files_from_glob = collect_files(&[glob_pattern], None);
        let _ = fs::remove_dir_all(&root);

        let mut expected = vec![component_dir.join("Nested.vue"), src.join("App.vue")];
        expected.sort();
        assert_eq!(files_from_dir, expected);
        assert_eq!(files_from_glob, expected);
    }

    #[test]
    fn explicit_relative_glob_respects_nested_gitignore() {
        let cwd = std::env::current_dir().unwrap();
        let relative_root = PathBuf::from("tests")
            .join(unique_case_dir("explicit-glob-ignore").file_name().unwrap());
        let root = cwd.join(&relative_root);
        let source = root.join("apps/web/src/App.vue");
        let dependency = root.join("apps/web/node_modules/dependency/Hidden.vue");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::create_dir_all(dependency.parent().unwrap()).unwrap();
        fs::write(&source, "<template><main /></template>").unwrap();
        fs::write(&dependency, "<template><aside /></template>").unwrap();
        fs::write(root.join("apps/web/.gitignore"), "node_modules/\n").unwrap();

        let pattern = relative_root
            .join("apps/**/*.vue")
            .to_string_lossy()
            .into_owned();
        let files = collect_files(&[pattern], None);
        let _ = fs::remove_dir_all(&root);

        assert_eq!(files, vec![relative_root.join("apps/web/src/App.vue")]);
    }

    #[test]
    fn default_absolute_glob_under_cwd_respects_gitignore() {
        // `vize fmt` in a Vite project roots its default globs at the project
        // root; ignored build output must not be reformatted (Void's dist/.void).
        let cwd = std::env::current_dir().unwrap();
        let root = cwd
            .join("tests")
            .join(unique_case_dir("absolute-glob-ignore").file_name().unwrap());
        let source = root.join("pages/index.vue");
        let _ = fs::remove_dir_all(&root);
        for file in [
            &source,
            &root.join("dist/index.vue"),
            &root.join(".void/entry.vue"),
        ] {
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, "<template><main /></template>").unwrap();
        }
        fs::write(root.join(".gitignore"), "dist\n.void\n").unwrap();

        let files = collect_files(
            &[root.join("**/*.vue").to_string_lossy().into_owned()],
            None,
        );
        let _ = fs::remove_dir_all(&root);

        assert_eq!(files, vec![source]);
    }

    #[test]
    fn relative_recursive_globs_include_dot_directories() {
        let cwd = std::env::current_dir().unwrap();
        let relative_root =
            PathBuf::from("tests").join(unique_case_dir("hidden-glob").file_name().unwrap());
        let root = cwd.join(&relative_root);
        let source = root.join("src/App.vue");
        let hidden = root.join("docs/.vitepress/components/DownloadPage.vue");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::create_dir_all(hidden.parent().unwrap()).unwrap();
        fs::write(&source, "<template><main /></template>").unwrap();
        fs::write(&hidden, "<template><aside /></template>").unwrap();

        let implicit = collect_files(&[relative_root.join("**/*.vue").to_string_lossy()], None);
        let explicit = collect_files(
            &[relative_root
                .join("docs/.vitepress/components/**/*.vue")
                .to_string_lossy()],
            None,
        );
        let _ = fs::remove_dir_all(&root);

        let mut expected = vec![
            relative_root.join("docs/.vitepress/components/DownloadPage.vue"),
            relative_root.join("src/App.vue"),
        ];
        expected.sort();
        assert_eq!(implicit, expected);
        assert_eq!(
            explicit,
            vec![relative_root.join("docs/.vitepress/components/DownloadPage.vue")]
        );
    }

    #[test]
    fn discovery_prunes_root_and_nested_installed_dependencies() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let source = root.join("src/main.ts");
        let dependencies = [
            root.join("node_modules/dep/index.ts"),
            root.join("apps/web/node_modules/dep/index.ts"),
            root.join("node_modules/.vize/corsa-overlay/index.ts"),
        ];
        for file in std::iter::once(&source).chain(&dependencies) {
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, "const   a=1\n").unwrap();
        }
        for pattern in [root.to_path_buf(), root.join("**/*.ts")] {
            assert_eq!(
                collect_files(&[pattern.to_string_lossy()], None),
                vec![source.clone()]
            );
        }
        for dependency in dependencies {
            assert!(collect_files(&[dependency.to_string_lossy()], None).is_empty());
        }
    }

    #[test]
    fn collect_files_includes_existing_bracket_paths() {
        let root = unique_case_dir("bracket-pages");
        let dynamic = root.join("pages/[id].vue");
        let nested = root.join("pages/[id]/index.vue");
        let plain = root.join("pages/plain.vue");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(nested.parent().unwrap()).unwrap();
        let body = "<script setup lang=\"ts\">\nconst a = {b:1}\n</script>\n";
        fs::write(&dynamic, body).unwrap();
        fs::write(&nested, body).unwrap();
        fs::write(&plain, body).unwrap();

        let files = collect_files(
            &[
                dynamic.to_string_lossy().into_owned(),
                plain.to_string_lossy().into_owned(),
                nested.to_string_lossy().into_owned(),
            ],
            None,
        );
        let _ = fs::remove_dir_all(&root);

        let mut expected = vec![dynamic, nested, plain];
        expected.sort();
        assert_eq!(files, expected);
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
