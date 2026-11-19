//! Config-layer merging for the full typed rule options.
use super::ConfigLintRuleOptions;

impl ConfigLintRuleOptions {
    /// Apply a later config layer to this option set.
    pub fn merge_from(&mut self, overlay: &Self) {
        self.stable.merge_from(&overlay.stable);
        if let Some(options) = &overlay.valid_v_on {
            self.valid_v_on = Some(options.clone());
        }
        if let Some(options) = overlay.valid_v_slot {
            self.valid_v_slot = Some(options);
        }
        if let Some(options) = overlay.define_props_destructuring {
            self.define_props_destructuring = Some(options);
        }
        if let Some(options) = &overlay.define_macros_order {
            self.define_macros_order = Some(options.clone());
        }
        if let Some(options) = &overlay.no_empty_palpable_content {
            self.no_empty_palpable_content = Some(options.clone());
        }
        if let Some(options) = overlay.strict_boolean_expressions {
            self.strict_boolean_expressions = Some(options);
        }
        if let Some(options) = &overlay.component_name_in_template_casing {
            self.component_name_in_template_casing = Some(options.clone());
        }
        if let Some(options) = &overlay.custom_event_name_casing {
            self.custom_event_name_casing = Some(*options);
        }
        if let Some(options) = overlay.prop_name_casing {
            self.prop_name_casing = Some(options);
        }
        if let Some(style) = overlay.slot_name_casing {
            self.slot_name_casing = Some(style);
        }
        if let Some(options) = &overlay.no_mutating_props {
            self.no_mutating_props = Some(*options);
        }
        if let Some(options) = &overlay.multi_word_component_names {
            self.multi_word_component_names = Some(options.clone());
        }
        if let Some(options) = overlay.html_button_has_type {
            self.html_button_has_type = Some(options);
        }
        if let Some(options) = overlay.no_use_v_if_with_v_for {
            self.no_use_v_if_with_v_for = Some(options);
        }
        if let Some(options) = &overlay.warn_custom_directive {
            self.warn_custom_directive = Some(options.clone());
        }
        if let Some(options) = &overlay.no_bare_strings_in_template {
            self.no_bare_strings_in_template = Some(options.clone());
        }
        if let Some(options) = &overlay.anchor_has_content {
            self.anchor_has_content = Some(options.clone());
        }
        if let Some(options) = &overlay.label_has_for {
            self.label_has_for = Some(options.clone());
        }
        if let Some(options) = &overlay.enforce_style_attribute {
            self.enforce_style_attribute = Some(options.clone());
        }
        if let Some(options) = &overlay.no_restricted_syntax {
            self.no_restricted_syntax = Some(options.clone());
        }
        if let Some(options) = overlay.quote_props {
            self.quote_props = Some(options);
        }
        if let Some(options) = overlay.sort_keys {
            self.sort_keys = Some(options);
        }
        if let Some(options) = &overlay.sfc_element_order {
            self.sfc_element_order = Some(options.clone());
        }
        if let Some(options) = &overlay.html_self_closing {
            self.html_self_closing = Some(*options);
        }
        if let Some(options) = overlay.html_quotes {
            self.html_quotes = Some(options);
        }
        if let Some(style) = overlay.mustache_interpolation_spacing {
            self.mustache_interpolation_spacing = Some(style);
        }
        if let Some(options) = &overlay.v_on_event_hyphenation {
            self.v_on_event_hyphenation = Some(options.clone());
        }
        if let Some(style) = overlay.v_bind_style {
            self.v_bind_style = Some(style);
        }
        if let Some(style) = overlay.v_on_style {
            self.v_on_style = Some(style);
        }
        if let Some(style) = overlay.v_slot_style {
            self.v_slot_style = Some(style);
        }
        if let Some(options) = &overlay.attribute_hyphenation {
            self.attribute_hyphenation = Some(options.clone());
        }
        if let Some(options) = &overlay.musea_prefer_design_tokens {
            self.musea_prefer_design_tokens = Some(options.clone());
        }
        if let Some(options) = &overlay.require_component_registration {
            self.require_component_registration = Some(options.clone());
        }
        if let Some(options) = &overlay.block_lang {
            self.block_lang = Some(options.clone());
        }
        self.configured.merge_from(&overlay.configured);
        self.type_aware.merge_from(&overlay.type_aware);
    }
}
