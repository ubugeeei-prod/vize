//! Apply the configurable Vue rule options by replacing the registered rules.

use super::super::config::Linter;
use crate::rules::opinionated::vue::{NoUnusedScriptVars, PreferConst};
use crate::rules::vue::configured::{
    MaxProps, MaxTemplateDepth, NoRestrictedBlock, NoRestrictedCallAfterAwait, NoRestrictedClass,
    NoRestrictedComponentNames, NoRestrictedComponentOptions, NoRestrictedCustomEvent,
    NoRestrictedHtmlElements, NoRestrictedProps, NoRestrictedStaticAttribute, NoRestrictedVOn,
    NoUnsupportedFeatures,
};
use vize_l0::config::ConfiguredVueRuleOptions;

pub(super) fn apply(mut linter: Linter, options: &ConfiguredVueRuleOptions) -> Linter {
    if let Some(value) = &options.max_props {
        linter
            .registry
            .replace(Box::new(MaxProps::from_options(value)));
    }
    if let Some(value) = &options.max_template_depth {
        linter
            .registry
            .replace(Box::new(MaxTemplateDepth::from_options(value)));
    }
    if let Some(value) = &options.no_restricted_block {
        linter
            .registry
            .replace(Box::new(NoRestrictedBlock::from_options(value)));
    }
    if let Some(value) = &options.no_restricted_call_after_await {
        linter
            .registry
            .replace(Box::new(NoRestrictedCallAfterAwait::from_options(value)));
    }
    if let Some(value) = &options.no_restricted_class {
        linter
            .registry
            .replace(Box::new(NoRestrictedClass::from_options(value)));
    }
    if let Some(value) = &options.no_restricted_component_names {
        linter
            .registry
            .replace(Box::new(NoRestrictedComponentNames::from_options(value)));
    }
    if let Some(value) = &options.no_restricted_component_options {
        linter
            .registry
            .replace(Box::new(NoRestrictedComponentOptions::from_options(value)));
    }
    if let Some(value) = &options.no_restricted_custom_event {
        linter
            .registry
            .replace(Box::new(NoRestrictedCustomEvent::from_options(value)));
    }
    if let Some(value) = &options.no_restricted_html_elements {
        linter
            .registry
            .replace(Box::new(NoRestrictedHtmlElements::from_options(value)));
    }
    if let Some(value) = &options.no_restricted_props {
        linter
            .registry
            .replace(Box::new(NoRestrictedProps::from_options(value)));
    }
    if let Some(value) = &options.no_restricted_static_attribute {
        linter
            .registry
            .replace(Box::new(NoRestrictedStaticAttribute::from_options(value)));
    }
    if let Some(value) = &options.no_restricted_v_on {
        linter
            .registry
            .replace(Box::new(NoRestrictedVOn::from_options(value)));
    }
    if let Some(value) = &options.no_unsupported_features {
        linter
            .registry
            .replace(Box::new(NoUnsupportedFeatures::from_options(value)));
    }
    if let Some(value) = &options.no_unused_script_vars {
        linter
            .registry
            .replace(Box::new(NoUnusedScriptVars::from_options(value)));
    }
    if let Some(value) = &options.prefer_const {
        linter
            .registry
            .replace(Box::new(PreferConst::from_options(value)));
    }
    linter
}
