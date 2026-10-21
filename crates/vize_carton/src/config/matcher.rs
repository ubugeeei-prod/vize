//! Shared ordered glob semantics for lint execution and inspection.

use crate::String;
use globset::{GlobBuilder, GlobMatcher};
use std::path::{Component, Path, PathBuf};

use crate::config::CompilerConfigEntry;

/// The compiler whitespace policy that the formatter must preserve for each file.
///
/// Entries use the same declaration-order matching as the build command: every
/// matching entry layers over the root policy and the last whitespace override
/// wins. Modes with output-sensitive authored text are kept verbatim by Glyph.
pub struct CompilerWhitespacePlan {
    root_preserves_text: bool,
    scopes: Vec<(LintPlanScope, bool)>,
    cwd: PathBuf,
}

impl CompilerWhitespacePlan {
    pub fn new(root: Option<&str>, entries: &[CompilerConfigEntry], config_dir: &Path) -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let scopes = entries
            .iter()
            .filter_map(|entry| {
                let whitespace = entry.whitespace.as_deref()?;
                Some((
                    LintPlanScope::new(
                        entry.base_path.as_deref(),
                        entry.files.as_deref(),
                        &entry.ignores,
                        config_dir,
                        &cwd,
                    ),
                    preserves_authored_text(Some(whitespace)),
                ))
            })
            .collect();
        Self {
            root_preserves_text: preserves_authored_text(root),
            scopes,
            cwd,
        }
    }

    pub fn preserves_text_for(&self, path: &Path) -> bool {
        let path = absolute_path(path, &self.cwd);
        let mut preserve = self.root_preserves_text;
        for (scope, scoped_preserve) in &self.scopes {
            if scope.matches(&path) {
                preserve = *scoped_preserve;
            }
        }
        preserve
    }

    pub fn root_preserves_text(&self) -> bool {
        self.root_preserves_text
    }
}

impl Default for CompilerWhitespacePlan {
    fn default() -> Self {
        Self::new(None, &[], Path::new("."))
    }
}

/// True when formatting authored template text can change compiler output.
pub fn preserves_authored_text(whitespace: Option<&str>) -> bool {
    matches!(
        whitespace,
        Some("preserve" | "vue2-preserve" | "vue2-line-breaks")
    )
}

/// Ordered, escaped ignore sequences for host-projected project settings.
///
/// Patterns with a common base retain their declaration order;
/// groups with different bases form independent ignore scopes.
pub struct ProjectIgnoreSet {
    scopes: Vec<LintPlanScope>,
    cwd: PathBuf,
}

impl ProjectIgnoreSet {
    pub fn new(ignores: &[crate::config::ConfigEntryIgnore], config_dir: &Path) -> Option<Self> {
        if ignores.is_empty() {
            return None;
        }
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let config_dir = absolute_path(config_dir, &cwd);
        let mut groups: Vec<(Option<String>, Vec<String>)> = Vec::new();
        for ignore in ignores {
            let mut base = resolve_base_dir(ignore.base_path.as_deref(), &config_dir);
            let (sign, pattern) = ignore
                .pattern
                .strip_prefix('!')
                .map_or(("", ignore.pattern.as_str()), |pattern| ("!", pattern));
            // An authored absolute pattern has the same matching meaning as
            // before projection. Rebase it without interpreting glob syntax;
            // an in-project absolute negative stays in the preceding sequence.
            let path = Path::new(pattern);
            let pattern = if path.is_absolute() {
                let relative = if let Ok(relative) = path.strip_prefix(&base) {
                    relative
                } else {
                    base = path.ancestors().last().unwrap_or(path).to_path_buf();
                    path.strip_prefix(&base).unwrap_or(path)
                };
                normalize_path(relative)
            } else {
                String::from(pattern)
            };
            let base = Some(normalize_path(&base));
            let group_index = groups
                .iter()
                .position(|(existing, _)| *existing == base)
                .unwrap_or_else(|| {
                    groups.push((base, Vec::new()));
                    groups.len() - 1
                });
            let Some((_, patterns)) = groups.get_mut(group_index) else {
                continue;
            };
            patterns.push(crate::cstr!("{sign}{pattern}"));
            // Preserve the existing nested dependency ignore expansion without
            // changing authored negation or escaped metacharacters.
            let suffix = "node_modules/**";
            if let Some(prefix) = pattern.strip_suffix(suffix)
                && !pattern.ends_with("**/node_modules/**")
            {
                patterns.push(crate::cstr!("{sign}{prefix}**/{suffix}"));
            }
        }
        Some(Self {
            scopes: groups
                .into_iter()
                .map(|(base, patterns)| {
                    LintPlanScope::new(base.as_deref(), None, &patterns, &config_dir, &cwd)
                })
                .collect(),
            cwd,
        })
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        let absolute = absolute_path(path, &self.cwd);
        self.scopes.iter().any(|scope| scope.ignores(&absolute))
    }
}

