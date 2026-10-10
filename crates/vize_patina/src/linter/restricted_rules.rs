//! Builder methods that thread project-local options into configurable rules.
//!
//! These rules are `&'static` singletons in the registry, so a configured rule
//! is built as a boxed instance and stored in [`Linter::script_rule_overrides`];
//! the script-rule dispatch prefers the override over the static singleton.
//!
//! Refs: #1891 (project-local custom rules during migration).

use super::config::Linter;
use crate::preset::LintPreset;
use crate::rules::opinionated::vue::{
    ComponentCasing, ComponentNameInTemplateCasing, ComponentNameInTemplateCasingNuxt,
    HtmlSelfClosing, HtmlSelfClosingNuxt, HtmlSelfClosingOptions, VOnEventHyphenation,
    VOnEventHyphenationStyle,
};
use crate::rules::script::{CustomEventNameCasing, EventNameCasing};
use crate::rules::script::{NoRestrictedMembers, RestrictedGlobals};
use crate::rules::vue::{
    AttributeHyphenation, HyphenationStyle, NoMutatingProps, NoMutatingPropsOptions,
    SfcElementOrder, SfcElementOrderOptions,
};
use vize_l0::String;

impl Linter {
    /// Configure slot modifiers without enabling an unselected rule.
    pub fn with_valid_v_slot_allow_modifiers(mut self, allow_modifiers: bool) -> Self {
        if self.registry.has_rule("vue/valid-v-slot") {
            self.registry
                .replace(Box::new(crate::rules::vue::ValidVSlot::configured(
                    allow_modifiers,
                )));
        }
        self
    }

    /// Configure `script/define-props-destructuring` without enabling the rule.
    #[inline]
    pub fn with_define_props_destructuring(
        mut self,
        mode: vize_l0::config::PropsDestructureMode,
    ) -> Self {
        self.script_rule_overrides.insert(
            "script/define-props-destructuring",
            Box::new(crate::rules::script::DefinePropsDestructuring::configured(
                mode,
            )),
        );
        self
    }

    /// Configure known content-providing directives without enabling the rule.
    pub fn with_palpable_content_directives(mut self, names: Vec<String>) -> Self {
        if self.registry.has_rule("html/no-empty-palpable-content") {
            if names.is_empty() {
                self.registry
                    .replace(Box::new(crate::rules::html::NoEmptyPalpableContent));
            } else {
                self.registry.replace(Box::new(
                    crate::rules::html::NoEmptyPalpableContent::with_content_directives(names),
                ));
            }
        }
        self
    }

    /// Configure strict boolean allowances without enabling the opt-in rule.
    pub fn with_strict_boolean_expressions_options(
        mut self,
        options: vize_l0::config::StrictBooleanExpressionsOptions,
    ) -> Self {
        self.strict_boolean_options = options;
        self
    }

    /// Configure the deny list for `script/no-restricted-globals`.
    ///
    /// Each entry is `(name, optional message)`. A non-empty list **replaces**
    /// the rule's built-in defaults; an empty list leaves the defaults in place.
    /// Enabling the rule itself is still governed by the usual rule-enable
    /// configuration; this only customizes its data.
    #[inline]
    pub fn with_restricted_globals(mut self, globals: Vec<(String, Option<String>)>) -> Self {
        if globals.is_empty() {
            return self;
        }
        self.script_rule_overrides.insert(
            "script/no-restricted-globals",
            Box::new(RestrictedGlobals::configured(globals)),
        );
        self
    }

    /// Configure the member-access deny list for `script/no-restricted-members`.
    ///
    /// Each entry is `(object, property, optional message)`. The rule is off
    /// unless this list is non-empty (there is no built-in default).
    #[inline]
    pub fn with_restricted_members(
        mut self,
        members: Vec<(String, String, Option<String>)>,
    ) -> Self {
        if members.is_empty() {
            return self;
        }
        self.script_rule_overrides.insert(
            "script/no-restricted-members",
            Box::new(NoRestrictedMembers::configured(members)),
        );
        self
    }

    /// Apply both project-local script-rule deny lists in one call.
    #[inline]
    pub fn with_restricted_rules(
        self,
        globals: Vec<(String, Option<String>)>,
        members: Vec<(String, String, Option<String>)>,
    ) -> Self {
        self.with_restricted_globals(globals)
            .with_restricted_members(members)
    }

    /// Configure `vue/component-name-in-template-casing`.
    #[inline]
    pub fn with_component_name_in_template_casing(self, casing: ComponentCasing) -> Self {
        self.with_component_name_in_template_casing_policy(casing, true, Vec::new())
    }

    /// Configure authored registration scope without enabling the rule.
    pub fn with_component_name_in_template_casing_policy(
        mut self,
        casing: ComponentCasing,
        registered_components_only: bool,
        globals: Vec<String>,
    ) -> Self {
        let policy = ComponentNameInTemplateCasing::new(casing)
            .with_registered_components_only(registered_components_only)
            .with_globals(globals);
        if matches!(self.preset, Some(LintPreset::Nuxt)) {
            self.registry
                .replace(Box::new(ComponentNameInTemplateCasingNuxt::with_policy(
                    policy,
                )));
        } else {
            self.registry.replace(Box::new(policy));
        }
        self
    }

    /// Configure `vue/html-self-closing`.
    #[inline]
    pub fn with_html_self_closing_options(mut self, options: HtmlSelfClosingOptions) -> Self {
        if matches!(self.preset, Some(LintPreset::Nuxt)) {
            self.registry
                .replace(Box::new(HtmlSelfClosingNuxt::new(options)));
        } else {
            self.registry
                .replace(Box::new(HtmlSelfClosing::new(options)));
        }
        self
    }

    /// Configure `vue/no-mutating-props`.
    #[inline]
    pub fn with_no_mutating_props_options(mut self, options: NoMutatingPropsOptions) -> Self {
        self.registry
            .replace(Box::new(NoMutatingProps::new(options)));
        self
    }

    /// Configure global component names without enabling the rule.
    pub fn with_component_registration_globals(mut self, globals: Vec<String>) -> Self {
        if self.registry.has_rule("vue/require-component-registration") {
            self.registry.replace(Box::new(
                crate::rules::opinionated::vue::RequireComponentRegistration {
                    ignore_globals: globals,
                    nuxt_mode: matches!(self.preset, Some(LintPreset::Nuxt)),
                },
            ));
        }
        self
    }

    /// Configure `vue/sfc-element-order`.
    #[inline]
    pub fn with_sfc_element_order_options(mut self, options: SfcElementOrderOptions) -> Self {
        self.registry
            .replace(Box::new(SfcElementOrder::new(options)));
        self
    }

    /// Configure `vue/v-on-event-hyphenation`.
    #[inline]
    pub fn with_v_on_event_hyphenation(mut self, style: VOnEventHyphenationStyle) -> Self {
        self.registry
            .replace(Box::new(VOnEventHyphenation::new(style)));
        self
    }

    /// Configure `vue/attribute-hyphenation`.
    #[inline]
    pub fn with_attribute_hyphenation(mut self, style: HyphenationStyle) -> Self {
        self.registry
            .replace(Box::new(AttributeHyphenation::new(style)));
        self
    }

    /// Configure `script/custom-event-name-casing`.
    #[inline]
    pub fn with_custom_event_name_casing(mut self, casing: EventNameCasing) -> Self {
        self.script_rule_overrides.insert(
            "script/custom-event-name-casing",
            Box::new(CustomEventNameCasing::new(casing)),
        );
        self
    }
}
