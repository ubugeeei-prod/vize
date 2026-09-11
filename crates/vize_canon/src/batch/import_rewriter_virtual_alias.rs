//! Virtual-project policy for aliased `.vue` specifier rewrites.
//!
//! The generic rewriter rewrites `@/Foo.vue` to `@/Foo.vue.ts`, which is correct
//! only when the virtual tsconfig carries a matching `paths` alias. Without one,
//! TypeScript and vue-tsc resolve the authored specifier as-is; appending `.ts`
//! changes the unresolved target and can introduce false TS2307 diagnostics.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use vize_carton::{String, cstr};

use super::virtual_rewrite::{append_extension, resolve_source_path};

#[derive(Debug, Default)]
pub(crate) struct VirtualAliasRewritePolicy {
    patterns: Vec<PathAliasPattern>,
}

#[derive(Debug)]
struct PathAliasPattern {
    prefix: String,
    suffix: String,
    wildcard: bool,
    targets: Vec<String>,
}

impl VirtualAliasRewritePolicy {
    #[allow(clippy::disallowed_types)]
    pub(crate) fn from_paths(paths: &Map<std::string::String, Value>) -> Self {
        let patterns = paths
            .iter()
            .filter_map(|(pattern, targets)| {
                let targets = first_party_alias_targets(targets);
                (!targets.is_empty()).then(|| PathAliasPattern::new(pattern, targets))
            })
            .collect();
        Self { patterns }
    }

    pub(crate) fn source_may_contain_rewritable_alias(&self, source: &str) -> bool {
        self.patterns
            .iter()
            .any(|pattern| pattern.source_may_contain_alias(source))
    }

    pub(crate) fn should_rewrite_vue_specifier(&self, specifier: &str) -> bool {
        if !is_policy_controlled_vue_specifier(specifier) {
            return true;
        }
        self.matches(cstr!("{specifier}.ts").as_str())
    }

    pub(crate) fn rewrite_extensionless_vue_specifier(
        &self,
        specifier: &str,
        project_root: &Path,
    ) -> Option<String> {
        if Path::new(specifier).extension().is_some() {
            return None;
        }
        let suffix = self.extensionless_vue_rewrite_suffix(specifier, project_root)?;
        Some(cstr!("{specifier}{suffix}"))
    }

    fn matches(&self, specifier: &str) -> bool {
        self.patterns
            .iter()
            .any(|pattern| pattern.matches(specifier))
    }

    fn extensionless_vue_rewrite_suffix(
        &self,
        specifier: &str,
        project_root: &Path,
    ) -> Option<&'static str> {
        let mut best: Option<(usize, &'static str)> = None;
        for pattern in &self.patterns {
            if best
                .as_ref()
                .is_some_and(|(len, _)| pattern.key_len() <= *len)
            {
                continue;
            }
            let Some(suffix) = pattern.extensionless_vue_rewrite_suffix(specifier, project_root)
            else {
                continue;
            };
            best = Some((pattern.key_len(), suffix));
        }
        best.map(|(_, suffix)| suffix)
    }
}

impl PathAliasPattern {
    fn new(pattern: &str, targets: Vec<String>) -> Self {
        let Some(wildcard) = pattern.find('*') else {
            return Self {
                prefix: pattern.into(),
                suffix: String::default(),
                wildcard: false,
                targets,
            };
        };
        Self {
            prefix: pattern[..wildcard].into(),
            suffix: pattern[wildcard + 1..].into(),
            wildcard: true,
            targets,
        }
    }

    fn source_may_contain_alias(&self, source: &str) -> bool {
        if self.prefix.is_empty() {
            source.contains("import") || source.contains("export")
        } else {
            source.contains(self.prefix.as_str())
        }
    }

    fn matches(&self, specifier: &str) -> bool {
        if self.wildcard {
            specifier.starts_with(self.prefix.as_str())
                && specifier.ends_with(self.suffix.as_str())
                && specifier.len() >= self.prefix.len() + self.suffix.len()
        } else {
            specifier == self.prefix
        }
    }

    fn key_len(&self) -> usize {
        self.prefix.len() + self.suffix.len()
    }

    fn extensionless_vue_rewrite_suffix(
        &self,
        specifier: &str,
        project_root: &Path,
    ) -> Option<&'static str> {
        for target in &self.targets {
            let Some(target) = self.substitute_target(specifier, target.as_str()) else {
                continue;
            };
            let target = target_path(project_root, target.as_str());
            let Some(resolved) = resolve_source_path(target.as_path()) else {
                continue;
            };
            let suffix = vue_rewrite_suffix_for_resolved_target(target.as_path(), &resolved)?;
            return self
                .matches(cstr!("{specifier}{suffix}").as_str())
                .then_some(suffix);
        }
        None
    }

    fn substitute_target(&self, specifier: &str, target: &str) -> Option<String> {
        if !self.matches(specifier) {
            return None;
        }
        if !self.wildcard {
            return Some(target.into());
        }
        let capture_end = specifier.len().checked_sub(self.suffix.len())?;
        let capture = specifier.get(self.prefix.len()..capture_end)?;
        target.find('*').map(|wildcard| {
            cstr!(
                "{}{}{}",
                &target[..wildcard],
                capture,
                &target[wildcard + 1..]
            )
        })
    }
}

