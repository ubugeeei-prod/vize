//! vue/match-component-file-name
//!
//! The component `name`, and the name passed to `app.component`, must match
//! the file stem. The default comparison is upstream's: `pascalCase(name)`
//! or `kebabCase(name)` equals the stem unchanged. `.vue` and `.jsx` are
//! checked. Several components in one file stay silent.

mod casing;
mod scan;

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use scan::{Scan, scan_block};

use vize_l0::{String, cstr};

const RULE: &str = "vue/match-component-file-name";
const HELP: &str = "Rename the component so its name matches the file name.";

static META: RuleMeta = RuleMeta {
    name: RULE,
    description: "Require a component name to match its file name",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Implements the `vue/match-component-file-name` lint rule.
pub struct MatchComponentFileName;

pub(super) fn message(name: &str, filename: &str) -> String {
    cstr!("Component name `{name}` should match file name `{filename}`.")
}

impl Rule for MatchComponentFileName {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let Some(stem) = checked_stem(ctx.filename) else {
            return;
        };
        let (Some(descriptor), Some(scripts)) = (ctx.sfc_descriptor(), ctx.sfc_scripts()) else {
            return;
        };

        let mut scan = Scan::default();
        if let Some(script) = descriptor.script.as_ref()
            && script.src.is_none()
            && !script.content.is_empty()
            && let Some(parsed) = scripts.script()
        {
            scan_block(script, parsed, false, stem, &mut scan);
        }
        if let Some(script) = descriptor.script_setup.as_ref()
            && script.src.is_none()
            && !script.content.is_empty()
            && let Some(parsed) = scripts.script_setup()
        {
            scan_block(script, parsed, true, stem, &mut scan);
        }
        if scan.count > 1 {
            return;
        }
        for miss in scan.misses {
            ctx.report(
                LintDiagnostic::warn(RULE, message(&miss.name, stem), miss.start, miss.end)
                    .with_help(HELP),
            );
        }
    }
}

fn checked_stem(filename: &str) -> Option<&str> {
    let base = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    let (stem, extension) = base.rsplit_once('.')?;
    if stem.is_empty() || !matches!(extension, "vue" | "jsx") {
        return None;
    }
    // Nuxt registers `Foo.client.vue` as `Foo`; a stem that is only a mode
    // suffix keeps its spelling.
    let name = crate::component_name::strip_nuxt_mode_suffixes(stem);
    Some(if name.is_empty() { stem } else { name })
}

#[cfg(test)]
mod tests {
    use super::{MatchComponentFileName, RULE, message};
    use crate::diagnostic::Severity;
    use crate::linter::Linter;
    use crate::preset::LintPreset;
    use crate::rule::{Rule, RuleRegistry};
    use vize_l0::String;

