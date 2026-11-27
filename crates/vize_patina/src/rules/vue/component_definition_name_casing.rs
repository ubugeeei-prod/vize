//! vue/component-definition-name-casing
//!
//! Enforce specific casing for the component definition name.
//!
//! Component file names use PascalCase or kebab-case; this checks the filename
//! of .vue files. Both are conventional in Vue, and a project that settles on
//! kebab-case for its single-file components is not doing anything the style
//! guide argues against, so neither is reported.
//!
//! Mixed shapes stay reported, because they are neither convention: camelCase
//! (`myComponent`), names with underscores, and anything mixing a capital into
//! a hyphenated name (`my-Component`).
//!
//! Nuxt's mode suffixes (`.client`, `.server`, `.global`, `.island`) are not
//! part of the component name, so they are removed before the check. Other
//! dots stay part of the name and are reported.
//!
//! ## Examples
//!
//! ### Invalid
//! ```text
//! myComponent.vue      -> should be MyComponent.vue or my-component.vue
//! my-Component.vue     -> should be MyComponent.vue or my-component.vue
//! myComponent.client.vue
//! page.block.vue
//! ```
//!
//! ### Valid
//! ```text
//! MyComponent.vue
//! UI.vue
//! my-component.vue
//! index.vue
//! App.vue
//! ClientBox.client.vue
//! ServerTime.server.vue
//! GlobalThing.global.vue
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::RootNode;

