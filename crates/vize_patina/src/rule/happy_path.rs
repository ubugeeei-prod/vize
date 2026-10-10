//! General-purpose default registry construction.

use super::RuleRegistry;

impl RuleRegistry {
    /// Create the default happy-path registry.
    ///
    /// This focuses on broad correctness, security, and accessibility checks
    /// without enforcing stronger stylistic or framework-specific conventions.
    pub fn with_happy_path() -> Self {
        let mut registry = Self::with_capacity(Self::HAPPY_PATH_CAPACITY);
        // Vue correctness rules.
        registry.register(Box::new(crate::rules::vue::RequireVForKey));
        registry.register(Box::new(crate::rules::vue::ValidVFor));
        registry.register(Box::new(crate::rules::vue::NoUseVIfWithVFor));
        registry.register(Box::new(crate::rules::vue::NoUnusedVars::default()));
        registry.register(Box::new(crate::rules::vue::NoDuplicateAttributes::default()));
        registry.register(Box::new(crate::rules::vue::NoTemplateKey));
        registry.register(Box::new(crate::rules::vue::NoTextareaMustache));
        registry.register(Box::new(crate::rules::vue::ValidVElse));
        registry.register(Box::new(crate::rules::vue::ValidVIf));
        registry.register(Box::new(crate::rules::vue::ValidVOn));
        registry.register(Box::new(crate::rules::vue::ValidVBind));
        registry.register(Box::new(crate::rules::vue::ValidVModel));
        registry.register(Box::new(crate::rules::vue::ValidVShow));
        registry.register(Box::new(crate::rules::vue::NoDupeVElseIf));
        registry.register(Box::new(
            crate::rules::vue::NoReservedComponentNames::default(),
        ));
        registry.register(Box::new(crate::rules::vue::ComponentDefinitionNameCasing));
        registry.register(Box::new(crate::rules::vue::HtmlQuotes::default()));
        registry.register(Box::new(
            crate::rules::vue::MustacheInterpolationSpacing::default(),
        ));
        registry.register(Box::new(crate::rules::vue::NoLoneTemplate));
        registry.register(Box::new(crate::rules::vue::NoMultiSpaces::default()));
        registry.register(Box::new(crate::rules::vue::PropNameCasing::default()));
        registry.register(Box::new(crate::rules::vue::VOnStyle::default()));
        registry.register(Box::new(crate::rules::vue::VSlotStyle::default()));
        registry.register(Box::new(crate::rules::vue::ValidVSlot));
        registry.register(Box::new(crate::rules::vue::NoChildContent));
        registry.register(Box::new(crate::rules::vue::ValidAttributeName));
        registry.register(Box::new(crate::rules::vue::AttributeHyphenation::default()));
        registry.register(Box::new(crate::rules::vue::AttributeOrder));
        registry.register(Box::new(crate::rules::vue::NoVTextVHtmlOnComponent));
        registry.register(Box::new(crate::rules::vue::RequireComponentIs));
        registry.register(Box::new(crate::rules::vue::RequireScopedStyle));
        registry.register(Box::new(crate::rules::vue::SfcElementOrder::default()));
        registry.register(Box::new(crate::rules::vue::SingleStyleBlock));
        registry.register(Box::new(crate::rules::vue::NoUselessTemplateAttributes));
        registry.register(Box::new(crate::rules::vue::NoDeprecatedSlotAttribute));
        registry.register(Box::new(crate::rules::vue::NoDeprecatedVBindSync));
        registry.register(Box::new(crate::rules::vue::NoDeprecatedVOnNativeModifier));
        registry.register(Box::new(crate::rules::vue::NoDeprecatedSlotScopeAttribute));
        registry.register(Box::new(crate::rules::vue::NoDeprecatedScopeAttribute));
        registry.register(Box::new(crate::rules::vue::NoDeprecatedVOnNumberModifiers));
        registry.register(Box::new(crate::rules::vue::NoDeprecatedFunctionalTemplate));
        crate::rules::vue::register_valid_directives(&mut registry);
        registry.register(Box::new(crate::rules::vapor::NoVueLifecycleEvents));
        crate::rules::vue::register_security(&mut registry);
        // Accessibility rules with broadly applicable guidance.
        registry.register(Box::new(crate::rules::a11y::AnchorHasContent));
        registry.register(Box::new(crate::rules::a11y::HeadingHasContent));
        registry.register(Box::new(crate::rules::a11y::IframeHasTitle));
        registry.register(Box::new(crate::rules::a11y::NoDistractingElements));
        registry.register(Box::new(crate::rules::a11y::NoIForIcon));
        registry.register(Box::new(crate::rules::a11y::TabindexNoPositive));
        registry.register(Box::new(crate::rules::a11y::ClickEventsHaveKeyEvents));
        registry.register(Box::new(crate::rules::a11y::FormControlHasLabel));
        registry.register(Box::new(crate::rules::a11y::AriaProps));
        registry.register(Box::new(crate::rules::a11y::AriaRole::default()));
        registry.register(Box::new(crate::rules::a11y::NoAriaHiddenOnFocusable));
        registry.register(Box::new(crate::rules::a11y::NoAccessKey));
        registry.register(Box::new(crate::rules::a11y::NoAutofocus));
        registry.register(Box::new(crate::rules::a11y::NoRolePresentationOnFocusable));
        registry.register(Box::new(crate::rules::a11y::AriaUnsupportedElements));
        registry.register(Box::new(crate::rules::a11y::NoRedundantRoles));
        registry.register(Box::new(crate::rules::a11y::MouseEventsHaveKeyEvents));
        registry.register(Box::new(crate::rules::a11y::AltText));
        registry.register(Box::new(crate::rules::a11y::AnchorIsValid));
        registry.register(Box::new(crate::rules::a11y::LabelHasFor));
        registry.register(Box::new(crate::rules::a11y::InteractiveSupportsFocus));
        registry.register(Box::new(crate::rules::a11y::RoleHasRequiredAriaProps));
        registry.register(Box::new(crate::rules::a11y::MediaHasCaption));
        registry.register(Box::new(crate::rules::a11y::NoStaticElementInteractions));
        registry.register(Box::new(crate::rules::a11y::NoReferToNonExistentId));
        registry.register(Box::new(crate::rules::vue::PermittedContents));

        // HTML conformance rules.
        registry.register(Box::new(crate::rules::html::DeprecatedElement));
        registry.register(Box::new(crate::rules::html::DeprecatedAttr));
        registry.register(Box::new(crate::rules::html::NoConsecutiveBr));
        registry.register(Box::new(crate::rules::html::IdDuplication));
        registry.register(Box::new(crate::rules::html::NoDuplicateDt));
        registry.register(Box::new(crate::rules::html::NoEmptyPalpableContent));
        registry.register(Box::new(crate::rules::html::RequireDatetime));

        // SSR rules.
        registry.register(Box::new(crate::rules::ssr::NoBrowserGlobalsInSsr));
        registry.register(Box::new(crate::rules::ssr::NoHydrationMismatch));

        // Semantic analysis rules.
        registry.register(Box::new(crate::rules::vue::NoUnusedComponents::default()));
        registry.register(Box::new(crate::rules::vue::NoMutatingProps::default()));
        registry.register(Box::new(crate::rules::vue::NoUnusedProperties::default()));
        #[cfg(not(target_arch = "wasm32"))]
        registry.register(Box::new(
            crate::rules::type_aware::RequireTypedProps::default(),
        ));
        #[cfg(not(target_arch = "wasm32"))]
        registry.register(Box::new(
            crate::rules::type_aware::RequireTypedEmits::default(),
        ));
        registry
    }
}
