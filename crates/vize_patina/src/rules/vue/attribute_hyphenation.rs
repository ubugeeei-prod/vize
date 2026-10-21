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
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::{String, ToCompactString, camelize, hyphenate, is_native_tag};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode};

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
    pub style: HyphenationStyle,
    /// Attributes to ignore
    pub ignore: Vec<String>,
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
        }
    }
}

impl AttributeHyphenation {
    pub fn new(style: HyphenationStyle) -> Self {
        Self {
            style,
            ..Self::default()
        }
    }

    fn is_custom_component(element: &ElementNode<'_>) -> bool {
        let tag = element.tag;
        // Custom components are either:
        // 1. PascalCase (starts with uppercase)
        // 2. Contains hyphen (kebab-case component)
        // 3. Not a known native HTML/SVG/MathML element
        // 4. A native element with an `is` customization
        if tag.chars().next().is_some_and(|c| c.is_uppercase()) {
            return true;
        }
        if tag.contains('-') {
            return true;
        }
        if !is_native_tag(tag) {
            return true;
        }
        element.props.iter().any(Self::is_customized_builtin)
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

    fn static_directive_arg<'a>(dir: &DirectiveNode<'a>) -> Option<&'a str> {
        if !matches!(dir.name, "bind" | "model") {
            return None;
        }

        match dir.arg.as_ref()? {
            ExpressionNode::Simple(s) if s.is_static => Some(s.content),
            _ => None,
        }
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

    fn is_customized_builtin(prop: &PropNode<'_>) -> bool {
        match prop {
            PropNode::Attribute(attr) => attr.name == "is",
            PropNode::Directive(dir) if dir.name == "is" => true,
            PropNode::Directive(dir) if dir.name == "bind" => {
                Self::static_directive_arg(dir) == Some("is")
            }
            PropNode::Directive(_) => false,
        }
    }
}

impl Rule for AttributeHyphenation {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Only check custom components
        if !Self::is_custom_component(element) {
            return;
        }

        for prop in &element.props {
            let (name, loc) = match prop {
                PropNode::Attribute(attr) => (attr.name, &attr.name_loc),
                PropNode::Directive(dir) => {
                    let Some(ExpressionNode::Simple(arg)) = dir.arg.as_ref() else {
                        continue;
                    };
                    if !matches!(dir.name, "bind" | "model") || !arg.is_static {
                        continue;
                    }
                    (arg.content, &arg.loc)
                }
            };

            // Skip ignored attributes
            if self.should_ignore(name) {
                continue;
            }

            // Skip directive shorthand attributes when they were parsed as plain attributes.
            if name.starts_with("v-") || name.starts_with('@') || name.starts_with('#') {
                continue;
            }

            match self.style {
                HyphenationStyle::Always => {
                    if Self::requires_hyphenation(name) {
                        report(
                            ctx,
                            "vue/attribute-hyphenation.message",
                            "vue/attribute-hyphenation.help",
                            loc.span.start,
                            loc.span.end,
                            hyphenate(name),
                        );
                    }
                }
                HyphenationStyle::Never => {
                    if Self::forbids_hyphenation(name) {
                        report(
                            ctx,
                            "vue/attribute-hyphenation.message_never",
                            "vue/attribute-hyphenation.help_never",
                            loc.span.start,
                            loc.span.end,
                            camelize(name),
                        );
                    }
                }
            }
        }
    }
}

fn report(
    ctx: &mut LintContext<'_>,
    message: &str,
    help: &str,
    start: u32,
    end: u32,
    replacement: impl Into<String>,
) {
    let message = ctx.t(message).into_owned();
    let help = ctx.t(help).into_owned();
    ctx.report(
        LintDiagnostic::warn(META.name, message, start, end)
            .with_help(help.clone())
            .with_fix(Fix::new(help, TextEdit::replace(start, end, replacement))),
    );
}

