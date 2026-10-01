//! Pure linter projections, retaining configuration declaration order.

use super::ConfigDocument;
use crate::config::model::RawVizeConfig;
use crate::config::{
    ConfigEntryIgnore, LinterConfig, LinterConfigEntry, LinterConfigPlan,
    LinterConfigPlanWithConfigRuleOptions, LinterConfigPlanWithRuleOptions,
};

impl ConfigDocument {
    /// Derive the effective linter settings from this document.
    pub fn linter(&self) -> LinterConfig {
        load_linter_from_raw_config(&self.0)
    }
    /// Derive the ordered linter plan with stable rule options.
    pub fn linter_plan(&self) -> LinterConfigPlan {
        load_linter_plan_from_raw_config(&self.0)
    }
    /// Derive the ordered plan with entry-local stable rule options.
    pub fn linter_plan_with_rule_options(&self) -> LinterConfigPlanWithRuleOptions {
        load_linter_plan_with_rule_options_from_raw_config(&self.0)
    }
    /// Derive the ordered plan with the full typed rule option vocabulary.
    pub fn linter_plan_with_config_rule_options(&self) -> LinterConfigPlanWithConfigRuleOptions {
        load_linter_plan_with_config_rule_options_from_raw_config(&self.0)
    }
}

fn load_linter_from_raw_config(config: &RawVizeConfig) -> LinterConfig {
    let mut linter = LinterConfig::from(config.linter.clone());
    if linter.preset.is_none() {
        linter.preset = common_entry_linter_preset(config);
    }
    linter
}

fn common_entry_linter_preset(config: &RawVizeConfig) -> Option<crate::String> {
    let mut common_preset: Option<crate::String> = None;
    for entry in config.entries.as_deref().unwrap_or_default() {
        let entry_linter = LinterConfig::from(entry.linter.clone());
        let Some(entry_preset) = entry_linter.preset else {
            continue;
        };
        if common_preset
            .as_ref()
            .is_some_and(|preset| preset.as_str() != entry_preset.as_str())
        {
            return None;
        }
        common_preset = Some(entry_preset);
    }
    common_preset
}

fn load_linter_plan_from_raw_config(config: &RawVizeConfig) -> LinterConfigPlan {
    let entries = config
        .entries
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter_map(|entry| {
            let linter = LinterConfig::from(entry.linter.clone());
            (!linter.rules.is_empty()).then(|| LinterConfigEntry {
                base_path: entry.base_path.clone(),
                files: entry.files.clone(),
                ignores: entry.ignores.clone().unwrap_or_default(),
                rules: linter.rules,
            })
        })
        .collect();
    LinterConfigPlan {
        base: load_linter_from_raw_config(config),
        entries,
        global_ignores: global_ignores_from_raw_config(config),
        rule_options: config.linter.rule_options().stable_options().clone(),
    }
}

fn load_linter_plan_with_rule_options_from_raw_config(
    config: &RawVizeConfig,
) -> LinterConfigPlanWithRuleOptions {
    let mut entries = Vec::new();
    let mut entry_rule_options = Vec::new();
    for entry in config.entries.as_deref().unwrap_or_default() {
        let rule_options = entry.linter.rule_options().stable_options().clone();
        let linter = LinterConfig::from(entry.linter.clone());
        if linter.rules.is_empty() && rule_options.is_empty() {
            continue;
        }
        entries.push(LinterConfigEntry {
            base_path: entry.base_path.clone(),
            files: entry.files.clone(),
            ignores: entry.ignores.clone().unwrap_or_default(),
            rules: linter.rules,
        });
        entry_rule_options.push(rule_options);
    }
    LinterConfigPlanWithRuleOptions {
        plan: LinterConfigPlan {
            base: load_linter_from_raw_config(config),
            entries,
            global_ignores: global_ignores_from_raw_config(config),
            rule_options: config.linter.rule_options().stable_options().clone(),
        },
        entry_rule_options,
    }
}

fn load_linter_plan_with_config_rule_options_from_raw_config(
    config: &RawVizeConfig,
) -> LinterConfigPlanWithConfigRuleOptions {
    let mut entries = Vec::new();
    let mut entry_rule_options = Vec::new();
    for entry in config.entries.as_deref().unwrap_or_default() {
        let rule_options = entry.linter.rule_options().clone();
        let linter = LinterConfig::from(entry.linter.clone());
        if linter.rules.is_empty() && rule_options.is_empty() {
            continue;
        }
        entries.push(LinterConfigEntry {
            base_path: entry.base_path.clone(),
            files: entry.files.clone(),
            ignores: entry.ignores.clone().unwrap_or_default(),
            rules: linter.rules,
        });
        entry_rule_options.push(rule_options);
    }
    LinterConfigPlanWithConfigRuleOptions {
        plan: LinterConfigPlan {
            base: load_linter_from_raw_config(config),
            entries,
            global_ignores: global_ignores_from_raw_config(config),
            rule_options: config.linter.rule_options().stable_options().clone(),
        },
        rule_options: config.linter.rule_options().clone(),
        entry_rule_options,
    }
}

fn global_ignores_from_raw_config(config: &RawVizeConfig) -> Vec<ConfigEntryIgnore> {
    config
        .ignores
        .as_deref()
        .unwrap_or_default()
        .iter()
        .cloned()
        .map(|pattern| ConfigEntryIgnore {
            // Preserve the established top-level ignore contract: patterns are
            // resolved from the config directory, independently of `basePath`.
            base_path: None,
            pattern,
        })
        .collect()
}
