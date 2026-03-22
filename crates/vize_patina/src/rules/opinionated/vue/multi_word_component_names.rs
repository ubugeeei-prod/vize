//! vue/multi-word-component-names
//!
//! Require component names to be multi-word.
//!
//! This rule enforces that component names are always multi-word,
//! except for root `App` components. This prevents conflicts with
//! existing and future HTML elements, since all HTML elements are
//! a single word.
//!
//! This rule checks the **component definition** (filename), NOT the
//! names of other components used in the template. This matches the
//! behavior of eslint-plugin-vue.
//!
//! ## Examples
//!
//! ### Invalid (filename)
//! ```text
//! Item.vue
//! Table.vue
//! ```
//!
//! ### Valid (filename)
//! ```text
//! TodoItem.vue
//! DataTable.vue
//! AppHeader.vue
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::RootNode;

static META: RuleMeta = RuleMeta {
    name: "vue/multi-word-component-names",
    description: "Require component names to be multi-word",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Require component names to be multi-word
pub struct MultiWordComponentNames {
    /// Component names to ignore (e.g., "App", "Nuxt")
    pub ignore: Vec<&'static str>,
}

impl Default for MultiWordComponentNames {
    fn default() -> Self {
        Self {
            ignore: vec!["App", "Nuxt", "NuxtPage", "NuxtLayout"],
        }
    }
}

impl MultiWordComponentNames {
    fn is_multi_word(name: &str) -> bool {
        // kebab-case: check for hyphen
        if name.contains('-') {
            return true;
        }
        // PascalCase: count uppercase letters (at least 2 means multi-word)
        let uppercase_count = name.chars().filter(|c| c.is_uppercase()).count();
        uppercase_count >= 2
    }

    /// Extract the component name from a filename.
    /// e.g., "MyComponent.vue" → "MyComponent", "pages/index.vue" → "index"
    fn extract_component_name(filename: &str) -> Option<&str> {
        // Get the file stem (without extension and path)
        let basename = filename.rsplit('/').next().unwrap_or(filename);
        let basename = basename.rsplit('\\').next().unwrap_or(basename);
        // Remove .vue extension
        basename.strip_suffix(".vue")
    }
}

impl Rule for MultiWordComponentNames {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let filename = ctx.filename;

        // Only check .vue files
        let Some(component_name) = Self::extract_component_name(filename) else {
            return;
        };

        // Skip ignored components
        if self.ignore.contains(&component_name) {
            return;
        }

        // Skip index files (common in file-based routing)
        if component_name == "index" || component_name == "Index" {
            return;
        }

        // Skip filenames that start with lowercase (likely page routes, not components)
        // e.g., pages/about.vue, pages/login.vue
        if component_name
            .chars()
            .next()
            .map(|c| c.is_lowercase())
            .unwrap_or(false)
        {
            return;
        }

        // Skip filenames with brackets (dynamic routes like [id].vue, [...slug].vue)
        if component_name.starts_with('[') || component_name.starts_with('_') {
            return;
        }

        // Check if the component name is multi-word
        if !Self::is_multi_word(component_name) {
            ctx.error_with_help(
                ctx.t("vue/multi-word-component-names.message"),
                &root.loc,
                ctx.t("vue/multi-word-component-names.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MultiWordComponentNames;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(MultiWordComponentNames::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_multi_word_pascal_case_filename() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>hello</div>"#, "TodoItem.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_multi_word_kebab_case_filename() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>hello</div>"#, "todo-item.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_ignored_app() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>hello</div>"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_single_word_pascal_case() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>hello</div>"#, "Item.vue");
        assert_eq!(result.error_count, 1);
        assert!(result.diagnostics[0].message.contains("multi-word"));
    }

    #[test]
    fn test_does_not_flag_component_usage_in_template() {
        let linter = create_linter();
        // Using single-word component <Mfm> in template should NOT trigger
        let result = linter.lint_template(r#"<Mfm :text="text" />"#, "MyComponent.vue");
        assert_eq!(
            result.error_count, 0,
            "Should not flag component usage in template"
        );
    }

    #[test]
    fn test_skips_index_files() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>hello</div>"#, "index.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_skips_non_vue_files() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>hello</div>"#, "test.html");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_three_words() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>hello</div>"#, "UserProfileCard.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_filename_with_path() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>hello</div>"#, "src/components/Item.vue");
        assert_eq!(result.error_count, 1);
    }
}