fn target_path(project_root: &Path, target: &str) -> PathBuf {
    let path = Path::new(target);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    }
}

fn is_policy_controlled_vue_specifier(specifier: &str) -> bool {
    specifier.ends_with(".vue") && (specifier.starts_with("@/") || specifier.starts_with("~/"))
}

fn first_party_alias_targets(targets: &Value) -> Vec<String> {
    let Some(targets) = targets.as_array() else {
        return Vec::new();
    };
    targets
        .iter()
        .filter_map(Value::as_str)
        .filter(|target| !target.is_empty() && !target.contains("node_modules"))
        .map(String::from)
        .collect()
}

fn vue_rewrite_suffix_for_resolved_target(target: &Path, resolved: &Path) -> Option<&'static str> {
    if resolved
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("vue")
    {
        return None;
    }
    let canonical_resolved = vize_carton::path::canonicalize_non_verbatim(resolved);
    let direct_vue = if target.extension().and_then(|extension| extension.to_str()) == Some("vue") {
        target.to_path_buf()
    } else {
        append_extension(target, ".vue")
    };
    if canonical_resolved == vize_carton::path::canonicalize_non_verbatim(&direct_vue) {
        return Some(".vue.ts");
    }
    let index_vue = target.join("index.vue");
    (target.extension().is_none()
        && canonical_resolved == vize_carton::path::canonicalize_non_verbatim(&index_vue))
    .then_some("/index.vue.ts")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::json;
    use tempfile::TempDir;

    use super::VirtualAliasRewritePolicy;

    fn policy(paths: serde_json::Value) -> VirtualAliasRewritePolicy {
        VirtualAliasRewritePolicy::from_paths(paths.as_object().unwrap())
    }

    fn rewrite_extensionless(
        paths: serde_json::Value,
        files: &[&str],
        specifier: &str,
    ) -> Option<String> {
        let temp_dir = TempDir::new().unwrap();
        for file in files {
            let path = temp_dir.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "").unwrap();
        }
        policy(paths)
            .rewrite_extensionless_vue_specifier(specifier, temp_dir.path())
            .map(|specifier| specifier.to_string())
    }

    #[test]
    fn aliased_vue_specifier_requires_a_matching_path_alias() {
        let empty = policy(json!({}));
        assert!(!empty.should_rewrite_vue_specifier("@/App.vue"));
        assert!(!empty.should_rewrite_vue_specifier("~/App.vue"));
        assert!(empty.should_rewrite_vue_specifier("./App.vue"));

        let configured = policy(json!({
            "@/*": ["src/*"],
            "~/*": ["app/*"],
            "#/*.vue": ["src/*.vue"]
        }));
        assert!(configured.should_rewrite_vue_specifier("@/App.vue"));
        assert!(configured.should_rewrite_vue_specifier("~/App.vue"));
        assert!(configured.should_rewrite_vue_specifier("#/App.vue"));
    }

    #[test]
    fn vue_suffixed_alias_keys_do_not_claim_rewritten_specifiers() {
        let configured = policy(json!({
            "@/*.vue": ["src/*.vue"]
        }));

        assert!(!configured.should_rewrite_vue_specifier("@/App.vue"));
    }

    #[test]
    fn dependency_only_aliases_do_not_claim_first_party_vue_specifiers() {
        let configured = policy(json!({
            "@/*": ["node_modules/@scope/*"]
        }));

        assert!(!configured.should_rewrite_vue_specifier("@/App.vue"));
    }

    #[test]
    fn empty_and_non_string_targets_do_not_claim_first_party_aliases() {
        let configured = policy(json!({
            "@/*": ["", 42, null]
        }));

        assert!(!configured.should_rewrite_vue_specifier("@/App.vue"));
    }

    #[test]
    fn a_mixed_target_list_can_still_claim_a_first_party_alias() {
        let configured = policy(json!({
            "@/*": ["node_modules/@scope/*", "src/*"]
        }));

        assert!(configured.should_rewrite_vue_specifier("@/App.vue"));
    }

    #[test]
    fn extensionless_aliases_stop_at_the_first_resolvable_paths_target() {
        let paths = json!({
            "@/*": ["src/*.ts", "src/*.vue"]
        });

        assert_eq!(
            rewrite_extensionless(
                paths.clone(),
                &["src/Button.ts", "src/Button.vue"],
                "@/Button"
            ),
            None,
            "a later Vue target must not override the first resolvable TS target"
        );
        assert_eq!(
            rewrite_extensionless(paths, &["src/Button.vue"], "@/Button"),
            Some("@/Button.vue.ts".into()),
            "unresolved earlier targets should still fall through to the first resolvable Vue target"
        );
    }
}
