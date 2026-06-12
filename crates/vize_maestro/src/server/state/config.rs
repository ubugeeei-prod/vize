//! Workspace/LSP config loading and feature application.

use std::path::Path;
use std::sync::atomic::Ordering;

use vize_carton::config::{LinterConfig, TypeCheckerConfig};

#[cfg(feature = "glyph")]
use vize_carton::config::FormatterConfig;

use super::ServerState;
use super::features::LspConfigSection;

impl ServerState {
    /// Apply LSP initialization options sent by an editor client.
    pub fn apply_lsp_initialization_options(&self, options: Option<&serde_json::Value>) {
        let Some(options) = options else {
            return;
        };

        match serde_json::from_value::<LspConfigSection>(options.clone()) {
            Ok(config) => self.apply_lsp_config(config, "initializationOptions"),
            Err(error) => {
                tracing::warn!(
                    "Failed to parse LSP initializationOptions: {}. Keeping current LSP options.",
                    error
                );
            }
        }
    }

    /// Get a clone of the current type checker config.
    #[inline]
    pub fn get_type_checker_config(&self) -> TypeCheckerConfig {
        self.type_checker_config.read().clone()
    }

    /// Get a clone of the current linter config.
    #[inline]
    pub fn get_linter_config(&self) -> LinterConfig {
        self.linter_config.read().clone()
    }

    /// Get the configured Vue dialect override, if any.
    #[inline]
    pub fn get_dialect_config(&self) -> Option<vize_carton::dialect::VueDialect> {
        *self.dialect_config.read()
    }

    /// Set the Vue dialect override (`None` re-enables structural detection).
    #[inline]
    pub fn set_dialect_config(&self, dialect: Option<vize_carton::dialect::VueDialect>) {
        *self.dialect_config.write() = dialect;
    }

    fn apply_type_checker_config(&self, config: TypeCheckerConfig, source: &str) {
        *self.type_checker_config.write() = config;
        tracing::info!("Loaded type checker config from {}", source);
    }

    fn apply_config_features(&self, features: vize_carton::config::ConfigFeatureFlags) {
        *self.type_checker_options_api.write() = features.type_checker_options_api;
        *self.type_checker_legacy_vue2.write() = features.type_checker_legacy_vue2;
        *self.type_checker_jsx_typecheck.write() = features.type_checker_jsx_typecheck;
        if let Some(enabled) = features.language_server_legacy_vue2 {
            let mut lsp_features = self.lsp_features.write();
            lsp_features.legacy_vue2 = enabled;
        }
    }

    fn apply_linter_config(&self, config: LinterConfig, source: &str) {
        *self.linter_config.write() = config;
        tracing::info!("Loaded linter config from {}", source);
    }

    fn apply_lsp_config(&self, config: LspConfigSection, source: &str) {
        let mut features = self.lsp_features.write();
        config.apply_to(&mut features);
        self.lsp_typecheck_enabled
            .store(features.typecheck, Ordering::SeqCst);
        tracing::info!("Loaded LSP config from {}: {:?}", source, *features);
    }

    /// Load all workspace-scoped options from `vize.config.pkl` (preferred) or JSON.
    pub fn load_workspace_config(&self, dir: &Path) {
        let (loaded, linter_config) =
            vize_carton::config::load_config_and_linter_with_features_and_source(Some(dir));
        if let Some(source_path) = loaded.source_path {
            let source = source_path.display().to_string();
            let config = loaded.config;
            #[cfg(feature = "glyph")]
            {
                *self.format_options.write() = format_options_from_config(&config.formatter);
                tracing::info!("Loaded format config from {}", source);
            }
            self.apply_linter_config(linter_config, &source);
            self.apply_type_checker_config(config.type_checker, &source);
            self.apply_lsp_config(config.language_server.into(), &source);
            self.apply_config_features(loaded.features);
            self.set_dialect_config(config.dialect);
        }
    }

