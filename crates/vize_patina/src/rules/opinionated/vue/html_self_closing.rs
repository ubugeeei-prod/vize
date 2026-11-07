//! vue/html-self-closing
//!
//! Enforce self-closing style for HTML elements.
//!
//! ## Examples
//!
//! ### Invalid (default config)
//! ```vue
//! <div></div>  <!-- should be <div /> when empty -->
//! <img />      <!-- should be <img> -->
//! <br />       <!-- should be <br> -->
//! ```
//!
//! ### Valid
//! ```vue
//! <div />
//! <img>
//! <br>
//! <div>content</div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::opinionated::vue::authored_element::closing;
use vize_l0::{Span, is_html_tag, is_math_ml_tag, is_svg_tag, is_void_tag};
use vize_relief::{ElementNode, ElementType, ExpressionNode, Namespace, PropNode};

mod fix;

static META: RuleMeta = RuleMeta {
    name: "vue/html-self-closing",
    description: "Enforce self-closing style",
    category: RuleCategory::StronglyRecommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Per-element self-closing policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlSelfClosingStyle {
    /// Empty elements must use self-closing syntax.
    Always,
    /// Elements must use separate start and end tags.
    Never,
    /// Both forms are accepted.
    Any,
}

/// Self-closing policy for HTML element families.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HtmlSelfClosingHtmlOptions {
    pub void: HtmlSelfClosingStyle,
    pub normal: HtmlSelfClosingStyle,
    pub component: HtmlSelfClosingStyle,
}

impl Default for HtmlSelfClosingHtmlOptions {
    fn default() -> Self {
        Self {
            void: HtmlSelfClosingStyle::Never,
            normal: HtmlSelfClosingStyle::Always,
            component: HtmlSelfClosingStyle::Always,
        }
    }
}

/// Config for `vue/html-self-closing`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HtmlSelfClosingOptions {
    pub html: HtmlSelfClosingHtmlOptions,
    pub svg: HtmlSelfClosingStyle,
    pub math: HtmlSelfClosingStyle,
}

impl Default for HtmlSelfClosingOptions {
    fn default() -> Self {
        Self {
            html: HtmlSelfClosingHtmlOptions::default(),
            svg: HtmlSelfClosingStyle::Always,
            math: HtmlSelfClosingStyle::Always,
        }
    }
}

/// HTML self-closing style rule
#[derive(Default)]
pub struct HtmlSelfClosing {
    options: HtmlSelfClosingOptions,
}

impl HtmlSelfClosing {
    pub const fn new(options: HtmlSelfClosingOptions) -> Self {
        Self { options }
    }
}

impl Rule for HtmlSelfClosing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        check_element(ctx, element, self.options, false);
    }
}

/// Nuxt-preset variant of [`HtmlSelfClosing`] that also exempts
/// framework-registered Vuetify 2 components (`v-*` tags) from
/// self-closing diagnostics.
///
/// Vuetify components are globally registered so the linter cannot determine
/// their preferred closing style from source. Enabling the exemption in the
/// Nuxt preset keeps real Nuxt + Vuetify projects out of a self-closing
/// warning storm without loosening the rule for other presets.
pub(crate) struct HtmlSelfClosingNuxt {
    options: HtmlSelfClosingOptions,
}

impl HtmlSelfClosingNuxt {
    pub(crate) const fn new(options: HtmlSelfClosingOptions) -> Self {
        Self { options }
    }
}

impl Default for HtmlSelfClosingNuxt {
    fn default() -> Self {
        Self::new(HtmlSelfClosingOptions::default())
    }
}

impl Rule for HtmlSelfClosingNuxt {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        check_element(ctx, element, self.options, true);
    }
}

fn check_element<'a>(
    ctx: &mut LintContext<'a>,
    element: &ElementNode<'a>,
    options: HtmlSelfClosingOptions,
    allow_vuetify_tags: bool,
) {
    // vue-eslint-parser treats the contents of <noscript> as raw text. Relief
    // retains the authored nodes for editor features, but this compatibility
    // rule must not lint markup that upstream cannot observe.
    if ctx
        .element_stack
        .iter()
        .take(ctx.element_stack.len().saturating_sub(1))
        .any(|ancestor| ancestor.tag == "noscript")
    {
        return;
    }

    let tag = element.tag;
    if allow_vuetify_tags && is_vuetify_tag(tag) {
        return;
    }

    let (style, message) = match classify_element(element) {
        SelfClosingElementKind::VoidHtml => {
            (options.html.void, ctx.t("vue/html-self-closing.void"))
        }
        SelfClosingElementKind::NormalHtml => {
            (options.html.normal, ctx.t("vue/html-self-closing.empty"))
        }
        SelfClosingElementKind::Component => (
            options.html.component,
            ctx.t("vue/html-self-closing.component"),
        ),
        SelfClosingElementKind::Svg => (options.svg, ctx.t("vue/html-self-closing.empty")),
        SelfClosingElementKind::Math => (options.math, ctx.t("vue/html-self-closing.empty")),
        SelfClosingElementKind::Other => return,
    };
    if matches!(style, HtmlSelfClosingStyle::Any) {
        return;
    }

    let is_self_closing = element.is_self_closing || authored_self_closing(ctx, element);
    let is_empty = authored_empty(ctx, element);

    match style {
        HtmlSelfClosingStyle::Always if is_empty && !is_self_closing => {
            report(
                ctx,
                element,
                style,
                message,
                ctx.t("vue/html-self-closing.help"),
            );
        }
        HtmlSelfClosingStyle::Never if is_self_closing => {
            report(
                ctx,
                element,
                style,
                ctx.t("vue/html-self-closing.never"),
                ctx.t("vue/html-self-closing.never_help"),
            );
        }
        _ => {}
    }
}

