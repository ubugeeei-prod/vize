//! vue/attribute-hyphenation
//!
//! Enforce attribute naming style on custom components.
//!
//! ## Examples
//!
//! ### Invalid (default: always)
//! ```vue
//! <MyComponent myProp="value" />
//! <MyComponent :myProp="value" />
//! ```
//!
//! ### Valid
//! ```vue
//! <MyComponent my-prop="value" />
//! <MyComponent :my-prop="value" />
//! <IconButton ariaLabel="Close" :dataTestid="id" />
//! ```
//!
//! `aria*` / `data*` camelCase names are exempt in `always` mode: template type
//! checkers (vue-tsc and Vize) treat hyphenated `aria-*` / `data-*` names as
//! fallthrough attributes instead of camelizing them into props, so writing a
//! component's `ariaLabel` prop as `aria-label` silently drops its type check —
//! and cannot satisfy a *required* `ariaLabel` prop at all.

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::ir::ByteRange;
use crate::markup::{MarkupContext, MarkupElement, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::vue::require_v_for_key::markup_element_is_custom_component;
use vize_l0::{String, ToCompactString, camelize, hyphenate};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/attribute-hyphenation",
    description: "Enforce attribute naming style on custom components",
    category: RuleCategory::StronglyRecommended,
    fixable: true,
    default_severity: Severity::Warning,
};

const SVG_ATTRIBUTES_WEIRD_CASE: &[&str] = &[
    "attributeName",
    "attributeType",
    "baseFrequency",
    "baseProfile",
    "calcMode",
    "clipPathUnits",
    "contentScriptType",
    "contentStyleType",
    "diffuseConstant",
    "edgeMode",
    "externalResourcesRequired",
    "filterRes",
    "filterUnits",
    "glyphRef",
    "gradientTransform",
    "gradientUnits",
    "kernelMatrix",
    "kernelUnitLength",
    "keyPoints",
    "keySplines",
    "keyTimes",
    "lengthAdjust",
    "limitingConeAngle",
    "markerHeight",
    "markerUnits",
    "markerWidth",
    "maskContentUnits",
    "maskUnits",
    "numOctaves",
    "pathLength",
    "patternContentUnits",
    "patternTransform",
    "patternUnits",
    "pointsAtX",
    "pointsAtY",
    "pointsAtZ",
    "preserveAlpha",
    "preserveAspectRatio",
    "primitiveUnits",
    "referrerPolicy",
    "refX",
    "refY",
    "repeatCount",
    "repeatDur",
    "requiredExtensions",
    "requiredFeatures",
    "specularConstant",
    "specularExponent",
    "spreadMethod",
    "startOffset",
    "stdDeviation",
    "stitchTiles",
    "surfaceScale",
    "systemLanguage",
    "tableValues",
    "targetX",
    "targetY",
    "textLength",
    "viewBox",
    "viewTarget",
    "xChannelSelector",
    "yChannelSelector",
    "zoomAndPan",
];

/// Attribute hyphenation style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HyphenationStyle {
    /// Require hyphenated attribute names: my-prop
    #[default]
    Always,
    /// Allow camelCase: myProp
    Never,
}

/// Attribute hyphenation rule
pub struct AttributeHyphenation {
    /// Required casing for authored custom-component attributes.
    pub style: HyphenationStyle,
    /// Attributes to ignore
    pub ignore: Vec<String>,
    /// Case-sensitive authored tag names whose attributes are exempt.
    /// Regex policies are not part of this literal-only rule option.
    pub ignore_tags: Vec<String>,
}

impl Default for AttributeHyphenation {
    fn default() -> Self {
        Self {
            style: HyphenationStyle::Always,
            ignore: vec![
                // Common data attributes
                "data-".to_compact_string(),
                "aria-".to_compact_string(),
                // Vue specific
                "slot-scope".to_compact_string(),
            ],
            ignore_tags: Vec::new(),
        }
    }
}

impl AttributeHyphenation {
    /// Create the rule with the required attribute-name style.
    pub fn new(style: HyphenationStyle) -> Self {
        Self {
            style,
            ..Self::default()
        }
    }

    fn should_ignore(&self, name: &str) -> bool {
        if SVG_ATTRIBUTES_WEIRD_CASE
            .iter()
            .any(|attr| name.contains(attr))
        {
            return true;
        }

        self.ignore
            .iter()
            .any(|pattern| name.contains(pattern.as_str()))
    }

    fn requires_hyphenation(name: &str) -> bool {
        name.chars().any(char::is_uppercase) && !Self::is_camel_aria_or_data_prop(name)
    }

    /// `ariaLabel`, `dataTestid`: props whose hyphenated spelling would be
    /// type-checked as a fallthrough `aria-*` / `data-*` attribute instead.
    fn is_camel_aria_or_data_prop(name: &str) -> bool {
        ["aria", "data"].iter().any(|prefix| {
            name.strip_prefix(prefix)
                .and_then(|rest| rest.chars().next())
                .is_some_and(char::is_uppercase)
        })
    }

    fn forbids_hyphenation(name: &str) -> bool {
        name.contains('-')
    }

    fn check(&self, ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        if !markup_element_is_custom_component(element)
            || self
                .ignore_tags
                .iter()
                .any(|tag| tag.as_str() == element.tag())
        {
            return;
        }

        element.walk_attributes(&mut |attribute| {
            self.check_name(
                ctx,
                attribute.name(),
                attribute.name_range(),
                attribute.range(),
            );
        });
        element.walk_directives(&mut |directive| {
            if !matches!(directive.name(), "bind" | "model") {
                return;
            }
            let Some(name) = directive.arg_name() else {
                return;
            };
            if !directive.is_static_arg_exact(name) {
                return;
            }
            let Some(name_range) = directive.argument_range() else {
                return;
            };
            self.check_name(ctx, name, name_range, directive.range());
        });
    }

    fn check_name(
        &self,
        ctx: &mut LintContext<'_>,
        name: &str,
        name_range: ByteRange,
        range: ByteRange,
    ) {
        if self.should_ignore(name)
            || name.starts_with("v-")
            || name.starts_with('@')
            || name.starts_with('#')
        {
            return;
        }
        match self.style {
            HyphenationStyle::Always if Self::requires_hyphenation(name) => report(
                ctx,
                "vue/attribute-hyphenation.message",
                "vue/attribute-hyphenation.help",
                range,
                name_range,
                hyphenate(name),
            ),
            HyphenationStyle::Never if Self::forbids_hyphenation(name) => report(
                ctx,
                "vue/attribute-hyphenation.message_never",
                "vue/attribute-hyphenation.help_never",
                range,
                name_range,
                camelize(name),
            ),
            HyphenationStyle::Always | HyphenationStyle::Never => {}
        }
    }
}

impl MarkupRule for AttributeHyphenation {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        self.check(ctx.lint(), element);
    }
}

impl Rule for AttributeHyphenation {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        self.check(ctx, &MarkupElement::new(element));
    }
}

fn report(
    ctx: &mut LintContext<'_>,
    message: &str,
    help: &str,
    range: ByteRange,
    name: ByteRange,
    replacement: impl Into<String>,
) {
    let message = ctx.t(message).into_owned();
    let help = ctx.t(help).into_owned();
    let edit = TextEdit::replace(name.start, name.end, replacement);
    ctx.report(
        LintDiagnostic::warn(META.name, message, range.start, range.end)
            .with_help(help.clone())
            .with_fix(Fix::new(help, edit)),
    );
}

#[cfg(test)]
mod ignore_tags_tests;
#[cfg(test)]
mod tests;