    /// Load LSP options from `vize.config.pkl` (preferred) or `vize.config.json`.
    pub fn load_lsp_config(&self, dir: &Path) {
        let (loaded, linter_config) =
            vize_carton::config::load_config_and_linter_with_features_and_source(Some(dir));
        if let Some(source_path) = loaded.source_path {
            let source = source_path.display().to_string();
            self.apply_linter_config(linter_config, &source);
            self.apply_type_checker_config(loaded.config.type_checker, &source);
            self.apply_lsp_config(loaded.config.language_server.into(), &source);
            self.apply_config_features(loaded.features);
            self.set_dialect_config(loaded.config.dialect);
        }
    }

    /// Get a clone of the current format options.
    #[cfg(feature = "glyph")]
    #[inline]
    pub fn get_format_options(&self) -> vize_glyph::FormatOptions {
        self.format_options.read().clone()
    }

    /// Load format options from `vize.config.json` in the given directory.
    #[cfg(feature = "glyph")]
    pub fn load_format_config(&self, dir: &Path) {
        let loaded = vize_carton::config::load_config_with_source(Some(dir));
        if let Some(source_path) = loaded.source_path {
            *self.format_options.write() = format_options_from_config(&loaded.config.formatter);
            tracing::info!("Loaded format config from {}", source_path.display());
        }
    }
}

#[cfg(feature = "glyph")]
fn format_options_from_config(config: &FormatterConfig) -> vize_glyph::FormatOptions {
    vize_glyph::FormatOptions {
        print_width: config.print_width,
        tab_width: config.tab_width,
        use_tabs: config.use_tabs,
        semi: config.semi,
        single_quote: config.single_quote,
        jsx_single_quote: config.jsx_single_quote,
        trailing_comma: match config.trailing_comma {
            vize_carton::config::TrailingComma::None => vize_glyph::TrailingComma::None,
            vize_carton::config::TrailingComma::Es5 => vize_glyph::TrailingComma::Es5,
            vize_carton::config::TrailingComma::All => vize_glyph::TrailingComma::All,
        },
        bracket_spacing: config.bracket_spacing,
        bracket_same_line: config.bracket_same_line,
        arrow_parens: match config.arrow_parens {
            vize_carton::config::ArrowParens::Always => vize_glyph::ArrowParens::Always,
            vize_carton::config::ArrowParens::Avoid => vize_glyph::ArrowParens::Avoid,
        },
        end_of_line: match config.end_of_line {
            vize_carton::config::EndOfLine::Lf => vize_glyph::EndOfLine::Lf,
            vize_carton::config::EndOfLine::Crlf => vize_glyph::EndOfLine::Crlf,
            vize_carton::config::EndOfLine::Cr => vize_glyph::EndOfLine::Cr,
            vize_carton::config::EndOfLine::Auto => vize_glyph::EndOfLine::Auto,
        },
        quote_props: match config.quote_props {
            vize_carton::config::QuoteProps::AsNeeded => vize_glyph::QuoteProps::AsNeeded,
            vize_carton::config::QuoteProps::Consistent => vize_glyph::QuoteProps::Consistent,
            vize_carton::config::QuoteProps::Preserve => vize_glyph::QuoteProps::Preserve,
        },
        single_attribute_per_line: config.single_attribute_per_line,
        vue_indent_script_and_style: config.vue_indent_script_and_style,
        sort_attributes: config.sort_attributes,
        attribute_sort_order: match config.attribute_sort_order {
            vize_carton::config::AttributeSortOrder::Alphabetical => {
                vize_glyph::AttributeSortOrder::Alphabetical
            }
            vize_carton::config::AttributeSortOrder::AsWritten => {
                vize_glyph::AttributeSortOrder::AsWritten
            }
        },
        merge_bind_and_non_bind_attrs: config.merge_bind_and_non_bind_attrs,
        max_attributes_per_line: config.max_attributes_per_line,
        attribute_groups: config.attribute_groups.clone(),
        normalize_directive_shorthands: config.normalize_directive_shorthands,
        sort_blocks: config.sort_blocks,
    }
}
