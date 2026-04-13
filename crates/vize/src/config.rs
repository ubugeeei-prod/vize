//! Configuration file loading for vize.
//!
//! Reads `vize.config.pkl` (preferred) or `vize.config.json` from the current
//! working directory. Also provides JSON Schema generation for editor autocompletion.

#![allow(clippy::disallowed_types)]

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Top-level vize configuration.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct VizeConfig {
    /// JSON Schema reference (for editor autocompletion).
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Type checking configuration.
    #[serde(default)]
    pub check: CheckConfig,

    /// Formatting configuration.
    #[cfg(feature = "glyph")]
    #[serde(default)]
    pub fmt: vize_glyph::FormatOptions,
}

/// Configuration for the `check` command.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CheckConfig {
    /// Path to a `.d.ts` file that augments `ComponentCustomProperties`.
    ///
    /// The file should follow Vue's standard module augmentation pattern:
    /// ```ts
    /// declare module 'vue' {
    ///   interface ComponentCustomProperties {
    ///     $t: (...args: any[]) => string
    ///   }
    /// }
    /// ```
    ///
    /// Resolved relative to `vize.config.json`.
    /// When omitted or null, no plugin globals are declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub globals: Option<String>,

    /// Override the number of parallel Corsa servers used by `vize check`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub servers: Option<usize>,
}

/// Load configuration from `vize.config.pkl` (preferred) or `vize.config.json`.
///
/// PKL takes priority when both files exist. If the PKL file exists but parsing
/// fails (e.g. `pkl` binary not on PATH), falls back to defaults with a warning.
pub fn load_config(dir: Option<&Path>) -> VizeConfig {
    let base = dir
        .map(|d| d.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    // Try PKL first
    let pkl_path = base.join("vize.config.pkl");
    if pkl_path.exists() {
        match rpkl::from_config::<VizeConfig>(&pkl_path) {
            Ok(config) => return config,
            Err(e) => {
                eprintln!(
                    "\x1b[33mWarning:\x1b[0m Failed to parse {}: {}",
                    pkl_path.display(),
                    e
                );
                return VizeConfig::default();
            }
        }
    }

    // Fall back to JSON
    let json_path = base.join("vize.config.json");
    if !json_path.exists() {
        return VizeConfig::default();
    }

    match std::fs::read_to_string(&json_path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!(
                    "\x1b[33mWarning:\x1b[0m Failed to parse {}: {}",
                    json_path.display(),
                    e
                );
                VizeConfig::default()
            }
        },
        Err(e) => {
            eprintln!(
                "\x1b[33mWarning:\x1b[0m Failed to read {}: {}",
                json_path.display(),
                e
            );
            VizeConfig::default()
        }
    }
}

