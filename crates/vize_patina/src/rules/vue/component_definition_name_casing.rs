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
//! (`myComponent`), SCREAMING_CASE, and anything mixing a capital into a
//! hyphenated name (`my-Component`).
//!
//! ## Examples
//!
//! ### Invalid
//! ```text
//! myComponent.vue      -> should be MyComponent.vue or my-component.vue
//! my-Component.vue     -> should be MyComponent.vue or my-component.vue
//! ```
//!
//! ### Valid
//! ```text
//! MyComponent.vue
//! my-component.vue
//! index.vue
//! App.vue
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

fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let Some(first) = s.chars().next() else {
        return false;
    };
    if !first.is_ascii_uppercase() {
        return false;
    }
    // Must not contain hyphens
    if s.contains('-') {
        return false;
    }
    // Must not be all uppercase (SCREAMING_CASE)
    if s.chars()
        .all(|c| c.is_ascii_uppercase() || !c.is_alphabetic())
    {
        return false;
    }
    true
}

/// A hyphen-separated lowercase name: `my-component`, `job-board-2`.
///
/// Single-segment names never reach here — the caller returns early on an
/// all-lowercase stem — so a lone `-` boundary is what this actually decides.
fn is_kebab_case(s: &str) -> bool {
    if !s
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return false;
    }
    // A leading, trailing, or doubled hyphen leaves an empty segment.
    let mut segments = s.split('-');
    segments.next().is_some_and(|first| {
        first.starts_with(|c: char| c.is_ascii_lowercase())
            && segments.all(|segment| !segment.is_empty())
    })
}

fn is_nuxt_route_file(filename: &str) -> bool {
    filename
        .replace('\\', "/")
        .split('/')
        .any(|segment| segment == "pages")
}

/// Common exception filenames that don't need PascalCase
const EXCEPTION_NAMES: &[&str] = &["App"];

impl Rule for ComponentDefinitionNameCasing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let filename = ctx.filename;
        if !filename.ends_with(".vue") {
            return;
        }

        if is_nuxt_route_file(filename) {
            return;
        }

        // Extract the stem name (without extension and path)
        let stem = filename
            .rsplit('/')
            .next()
            .unwrap_or(filename)
            .rsplit('\\')
            .next()
            .unwrap_or(filename)
            .trim_end_matches(".vue");

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

    #[test]
    fn test_valid_non_vue_file() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Content</div>"#, "test.html");
        assert_eq!(result.warning_count, 0);
    }
}
