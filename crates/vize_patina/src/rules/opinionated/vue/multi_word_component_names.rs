//! vue/multi-word-component-names
//!
//! Require component names to be multi-word.
//!
//! This rule enforces that component names are always multi-word,
//! except for root `App` components. This prevents conflicts with
//! existing and future HTML elements, since all HTML elements are
//! a single word.
//!
//! This rule checks the explicit component `name` when one is declared and
//! otherwise falls back to the SFC filename. It does not inspect component
//! usages in the template. This matches eslint-plugin-vue.
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

mod scan;

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use scan::{NameFinding, Scan, scan_normal_script, scan_setup_script};
use vize_relief::RootNode;

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
            ignore: vec!["App", "app"],
        }
    }
}

impl MultiWordComponentNames {
    fn is_multi_word(name: &str) -> bool {
        kebab_case(name).split('-').count() > 1
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

    fn is_valid_name(&self, name: &str) -> bool {
        self.ignore
            .iter()
            .any(|ignore| *ignore == name || (is_pascal_case(ignore) && kebab_case(ignore) == name))
            || is_vue_builtin(name)
            || Self::is_multi_word(name)
    }

    fn report(&self, ctx: &mut LintContext<'_>, finding: NameFinding) {
        if self.is_valid_name(finding.name.as_str()) {
            return;
        }
        ctx.error_at_with_help(
            ctx.t_fmt(
                "vue/multi-word-component-names.message",
                &[("name", finding.name.as_str())],
            ),
            ByteRange::new(finding.start, finding.end),
            ctx.t("vue/multi-word-component-names.help"),
        );
    }

    fn report_filename(&self, ctx: &mut LintContext<'_>) {
        if ctx.filename.ends_with(".art.vue") {
            return;
        }
        let Some(name) = Self::extract_component_name(ctx.filename) else {
            return;
        };
        self.report(
            ctx,
            NameFinding {
                name: name.into(),
                start: 0,
                end: 0,
            },
        );
    }
}

impl Rule for MultiWordComponentNames {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let (Some(descriptor), Some(scripts)) = (ctx.sfc_descriptor(), ctx.sfc_scripts()) else {
            return;
        };
        let mut scan = Scan::default();
        if let Some(script) = descriptor.script.as_ref()
            && script.src.is_none()
            && let Some(parsed) = scripts.script()
        {
            scan_normal_script(parsed, script.loc.start as u32, &mut scan);
        }
        if let Some(script) = descriptor.script_setup.as_ref()
            && script.src.is_none()
            && let Some(parsed) = scripts.script_setup()
        {
            scan_setup_script(parsed, script.loc.start as u32, &mut scan);
        }
        for finding in scan.findings {
            self.report(ctx, finding);
        }
        if !scan.has_name && (scan.has_vue || !scan.has_non_vue_program) {
            self.report_filename(ctx);
        }
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        if ctx.sfc_descriptor().is_none() {
            self.report_filename(ctx);
        }
    }
}

fn kebab_case(value: &str) -> vize_l0::String {
    let dashed: Vec<char> = value.replace('_', "-").chars().collect();
    let mut out = vize_l0::String::with_capacity(dashed.len() + 4);
    for (index, ch) in dashed.iter().copied().enumerate() {
        if ch.is_ascii_uppercase()
            && index > 0
            && dashed
                .get(index - 1)
                .is_some_and(|previous| previous.is_ascii_alphanumeric() || *previous == '_')
        {
            out.push('-');
        }
        out.extend(ch.to_lowercase());
    }
    out
}

fn is_pascal_case(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|first| !first.is_ascii_lowercase())
        && !value.contains(['-', '_', ' '])
}

fn is_vue_builtin(name: &str) -> bool {
    matches!(
        name,
        "template"
            | "slot"
            | "component"
            | "Component"
            | "transition"
            | "Transition"
            | "transition-group"
            | "TransitionGroup"
            | "keep-alive"
            | "KeepAlive"
            | "teleport"
            | "Teleport"
            | "suspense"
            | "Suspense"
    )
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
        insta::assert_debug_snapshot!(result.diagnostics);
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
    fn test_reports_index_files_like_upstream() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>hello</div>"#, "index.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_accepts_camel_case_multi_word_name() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div />"#, "todoItem.vue");
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

    #[test]
    fn sfc_filename_fallback_reports_the_program_start() {
        let result = create_linter().lint_sfc("<template><div /></template>", "Item.vue");
        assert_eq!(result.error_count, 1);
        assert_eq!(
            (result.diagnostics[0].start, result.diagnostics[0].end),
            (0, 0)
        );
    }

    #[test]
    fn explicit_options_name_owns_the_diagnostic_and_suppresses_the_filename() {
        let source = "<script>export default { name: 'Item' }</script><template><div /></template>";
        let result = create_linter().lint_sfc(source, "SingleFile.vue");
        let start = source.find("'Item'").unwrap() as u32;
        assert_eq!(result.error_count, 1);
        assert_eq!(
            (result.diagnostics[0].start, result.diagnostics[0].end),
            (start, start + 6)
        );

        let valid = source.replace("'Item'", "'TodoItem'");
        assert_eq!(create_linter().lint_sfc(&valid, "Item.vue").error_count, 0);
    }

    #[test]
    fn define_options_and_dynamic_names_follow_upstream_fallback_rules() {
        let source =
            "<script setup>defineOptions({ name: 'Item' })</script><template><div /></template>";
        let result = create_linter().lint_sfc(source, "MultiWord.vue");
        let start = source.find("'Item'").unwrap() as u32;
        assert_eq!(result.error_count, 1);
        assert_eq!(result.diagnostics[0].start, start);

        let dynamic = "<script setup>defineOptions({ name: componentName })</script><template><div /></template>";
        assert_eq!(create_linter().lint_sfc(dynamic, "Item.vue").error_count, 0);
    }

    #[test]
    fn ordinary_non_component_program_does_not_infer_a_component_name() {
        let source = "<script>console.log('utility')</script><template><div /></template>";
        assert_eq!(create_linter().lint_sfc(source, "Item.vue").error_count, 0);
    }
}