/// JSON Schema for `vize.config.json`.
pub const VIZE_CONFIG_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Vize Configuration",
  "description": "Configuration file for vize - High-performance Vue.js toolchain",
  "type": "object",
  "properties": {
    "$schema": {
      "type": "string",
      "description": "JSON Schema reference for editor autocompletion"
    },
    "check": {
      "type": "object",
      "description": "Type checking configuration",
      "properties": {
        "globals": {
          "type": "string",
          "description": "Path to a .d.ts file that augments ComponentCustomProperties with template globals (e.g. $t, $route). Resolved relative to vize.config.json.",
          "examples": ["globals.d.ts", "./types/globals.d.ts"]
        },
        "servers": {
          "type": "integer",
          "minimum": 1,
          "description": "Override the number of parallel Corsa language servers used by `vize check`."
        }
      },
      "additionalProperties": false
    },
    "fmt": {
      "type": "object",
      "description": "Formatting configuration (Prettier-compatible)",
      "properties": {
        "printWidth": { "type": "integer", "default": 100, "description": "Maximum line width" },
        "tabWidth": { "type": "integer", "default": 2, "description": "Number of spaces per indentation level" },
        "useTabs": { "type": "boolean", "default": false, "description": "Use tabs instead of spaces" },
        "semi": { "type": "boolean", "default": true, "description": "Print semicolons at the ends of statements" },
        "singleQuote": { "type": "boolean", "default": false, "description": "Use single quotes instead of double quotes" },
        "jsxSingleQuote": { "type": "boolean", "default": false, "description": "Use single quotes in JSX" },
        "trailingComma": { "type": "string", "enum": ["none", "es5", "all"], "default": "all", "description": "Print trailing commas wherever possible" },
        "bracketSpacing": { "type": "boolean", "default": true, "description": "Print spaces between brackets in object literals" },
        "bracketSameLine": { "type": "boolean", "default": false, "description": "Put > of multi-line element at end of last line" },
        "arrowParens": { "type": "string", "enum": ["always", "avoid"], "default": "always", "description": "Include parens around sole arrow function parameter" },
        "endOfLine": { "type": "string", "enum": ["lf", "crlf", "cr", "auto"], "default": "lf", "description": "End of line style" },
        "quoteProps": { "type": "string", "enum": ["as-needed", "consistent", "preserve"], "default": "as-needed" },
        "singleAttributePerLine": { "type": "boolean", "default": false, "description": "Put each HTML attribute on its own line" },
        "vueIndentScriptAndStyle": { "type": "boolean", "default": false, "description": "Indent script and style tags in Vue files" },
        "sortAttributes": { "type": "boolean", "default": true, "description": "Sort HTML attributes in template" },
        "attributeSortOrder": { "type": "string", "enum": ["alphabetical", "as-written"], "default": "alphabetical", "description": "Sort order within attribute groups" },
        "mergeBindAndNonBindAttrs": { "type": "boolean", "default": false, "description": "Merge :xxx and xxx attributes for sorting" },
        "maxAttributesPerLine": { "type": "integer", "minimum": 1, "description": "Max attributes per line before wrapping" },
        "attributeGroups": { "type": "array", "items": { "type": "array", "items": { "type": "string" } }, "description": "Custom attribute sort groups (overrides Vue style guide order)" },
        "normalizeDirectiveShorthands": { "type": "boolean", "default": true, "description": "Normalize v-bind:/v-on:/v-slot: to :/@ /#" }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}"#;

#[cfg(all(test, feature = "glyph"))]
mod tests {
    use super::load_config;

    #[test]
    fn load_config_returns_defaults_when_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = load_config(Some(dir.path()));
        assert_eq!(config.fmt.print_width, 100);
        assert_eq!(config.fmt.tab_width, 2);
        assert!(!config.fmt.use_tabs);
        assert!(config.fmt.semi);
        assert!(!config.fmt.single_quote);
        assert!(config.fmt.sort_attributes);
        assert!(config.fmt.normalize_directive_shorthands);
    }

    #[test]
    fn load_config_parses_fmt_section() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.json");
        std::fs::write(
            &config_path,
            r#"{
                "fmt": {
                    "printWidth": 80,
                    "tabWidth": 4,
                    "useTabs": true,
                    "semi": false,
                    "singleQuote": true,
                    "sortAttributes": false,
                    "normalizeDirectiveShorthands": false
                }
            }"#,
        )
        .unwrap();

        let config = load_config(Some(dir.path()));
        assert_eq!(config.fmt.print_width, 80);
        assert_eq!(config.fmt.tab_width, 4);
        assert!(config.fmt.use_tabs);
        assert!(!config.fmt.semi);
        assert!(config.fmt.single_quote);
        assert!(!config.fmt.sort_attributes);
        assert!(!config.fmt.normalize_directive_shorthands);
    }

    #[test]
    fn load_config_partial_fmt_uses_defaults_for_missing() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.json");
        std::fs::write(&config_path, r#"{ "fmt": { "printWidth": 120 } }"#).unwrap();

        let config = load_config(Some(dir.path()));
        assert_eq!(config.fmt.print_width, 120);
        // defaults preserved
        assert_eq!(config.fmt.tab_width, 2);
        assert!(!config.fmt.use_tabs);
        assert!(config.fmt.semi);
    }

    #[test]
    fn load_config_returns_defaults_on_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.json");
        std::fs::write(&config_path, "not valid json {{{").unwrap();

        let config = load_config(Some(dir.path()));
        // should fall back to defaults
        assert_eq!(config.fmt.print_width, 100);
        assert_eq!(config.fmt.tab_width, 2);
    }

    #[test]
    fn load_config_with_check_and_fmt() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.json");
        std::fs::write(
            &config_path,
            r#"{
                "check": { "globals": "globals.d.ts", "servers": 6 },
                "fmt": { "singleQuote": true, "maxAttributesPerLine": 3 }
            }"#,
        )
        .unwrap();

        let config = load_config(Some(dir.path()));
        // check section
        let globals = config.check.globals.unwrap();
        assert_eq!(globals, "globals.d.ts");
        assert_eq!(config.check.servers, Some(6));
        // fmt section
        assert!(config.fmt.single_quote);
        assert_eq!(config.fmt.max_attributes_per_line, Some(3));
    }

    #[test]
    #[ignore = "requires pkl runtime installed"]
    fn load_config_parses_pkl() {
        let dir = tempfile::tempdir().unwrap();
        let pkl_path = dir.path().join("vize.config.pkl");
        std::fs::write(&pkl_path, "check {\n    globals = \"globals.d.ts\"\n}\n").unwrap();

        let config = load_config(Some(dir.path()));
        assert_eq!(config.check.globals.as_deref(), Some("globals.d.ts"));
    }
}

/// Write the JSON Schema to `node_modules/.vize/vize.config.schema.json`.
pub fn write_schema(dir: Option<&Path>) {
    let base = dir
        .map(|d| d.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let schema_dir = base.join("node_modules/.vize");
    if std::fs::create_dir_all(&schema_dir).is_ok() {
        let schema_path = schema_dir.join("vize.config.schema.json");
        let _ = std::fs::write(&schema_path, VIZE_CONFIG_SCHEMA);
    }
}
