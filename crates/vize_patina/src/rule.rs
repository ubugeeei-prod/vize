//! Rule trait and registry for lint rules.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::preset::LintPreset;
use vize_relief::ast::{DirectiveNode, ElementNode, ForNode, IfNode, InterpolationNode, RootNode};

/// Rule category for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCategory {
    /// Essential rules (vue/essential) - prevent errors
    Essential,
    /// Strongly recommended rules (vue/strongly-recommended)
    StronglyRecommended,
    /// Recommended rules (vue/recommended)
    Recommended,
    /// Vapor mode specific rules
    Vapor,
    /// Musea (Art file / Storybook) specific rules
    Musea,
    /// Accessibility (a11y) rules
    Accessibility,
    /// HTML conformance rules
    HtmlConformance,
    /// Type-aware rules (require semantic analysis)
    TypeAware,
}

/// Rule metadata
pub struct RuleMeta {
    /// Rule name (e.g., "vue/require-v-for-key")
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Rule category
    pub category: RuleCategory,
    /// Whether rule is auto-fixable
    pub fixable: bool,
    /// Default severity
    pub default_severity: Severity,
}

/// Rule trait for implementing lint rules
///
/// Rules implement visitor-like methods that are called during AST traversal.
/// Each method receives a mutable reference to LintContext for reporting diagnostics.
pub trait Rule: Send + Sync {
    /// Get rule metadata
    fn meta(&self) -> &'static RuleMeta;

    /// Run on the full SFC source before template extraction.
    #[allow(unused_variables)]
    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {}

    /// Run on template root node (called once per template)
    #[allow(unused_variables)]
    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {}

    /// Called when entering an element node
    #[allow(unused_variables)]
    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {}

    /// Called when exiting an element node
    #[allow(unused_variables)]
    fn exit_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {}

    /// Called for each directive on an element
    #[allow(unused_variables)]
    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
    }

    /// Called for v-for nodes
    #[allow(unused_variables)]
    fn check_for<'a>(&self, ctx: &mut LintContext<'a>, for_node: &ForNode<'a>) {}

    /// Called for v-if nodes
    #[allow(unused_variables)]
    fn check_if<'a>(&self, ctx: &mut LintContext<'a>, if_node: &IfNode<'a>) {}

    /// Called for interpolation nodes {{ expr }}
    #[allow(unused_variables)]
    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
    }
}

/// Registry holding all enabled lint rules
pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
    const ESSENTIAL_CAPACITY: usize = 32;
    const HAPPY_PATH_CAPACITY: usize = 90;
    /// Create a new empty registry
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            rules: Vec::with_capacity(capacity),
        }
    }

    /// Register a rule
    pub fn register(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    /// Add a rule (alias for register)
    pub fn add(&mut self, rule: Box<dyn Rule>) {
        self.register(rule);
    }

    /// Get all registered rules
    pub fn rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }

    /// Check whether a rule with the given name is registered.
    pub fn has_rule(&self, name: &str) -> bool {
        self.rules.iter().any(|rule| rule.meta().name == name)
    }

    /// Create a registry for a named preset.
    pub fn with_preset(preset: LintPreset) -> Self {
        match preset {
            LintPreset::HappyPath => Self::with_happy_path(),
            LintPreset::Opinionated => Self::with_opinionated(),
            LintPreset::Essential => Self::with_essential(),
            LintPreset::Nuxt => Self::with_nuxt(),
        }
    }

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
        registry.register(Box::new(crate::rules::vue::PropNameCasing));
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
        registry.register(Box::new(crate::rules::vue::SfcElementOrder));
        registry.register(Box::new(crate::rules::vue::SingleStyleBlock));
        registry.register(Box::new(crate::rules::vue::NoUselessTemplateAttributes));
        registry.register(Box::new(crate::rules::vue::ValidVMemo));
        registry.register(Box::new(crate::rules::vapor::NoVueLifecycleEvents));

        // Security rules.
        registry.register(Box::new(crate::rules::vue::NoVHtml));
        registry.register(Box::new(crate::rules::vue::NoUnsafeUrl));

        // Accessibility rules with broadly applicable guidance.
        registry.register(Box::new(crate::rules::a11y::ImgAlt));
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
        registry.register(Box::new(crate::rules::vue::NoMutatingProps));
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

    /// Backward-compatible alias for the default preset.
    pub fn with_recommended() -> Self {
        Self::with_happy_path()
    }

    /// Create registry with only essential rules (errors only)
    ///
    /// Use this for minimal checking that only catches definite errors.
    pub fn with_essential() -> Self {
        let mut registry = Self::with_capacity(Self::ESSENTIAL_CAPACITY);

        // Vue Essential Rules only
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
        registry.register(Box::new(crate::rules::vue::ValidVSlot));
        registry.register(Box::new(
            crate::rules::vue::MultiWordComponentNames::default(),
        ));
        registry.register(Box::new(crate::rules::vue::NoChildContent));
        registry.register(Box::new(crate::rules::vue::ValidAttributeName));
        registry.register(Box::new(crate::rules::vue::NoVTextVHtmlOnComponent));
        registry.register(Box::new(crate::rules::vue::RequireComponentIs));
        registry.register(Box::new(crate::rules::vue::NoUselessTemplateAttributes));
        registry.register(Box::new(crate::rules::vue::ValidVMemo));
        registry.register(Box::new(crate::rules::vue::UseVOnExact));

        // Security Rules
        registry.register(Box::new(crate::rules::vue::NoVHtml));
        registry.register(Box::new(crate::rules::vue::NoUnsafeUrl));

        // HTML Conformance (essential)
        registry.register(Box::new(crate::rules::html::IdDuplication));

        registry
    }

    /// Create registry with the strongest built-in preset enabled.
    pub fn with_opinionated() -> Self {
        let mut registry = Self::with_happy_path();
        crate::rules::opinionated::register(&mut registry);

        registry
    }

    /// Create registry with all available rules (including opt-in).
    pub fn with_all() -> Self {
        Self::with_opinionated()
    }

    /// Create registry with Nuxt-friendly rules (auto-imports enabled).
    pub fn with_nuxt() -> Self {
        let mut registry = Self::with_happy_path();
        crate::rules::opinionated::register_nuxt(&mut registry);

        registry
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::with_preset(LintPreset::default())
    }
}
