//! One extension and diagnostic policy for a TypeScript project snapshot.
//!
//! Source discovery, virtual-project materialization, and Corsa diagnostic
//! requests must agree on the same effective `allowJs` / `checkJs` values.
//! Keeping the classification here prevents each layer from growing a slightly
//! different extension whitelist.
#![allow(clippy::disallowed_types)] // serde_json::Map keys are std::string::String.

use std::path::Path;

use serde_json::{Map, Value};

use super::declaration_path::is_declaration_file;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct SourceFilePolicy {
    allow_javascript: bool,
    check_javascript: bool,
}

impl SourceFilePolicy {
    pub(crate) fn from_compiler_options(
        compiler_options: &Map<std::string::String, Value>,
    ) -> Self {
        Self {
            allow_javascript: option_enabled(compiler_options, "allowJs"),
            check_javascript: option_enabled(compiler_options, "checkJs"),
        }
    }

    pub(crate) fn allows_javascript(self) -> bool {
        self.allow_javascript
    }

    pub(crate) fn checks_javascript(self) -> bool {
        self.check_javascript
    }

    pub(crate) fn accepts_project_source(self, path: &Path) -> bool {
        is_typescript_family(path)
            || is_vue(path)
            || is_declaration_file(path)
            || self.allow_javascript && is_javascript_family(path)
    }

    pub(crate) fn accepts_diagnostic_input(self, path: &Path) -> bool {
        is_typescript_family(path) || self.allow_javascript && is_javascript_family(path)
    }
}

fn option_enabled(options: &Map<std::string::String, Value>, name: &str) -> bool {
    options.get(name).and_then(Value::as_bool).unwrap_or(false)
}

fn is_typescript_family(path: &Path) -> bool {
    matches!(extension(path), Some("ts" | "tsx" | "mts" | "cts"))
}

fn is_javascript_family(path: &Path) -> bool {
    matches!(extension(path), Some("js" | "jsx" | "mjs" | "cjs"))
}

fn is_vue(path: &Path) -> bool {
    extension(path) == Some("vue")
}

fn extension(path: &Path) -> Option<&str> {
    path.extension().and_then(|extension| extension.to_str())
}

#[cfg(test)]
mod tests {
    use super::SourceFilePolicy;
    use serde_json::json;
    use std::path::Path;

    fn policy(options: serde_json::Value) -> SourceFilePolicy {
        SourceFilePolicy::from_compiler_options(options.as_object().unwrap())
    }

    #[test]
    fn allowjs_controls_membership_while_checkjs_only_controls_diagnostics() {
        let typescript_only = policy(json!({ "checkJs": true }));
        let allow_js = policy(json!({ "allowJs": true, "checkJs": false }));

        for path in [
            "App.vue",
            "entry.ts",
            "view.tsx",
            "module.mts",
            "config.cts",
        ] {
            assert!(
                typescript_only.accepts_project_source(Path::new(path)),
                "{path}"
            );
            assert!(typescript_only.accepts_diagnostic_input(Path::new(path)) || path == "App.vue");
        }
        for path in ["entry.js", "view.jsx", "module.mjs", "config.cjs"] {
            assert!(
                !typescript_only.accepts_project_source(Path::new(path)),
                "{path}"
            );
            assert!(allow_js.accepts_project_source(Path::new(path)), "{path}");
            assert!(allow_js.accepts_diagnostic_input(Path::new(path)), "{path}");
        }
        assert!(!typescript_only.allows_javascript());
        assert!(typescript_only.checks_javascript());
        assert!(allow_js.allows_javascript());
        assert!(!allow_js.checks_javascript());
    }
}