    fn lint(source: &str, filename: &str) -> crate::linter::LintResult {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(MatchComponentFileName));
        Linter::with_registry(registry).lint_sfc(source, filename)
    }

    #[test]
    fn reports_a_name_that_misses_the_pascal_case_stem() {
        let cases = [
            (
                "<script>\nexport default { name: 'Other' }\n</script>\n",
                "components/MyComponent.vue",
            ),
            (
                "<script>\nexport default defineComponent({ name: 'Other' })\n</script>\n",
                "MyComponent.vue",
            ),
            (
                "<script setup>\ndefineOptions({ name: `Other` })\n</script>\n",
                "MyComponent.vue",
            ),
            (
                "<script>\napp.component('Other', {})\n</script>\n",
                "MyComponent.vue",
            ),
        ];
        for (source, filename) in cases {
            let result = lint(source, filename);
            assert_eq!(result.warning_count, 1, "{filename}");
            assert_eq!(result.error_count, 0, "{filename}");
            assert_eq!(result.diagnostics[0].rule_name, RULE);
            assert_eq!(
                result.diagnostics[0].message.as_str(),
                message("Other", "MyComponent")
            );
        }
    }

    #[test]
    fn allows_the_upstream_default_case_and_multiple_components() {
        let silent = [
            (
                "<script>\nexport default { name: 'MyComponent' }\n</script>\n",
                "MyComponent.vue",
            ),
            (
                "<script>\nexport default { name: 'my-component' }\n</script>\n",
                "MyComponent.vue",
            ),
            (
                "<script>\nexport default { name: 'myComponent' }\n</script>\n",
                "src/MyComponent.vue",
            ),
            (
                "<script>\nexport default { name: 'my_component' }\n</script>\n",
                "MyComponent.vue",
            ),
            (
                "<script>\nexport default { name: 'MyComponent' }\n</script>\n",
                "my-component.vue",
            ),
            (
                "<script>\nexport default { render() { return null } }\n</script>\n",
                "MyComponent.vue",
            ),
            (
                "<script>\nexport default { name: 'Other' }\n</script>\n<script setup>\ndefineOptions({ name: 'AlsoOther' })\n</script>\n",
                "MyComponent.vue",
            ),
            (
                "<script>\nexport default { name: 'Other' }\n</script>\n",
                "MyComponent.js",
            ),
            (
                "<script setup>\ndefineOptions({ name: 'MyComponent' })\n</script>\n",
                "MyComponent.jsx",
            ),
        ];
        for (source, filename) in silent {
            let result = lint(source, filename);
            assert_eq!(result.warning_count, 0, "{filename}\n{source}");
            assert!(result.diagnostics.is_empty(), "{filename}");
        }
    }

    // Nuxt registers `SearchBox.client.vue` as `SearchBox`, so its mode
    // suffixes are not part of the compared stem.
    #[test]
    fn nuxt_mode_suffixes_are_not_part_of_the_file_name() {
        let source = include_str!(
            "../../../../../tests/_fixtures/differential/linter/component-name-nuxt-mode-suffixes/Named.vue.txt"
        );
        let cases = include_str!(
            "../../../../../tests/_fixtures/differential/linter/component-name-nuxt-mode-suffixes/match-component-file-name.txt"
        );
        let mut checked = 0;
        for line in cases.lines() {
            let (expected, filename) = line.split_once(' ').unwrap();
            let result = lint(source, filename);
            assert_eq!(
                result.warning_count,
                expected.parse::<usize>().unwrap(),
                "{filename}"
            );
            checked += 1;
        }
        assert_eq!(checked, 9);

        let result = lint(source, "app/components/Other.client.vue");
        assert_eq!(
            result.diagnostics[0].message.as_str(),
            message("SearchBox", "Other")
        );
        // A stem that is only a mode suffix keeps its spelling.
        let result = lint(source, ".client.vue");
        assert_eq!(
            result.diagnostics[0].message.as_str(),
            message("SearchBox", ".client")
        );
    }

    #[test]
    fn stays_opt_in() {
        assert!(!RuleRegistry::with_essential().has_rule(RULE));
        assert!(!RuleRegistry::with_happy_path().has_rule(RULE));
        assert!(RuleRegistry::with_opt_in_rules().has_rule(RULE));
        assert_eq!(
            MatchComponentFileName.meta().default_severity,
            Severity::Warning
        );

        let result = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![String::from(RULE)]))
            .lint_sfc(
                "<script>\nexport default { name: 'Other' }\n</script>\n",
                "MyComponent.vue",
            );
        let reports: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == RULE)
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert_eq!(reports, [message("Other", "MyComponent").as_str()]);
    }

    #[test]
    fn shared_jsx_parse_preserves_the_complete_name_diagnostic() {
        let source = "<script lang=\"jsx\">\nexport default { name: 'Other', render: () => <div /> }\n</script>\n";
        let actual = lint(source, "MyComponent.vue");
        let start = source.find("'Other'").unwrap() as u32;
        let expected = crate::linter::LintResult {
            filename: "MyComponent.vue".into(),
            diagnostics: vec![
                crate::diagnostic::LintDiagnostic::warn(
                    RULE,
                    message("Other", "MyComponent"),
                    start,
                    start + "'Other'".len() as u32,
                )
                .with_help(super::HELP),
            ],
            error_count: 0,
            warning_count: 1,
        };
        assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
    }
}
