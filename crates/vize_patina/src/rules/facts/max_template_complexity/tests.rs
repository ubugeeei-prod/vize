use super::MaxTemplateComplexity;
use crate::diagnostic::Severity;
use crate::linter::Linter;
use crate::preset::LintPreset;
use crate::rule::RuleRegistry;
use crate::{LintResult, OutputFormat, format_results};

const RULE: &str = "vue/max-template-complexity";

fn lint(source: &str) -> LintResult {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(MaxTemplateComplexity));
    Linter::with_registry(registry).lint_sfc(source, "Dashboard.vue")
}

/// Every construct the metric counts, nested three deep: cyclomatic 13,
/// cognitive 25 — over both thresholds.
const COMPLEX: &str = r#"<script setup lang="ts">
defineProps<{ rows: Row[] }>()
</script>

<template>
  <section>
    <h1>{{ user ? user.name : 'Guest' }}</h1>
    <DataTable :rows="rows">
      <template #cell="{ row, column }">
        <span v-if="column.key === 'status'" :class="row.active ? 'on' : 'off'">
          {{ row.status ?? 'unknown' }}
        </span>
        <a v-else-if="column.key === 'link' && row.url" :href="row.url">{{ row.label }}</a>
        <template v-else>
          <em v-for="tag in row.tags" :key="tag">
            <b v-if="tag.pinned || tag.starred">{{ tag.hot ? '!' : '' }}</b>
          </em>
        </template>
      </template>
    </DataTable>
    <p v-if="!rows.length && !loading">No data</p>
  </section>
</template>
"#;

fn slice(source: &str, start: u32, end: u32) -> &str {
    &source[start as usize..end as usize]
}

#[test]
fn a_component_over_the_thresholds_warns_with_its_top_contributors() {
    let result = lint(COMPLEX);
    assert_eq!((result.error_count, result.warning_count), (0, 1));
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.rule_name, "vue/max-template-complexity");
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert_eq!(
        diagnostic.message.as_str(),
        "Template complexity is too high: cyclomatic 13 (limit 11), cognitive 25 (limit 16)"
    );
    assert_eq!(
        slice(COMPLEX, diagnostic.start, diagnostic.end),
        "<template>"
    );
    let labels: Vec<(&str, &str)> = diagnostic
        .labels
        .iter()
        .map(|label| {
            (
                label.message.as_str(),
                slice(COMPLEX, label.start, label.end),
            )
        })
        .collect();
    assert_eq!(
        labels,
        vec![
            (
                "v-if at nesting 1: +2 cognitive, +1 cyclomatic",
                "column.key === 'status'"
            ),
            (
                "conditional at nesting 2: +3 cognitive, +1 cyclomatic",
                "row.active ? 'on' : 'off'"
            ),
            (
                "v-for at nesting 2: +3 cognitive, +1 cyclomatic",
                "row.tags"
            ),
            (
                "v-if at nesting 3: +4 cognitive, +1 cyclomatic",
                "tag.pinned || tag.starred"
            ),
            (
                "conditional at nesting 4: +5 cognitive, +1 cyclomatic",
                "tag.hot ? '!' : ''"
            ),
        ]
    );
    assert_eq!(
        diagnostic.help.as_deref(),
        Some(
            "Move the deepest branches or loops into child components: a child's complexity \
             never counts toward this component's own score"
        )
    );
}

#[test]
fn a_component_within_the_thresholds_is_silent() {
    let source = r#"<template>
  <p v-if="a && b">{{ c ? d : e }}</p>
  <p v-else>none</p>
  <li v-for="x in xs" :key="x">{{ x }}</li>
</template>
"#;
    let result = lint(source);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn no_preset_carries_the_rule_and_naming_it_enables_it() {
    for preset in LintPreset::ALL {
        let result = Linter::with_preset(preset).lint_sfc(COMPLEX, "Dashboard.vue");
        assert!(
            result.diagnostics.iter().all(|d| d.rule_name != RULE),
            "{preset:?} must not carry {RULE}"
        );
    }
    let enabled = Linter::with_preset(LintPreset::Opinionated)
        .with_additional_rules(vec![RULE.into()])
        .lint_sfc(COMPLEX, "Dashboard.vue");
    let fired: Vec<_> = enabled
        .diagnostics
        .iter()
        .filter(|d| d.rule_name == RULE)
        .map(|d| d.message.as_str())
        .collect();
    assert_eq!(
        fired,
        vec!["Template complexity is too high: cyclomatic 13 (limit 11), cognitive 25 (limit 16)"]
    );
}

#[test]
fn foreign_and_external_templates_are_not_judged() {
    let pug = "<template lang=\"pug\">\ndiv(v-if=\"a && b && c\")\n</template>\n";
    assert_eq!(lint(pug).diagnostics.len(), 0);
    let external = "<template src=\"./dashboard.html\"></template>\n";
    assert_eq!(lint(external).diagnostics.len(), 0);
    assert_eq!(
        lint("<script setup>\nconst a = 1\n</script>\n")
            .diagnostics
            .len(),
        0
    );
}

#[test]
fn the_text_output_shows_where_the_complexity_comes_from() {
    let result = lint(COMPLEX);
    let output = format_results(
        &[result],
        &[(
            vize_s0::String::from("Dashboard.vue"),
            vize_s0::String::from(COMPLEX),
        )],
        OutputFormat::Text,
    );
    #[allow(clippy::disallowed_macros)]
    {
        insta::assert_snapshot!(strip_ansi(&output));
    }
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(ch);
        }
    }
    // Gutter lines end in padding; trim it so the snapshot has none.
    out.lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
}