pub struct LintPlanScope {
    base_dir: PathBuf,
    files: Option<GlobSequence>,
    ignores: GlobSequence,
}

pub struct GlobSequence {
    steps: Vec<GlobStep>,
    has_steps: bool,
    has_positive_source: bool,
}

struct GlobStep {
    negated: bool,
    matcher: GlobMatcher,
}

impl LintPlanScope {
    pub fn new(
        base_path: Option<&str>,
        files: Option<&[String]>,
        ignores: &[String],
        config_dir: &Path,
        cwd: &Path,
    ) -> Self {
        Self {
            base_dir: resolve_base_dir(base_path, &absolute_path(config_dir, cwd)),
            files: files.map(GlobSequence::new),
            ignores: GlobSequence::new(ignores),
        }
    }

    pub fn matches(&self, file: &Path) -> bool {
        let Some(relative) = self.relative(file) else {
            return false;
        };
        self.files
            .as_ref()
            .is_none_or(|patterns| patterns.matches_files(relative.as_str()))
            && !self.ignores.matches_ignore(relative.as_str())
    }

    pub fn ignores(&self, file: &Path) -> bool {
        self.relative(file)
            .is_some_and(|relative| self.ignores.matches_ignore(relative.as_str()))
    }

    fn relative(&self, file: &Path) -> Option<String> {
        file.strip_prefix(&self.base_dir).ok().map(normalize_path)
    }
}

impl GlobSequence {
    pub fn new(patterns: &[String]) -> Self {
        let steps = patterns
            .iter()
            .filter_map(|source| {
                // `\` stays a path separator for Windows patterns (`src\**\*.vue`).
                // Once a pattern uses `/`, `\`, `[`, `*`, `?`, `{`, and `}` are escapes.
                let slashed = normalize_entry_glob(source);
                let (negated, rest) = slashed
                    .strip_prefix('!')
                    .map_or((false, slashed.as_str()), |pattern| (true, pattern));
                let pattern = strip_leading_current_dir(rest);
                if pattern.is_empty() {
                    return None;
                }
                match GlobBuilder::new(pattern)
                    .literal_separator(true)
                    .backslash_escape(false)
                    .build()
                {
                    Ok(glob) => Some(GlobStep {
                        negated,
                        matcher: glob.compile_matcher(),
                    }),
                    Err(error) => {
                        eprintln!("[vize] Ignoring invalid entry glob '{source}': {error}");
                        None
                    }
                }
            })
            .collect::<Vec<_>>();
        let has_positive_source = steps.iter().any(|step| !step.negated);
        Self {
            has_steps: !steps.is_empty(),
            steps,
            has_positive_source,
        }
    }

    pub fn matches_files(&self, file: &str) -> bool {
        if !self.has_steps {
            return false;
        }
        let mut matched = !self.has_positive_source;
        for step in &self.steps {
            if matches_file_or_parent(&step.matcher, file) {
                matched = !step.negated;
            }
        }
        matched
    }

    fn matches_ignore(&self, file: &str) -> bool {
        let mut ignored = false;
        for step in &self.steps {
            if matches_file_or_parent(&step.matcher, file) {
                ignored = !step.negated;
            }
        }
        ignored
    }
}

