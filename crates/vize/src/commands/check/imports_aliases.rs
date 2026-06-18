use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_carton::{FxHashSet, String};

use super::path_cache::CanonicalPathCache;
use super::tsconfig_inputs::{parse_jsonc_value, read_extends_entries, resolve_extended_tsconfig};

#[derive(Default)]
pub(super) struct PathAliasResolver {
    aliases: Vec<PathAlias>,
}

struct PathAlias {
    prefix: String,
    suffix: String,
    has_wildcard: bool,
    targets: Vec<String>,
    base_dir: PathBuf,
}

impl PathAliasResolver {
    pub(super) fn from_tsconfig(tsconfig_path: Option<&Path>) -> Self {
        let Some(tsconfig_path) = tsconfig_path else {
            return Self::default();
        };
        let mut seen = FxHashSet::default();
        load_aliases(tsconfig_path, &mut seen).unwrap_or_default()
    }

    pub(super) fn resolve(
        &self,
        specifier: &str,
        canonical_paths: &mut CanonicalPathCache,
        include_jsx: bool,
        resolve_base: impl Fn(&Path, &mut CanonicalPathCache, bool) -> Option<PathBuf>,
    ) -> Option<PathBuf> {
        for alias in &self.aliases {
            let Some(matched) = alias.match_specifier(specifier) else {
                continue;
            };
            for target in &alias.targets {
                let target = if target.contains('*') {
                    alias.base_dir.join(target.replace('*', matched))
                } else {
                    alias.base_dir.join(target.as_str())
                };
                if let Some(resolved) = resolve_base(&target, canonical_paths, include_jsx) {
                    return Some(resolved);
                }
            }
        }
        None
    }
}

impl PathAlias {
    fn match_specifier<'a>(&self, specifier: &'a str) -> Option<&'a str> {
        if !self.has_wildcard {
            return (self.prefix == specifier).then_some("");
        }
        specifier
            .strip_prefix(self.prefix.as_str())?
            .strip_suffix(self.suffix.as_str())
    }
}

fn load_aliases(
    tsconfig_path: &Path,
    seen: &mut FxHashSet<PathBuf>,
) -> std::io::Result<PathAliasResolver> {
    let tsconfig_path = tsconfig_path
        .canonicalize()
        .unwrap_or_else(|_| tsconfig_path.to_path_buf());
    if !seen.insert(tsconfig_path.clone()) {
        return Ok(PathAliasResolver::default());
    }

    let content = std::fs::read_to_string(&tsconfig_path)?;
    let value = parse_jsonc_value(&content).unwrap_or(Value::Null);
    let dir = tsconfig_path.parent().unwrap_or(Path::new("."));

    let mut resolver = PathAliasResolver::default();
    for extends in read_extends_entries(&value) {
        if let Some(extended) = resolve_extended_tsconfig(&tsconfig_path, &extends) {
            resolver = load_aliases(&extended, seen)?;
        }
    }

    let Some(options) = value.get("compilerOptions").and_then(Value::as_object) else {
        return Ok(resolver);
    };
    let base_dir = options
        .get("baseUrl")
        .and_then(Value::as_str)
        .map(|base| dir.join(base))
        .unwrap_or_else(|| dir.to_path_buf());
    let Some(paths) = options.get("paths").and_then(Value::as_object) else {
        return Ok(resolver);
    };

    resolver.aliases.clear();
    for (pattern, targets) in paths {
        let Some(targets) = targets.as_array() else {
            continue;
        };
        let (prefix, suffix, has_wildcard) = split_pattern(pattern);
        let targets = targets
            .iter()
            .filter_map(Value::as_str)
            .map(String::from)
            .collect::<Vec<_>>();
        if !targets.is_empty() {
            resolver.aliases.push(PathAlias {
                prefix,
                suffix,
                has_wildcard,
                targets,
                base_dir: base_dir.clone(),
            });
        }
    }
    resolver
        .aliases
        .sort_by_key(|alias| std::cmp::Reverse(alias.prefix.len()));
    Ok(resolver)
}

fn split_pattern(pattern: &str) -> (String, String, bool) {
    match pattern.split_once('*') {
        Some((prefix, suffix)) => (prefix.into(), suffix.into(), true),
        None => (pattern.into(), String::default(), false),
    }
}

#[cfg(test)]
mod tests {
    use super::PathAliasResolver;
    use crate::commands::check::{
        imports::{collect_transitive_local_imports, resolve_import_base},
        path_cache::CanonicalPathCache,
    };
    use std::path::{Path, PathBuf};

    fn write(root: &Path, rel: &str, contents: &str) -> PathBuf {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn resolves_wildcard_alias_to_vue_source() {
        let root = tempfile::tempdir().unwrap();
        let keyboard = write(
            root.path(),
            "src/keyboards/EnglishKeyboard.vue",
            "<template />",
        );
        std::fs::write(
            root.path().join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "~/*": ["*"]
    }
  }
}"#,
        )
        .unwrap();

        let resolver =
            PathAliasResolver::from_tsconfig(Some(root.path().join("tsconfig.json").as_path()));
        let resolved = resolver.resolve(
            "~/src/keyboards/EnglishKeyboard.vue",
            &mut CanonicalPathCache::default(),
            false,
            resolve_import_base,
        );

        assert_eq!(resolved, Some(keyboard.canonicalize().unwrap()));
    }

    #[test]
    fn exact_alias_does_not_match_prefix() {
        let root = tempfile::tempdir().unwrap();
        let entry = write(root.path(), "src/exact.ts", "export const exact = 1;");
        let prefix = write(root.path(), "src/prefix.ts", "export const prefix = 1;");
        std::fs::write(
            root.path().join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@app": ["src/exact.ts"],
      "@app/*": ["src/*"]
    }
  }
}"#,
        )
        .unwrap();

        let resolver =
            PathAliasResolver::from_tsconfig(Some(root.path().join("tsconfig.json").as_path()));
        let mut canonical_paths = CanonicalPathCache::default();
        let resolved_exact =
            resolver.resolve("@app", &mut canonical_paths, false, resolve_import_base);
        let resolved_prefix = resolver.resolve(
            "@app/prefix",
            &mut canonical_paths,
            false,
            resolve_import_base,
        );

        assert_eq!(resolved_exact, Some(entry.canonicalize().unwrap()));
        assert_eq!(resolved_prefix, Some(prefix.canonicalize().unwrap()));
    }

    #[test]
    fn collector_registers_tsconfig_alias_vue_dependencies() {
        let root = tempfile::tempdir().unwrap();
        let entry = write(
            root.path(),
            "src/Entry.vue",
            r#"<script lang="ts">
import EnglishKeyboard from "~/src/keyboards/EnglishKeyboard.vue";
void EnglishKeyboard;
</script>
"#,
        );
        let keyboard = write(
            root.path(),
            "src/keyboards/EnglishKeyboard.vue",
            "<template />",
        );
        std::fs::write(
            root.path().join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "~/*": ["*"]
    }
  }
}"#,
        )
        .unwrap();

        let resolver =
            PathAliasResolver::from_tsconfig(Some(root.path().join("tsconfig.json").as_path()));
        let discovered = collect_transitive_local_imports(
            &[entry],
            root.path(),
            &mut CanonicalPathCache::default(),
            false,
            Some(&resolver),
        );

        assert_eq!(discovered, vec![keyboard.canonicalize().unwrap()]);
    }
}
