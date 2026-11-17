use std::path::PathBuf;

use serde_json::json;
use vize_l0::FxHashMap;
use vize_patina::plugin_batch::directives::DIRECTIVE_CONTRACT_RULE;
use vize_patina::plugin_batch::lint::host::{PluginLintHost, PluginLintOutcome};
use vize_patina::{LintDiagnostic, LintResult};

use super::PluginLint;
use crate::config::plugin_plan::{VerifiedPluginPlan, verify_plugin_plan};

const TRACK: &str =
    "<template>\n  <a v-track.strict.once v-track.bogus.strict v-track>go</a>\n</template>\n";

/// The declared-capabilities fixture reduced to its one directive plugin.
fn directive_only_plan() -> VerifiedPluginPlan {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../tests/_fixtures/plugin-plan/resolve/declared-capabilities.json"
    ))
    .unwrap();
    let mut tracking = fixture["expect"]["plan"]["plugins"][2].clone();
    tracking["index"] = json!(0);
    let plan = json!({
        "claims": [{ "key": "vue.directive:track", "owner": "@acme/tracking" }],
        "planSchema": 1,
        "plugins": [tracking],
    });
    verify_plugin_plan(&plan, None).unwrap()
}

fn reports(diagnostics: &[LintDiagnostic]) -> Vec<(&'static str, &str, &str)> {
    diagnostics
        .iter()
        .map(|diagnostic| {
            let span = TRACK
                .get(diagnostic.start as usize..diagnostic.end as usize)
                .unwrap_or_default();
            (diagnostic.rule_name, diagnostic.message.as_str(), span)
        })
        .collect()
}

const EXPECTED: [(&str, &str, &str); 2] = [
    (
        DIRECTIVE_CONTRACT_RULE,
        "v-track does not declare the modifier `bogus` (declared by @acme/tracking)",
        "v-track.bogus.strict",
    ),
    (
        DIRECTIVE_CONTRACT_RULE,
        "v-track requires the modifier `strict` (declared by @acme/tracking)",
        "v-track",
    ),
];

#[test]
fn declared_directives_lint_natively_without_a_sidecar() {
    let lint = PluginLint::plan(&directive_only_plan(), &FxHashMap::default(), None)
        .unwrap()
        .unwrap();
    assert!(lint.sidecar.is_none());
    let result = LintResult {
        filename: "Track.vue".into(),
        diagnostics: Vec::new(),
        error_count: 0,
        warning_count: 0,
    };
    let mut results = vec![(
        PathBuf::from("Track.vue"),
        "Track.vue".into(),
        TRACK.into(),
        result,
    )];
    let receipt = lint.run(&mut results).unwrap();
    assert_eq!(receipt.documents, 1);
    let (_, _, _, result) = &results[0];
    assert_eq!((result.error_count, result.warning_count), (2, 0));
    assert_eq!(reports(&result.diagnostics), EXPECTED);
}

#[test]
fn the_editor_host_checks_directives_without_a_sidecar() {
    let host = PluginLintHost::for_plan(&directive_only_plan(), None, &FxHashMap::default(), "lsp")
        .unwrap();
    let found = match host.lint(TRACK, "Track.vue", &|| false) {
        PluginLintOutcome::Diagnostics(found) => found,
        PluginLintOutcome::Cancelled | PluginLintOutcome::Failed(_) => Vec::new(),
    };
    assert_eq!(reports(&found), EXPECTED);
}
