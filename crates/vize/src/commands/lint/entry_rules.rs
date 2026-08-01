//! Batch resolution of declaration-ordered `entries[].linter.rules`.

use globset::{GlobBuilder, GlobMatcher};
use std::path::{Component, Path, PathBuf};
use vize_carton::{
    FxHashMap, String,
    config::{LintRuleOptions, LintRuleSeverity, LinterConfigPlan, LinterFeatureFlags},
};
use vize_patina::{HelpLevel, LintPreset, Linter, Severity};

use super::LintArgs;

pub(super) struct ResolvedLinterRuleGroups {
    pub(super) configs: Vec<crate::config::LinterConfig>,
    pub(super) file_config_indices: Vec<usize>,
}

impl ResolvedLinterRuleGroups {
    pub(super) fn build_linters(
        &self,
        preset: LintPreset,
        help_level: HelpLevel,
        args: &LintArgs,
        features: LinterFeatureFlags,
        rule_options: &LintRuleOptions,
        configured_corsa_path: Option<PathBuf>,
    ) -> Vec<Linter> {
        self.configs
            .iter()
            .map(|config| {
                let type_aware =
                    args.type_aware || args.strict_reactivity || config.type_aware_lint_enabled();
                let mut linter = Linter::with_preset(preset)
                    .with_additional_rules(config.enabled_rules())
                    .with_disabled_rules(config.disabled_rules())
                    .with_disabled_categories(config.disabled_categories())
                    .with_category_severity_overrides(severity_overrides(
                        config.category_severity_overrides(),
                    ))
                    .with_rule_severity_overrides(severity_overrides(
                        config.rule_severity_overrides(),
                    ))
                    .with_help_level(help_level)
                    .with_type_aware_lint(type_aware)
                    .with_vue_version(features.vue_version)
                    .with_vapor_mode(features.vapor)
                    .with_restricted_globals(rule_options.restricted_globals())
                    .with_restricted_members(rule_options.restricted_members());
                #[cfg(not(target_arch = "wasm32"))]
                {
                    linter = linter.with_corsa_path(configured_corsa_path.clone());
                }
                #[cfg(not(target_arch = "wasm32"))]
                if args.strict_reactivity {
                    linter = linter.with_rule(Box::new(
                        vize_patina::rules::type_aware::NoReactivityLoss::new(),
                    ));
                }
                linter
            })
            .collect()
    }
}

fn severity_overrides(entries: Vec<(String, LintRuleSeverity)>) -> Vec<(String, Severity)> {
    entries
        .into_iter()
        .filter_map(|(name, severity)| match severity {
            LintRuleSeverity::Off => None,
            LintRuleSeverity::Warn => Some((name, Severity::Warning)),
            LintRuleSeverity::Error => Some((name, Severity::Error)),
        })
        .collect()
}

pub(super) struct LinterRuleResolver {
    plan: LinterConfigPlan,
    scopes: Vec<EntryRuleScope>,
}

struct EntryRuleScope {
    base_dir: PathBuf,
    files: Option<GlobSequence>,
    ignores: GlobSequence,
}

struct GlobSequence {
    steps: Vec<GlobStep>,
    has_steps: bool,
    has_positive_source: bool,
}

struct GlobStep {
    negated: bool,
    matcher: GlobMatcher,
}

impl LinterRuleResolver {
    pub(super) fn new(plan: LinterConfigPlan, config_dir: &Path, cwd: &Path) -> Self {
        let config_dir = absolute_path(config_dir, cwd);
        let scopes = plan
            .entries
            .iter()
            .map(|entry| EntryRuleScope {
                base_dir: resolve_base_dir(entry.base_path.as_deref(), &config_dir),
                files: entry.files.as_deref().map(GlobSequence::new),
                ignores: GlobSequence::new(&entry.ignores),
            })
            .collect();
        Self { plan, scopes }
    }

    /// Resolve each distinct matching-entry signature exactly once for the batch.
    pub(super) fn resolve_files(&self, files: &[PathBuf], cwd: &Path) -> ResolvedLinterRuleGroups {
        if self.scopes.is_empty() {
            return ResolvedLinterRuleGroups {
                configs: vec![self.plan.base.clone()],
                file_config_indices: vec![0; files.len()],
            };
        }
        let mut signatures = FxHashMap::<Vec<usize>, usize>::default();
        let mut configs = Vec::new();
        let mut file_config_indices = Vec::with_capacity(files.len());
        for file in files {
            let signature = self.matching_entries(file, cwd);
            let index = match signatures.get(&signature) {
                Some(index) => *index,
                None => {
                    let index = configs.len();
                    configs.push(self.plan.resolve_matching_entries(&signature));
                    signatures.insert(signature, index);
                    index
                }
            };
            file_config_indices.push(index);
        }
        ResolvedLinterRuleGroups {
            configs,
            file_config_indices,
        }
    }

    fn matching_entries(&self, file: &Path, cwd: &Path) -> Vec<usize> {
        let file = absolute_path(file, cwd);
        self.scopes
            .iter()
            .enumerate()
            .filter_map(|(index, scope)| scope.matches(&file).then_some(index))
            .collect()
    }
}

impl EntryRuleScope {
    fn matches(&self, file: &Path) -> bool {
        let Ok(relative) = file.strip_prefix(&self.base_dir) else {
            return false;
        };
        let relative = normalize_path(relative);
        self.files
            .as_ref()
            .is_none_or(|patterns| patterns.matches_files(relative.as_str()))
            && !self.ignores.matches_ignore(relative.as_str())
    }
}

impl GlobSequence {
    fn new(patterns: &[String]) -> Self {
        let steps = patterns
            .iter()
            .filter_map(|source| {
                // Split the negation marker before stripping `./`, otherwise
                // `!./src/generated/**` keeps the `./` and silently stops
                // matching the relative paths it is meant to exclude.
                let slashed = source.replace('\\', "/");
                let (negated, rest) = slashed
                    .strip_prefix('!')
                    .map_or((false, slashed.as_str()), |rest| (true, rest));
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

    fn matches_files(&self, file: &str) -> bool {
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

/// Match `file` or any of its parent directories, so a directory-form pattern
/// such as `src` or `src/pages` covers the whole subtree. `file` is already
/// `/`-normalized, so ancestors are plain string slices.
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

fn absolute_path(path: &Path, cwd: &Path) -> PathBuf {
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

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").into()
}

#[cfg(test)]
#[path = "entry_rules_tests.rs"]
mod tests;