fn matches_file_or_parent(matcher: &GlobMatcher, file: &str) -> bool {
    if matcher.is_match(file) {
        return true;
    }
    let mut parent = file;
    while let Some((head, _)) = parent.rsplit_once('/') {
        if head.is_empty() {
            break;
        }
        if matcher.is_match(head) {
            return true;
        }
        parent = head;
    }
    false
}

fn resolve_base_dir(base_path: Option<&str>, config_dir: &Path) -> PathBuf {
    let Some(base_path) = base_path.filter(|path| !path.is_empty()) else {
        return config_dir.to_path_buf();
    };
    absolute_path(Path::new(&base_path.replace('\\', "/")), config_dir)
}

pub fn absolute_path(path: &Path, cwd: &Path) -> PathBuf {
    let normalized = PathBuf::from(path.to_string_lossy().replace('\\', "/"));
    let absolute = if normalized.is_absolute() {
        normalized
    } else {
        cwd.join(normalized)
    };
    normalize_lexically(&absolute)
}

fn normalize_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() && !path.is_absolute() {
                    normalized.push(component.as_os_str());
                }
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

fn normalize_entry_glob(source: &str) -> String {
    let escape_metacharacters = source.contains('/');
    let mut normalized = String::with_capacity(source.len());
    let mut characters = source.chars().peekable();
    while let Some(character) = characters.next() {
        if character != '\\' {
            normalized.push(character);
            continue;
        }
        match characters.peek().copied() {
            Some(next) if escape_metacharacters && is_glob_metacharacter(next) => {
                characters.next();
                push_literal_metacharacter(&mut normalized, next);
            }
            _ => normalized.push('/'),
        }
    }
    normalized
}

fn is_glob_metacharacter(character: char) -> bool {
    matches!(character, '[' | ']' | '*' | '?' | '{' | '}')
}

fn push_literal_metacharacter(pattern: &mut String, character: char) {
    match character {
        '[' => pattern.push_str("[[]"),
        ']' => pattern.push_str("[]]"),
        '*' => pattern.push_str("[*]"),
        '?' => pattern.push_str("[?]"),
        '{' => pattern.push_str("[{]"),
        '}' => pattern.push_str("[}]"),
        _ => pattern.push(character),
    }
}

fn strip_leading_current_dir(pattern: &str) -> &str {
    let mut stripped = pattern;
    while let Some(rest) = stripped.strip_prefix("./") {
        stripped = rest;
    }
    stripped
}

pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").into()
}

#[cfg(test)]
mod tests {
    use super::{CompilerWhitespacePlan, LintPlanScope, ProjectIgnoreSet};
    use crate::config::CompilerConfigEntry;
    use std::path::Path;

    fn scope(files: Option<&[&str]>, ignores: &[&str]) -> LintPlanScope {
        let files: Option<Vec<crate::String>> =
            files.map(|patterns| patterns.iter().copied().map(Into::into).collect());
        let ignores = ignores
            .iter()
            .copied()
            .map(Into::into)
            .collect::<Vec<crate::String>>();
        let root = Path::new("/");
        LintPlanScope::new(None, files.as_deref(), &ignores, root, root)
    }

    #[test]
    fn escaped_metacharacters_match_literals_in_files_and_ignores() {
        let root = Path::new("/");
        for (pattern, matched, unmatched) in [
            (
                "pages/users/\\[id\\].vue",
                "/pages/users/[id].vue",
                "/pages/users/id.vue",
            ),
            (
                "pages/users/\\].vue",
                "/pages/users/].vue",
                "/pages/users/x.vue",
            ),
            (
                "pages/users/\\*.vue",
                "/pages/users/*.vue",
                "/pages/users/id.vue",
            ),
            (
                "pages/users/\\?.vue",
                "/pages/users/?.vue",
                "/pages/users/a.vue",
            ),
            (
                "pages/users/\\{id\\}.vue",
                "/pages/users/{id}.vue",
                "/pages/users/id.vue",
            ),
            (
                "pages/users/[[]id].vue",
                "/pages/users/[id].vue",
                "/pages/users/id.vue",
            ),
        ] {
            let files = scope(Some(&[pattern]), &[]);
            assert!(files.matches(root.join(matched).as_path()), "{pattern}");
            assert!(!files.matches(root.join(unmatched).as_path()), "{pattern}");
            let ignored = scope(None, &[pattern]);
            assert!(ignored.ignores(root.join(matched).as_path()), "{pattern}");
            assert!(
                !ignored.ignores(root.join(unmatched).as_path()),
                "{pattern}"
            );
        }

        let windows = scope(Some(&[r"src\**\*.vue"]), &[]);
        assert!(windows.matches(Path::new("/src/components/App.vue")));
        assert!(!windows.matches(Path::new("/packages/App.vue")));

        let negated = scope(Some(&[r"src/**/*.vue", r"!.\src\generated\**"]), &[]);
        assert!(negated.matches(Path::new("/src/App.vue")));
        assert!(!negated.matches(Path::new("/src/generated/drop.vue")));
    }