static META: RuleMeta = RuleMeta {
    name: "vue/component-definition-name-casing",
    description: "Enforce PascalCase or kebab-case for component definition names",
    category: RuleCategory::StronglyRecommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Enforce PascalCase component definition names
#[derive(Default)]
pub struct ComponentDefinitionNameCasing;

use crate::component_name::{
    EXCEPTION_NAMES, definition_name_stem, is_kebab_case, is_nuxt_route_file, is_pascal_case,
};

impl Rule for ComponentDefinitionNameCasing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn template_hooks(&self) -> crate::rule::RuleHooks {
        crate::rule::RuleHooks::TEMPLATE
    }

    fn as_native_template_rule(&self) -> Option<&dyn crate::native::template::NativeTemplateRule> {
        Some(self)
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let filename = ctx.filename;
        // Musea Art files describe variants, not component definitions.
        if !filename.ends_with(".vue") || filename.ends_with(".art.vue") {
            return;
        }

        if is_nuxt_route_file(filename) {
            return;
        }

        let stem = definition_name_stem(filename);

        // Skip exception names
        if EXCEPTION_NAMES.contains(&stem) {
            return;
        }

        // Skip names starting with [ (dynamic routes)
        if stem.starts_with('[') {
            return;
        }

        // Skip single-word lowercase names (index, test, app, main, etc.)
        if stem.chars().all(|c| c.is_ascii_lowercase()) {
            return;
        }

        if !is_pascal_case(stem) && !is_kebab_case(stem) {
            ctx.warn_with_help(
                ctx.t_fmt(
                    "vue/component-definition-name-casing.message",
                    &[("name", stem)],
                ),
                &root.loc,
                ctx.t("vue/component-definition-name-casing.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentDefinitionNameCasing;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ComponentDefinitionNameCasing));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_pascal_case() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "MyComponent.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_single_letter_and_initialisms() {
        let linter = create_linter();
        for stem in ["A", "UI", "Foo2", "TheH1Title"] {
            let result = linter.lint_template(r#"<div>Content</div>"#, &format!("{stem}.vue"));
            assert_eq!(result.warning_count, 0, "{stem} must be valid PascalCase");
        }
    }

    #[test]
    fn test_invalid_underscored_uppercase_name() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "MY_COMPONENT.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_index() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "index.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_app() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_kebab_case() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "my-component.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_nuxt_pages_route_kebab_case() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "pages/job-board.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_nuxt_app_pages_route_kebab_case() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "app/pages/job-board.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_nuxt_src_pages_route_kebab_case() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "src/pages/job-board.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_camel_case() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "myComponent.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_with_path() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div>Content</div>"#, "src/components/MyComponent.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_component_kebab_case_with_path() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "src/components/job-board.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_kebab_case_with_digits() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "grid-2-col.vue");
        assert_eq!(result.warning_count, 0);
    }

    // Neither convention: a capital inside a hyphenated name is not kebab-case,
    // and the hyphen keeps it out of PascalCase.
    #[test]
    fn test_invalid_mixed_kebab_and_pascal() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "my-Component.vue");
        assert_eq!(result.warning_count, 1);
    }

    // An empty segment leaves a name that is neither convention.
    #[test]
    fn test_invalid_hyphen_boundaries() {
        let linter = create_linter();
        for stem in ["-my-component", "my-component-", "my--component"] {
            let result = linter.lint_template(r#"<div>Content</div>"#, &format!("{stem}.vue"));
            assert_eq!(result.warning_count, 1, "{stem} must stay reported");
        }
    }

    // Still reported: dots are neither convention, and these appear in real
    // fixtures (`page.block.vue`).
    #[test]
    fn test_invalid_dotted_name() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "page.block.vue");
        assert_eq!(result.warning_count, 1);
    }

    // Nuxt registers `ClientBox.client.vue` as `ClientBox`; the mode suffix is
    // not part of the name.
    #[test]
    fn test_valid_nuxt_mode_suffixes() {
        let linter = create_linter();
        for filename in [
            "components/ClientBox.client.vue",
            "components/ServerTime.server.vue",
            "components/GlobalThing.global.vue",
            "components/IslandCard.island.vue",
            "components/Counter.client.global.vue",
            "components/Panel.server.island.global.vue",
            "components/account-menu.client.vue",
            "components/card/index.client.vue",
        ] {
            let result = linter.lint_template(r#"<div>Content</div>"#, filename);
            assert_eq!(result.warning_count, 0, "{filename} must be valid");
        }
    }

    // The stripped name is still checked, and is the name the message shows.
    #[test]
    fn test_invalid_name_before_nuxt_mode_suffix() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "myComponent.client.vue");
        let messages: Vec<&str> = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert_eq!(
            messages,
            ["Component file name 'myComponent' should be PascalCase or kebab-case"]
        );
    }

    // Only Nuxt's mode suffixes are removed, and only in Nuxt's order: a mode
    // after `.global`, a doubled mode, or any other dotted part stays reported.
    #[test]
    fn test_invalid_other_dotted_suffixes() {
        let linter = create_linter();
        for filename in [
            "Foo.bar.vue",
            "Foo.Client.vue",
            "Foo.global.client.vue",
            "Foo.client.client.vue",
            "Foo.client.bar.vue",
        ] {
            let result = linter.lint_template(r#"<div>Content</div>"#, filename);
            assert_eq!(result.warning_count, 1, "{filename} must stay reported");
        }
    }

    #[test]
    fn pinned_nuxt_mode_suffix_filenames() {
        let linter = create_linter();
        let source = include_str!(
            "../../../../../tests/_fixtures/differential/linter/component-definition-nuxt-mode-suffixes/Component.vue.txt"
        );
        let cases = include_str!(
            "../../../../../tests/_fixtures/differential/linter/component-definition-nuxt-mode-suffixes/filenames.txt"
        );
        let mut checked = 0;
        for line in cases.lines() {
            let (expected, filename) = line.split_once(' ').unwrap();
            let result = linter.lint_sfc(source, filename);
            assert_eq!(
                result.warning_count,
                expected.parse::<usize>().unwrap(),
                "{filename}"
            );
            checked += 1;
        }
        assert_eq!(checked, 11);
    }

    #[test]
    fn test_valid_non_vue_file() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "test.html");
        assert_eq!(result.warning_count, 0);
    }
}
