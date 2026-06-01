//! Type checker for Vue SFC templates.

mod reporting;
pub(crate) mod runner;

pub use runner::TypeChecker;

#[cfg(test)]
mod tests {
    use super::TypeChecker;
    use crate::context::{Binding, BindingKind, TypeContext};
    use crate::types::{TypeInfo, TypeKind};

    fn create_test_context() -> TypeContext {
        let mut ctx = TypeContext::new();
        ctx.add_binding(
            "count",
            Binding::new(
                "count",
                TypeInfo::new("Ref<number>", TypeKind::Ref),
                BindingKind::Ref,
            ),
        );
        ctx.add_binding(
            "message",
            Binding::new("message", TypeInfo::string(), BindingKind::Const),
        );
        ctx.add_binding(
            "handleClick",
            Binding::new(
                "handleClick",
                TypeInfo::new("() => void", TypeKind::Function),
                BindingKind::Function,
            ),
        );
        ctx
    }

    #[test]
    fn test_check_interpolation() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<div>{{ count }}</div>"#;

        let result = checker.check_template(template, &ctx);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_check_unknown_identifier() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<div>{{ unknownVar }}</div>"#;

        let result = checker.check_template(template, &ctx);
        assert!(result.has_errors());
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_check_directive() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<div v-if="count > 0">visible</div>"#;

        let result = checker.check_template(template, &ctx);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_check_event_handler() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<button @click="handleClick">Click</button>"#;

        let result = checker.check_template(template, &ctx);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_check_vbind() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<input :value="message" />"#;

        let result = checker.check_template(template, &ctx);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_extract_identifiers() {
        let ids = TypeChecker::extract_identifiers("count + message.length");
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0].0, "count");
        assert_eq!(ids[1].0, "message");
        assert_eq!(ids[2].0, "length");
    }

    #[test]
    fn test_is_simple_identifier() {
        assert!(TypeChecker::is_simple_identifier("foo"));
        assert!(TypeChecker::is_simple_identifier("_bar"));
        assert!(TypeChecker::is_simple_identifier("$baz"));
        assert!(!TypeChecker::is_simple_identifier("foo.bar"));
        assert!(!TypeChecker::is_simple_identifier("foo[0]"));
        assert!(!TypeChecker::is_simple_identifier("123"));
    }

    #[test]
    fn test_get_completions() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<div>{{ }}</div>"#;

        let completions = checker.get_completions(template, 8, &ctx);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "count"));
        assert!(completions.iter().any(|c| c.label == "message"));
    }

    #[test]
    fn test_check_multiple_interpolations() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<div>{{ count }} - {{ message }}</div>"#;

        let result = checker.check_template(template, &ctx);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_check_multiple_unknown_identifiers() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<div>{{ unknownA }} {{ unknownB }}</div>"#;

        let result = checker.check_template(template, &ctx);
        assert!(result.has_errors());
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_check_keyword_not_flagged() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        // Keywords and literals should not be flagged as unknown
        let template = r#"<div>{{ true }}</div>"#;

        let result = checker.check_template(template, &ctx);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_check_ternary_expression() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        // Note: checker's simple extract_identifiers doesn't skip string literals,
        // so use identifiers that are in context
        let template = r#"<div>{{ count > 0 ? message : count }}</div>"#;

        let result = checker.check_template(template, &ctx);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_extract_identifiers_with_property_access() {
        // extract_identifiers extracts ALL identifier-like tokens including after dots
        let ids = TypeChecker::extract_identifiers("message.length");
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0].0, "message");
        assert_eq!(ids[1].0, "length");
    }

    #[test]
    fn test_extract_identifiers_complex() {
        let ids = TypeChecker::extract_identifiers("a + b * c");
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0].0, "a");
        assert_eq!(ids[1].0, "b");
        assert_eq!(ids[2].0, "c");
    }

    #[test]
    fn test_extract_identifiers_empty() {
        let ids = TypeChecker::extract_identifiers("");
        assert!(ids.is_empty());
    }

    #[test]
    fn test_extract_identifiers_numeric() {
        let ids = TypeChecker::extract_identifiers("42");
        // "42" starts with digit, not an ident start
        assert!(ids.is_empty() || ids.iter().all(|(name, _)| name.parse::<i32>().is_ok()));
    }

    #[test]
    fn test_is_simple_identifier_edge_cases() {
        assert!(!TypeChecker::is_simple_identifier(""));
        assert!(TypeChecker::is_simple_identifier("a"));
        assert!(TypeChecker::is_simple_identifier("$"));
        assert!(TypeChecker::is_simple_identifier("_"));
        assert!(!TypeChecker::is_simple_identifier("a b"));
        assert!(!TypeChecker::is_simple_identifier("a.b.c"));
    }

    #[test]
    fn test_check_v_bind_shorthand() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<div :class="message">content</div>"#;

        let result = checker.check_template(template, &ctx);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_check_v_on_shorthand() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<button @click="handleClick()">Click</button>"#;

        let result = checker.check_template(template, &ctx);
        assert!(!result.has_errors());
    }
}
