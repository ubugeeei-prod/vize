//! Shared ordered glob semantics for lint execution and inspection.

use crate::String;
use globset::{GlobBuilder, GlobMatcher};
use std::path::{Component, Path, PathBuf};

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
mod tests;
