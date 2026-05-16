//! Type checking service for Vue SFC files.
//!
//! Integrates vize_vitrine's strict type checker with the LSP server.
//! Uses croquis for semantic analysis and provides comprehensive type diagnostics.
//! Also supports batch type checking via the Corsa CLI.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

mod diagnostics;
mod type_context;

use super::IdeContext;

/// Batch type check result summary.
#[cfg(feature = "native")]
#[derive(Debug, Clone, Default)]
pub struct BatchTypeCheckSummary {
    /// Total number of files checked.
    pub file_count: usize,
    /// Number of errors.
    pub error_count: usize,
    /// Number of warnings.
    pub warning_count: usize,
    /// Whether the check succeeded (exit code 0).
    pub success: bool,
}

/// Type checking options for LSP.
#[derive(Debug, Clone)]
pub struct LspTypeCheckOptions {
    /// Enable strict mode (treats warnings as errors)
    pub strict: bool,
    /// Check props type definitions
    pub check_props: bool,
    /// Check emits type definitions
    pub check_emits: bool,
    /// Check template bindings
    pub check_template_bindings: bool,
    /// Check reactivity loss patterns
    pub check_reactivity: bool,
    /// Check setup context violations
    pub check_setup_context: bool,
    /// Check invalid exports in `<script setup>`
    pub check_invalid_exports: bool,
    /// Check fallthrough attrs with multi-root
    pub check_fallthrough_attrs: bool,
}

impl Default for LspTypeCheckOptions {
    fn default() -> Self {
        Self {
            strict: true, // Strict by default for IDE integration
            check_props: true,
            check_emits: true,
            check_template_bindings: true,
            check_reactivity: true,
            check_setup_context: true,
            check_invalid_exports: true,
            check_fallthrough_attrs: true,
        }
    }
}

/// Type checking service for providing type diagnostics and information.
pub struct TypeService;

impl TypeService {
    /// Get type information at a specific position.
    pub fn get_type_at(ctx: &IdeContext) -> Option<vize_canon::TypeInfo> {
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        };

        let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, options).ok()?;
        let template = descriptor.template.as_ref()?;

        // Check if offset is in template
        let template_start = template.loc.start;
        let template_end = template.loc.end;

        if ctx.offset < template_start || ctx.offset > template_end {
            return None;
        }

        // Convert SFC offset to template-relative offset
        let template_offset = ctx.offset - template_start;

        // Build type context
        let type_ctx = Self::build_type_context(&descriptor);

        // Get type at position
        let checker = vize_canon::TypeChecker::new();
        checker.get_type_at(&template.content, template_offset as u32, &type_ctx)
    }

    /// Get type-aware completions.
    pub fn get_completions(ctx: &IdeContext) -> Vec<vize_canon::CompletionItem> {
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
            return vec![];
        };

        let Some(ref template) = descriptor.template else {
            return vec![];
        };

        // Check if offset is in template
        let template_start = template.loc.start;
        let template_end = template.loc.end;

        if ctx.offset < template_start || ctx.offset > template_end {
            return vec![];
        }

        let template_offset = ctx.offset - template_start;

        // Build type context
        let type_ctx = Self::build_type_context(&descriptor);

        // Get completions
        let checker = vize_canon::TypeChecker::new();
        checker.get_completions(&template.content, template_offset as u32, &type_ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::TypeService;

    #[test]
    fn test_extract_identifier() {
        assert_eq!(
            TypeService::extract_identifier("count = 0"),
            Some("count".to_string())
        );
        assert_eq!(
            TypeService::extract_identifier("_private"),
            Some("_private".to_string())
        );
        assert_eq!(
            TypeService::extract_identifier("$refs"),
            Some("$refs".to_string())
        );
        assert_eq!(TypeService::extract_identifier("123abc"), None);
    }

    #[test]
    fn test_infer_binding_type() {
        let t = TypeService::infer_binding_type("= \"hello\"", "msg");
        assert_eq!(t.display, "string");

        let t = TypeService::infer_binding_type("= 42", "count");
        assert_eq!(t.display, "number");

        let t = TypeService::infer_binding_type("= true", "flag");
        assert_eq!(t.display, "boolean");

        let t = TypeService::infer_binding_type("= ref(0)", "count");
        assert_eq!(t.kind, vize_canon::TypeKind::Ref);
    }
}
