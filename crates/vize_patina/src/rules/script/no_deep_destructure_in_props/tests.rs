    use super::NoDeepDestructureInProps;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoDeepDestructureInProps::default()));
        linter
    }

    #[test]
    fn test_valid_simple_destructure() {
        let linter = create_linter();
        let result = linter.lint(
            "const { name, count = 0 } = defineProps<{ name: string }>()",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_destructure() {
        let linter = create_linter();
        let result = linter.lint("const props = defineProps<{ name: string }>()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_deep_destructure() {
        let linter = create_linter();
        let result = linter.lint(
            "const { user: { name } } = defineProps<{ user: User }>()",
            0,
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_very_deep_destructure() {
        let linter = create_linter();
        let result = linter.lint(
            "const /* props */ { config: { settings: { theme } } } = defineProps()",
            0,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_deep_destructure_with_defaults() {
        let linter = create_linter();
        let result = linter.lint(
            "const { user: { name } } = withDefaults(defineProps<{ user: User }>(), {})",
            0,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_with_higher_max_depth() {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoDeepDestructureInProps { max_depth: 2 }));
        let result = linter.lint(
            "const { user: { name } } = defineProps<{ user: User }>()",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_define_props_string_not_matched() {
        let linter = create_linter();
        let result = linter.lint(
            r#"const text = "const { user: { name } } = defineProps()""#,
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_unrelated_deep_destructure_not_matched() {
        let linter = create_linter();
        let result = linter.lint(
            "const { user: { name } } = createProps<{ user: User }>()",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }
