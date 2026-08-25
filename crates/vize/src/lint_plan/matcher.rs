//! Shared ordered glob semantics for lint execution and inspection.

use globset::{GlobBuilder, GlobMatcher};
use std::path::{Component, Path, PathBuf};
use vize_s0::String;

pub(crate) struct LintPlanScope {
    base_dir: PathBuf,
    files: Option<GlobSequence>,
    ignores: GlobSequence,
}

pub(crate) struct GlobSequence {
    steps: Vec<GlobStep>,
    has_steps: bool,
    has_positive_source: bool,
}

struct GlobStep {
    negated: bool,
    matcher: GlobMatcher,
}

impl LintPlanScope {
    pub(crate) fn new(
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

    pub(crate) fn matches(&self, file: &Path) -> bool {
        let Some(relative) = self.relative(file) else {
            return false;
        };
        self.files
            .as_ref()
            .is_none_or(|patterns| patterns.matches_files(relative.as_str()))
            && !self.ignores.matches_ignore(relative.as_str())
    }

    pub(crate) fn ignores(&self, file: &Path) -> bool {
        self.relative(file)
            .is_some_and(|relative| self.ignores.matches_ignore(relative.as_str()))
    }

    fn relative(&self, file: &Path) -> Option<String> {
        file.strip_prefix(&self.base_dir).ok().map(normalize_path)
    }
}

impl GlobSequence {
    pub(crate) fn new(patterns: &[String]) -> Self {
        let steps = patterns
            .iter()
            .filter_map(|source| {
                let slashed = source.replace('\\', "/");
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

    pub(crate) fn matches_files(&self, file: &str) -> bool {
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

pub(crate) fn absolute_path(path: &Path, cwd: &Path) -> PathBuf {
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

fn strip_leading_current_dir(pattern: &str) -> &str {
    let mut stripped = pattern;
    while let Some(rest) = stripped.strip_prefix("./") {
        stripped = rest;
    }
    stripped
}

pub(crate) fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").into()
}
