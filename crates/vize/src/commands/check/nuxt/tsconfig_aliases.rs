//! Helpers for detecting Nuxt virtual modules already handled by `tsconfig` paths.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;
use vize_carton::{FxHashSet, String};

use crate::commands::check::tsconfig_inputs::{
    parse_jsonc_value, read_extends_entries, resolve_extended_tsconfig,
};

pub(super) fn collect_explicit_virtual_module_aliases(
    tsconfig_path: Option<&Path>,
) -> FxHashSet<String> {
    let mut aliases = FxHashSet::default();
    let mut seen = FxHashSet::default();
    if let Some(tsconfig_path) = tsconfig_path {
        collect_explicit_virtual_module_aliases_inner(tsconfig_path, &mut aliases, &mut seen);
    }
    aliases
}

fn collect_explicit_virtual_module_aliases_inner(
    tsconfig_path: &Path,
    aliases: &mut FxHashSet<String>,
    seen: &mut FxHashSet<PathBuf>,
) {
    let resolved = vize_carton::path::canonicalize_non_verbatim(tsconfig_path);
    if !seen.insert(resolved.clone()) {
        return;
    }

    let Ok(content) = fs::read_to_string(&resolved) else {
        return;
    };
    let Ok(value) = parse_jsonc_value(&content) else {
        return;
    };

    for extends in read_extends_entries(&value) {
        if let Some(extended_path) = resolve_extended_tsconfig(&resolved, &extends) {
            collect_explicit_virtual_module_aliases_inner(&extended_path, aliases, seen);
        }
    }

    let Some(paths) = value
        .get("compilerOptions")
        .and_then(Value::as_object)
        .and_then(|compiler_options| compiler_options.get("paths"))
        .and_then(Value::as_object)
    else {
        return;
    };

    for alias in paths.keys().filter(|alias| is_nuxt_virtual_module(alias)) {
        aliases.insert(alias.as_str().into());
    }
}

fn is_nuxt_virtual_module(alias: &str) -> bool {
    matches!(alias, "#imports" | "#components" | "#app" | "@typed-router")
}

#[cfg(test)]
mod tests {
    use super::collect_explicit_virtual_module_aliases;

    #[test]
    fn collects_virtual_module_aliases_from_tsconfig_extends_chain() {
        let root =
            std::env::temp_dir().join(format!("vize-nuxt-tsconfig-aliases-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        std::fs::write(
            root.join("base.json"),
            r##"{
  "compilerOptions": {
    "paths": {
      "#imports": ["types/imports.ts"],
      "~/*": ["app/*"]
    }
  }
}"##,
        )
        .unwrap();
        std::fs::write(
            root.join("tsconfig.json"),
            r##"{
  "extends": "./base.json",
  "compilerOptions": {
    "paths": {
      "@typed-router": ["types/router.ts"],
      "#components": ["types/components.ts"]
    }
  }
}"##,
        )
        .unwrap();

        let aliases = collect_explicit_virtual_module_aliases(Some(&root.join("tsconfig.json")));

        assert!(aliases.contains("#imports"));
        assert!(aliases.contains("#components"));
        assert!(aliases.contains("@typed-router"));
        assert!(!aliases.contains("~/*"));

        let _ = std::fs::remove_dir_all(&root);
    }
}
