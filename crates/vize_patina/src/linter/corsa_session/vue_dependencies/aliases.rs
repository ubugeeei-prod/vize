//! Resolve explicit `.vue` imports through the effective tsconfig `paths`.

use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_l0::String;

#[derive(Default)]
pub(in crate::linter::corsa_session) struct PathAliases {
    project_root: PathBuf,
    patterns: Vec<Pattern>,
}

struct Pattern {
    key: String,
    prefix: String,
    suffix: String,
    targets: Vec<String>,
}

impl PathAliases {
    pub(in crate::linter::corsa_session) fn from_tsconfig(
        contents: &str,
        project_root: &Path,
    ) -> Self {
        let Some(paths) = serde_json::from_str::<Value>(contents)
            .ok()
            .and_then(|value| value.pointer("/compilerOptions/paths").cloned())
            .and_then(|value| value.as_object().cloned())
        else {
            return Self::default();
        };
        let patterns = paths
            .into_iter()
            .filter_map(|(key, targets)| {
                let targets = targets
                    .as_array()?
                    .iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect::<Vec<_>>();
                if targets.is_empty() {
                    return None;
                }
                let (prefix, suffix) = key.split_once('*').unwrap_or_else(|| (key.as_str(), ""));
                Some(Pattern {
                    key: key.as_str().into(),
                    prefix: prefix.into(),
                    suffix: suffix.into(),
                    targets,
                })
            })
            .collect();
        Self {
            project_root: project_root.to_path_buf(),
            patterns,
        }
    }

    pub(super) fn resolve(&self, specifier: &str) -> Option<PathBuf> {
        let mut selected: Option<(&Pattern, Option<&str>)> = None;
        for pattern in &self.patterns {
            if !pattern.key.contains('*') {
                if specifier == pattern.key {
                    selected = Some((pattern, None));
                    break;
                }
                continue;
            }
            let Some(capture) = specifier
                .strip_prefix(pattern.prefix.as_str())
                .and_then(|rest| rest.strip_suffix(pattern.suffix.as_str()))
            else {
                continue;
            };
            if selected
                .as_ref()
                .is_none_or(|(current, _)| pattern.prefix.len() > current.prefix.len())
            {
                selected = Some((pattern, Some(capture)));
            }
        }
        let (pattern, capture) = selected?;
        for target in &pattern.targets {
            let substituted = match capture {
                Some(capture) => {
                    let Some((prefix, suffix)) = target.split_once('*') else {
                        continue;
                    };
                    let mut value =
                        String::with_capacity(prefix.len() + capture.len() + suffix.len());
                    value.push_str(prefix);
                    value.push_str(capture);
                    value.push_str(suffix);
                    value
                }
                None => target.clone(),
            };
            let path = Path::new(substituted.as_str());
            let candidate = if path.is_absolute() {
                path.to_path_buf()
            } else {
                self.project_root.join(path)
            };
            if candidate.is_file() {
                return Some(candidate.canonicalize().unwrap_or(candidate));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::PathAliases;

    #[test]
    fn exact_and_longest_wildcard_aliases_resolve_existing_vue_files() {
        let root = tempfile::tempdir().unwrap();
        let exact = root.path().join("src/exact.vue");
        let broad = root.path().join("src/ui/panel.vue");
        let narrow = root.path().join("src/special/panel.vue");
        for path in [&exact, &broad, &narrow] {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "<template />").unwrap();
        }
        let config = serde_json::json!({
            "compilerOptions": {
                "paths": {
                    "#exact": [exact],
                    "#ui/*": [root.path().join("src/ui/*")],
                    "#ui/special/*": [root.path().join("src/special/*")]
                }
            }
        });
        let aliases = PathAliases::from_tsconfig(&config.to_string(), root.path());
        assert_eq!(
            aliases.resolve("#exact").unwrap(),
            exact.canonicalize().unwrap()
        );
        assert_eq!(
            aliases.resolve("#ui/panel.vue").unwrap(),
            broad.canonicalize().unwrap()
        );
        assert_eq!(
            aliases.resolve("#ui/special/panel.vue").unwrap(),
            narrow.canonicalize().unwrap()
        );
        assert!(aliases.resolve("#missing/panel.vue").is_none());
    }
}
