use vize_atelier_sfc::vite_plugin::{
    transform_css_vars_for_pipeline, transform_css_vars_for_pipeline_with_context,
};

const SOURCE: &str = include_str!("fixtures/fix-history/production-css-var-scanner.input.txt");

#[test]
fn production_pipeline_changes_only_authored_bindings() {
    // The four hashes are independently produced by Vue 3.5.40 compileStyle,
    // with id=scope and isProd=true. Literal/comment/string bytes stay authored.
    let expected = concat!(
        "/* v-bind(ignored); var(--scope-color) */\n",
        ".box {\n",
        "  content: \"v-bind(ignored) var(--scope-color)\";\n",
        "  --literal: var(--scope-color);\n",
        "  color: var(--v19ca5605);\n",
        "  border-color: var(--v07468a12);\n",
        "  width: var(--f7a4f774);\n",
        "  --unicode: var(--ef1742e8);\n",
        "}\n",
    );
    for scope in ["scope", "data-v-scope"] {
        for filename in [
            "/work/a/src/Box.vue",
            "/work/very-different-folder/src/Box.vue",
        ] {
            assert_eq!(
                transform_css_vars_for_pipeline_with_context(SOURCE, scope, Some(filename))
                    .as_str(),
                expected,
            );
        }
    }
    assert_ne!(
        transform_css_vars_for_pipeline_with_context(SOURCE, "other-scope", Some("/work/a.vue"))
            .as_str(),
        expected,
        "a different component scope must retain a different identity"
    );
}

#[test]
fn default_pipeline_and_malformed_tail_keep_complete_original_bytes() {
    let expected = concat!(
        "/* v-bind(ignored); var(--scope-color) */\n",
        ".box {\n",
        "  content: \"v-bind(ignored) var(--scope-color)\";\n",
        "  --literal: var(--scope-color);\n",
        "  color: var(--scope-color);\n",
        "  border-color: var(--scope-backgroundColor);\n",
        "  width: var(--f7a4f774);\n",
        "  --unicode: var(--ef1742e8);\n",
        "}\n",
    );
    for scope in ["scope", "data-v-scope"] {
        assert_eq!(
            transform_css_vars_for_pipeline(SOURCE, scope).as_str(),
            expected
        );
        assert_eq!(
            transform_css_vars_for_pipeline_with_context(SOURCE, scope, None).as_str(),
            expected,
        );
        for filename in [None, Some("/project/雪.vue")] {
            assert_eq!(
                transform_css_vars_for_pipeline_with_context(
                    ".box { color: v-bind(unclosed; }",
                    scope,
                    filename,
                )
                .as_str(),
                ".box { color: v-bind(unclosed; }",
            );
        }
    }
}