    #[test]
    fn project_ignore_sequences_preserve_root_negation_and_literal_metacharacters() {
        let project = tempfile::tempdir().unwrap();
        let root = project.path().join("app");
        let absolute = root.join("absolute/**");
        let outside = project.path().join("outside/**");
        let ignores = [
            "generated/**",
            outside.to_str().unwrap(),
            "!generated/KeepItem.vue",
            r"src/\[id\].vue",
            "node_modules/**",
            "!node_modules/keep.vue",
            absolute.to_str().unwrap(),
            "!absolute/KeepItem.vue",
        ]
        .into_iter()
        .map(|pattern| crate::config::ConfigEntryIgnore {
            base_path: Some(root.to_string_lossy().as_ref().into()),
            pattern: pattern.into(),
        })
        .collect::<Vec<_>>();
        let set = ProjectIgnoreSet::new(&ignores, project.path()).unwrap();
        let actual = [
            "app/generated/DropItem.vue",
            "app/generated/KeepItem.vue",
            "app/src/[id].vue",
            "app/src/id.vue",
            "generated/DropItem.vue",
            "app/src/node_modules/drop.vue",
            "app/node_modules/keep.vue",
            "app/absolute/DropItem.vue",
            "app/absolute/KeepItem.vue",
            "outside/DropItem.vue",
        ]
        .into_iter()
        .map(|name| (name, set.is_ignored(&project.path().join(name))))
        .collect::<Vec<_>>();
        assert_eq!(
            actual,
            vec![
                ("app/generated/DropItem.vue", true),
                ("app/generated/KeepItem.vue", false),
                ("app/src/[id].vue", true),
                ("app/src/id.vue", false),
                ("generated/DropItem.vue", false),
                ("app/src/node_modules/drop.vue", true),
                ("app/node_modules/keep.vue", false),
                ("app/absolute/DropItem.vue", true),
                ("app/absolute/KeepItem.vue", false),
                ("outside/DropItem.vue", true),
            ]
        );
    }

    #[test]
    fn compiler_whitespace_plan_matches_build_entry_order_per_file() {
        let project = tempfile::tempdir().unwrap();
        let entry = |files: &str, whitespace: &str| CompilerConfigEntry {
            base_path: None,
            files: Some(vec![files.into()]),
            ignores: Vec::new(),
            template_syntax: None,
            whitespace: Some(whitespace.into()),
            vapor: None,
            custom_elements: Vec::new(),
        };
        let plan = CompilerWhitespacePlan::new(
            Some("preserve"),
            &[
                entry("src/**", "condense"),
                entry("src/legacy/**", "vue2-preserve"),
                entry("src/line-breaks/**", "vue2-line-breaks"),
            ],
            project.path(),
        );

        assert!(plan.root_preserves_text());
        assert!(!plan.preserves_text_for(&project.path().join("src/App.vue")));
        assert!(plan.preserves_text_for(&project.path().join("src/legacy/App.vue")));
        assert!(plan.preserves_text_for(&project.path().join("src/line-breaks/App.vue")));
        assert!(plan.preserves_text_for(&project.path().join("outside/App.vue")));
    }
}
