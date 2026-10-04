//! Original source/options and exact historical witness identity.

use serde::Deserialize;
use vize_l0::String;
use vize_patina::rules::a11y::{NoAccessKey, NoAutofocus};
use vize_patina::{HelpLevel, Linter, Locale, Rule, RuleCategory, RuleRegistry, Severity};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[expect(
    dead_code,
    reason = "all historical witness fields remain in the complete Debug capture"
)]
pub(super) struct Case {
    pub(super) id: String,
    pub(super) fix: String,
    pub(super) parent: String,
    pub(super) witness_revision: String,
    pub(super) witness_blob: String,
    pub(super) rule_path: String,
    pub(super) test: String,
    pub(super) witness_kind: String,
    pub(super) source: String,
    pub(super) source_sha256: String,
    pub(super) filename: String,
    pub(super) entry: String,
    pub(super) rule: String,
    pub(super) vue_version: Option<String>,
    pub(super) vapor: Option<bool>,
    pub(super) locale: Option<String>,
    pub(super) help_level: Option<String>,
    pub(super) severity: Option<String>,
    pub(super) original_warning_count: usize,
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "all actual identity fields remain in the full Debug capture"
)]
pub(super) struct RuleIdentity {
    concrete_type: &'static str,
    name: &'static str,
    description: &'static str,
    category: RuleCategory,
    fixable: bool,
    default_severity: Severity,
    registry_names: Vec<&'static str>,
    locale: Locale,
    constructor_default_help: HelpLevel,
}

pub(super) fn configured(case: &Case) -> (Linter, RuleIdentity) {
    for revision in [
        &case.fix,
        &case.parent,
        &case.witness_revision,
        &case.witness_blob,
    ] {
        assert_eq!(revision.len(), 40, "full historical object identity");
    }
    assert_eq!(
        case.source_sha256.len(),
        64,
        "original source UTF8 identity"
    );
    assert_eq!(case.filename, "test.vue");
    assert_eq!(case.entry, "template");
    assert!(case.vue_version.is_none() && case.vapor.is_none());
    assert!(case.locale.is_none() && case.help_level.is_none() && case.severity.is_none());
    let (rule, concrete_type): (Box<dyn Rule>, _) = match case.rule.as_str() {
        "a11y/no-autofocus" => (Box::new(NoAutofocus), std::any::type_name::<NoAutofocus>()),
        "a11y/no-access-key" => (Box::new(NoAccessKey), std::any::type_name::<NoAccessKey>()),
        name => panic!("unknown original concrete rule: {name}"),
    };
    let meta = rule.meta();
    let mut registry = RuleRegistry::new();
    registry.register(rule);
    let registry_names = registry.rule_names().to_vec();
    assert_eq!(registry_names.as_slice(), &[case.rule.as_str()]);
    // Exactly the original helper: one concrete instance and no preset/setters.
    let linter = Linter::with_registry(registry);
    let identity = RuleIdentity {
        concrete_type,
        name: meta.name,
        description: meta.description,
        category: meta.category,
        fixable: meta.fixable,
        default_severity: meta.default_severity,
        registry_names,
        locale: linter.locale(),
        constructor_default_help: HelpLevel::default(),
    };
    (linter, identity)
}
