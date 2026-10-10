//! Typed config-only slot policy, separate from stable Rust struct literals.
use serde::{Deserialize, Serialize};

use super::ConfigLintRuleOptions;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ValidVSlotOptions {
    allow_modifiers: bool,
}

impl ConfigLintRuleOptions {
    /// Explicit slot modifier policy; options alone do not enable the rule.
    pub fn valid_v_slot_allow_modifiers(&self) -> Option<bool> {
        self.valid_v_slot.map(|options| options.allow_modifiers)
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigLintRuleOptions;
    use serde_json::json;

    fn options(value: serde_json::Value) -> ConfigLintRuleOptions {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn absence_default_and_explicit_policies_remain_distinct() {
        assert_eq!(options(json!({})).valid_v_slot_allow_modifiers(), None);
        assert_eq!(
            options(json!({"vue/valid-v-slot": {}})).valid_v_slot_allow_modifiers(),
            Some(false)
        );
        for allow in [false, true] {
            let value = json!({"vue/valid-v-slot": {"allowModifiers": allow}});
            let configured = options(value.clone());
            assert_eq!(configured.valid_v_slot_allow_modifiers(), Some(allow));
            assert_eq!(
                serde_json::to_value(configured).unwrap()["vue/valid-v-slot"],
                value["vue/valid-v-slot"]
            );
        }
        assert!(
            serde_json::to_value(options(json!({})))
                .unwrap()
                .get("vue/valid-v-slot")
                .is_none()
        );
    }

    #[test]
    fn explicit_false_overrides_true_and_absence_preserves_it() {
        let mut configured = options(json!({"vue/valid-v-slot": {"allowModifiers": true}}));
        configured.merge_from(&options(json!({})));
        assert_eq!(configured.valid_v_slot_allow_modifiers(), Some(true));
        configured.merge_from(&options(
            json!({"vue/valid-v-slot": {"allowModifiers": false}}),
        ));
        assert_eq!(configured.valid_v_slot_allow_modifiers(), Some(false));
        configured.merge_from(&options(json!({})));
        assert_eq!(configured.valid_v_slot_allow_modifiers(), Some(false));
    }

    #[test]
    fn option_type_and_unknown_fields_are_rejected() {
        for invalid in [
            json!({"vue/valid-v-slot": {"allowModifiers": "true"}}),
            json!({"vue/valid-v-slot": {"allowModifier": true}}),
            json!({"vue/valid-v-slot": true}),
        ] {
            assert!(serde_json::from_value::<ConfigLintRuleOptions>(invalid).is_err());
        }
    }
}
