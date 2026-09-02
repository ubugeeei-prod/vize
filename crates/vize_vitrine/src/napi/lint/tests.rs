use super::collect_patina_rule_metadata;

#[test]
fn patina_rule_metadata_includes_happy_path_membership() {
    let rules = collect_patina_rule_metadata();
    let require_scoped_style = rules
        .iter()
        .find(|rule| rule.name == "vue/require-scoped-style")
        .expect("vue/require-scoped-style should be exposed");

    assert_eq!(
        require_scoped_style.presets,
        vec!["general-recommended", "nuxt", "ecosystem", "opinionated"]
    );
}

#[test]
fn patina_rule_metadata_includes_opinionated_script_rules() {
    let rules = collect_patina_rule_metadata();
    let no_options_api = rules
        .iter()
        .find(|rule| rule.name == "script/no-options-api")
        .expect("script/no-options-api should be exposed");

    assert_eq!(no_options_api.presets, vec!["opinionated"]);
    assert_eq!(no_options_api.default_severity, "error");
}

#[test]
fn patina_rule_metadata_includes_ecosystem_rules() {
    let rules = collect_patina_rule_metadata();
    let void_link = rules
        .iter()
        .find(|rule| rule.name == "ecosystem/void-link-require-href")
        .expect("Void Vue Link rule should be exposed");

    assert_eq!(void_link.presets, vec!["ecosystem"]);
    assert_eq!(void_link.default_severity, "error");
}

#[test]
fn patina_rule_metadata_limits_nuxt_link_rule_to_nuxt() {
    let rules = collect_patina_rule_metadata();
    let prefer_nuxt_link = rules
        .iter()
        .find(|rule| rule.name == "ecosystem/nuxt-prefer-nuxt-link")
        .expect("NuxtLink preference rule should be exposed");

    assert_eq!(prefer_nuxt_link.presets, vec!["nuxt"]);
    assert_eq!(prefer_nuxt_link.default_severity, "warning");
}

#[test]
fn patina_rule_metadata_includes_musea_opt_in_rules() {
    let rules = collect_patina_rule_metadata();
    let require_title = rules
        .iter()
        .find(|rule| rule.name == "musea/require-title")
        .expect("Musea require-title rule should be exposed");
    let no_empty_variant = rules
        .iter()
        .find(|rule| rule.name == "musea/no-empty-variant")
        .expect("Musea no-empty-variant rule should be exposed");

    assert_eq!(require_title.category, "Musea");
    assert_eq!(require_title.presets, Vec::<&'static str>::new());
    assert_eq!(require_title.default_severity, "error");
    assert_eq!(no_empty_variant.default_severity, "warning");
}
