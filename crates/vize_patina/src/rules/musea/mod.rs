//! Musea lint rules for Art files (*.art.vue).
//!
//! These rules validate Art files used by the Musea component gallery
//! system (vize_musea).
//!
//! Art files have a specific structure:
//! - `<art>` block with metadata (title, component, etc.)
//! - `<variant>` blocks defining component variations
//! - Optional script and style blocks
//!
//! ## Performance
//!
//! The `MuseaLinter` uses single-pass scanning with SIMD-accelerated
//! pattern matching (memchr) for optimal performance.

mod no_empty_variant;
pub mod prefer_design_tokens;
mod require_component;
mod require_title;
mod unique_variant_names;
mod valid_variant;

pub use no_empty_variant::NoEmptyVariant;
pub use prefer_design_tokens::{PreferDesignTokens, PreferDesignTokensConfig};
pub use require_component::RequireComponent;
pub use require_title::RequireTitle;
pub use unique_variant_names::UniqueVariantNames;
pub use valid_variant::ValidVariant;

use memchr::memmem;
use vize_carton::FxHashSet;

use crate::diagnostic::{LintDiagnostic, Severity};

/// Musea Art file lint result
#[derive(Debug, Clone, Default)]
pub struct MuseaLintResult {
    /// Collected diagnostics
    pub diagnostics: Vec<LintDiagnostic>,
    /// Error count
    pub error_count: usize,
    /// Warning count
    pub warning_count: usize,
}

impl MuseaLintResult {
    /// Check if there are errors
    #[inline]
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Check if there are any diagnostics
    #[inline]
    pub fn has_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    /// Add a diagnostic
    #[inline]
    pub fn add_diagnostic(&mut self, diagnostic: LintDiagnostic) {
        match diagnostic.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
        }
        self.diagnostics.push(diagnostic);
    }
}

/// Musea rule metadata
pub struct MuseaRuleMeta {
    /// Rule name (e.g., "musea/require-title")
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Default severity
    pub default_severity: Severity,
}

/// Trait for Musea Art file lint rules
pub trait MuseaRule: Send + Sync {
    /// Get rule metadata
    fn meta(&self) -> &'static MuseaRuleMeta;

    /// Check the Art file source
    fn check(&self, source: &str, result: &mut MuseaLintResult);
}

/// High-performance Musea Art file linter using single-pass scanning
pub struct MuseaLinter {
    /// Whether to check require-title rule
    pub check_require_title: bool,
    /// Whether to check require-component rule
    pub check_require_component: bool,
    /// Whether to check valid-variant rule
    pub check_valid_variant: bool,
    /// Whether to check no-empty-variant rule
    pub check_no_empty_variant: bool,
    /// Whether to check unique-variant-names rule
    pub check_unique_variant_names: bool,
    /// Optional design token checker for prefer-design-tokens rule
    prefer_design_tokens: Option<PreferDesignTokens>,
}

impl MuseaLinter {
    /// Create a new Musea linter with all rules enabled
    #[inline]
    pub fn new() -> Self {
        Self {
            check_require_title: true,
            check_require_component: true,
            check_valid_variant: true,
            check_no_empty_variant: true,
            check_unique_variant_names: true,
            prefer_design_tokens: None,
        }
    }

    /// Set design token configuration for prefer-design-tokens rule
    #[inline]
    pub fn with_design_tokens(mut self, config: PreferDesignTokensConfig) -> Self {
        self.prefer_design_tokens = Some(PreferDesignTokens::new(config));
        self
    }

    /// Lint an Art file source using optimized single-pass scanning
    pub fn lint(&self, source: &str) -> MuseaLintResult {
        let mut result = MuseaLintResult::default();
        let bytes = source.as_bytes();

        // Phase 1: Check <art> block (single scan)
        self.check_art_block(bytes, &mut result);

        // Phase 2: Check <variant> blocks (single scan for all variant rules)
        self.check_variant_blocks(bytes, &mut result);

        // Phase 3: Check <style> blocks for hardcoded token values
        if let Some(ref token_rule) = self.prefer_design_tokens {
            token_rule.check(source, &mut result);
        }

        result
    }

    /// Check <art> block for required attributes
    #[inline]
    fn check_art_block(&self, bytes: &[u8], result: &mut MuseaLintResult) {
        // Find <art tag
        let Some(art_start) = memmem::find(bytes, b"<art") else {
            return;
        };

        // Find the end of the opening tag
        let Some(tag_end) = memchr::memchr(b'>', &bytes[art_start..]) else {
            return;
        };

        let art_tag = &bytes[art_start..art_start + tag_end];

        // Check for title attribute
        if self.check_require_title && !has_attribute(art_tag, b"title=") {
            result.add_diagnostic(
                LintDiagnostic::error(
                    "musea/require-title",
                    "Missing required 'title' attribute in <art> block",
                    art_start as u32,
                    (art_start + tag_end) as u32,
                )
                .with_help("Add a title attribute: <art title=\"Component Name\">"),
            );
        }

        // Check for component attribute
        if self.check_require_component && !has_attribute(art_tag, b"component=") {
            result.add_diagnostic(
                LintDiagnostic::warn(
                    "musea/require-component",
                    "Missing 'component' attribute in <art> block",
                    art_start as u32,
                    (art_start + tag_end) as u32,
                )
                .with_help("Add component=\"./Component.vue\""),
            );
        }
    }

