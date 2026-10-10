//! Config-layer merging for the full typed rule options.
use super::ConfigLintRuleOptions;

impl ConfigLintRuleOptions {
    /// Apply a later config layer to this option set.
    pub fn merge_from(&mut self, overlay: &Self) {
        self.stable.merge_from(&overlay.stable);
        if let Some(options) = overlay.define_props_destructuring {
            self.define_props_destructuring = Some(options);
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
        if let Some(options) = &overlay.no_mutating_props {
            self.no_mutating_props = Some(*options);
        }
        if let Some(options) = &overlay.sfc_element_order {
            self.sfc_element_order = Some(options.clone());
        }
        if let Some(options) = &overlay.html_self_closing {
            self.html_self_closing = Some(*options);
        }
        if let Some(style) = overlay.v_on_event_hyphenation {
            self.v_on_event_hyphenation = Some(style);
        }
        if let Some(style) = overlay.attribute_hyphenation {
            self.attribute_hyphenation = Some(style);
        }
        if let Some(options) = &overlay.musea_prefer_design_tokens {
            self.musea_prefer_design_tokens = Some(options.clone());
        }
        if let Some(options) = &overlay.require_component_registration {
            self.require_component_registration = Some(options.clone());
        }
    }
}