fn report(
    ctx: &mut LintContext<'_>,
    element: &ElementNode<'_>,
    style: HtmlSelfClosingStyle,
    message: std::borrow::Cow<'_, str>,
    help: std::borrow::Cow<'_, str>,
) {
    let span = diagnostic_span(ctx, element, style);
    let message: vize_l0::String = message.as_ref().into();
    let mut diagnostic = LintDiagnostic::warn(ctx.current_rule, message, span.start, span.end);
    if let Some(processed) = ctx.help_level().process(&help) {
        diagnostic = diagnostic.with_help(processed);
    }
    if !ctx.is_petite_vue()
        && let Some(fix) = fix::element_fix(ctx.source, element, style, &help)
    {
        diagnostic = diagnostic.with_fix(fix);
    }
    ctx.report(diagnostic);
}

fn authored_empty(ctx: &LintContext<'_>, element: &ElementNode<'_>) -> bool {
    if element.ns == Namespace::Html && is_void_tag(element.tag) {
        return true;
    }
    let Some(close) = closing(ctx.source, element) else {
        // Do not infer emptiness from a recovered parser node. Invalid or
        // mismatched end tags can leave a node with no children even though the
        // authored range is not provably empty.
        return !ctx.filename.ends_with(".vue") && element.children.is_empty();
    };
    ctx.source
        .get(element.loc.span.end as usize..close.start)
        .is_some_and(|content| content.trim().is_empty())
}

fn diagnostic_span(
    _ctx: &LintContext<'_>,
    element: &ElementNode<'_>,
    _style: HtmlSelfClosingStyle,
) -> Span {
    // eslint-plugin-vue reports the authored start tag for both directions.
    // The closing tag (or just its slash) is an implementation detail of the
    // edit and is a much less useful diagnostic anchor.
    element.loc.span
}

#[derive(Clone, Copy)]
enum SelfClosingElementKind {
    VoidHtml,
    NormalHtml,
    Component,
    Svg,
    Math,
    Other,
}

fn classify_element(element: &ElementNode<'_>) -> SelfClosingElementKind {
    let tag = element.tag;
    // A custom element named <view> is also an SVG tag. Relief consequently
    // propagates the SVG namespace to its children, while vue-eslint-parser
    // only enters SVG mode for an authored <svg>. HTML void tags retain their
    // HTML closing contract even in that recovered namespace.
    if is_void_tag(tag) {
        return SelfClosingElementKind::VoidHtml;
    }
    match element.ns {
        Namespace::Svg => return SelfClosingElementKind::Svg,
        Namespace::MathMl => return SelfClosingElementKind::Math,
        Namespace::Html => {}
    }

    if is_svg_tag(tag) {
        return SelfClosingElementKind::Svg;
    }

    if is_math_ml_tag(tag) {
        return SelfClosingElementKind::Math;
    }

    if element.tag_type == ElementType::Component
        || has_component_cast(element)
        || !is_html_element_name(tag)
    {
        return SelfClosingElementKind::Component;
    }

    if is_html_element_name(tag) {
        return SelfClosingElementKind::NormalHtml;
    }

    SelfClosingElementKind::Other
}

fn is_html_element_name(tag: &str) -> bool {
    if tag == "slot" {
        return true;
    }
    if tag.bytes().all(|b| !b.is_ascii_uppercase()) {
        return is_html_tag(tag);
    }
    is_html_tag(&tag.to_lowercase())
}

fn has_component_cast(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attribute) => attribute.name == "is",
        PropNode::Directive(directive) => {
            directive.name == "is"
                || (directive.name == "bind"
                    && matches!(
                        directive.arg.as_ref(),
                        Some(ExpressionNode::Simple(arg))
                            if arg.is_static && arg.content == "is"
                    ))
        }
    })
}

fn authored_self_closing(ctx: &LintContext<'_>, element: &ElementNode<'_>) -> bool {
    let start = element.loc.span.start as usize;
    let end = element.loc.span.end as usize;
    let Some(open_tag) = ctx.source.get(start..end) else {
        return false;
    };
    let mut saw_close = false;
    for byte in open_tag.bytes().rev() {
        match byte {
            b'>' if !saw_close => saw_close = true,
            b'/' if saw_close => return true,
            b' ' | b'\t' | b'\n' | b'\r' | 0x0c if saw_close => {}
            _ if saw_close => return false,
            _ => {}
        }
    }
    false
}

/// Matches the Vuetify `v-*` tag convention (e.g. `v-btn`, `v-dialog`).
fn is_vuetify_tag(tag: &str) -> bool {
    matches!(tag.as_bytes(), [b'v', b'-', third, ..] if third.is_ascii_lowercase())
}

#[cfg(test)]
mod tests;