#[cfg(test)]
mod tests {
    use super::{AttributeHyphenation, HyphenationStyle};
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(AttributeHyphenation::default()));
        Linter::with_registry(registry)
    }

    fn warning_count(template: &str) -> usize {
        create_linter()
            .lint_template(template, "test.vue")
            .warning_count
    }

    fn never_warning_count(template: &str) -> usize {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(AttributeHyphenation::new(HyphenationStyle::Never)));
        Linter::with_registry(registry)
            .lint_template(template, "test.vue")
            .warning_count
    }

    fn fixed(linter: &Linter, source: &str) -> String {
        let result = linter.lint_template(source, "test.vue");
        let mut fixed = source.to_owned();
        let mut edits: Vec<_> = result
            .diagnostics
            .iter()
            .flat_map(|diagnostic| diagnostic.fix.as_ref().unwrap().edits.iter())
            .collect();
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.start));
        for edit in edits {
            fixed.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
        }
        fixed
    }

    #[test]
    fn test_valid_hyphenated() {
        assert_eq!(warning_count(r#"<MyComponent my-prop="value" />"#), 0);
    }

    #[test]
    fn test_invalid_camel_case() {
        assert_eq!(warning_count(r#"<MyComponent myProp="value" />"#), 1);
    }

    #[test]
    fn test_invalid_uppercase_initial_and_lowercase_custom_component() {
        assert_eq!(
            warning_count(r#"<div><MyComponent DOMId="value" /><draggable itemKey="id" /></div>"#,),
            2,
        );
    }

    #[test]
    fn test_invalid_bound_camel_case_argument() {
        assert_eq!(warning_count(r#"<MyComponent :activeKey="value" />"#), 1);
    }

    #[test]
    fn test_invalid_model_camel_case_argument() {
        assert_eq!(
            warning_count(r#"<MyComponent v-model:activeKey.trim="value" />"#,),
            1,
        );
    }

    #[test]
    fn test_invalid_on_prefixed_prop() {
        assert_eq!(warning_count(r#"<MyComponent :onUndo="undo" />"#), 1);
    }

    #[test]
    fn test_valid_event_argument() {
        assert_eq!(warning_count(r#"<MyComponent @onUndo="undo" />"#), 0);
    }

    #[test]
    fn test_valid_dynamic_argument() {
        assert_eq!(warning_count(r#"<MyComponent :[myProp]="value" />"#), 0);
    }

    #[test]
    fn test_valid_dynamic_model_argument() {
        assert_eq!(
            warning_count(r#"<MyComponent v-model:[activeKey]="value" />"#),
            0
        );
    }

    #[test]
    fn test_valid_svg_weird_case_attributes() {
        assert_eq!(
            warning_count(
                r#"<MyIcon viewBox="0 0 16 16" :preserveAspectRatio="ratio" customCamel="x" />"#,
            ),
            1,
        );
    }

    #[test]
    fn test_invalid_customized_builtin() {
        assert_eq!(
            warning_count(r#"<div is="vue:MyRow" :rowData="row" /><div :is="MyRow" rowData />"#),
            2,
        );
    }

    #[test]
    fn test_valid_html_element() {
        assert_eq!(warning_count(r#"<div onClick="handler"></div>"#), 0);
    }

    #[test]
    fn test_valid_data_attribute() {
        assert_eq!(warning_count(r#"<MyComponent data-testId="123" />"#), 0);
    }

    #[test]
    fn test_never_invalid_hyphenated_attribute() {
        assert_eq!(never_warning_count(r#"<MyComponent my-prop="value" />"#), 1);
        assert_eq!(
            never_warning_count(r#"<MyComponent :active-key="value" />"#),
            1
        );
    }

    #[test]
    fn test_never_valid_camel_case_attribute_and_native_elements() {
        assert_eq!(
            never_warning_count(r#"<MyComponent myProp="value" /><div data-test="value" />"#),
            0
        );
    }

    #[test]
    fn test_camel_case_aria_and_data_props_are_allowed() {
        // Hyphenated `aria-*` / `data-*` names bypass prop type checking in
        // vue-tsc, so camelCase is the only spelling that type-checks a required
        // `ariaLabel` prop.
        assert_eq!(
            warning_count(
                r#"<div><IconButton ariaLabel="Close" /><MyComponent :ariaLabelledby="id" :dataSource="rows" /></div>"#,
            ),
            0,
        );
        assert_eq!(
            warning_count(r#"<IconButton aria-label="Close" data-testid="x" />"#),
            0
        );
    }

    #[test]
    fn test_names_that_only_start_like_aria_or_data_still_need_hyphens() {
        assert_eq!(
            warning_count(r#"<MyComponent dataset="x" ariadne="y" :databaseName="n" />"#),
            1,
        );
    }

    #[test]
    fn fixes_authored_static_and_bound_attribute_names() {
        let linter = create_linter();
        let source = r#"<UserBadge displayName="a" :activeKey="key" v-model:pageSize="size" />"#;
        let fixed = fixed(&linter, source);
        assert_eq!(
            fixed,
            r#"<UserBadge display-name="a" :active-key="key" v-model:page-size="size" />"#
        );
        assert_eq!(linter.lint_template(&fixed, "test.vue").warning_count, 0);
    }

    #[test]
    fn never_fixes_authored_attribute_names_to_camel_case() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(AttributeHyphenation::new(HyphenationStyle::Never)));
        let linter = Linter::with_registry(registry);
        let source = r#"<UserBadge display-name="a" :active-key="key" />"#;
        let fixed = fixed(&linter, source);
        assert_eq!(fixed, r#"<UserBadge displayName="a" :activeKey="key" />"#);
        assert_eq!(linter.lint_template(&fixed, "test.vue").warning_count, 0);
    }
}
