use super::{ESLINT_VUE_ALIASES, patina_rule};
use crate::linter::Linter;
use crate::preset::LintPreset;
use crate::rule::RuleRegistry;
use std::collections::BTreeSet;

#[test]
fn every_renamed_mapped_rule_has_its_alias() {
    let rule_map: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/patina-eslint-vue-rule-map.json"
    ))
    .unwrap();
    let renamed: BTreeSet<(String, String)> = rule_map["entries"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(_, entry)| entry["status"] == "mapped")
        .filter_map(|(upstream, entry)| {
            let patina = entry["patinaRule"].as_str()?;
            (patina != upstream).then(|| (upstream.clone(), patina.to_owned()))
        })
        .collect();
    let table: BTreeSet<(String, String)> = ESLINT_VUE_ALIASES
        .iter()
        .map(|(upstream, patina)| ((*upstream).to_owned(), (*patina).to_owned()))
        .collect();
    let extra: Vec<_> = table.difference(&renamed).collect();
    assert_eq!(
        extra,
        [&(
            "vue/no-shared-component-data".to_owned(),
            "script/no-deprecated-data-object-declaration".to_owned()
        )]
    );
    assert!(
        renamed.is_subset(&table),
        "{:?}",
        renamed.difference(&table)
    );
}

#[test]
fn the_alias_table_is_sorted_and_never_hides_a_registered_rule() {
    assert!(
        ESLINT_VUE_ALIASES
            .windows(2)
            .all(|pair| pair[0].0 < pair[1].0)
    );
    let mut registered: BTreeSet<&str> = RuleRegistry::with_all()
        .rule_names()
        .iter()
        .copied()
        .collect();
    registered.extend(
        RuleRegistry::with_opt_in_rules()
            .rule_names()
            .iter()
            .copied(),
    );
    registered.extend(
        crate::linter::script_rules::all_builtin_script_rule_names()
            .iter()
            .copied(),
    );
    let hidden: Vec<&str> = ESLINT_VUE_ALIASES
        .iter()
        .map(|(upstream, _)| *upstream)
        .filter(|upstream| registered.contains(upstream))
        .collect();
    assert_eq!(
        hidden,
        Vec::<&str>::new(),
        "upstream IDs that are Patina rules"
    );
    for (upstream, patina) in ESLINT_VUE_ALIASES {
        assert_eq!(patina_rule(upstream), *patina);
    }
    assert_eq!(patina_rule("vue/no-v-html"), "vue/no-v-html");
}

#[test]
fn an_upstream_disable_comment_suppresses_the_mapped_rule() {
    let source = "<template>\n  <!-- eslint-disable vue/prefer-separate-static-class -->\n  <div :class=\"'a'\" />\n</template>\n";
    let lint = |source: &str| {
        Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec!["vapor/prefer-static-class".into()]))
            .lint_sfc(source, "Alias.vue")
            .diagnostics
            .len()
    };
    assert_eq!(lint(source), 0);
    assert_eq!(
        lint(&source.replace("vue/prefer-separate-static-class", "vue/other-rule")),
        1
    );
}

fn reported(rules: &[&str], source: &str) -> Vec<(String, String)> {
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(rules.iter().map(|rule| (*rule).into()).collect()))
        .lint_sfc(source, "Alias.vue");
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let line = source
                .get(..diagnostic.start as usize)
                .map_or(0, |prefix| prefix.matches('\n').count() + 1);
            (diagnostic.rule_name.to_string(), format!("line {line}"))
        })
        .collect()
}

#[test]
fn template_directive_forms_resolve_renamed_ids() {
    let source = concat!(
        "<template>\n",
        "  <!-- eslint-disable-next-line vue/no-v-html, vue/prefer-separate-static-class -->\n",
        "  <div :class=\"'a'\" />\n",
        "  <div :class=\"'b'\" /><!-- eslint-disable-line vue/prefer-separate-static-class -->\n",
        "  <!-- eslint-disable vue/prefer-separate-static-class -->\n",
        "  <div :class=\"'c'\" />\n",
        "  <!-- eslint-enable vue/prefer-separate-static-class -->\n",
        "  <div :class=\"'d'\" />\n",
        "</template>\n",
    );
    assert_eq!(
        reported(&["vapor/prefer-static-class"], source),
        [("vapor/prefer-static-class".into(), "line 8".into())]
    );
}

#[test]
fn script_directive_forms_resolve_renamed_ids() {
    let source = concat!(
        "<script setup lang=\"ts\">\n",
        "import { ref } from 'vue'\n",
        "/* eslint-disable vue/require-typed-ref, no-console */\n",
        "const a = ref()\n",
        "/* eslint-enable */\n",
        "const b = ref() // eslint-disable-line vue/require-typed-ref\n",
        "// eslint-disable-next-line no-alert, vue/require-typed-ref\n",
        "const c = ref()\n",
        "const d = ref()\n",
        "</script>\n",
    );
    assert_eq!(
        reported(&["script/require-typed-ref"], source),
        [("script/require-typed-ref".into(), "line 9".into())]
    );
}

/// eslint-plugin-vue's `comment-directive` also reads JavaScript comments in
/// directive values and interpolations (airi callout.vue).
#[test]
fn expression_comment_pragmas_suppress_like_template_comments() {
    let source = concat!(
        "<template>\n",
        "  <div\n",
        "    :class=\"[\n",
        "      ...extra,\n",
        "      // eslint-disable-next-line vue/prefer-separate-static-class\n",
        "      'a b',\n",
        "      'c',\n",
        "    ]\"\n",
        "  />\n",
        "  <div :class=\"/* eslint-disable-line vue/prefer-separate-static-class */ 'd'\" />\n",
        "  <div :class=\"['// eslint-disable-next-line vue/prefer-separate-static-class', 'e']\" />\n",
        "  <p>{{ // eslint-disable-line vue/other-rule\n value }}</p>\n",
        "</template>\n",
    );
    assert_eq!(
        reported(&["vapor/prefer-static-class"], source),
        [
            ("vapor/prefer-static-class".into(), "line 7".into()),
            ("vapor/prefer-static-class".into(), "line 11".into()),
            ("vapor/prefer-static-class".into(), "line 11".into()),
        ]
    );
}