    /// Check all <variant> blocks in a single pass
    fn check_variant_blocks(&self, bytes: &[u8], result: &mut MuseaLintResult) {
        let variant_finder = memmem::Finder::new(b"<variant");
        let mut search_start = 0;
        let mut seen_names: FxHashSet<&[u8]> = FxHashSet::default();

        while let Some(variant_pos) = variant_finder.find(&bytes[search_start..]) {
            let abs_pos = search_start + variant_pos;
            let remaining = &bytes[abs_pos..];

            // Find the end of the opening tag
            let Some(tag_end) = memchr::memchr(b'>', remaining) else {
                break;
            };

            let variant_tag = &remaining[..tag_end];

            // Check for self-closing tag
            let is_self_closing = tag_end > 0 && remaining[tag_end - 1] == b'/';

            // Check valid-variant: name attribute required
            let name_value = extract_name_attr_bytes(variant_tag);

            if self.check_valid_variant && name_value.is_none() {
                result.add_diagnostic(
                    LintDiagnostic::error(
                        "musea/valid-variant",
                        "Missing required 'name' attribute in <variant> block",
                        abs_pos as u32,
                        (abs_pos + tag_end) as u32,
                    )
                    .with_help("Add name=\"variant-name\" to the variant"),
                );
            }

            // Check unique-variant-names
            if let Some(name) = name_value {
                if self.check_unique_variant_names {
                    if seen_names.contains(name) {
                        result.add_diagnostic(
                            LintDiagnostic::error(
                                "musea/unique-variant-names",
                                "Duplicate variant name",
                                abs_pos as u32,
                                (abs_pos + tag_end) as u32,
                            )
                            .with_help("Use a unique name for each variant"),
                        );
                    } else {
                        seen_names.insert(name);
                    }
                }
            }

            // Check no-empty-variant
            if self.check_no_empty_variant {
                if is_self_closing {
                    result.add_diagnostic(
                        LintDiagnostic::warn(
                            "musea/no-empty-variant",
                            "Empty self-closing <variant /> block",
                            abs_pos as u32,
                            (abs_pos + tag_end + 1) as u32,
                        )
                        .with_help("Add template content inside the variant"),
                    );
                    search_start = abs_pos + tag_end + 1;
                    continue;
                }

                // Find the closing tag
                let after_open = &remaining[tag_end + 1..];
                if let Some(close_pos) = memmem::find(after_open, b"</variant>") {
                    let content = &after_open[..close_pos];
                    if is_whitespace_only(content) {
                        result.add_diagnostic(
                            LintDiagnostic::warn(
                                "musea/no-empty-variant",
                                "Empty <variant> block with no content",
                                abs_pos as u32,
                                (abs_pos + tag_end + 1 + close_pos + 10) as u32,
                            )
                            .with_help("Add template content inside the variant"),
                        );
                    }
                    search_start = abs_pos + tag_end + 1 + close_pos + 10;
                } else {
                    break;
                }
            } else {
                search_start = abs_pos + tag_end + 1;
            }
        }
    }
}

impl Default for MuseaLinter {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a tag has an attribute (fast byte-level check)
#[inline]
fn has_attribute(tag: &[u8], attr: &[u8]) -> bool {
    memmem::find(tag, attr).is_some()
}

/// Extract the value of the name attribute from a tag (byte-level)
#[inline]
fn extract_name_attr_bytes(tag: &[u8]) -> Option<&[u8]> {
    // Find name=" or name='
    let name_pos = memmem::find(tag, b"name=")?;
    let after_eq = &tag[name_pos + 5..];

    // Skip whitespace
    let mut i = 0;
    while i < after_eq.len() && after_eq[i].is_ascii_whitespace() {
        i += 1;
    }

    if i >= after_eq.len() {
        return None;
    }

    let quote = after_eq[i];
    if quote != b'"' && quote != b'\'' {
        return None;
    }

    let after_quote = &after_eq[i + 1..];
    let end_quote = memchr::memchr(quote, after_quote)?;

    Some(&after_quote[..end_quote])
}

/// Check if bytes contain only whitespace
#[inline]
fn is_whitespace_only(bytes: &[u8]) -> bool {
    bytes.iter().all(|b| b.is_ascii_whitespace())
}

#[cfg(test)]
mod tests {
    use super::{extract_name_attr_bytes, MuseaLinter};

    #[test]
    fn test_lint_valid_art_file() {
        let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="default">
    <Button>Click me</Button>
  </variant>
</art>
"#;
        let linter = MuseaLinter::new();
        let result = linter.lint(source);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_lint_missing_title() {
        let source = r#"
<art component="./Button.vue">
  <variant name="default">
    <Button>Click me</Button>
  </variant>
</art>
"#;
        let linter = MuseaLinter::new();
        let result = linter.lint(source);
        assert!(result.has_errors());
    }

    #[test]
    fn test_lint_duplicate_variant_names() {
        let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="same">
    <Button>One</Button>
  </variant>
  <variant name="same">
    <Button>Two</Button>
  </variant>
</art>
"#;
        let linter = MuseaLinter::new();
        let result = linter.lint(source);
        assert!(result.has_errors());
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_lint_empty_variant() {
        let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="empty"></variant>
</art>
"#;
        let linter = MuseaLinter::new();
        let result = linter.lint(source);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_extract_name_attr() {
        assert_eq!(
            extract_name_attr_bytes(b"<variant name=\"test\""),
            Some(b"test".as_slice())
        );
        assert_eq!(
            extract_name_attr_bytes(b"<variant name='test'"),
            Some(b"test".as_slice())
        );
        assert_eq!(extract_name_attr_bytes(b"<variant "), None);
    }
}
