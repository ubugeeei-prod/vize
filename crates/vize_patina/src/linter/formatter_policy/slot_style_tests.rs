//! Slot options preserve positional defaults and the selected-rule boundary.
use super::*;
use serde_json::{Value, json};
use vize_l0::config::ConfigDocument;

const RULE: &str = "vue/v-slot-style";
const REASON: &str = "normalizeDirectiveShorthands=true conflicts with an active vue/v-slot-style longform policy; set formatter.normalizeDirectiveShorthands=false (CLI: --normalize-directive-shorthands=false) or use shorthand/v-slot for the affected slot positions";

fn validate(linter: Value, normalize: bool, editor: bool) -> Result<(), String> {
    let document: ConfigDocument = serde_json::from_value(json!({"linter":linter})).unwrap();
    let resolved = document
        .linter_plan_with_config_rule_options()
        .resolve_matching_entries(&[]);
    if editor {
        Linter::validate_editor_directive_normalization(
            &resolved.config,
            &resolved.rule_options,
            normalize,
        )
    } else {
        Linter::validate_directive_normalization(
            &resolved.config,
            &resolved.rule_options,
            normalize,
        )
    }
}

#[test]
fn uniform_and_each_positional_longform_conflict_but_bare_defaults_do_not() {
    for style in [
        json!("longform"),
        json!({"atComponent":"longform"}),
        json!({"default":"longform"}),
        json!({"named":"longform"}),
        json!({"atComponent":"longform","default":"v-slot","named":"longform"}),
    ] {
        let config =
            json!({"preset":"incremental","rules":{RULE:"error"},"ruleOptions":{RULE:style}});
        for editor in [false, true] {
            assert_eq!(validate(config.clone(), true, editor), Err(REASON.into()));
            assert_eq!(validate(config.clone(), false, editor), Ok(()));
        }
    }
    for style in [
        json!("shorthand"),
        json!({}),
        json!({"default":"v-slot"}),
        json!({"atComponent":"v-slot","default":"shorthand","named":"shorthand"}),
    ] {
        let config =
            json!({"preset":"incremental","rules":{RULE:"error"},"ruleOptions":{RULE:style}});
        assert_eq!(validate(config.clone(), true, false), Ok(()));
        assert_eq!(validate(config, true, true), Ok(()));
    }
    assert_eq!(
        validate(json!({"preset":"opinionated"}), true, false),
        Ok(())
    );
}

#[test]
fn slot_configuration_does_not_append_an_unselected_rule_or_undo_host_categories() {
    for config in [
        json!({"preset":"incremental","ruleOptions":{RULE:"longform"}}),
        json!({"preset":"incremental","categories":{"style":"off"},"ruleOptions":{RULE:"longform"}}),
        json!({"preset":"opinionated","rules":{RULE:"off"},"ruleOptions":{RULE:"longform"}}),
        json!({"enabled":false,"preset":"opinionated","ruleOptions":{RULE:"longform"}}),
        json!({"preset":"opinionated","rules":{"vue/no-v-html":"error"},"ruleOptions":{RULE:"longform"}}),
        json!({"preset":"incremental","ruleOptions":{RULE:"longform","script/no-restricted-members":{"members":[{"object":"window","property":"localStorage"}]}}}),
        json!({"preset":"incremental","ruleOptions":{RULE:"longform","musea/prefer-design-tokens":{"tokens":[{"path":"color.primary","value":"#3b82f6"}]}}}),
    ] {
        assert_eq!(validate(config.clone(), true, false), Ok(()), "{config}");
        assert_eq!(validate(config.clone(), true, true), Ok(()), "{config}");
    }
    for config in [
        json!({"preset":"incremental","rules":{RULE:"warn"},"ruleOptions":{RULE:"longform"}}),
        json!({"preset":"incremental","rules":{RULE:"error"},"ruleOptions":{RULE:"longform"}}),
        json!({"preset":"opinionated","ruleOptions":{RULE:"longform"}}),
        json!({"preset":"all","ruleOptions":{RULE:"longform"}}),
        json!({"ruleOptions":{RULE:"longform"}}),
    ] {
        assert_eq!(
            validate(config.clone(), true, false),
            Err(REASON.into()),
            "{config}"
        );
        assert_eq!(
            validate(config.clone(), true, true),
            Err(REASON.into()),
            "{config}"
        );
    }
    // Slot's StronglyRecommended category is included by the actual CLI
    // category matcher; the editor currently omits that category pass.
    for preset in ["incremental", "opinionated"] {
        let config = json!({"preset":preset,"rules":{RULE:"error"},"categories":{"style":"off"},"ruleOptions":{RULE:"longform"}});
        assert_eq!(validate(config.clone(), true, false), Ok(()));
        assert_eq!(validate(config, true, true), Err(REASON.into()));
    }
}

#[test]
fn existing_bind_and_event_reasons_keep_priority_and_unknown_presets_still_refuse() {
    for (name, reason) in [
        (
            "vue/v-bind-style",
            "normalizeDirectiveShorthands=true conflicts with active vue/v-bind-style=longform; set formatter.normalizeDirectiveShorthands=false (CLI: --normalize-directive-shorthands=false) or use shorthand binding style",
        ),
        (
            "vue/v-on-style",
            "normalizeDirectiveShorthands=true conflicts with active vue/v-on-style=longform; set formatter.normalizeDirectiveShorthands=false (CLI: --normalize-directive-shorthands=false) or use shorthand event style",
        ),
    ] {
        let config = json!({"preset":"incremental","rules":{RULE:"error",name:"error"},"ruleOptions":{RULE:"longform",name:"longform"}});
        assert_eq!(validate(config.clone(), true, false), Err(reason.into()));
        assert_eq!(validate(config.clone(), true, true), Err(reason.into()));
        assert_eq!(validate(config, false, false), Ok(()));
    }
    let config =
        json!({"preset":"unrecognized","rules":{RULE:"error"},"ruleOptions":{RULE:"longform"}});
    assert_eq!(
        validate(config, true, false),
        Err("Unknown lint preset 'unrecognized'".into())
    );
}
