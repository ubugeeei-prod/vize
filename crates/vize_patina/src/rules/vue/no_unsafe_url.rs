//! vue/no-unsafe-url
//!
//! Warn about potentially unsafe URL bindings.
//!
//! URLs in navigation and active-content attributes can be exploited for XSS
//! attacks using `javascript:` protocol or executable data URLs.
//!
//! ## Security Risks
//!
//! - JavaScript execution via `javascript:` protocol
//! - Data exfiltration via malicious URLs
//! - Phishing through open redirects
//!
//! ## Examples
//!
//! ### Requires Attention
//! ```vue
//! <!-- User-provided URLs need sanitization -->
//! <a :href="userProvidedUrl">Link</a>
//! <iframe :src="dynamicUrl"></iframe>
//! ```
//!
//! ### Safe Patterns
//! ```vue
//! <!-- Trusted static URLs are safe -->
//! <a href="/about">About</a>
//!
//! <!-- Computed URLs with validation/sanitization -->
//! <a :href="safeUrl(userProvidedUrl)">Link</a>
//!
//! <!-- Using router-link instead of href -->
//! <router-link :to="{ name: 'profile', params: { id } }">Profile</router-link>
//!
//! <!-- Image and media URLs do not execute javascript: URLs -->
//! <img :src="imageUrl" />
//! ```
//!
//! ## Best Practices
//!
//! 1. Sanitize URLs on the backend before storing
//! 2. Use `@braintree/sanitize-url` for frontend validation
//! 3. Prefer `<router-link>` over `<a :href="">`

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{
    MarkupBinding, MarkupBindingKind, MarkupContext, MarkupDirective, MarkupElement,
    MarkupElementKind, MarkupRule, MarkupTraversal,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::url::is_unsafe_url;
use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_l0::is_platform_tag;
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-unsafe-url",
    description: "Warn about potentially unsafe URL bindings",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// No unsafe URL binding rule
#[derive(Default)]
pub struct NoUnsafeUrl;

/// Only inspect URLs where the browser can navigate or run active content.
/// Component props have no known DOM destination, and image/media resource
/// attributes do not execute `javascript:` URLs.
fn is_unsafe_url_sink(name: &str, element: &MarkupElement<'_>) -> bool {
    let tag = element.tag();
    if element.kind() != MarkupElementKind::Element
        || !is_platform_tag(tag)
        || !element.is_unqualified_tag_exact(tag)
    {
        return false;
    }

    if name.eq_ignore_ascii_case("href") || name.eq_ignore_ascii_case("xlink:href") {
        matches!(tag, "a" | "area" | "base")
            || (tag == "script" && name.eq_ignore_ascii_case("xlink:href"))
    } else if name.eq_ignore_ascii_case("src") {
        matches!(tag, "iframe" | "frame" | "embed" | "script")
    } else if name.eq_ignore_ascii_case("data") {
        tag == "object"
    } else if name.eq_ignore_ascii_case("action") {
        tag == "form"
    } else if name.eq_ignore_ascii_case("formaction") {
        matches!(tag, "button" | "input")
    } else {
        false
    }
}

fn is_router_link_tag(tag: &str) -> bool {
    tag == "router-link" || tag == "RouterLink" || tag == "nuxt-link" || tag == "NuxtLink"
}

fn is_slot_tag(tag: &str) -> bool {
    tag == "slot"
}

fn expression_is_sanitizer_call(source: &str) -> bool {
    if !source.contains('(') {
        return false;
    }

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let Ok(parsed) = Parser::new(&allocator, source, source_type).parse_expression() else {
        return false;
    };
    let Some(rest) = source.get(parsed.span().end as usize..) else {
        return false;
    };
    if !rest.trim().is_empty() {
        return false;
    }

    let Expression::CallExpression(call) = parsed.get_inner_expression() else {
        return false;
    };

    match call.callee.get_inner_expression() {
        Expression::Identifier(callee) => is_sanitizer_callee_name(callee.name.as_str()),
        Expression::StaticMemberExpression(member) => {
            is_sanitizer_callee_name(member.property.name.as_str())
        }
        _ => false,
    }
}

fn is_sanitizer_callee_name(name: &str) -> bool {
    matches!(
        name,
        "sanitize"
            | "sanitizeUrl"
            | "sanitizeURL"
            | "sanitizedUrl"
            | "sanitizedURL"
            | "safe"
            | "safeUrl"
            | "safeURL"
            | "toSafeUrl"
            | "toSafeURL"
    )
}

fn is_hash_prefixed_expression(value: &str) -> bool {
    if value.starts_with("`#") && value.ends_with('`') {
        return true;
    }

    let Some((literal, rest)) = static_string_literal_prefix(value) else {
        return false;
    };
    if !literal.starts_with('#') {
        return false;
    }

    let rest = rest.trim_start();
    rest.is_empty() || rest.starts_with('+')
}

fn static_string_literal_prefix(value: &str) -> Option<(&str, &str)> {
    let bytes = value.as_bytes();
    let quote = *bytes.first()?;
    if !matches!(quote, b'\'' | b'"') {
        return None;
    }

    let mut escaped = false;
    for (index, &byte) in bytes.iter().enumerate().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' {
            escaped = true;
            continue;
        }
        if byte == quote {
            return Some((value.get(1..index)?, value.get(index + 1..)?));
        }
    }

    None
}

impl NoUnsafeUrl {
    fn check_static<'a>(ctx: &mut LintContext<'a>, element: &MarkupElement<'a>) {
        if is_router_link_tag(element.tag()) {
            return;
        }

        element.walk_bindings(&mut |binding| {
            if binding.kind() != MarkupBindingKind::Attribute {
                return;
            }
            let Some(attr_name) = binding.authored_argument(ctx.source) else {
                return;
            };
            if !is_unsafe_url_sink(attr_name, element) {
                return;
            }
            let Some(value) = binding.static_value() else {
                return;
            };
            if !is_unsafe_url(value) {
                return;
            }

            ctx.warn_at_with_help(
                ctx.t("vue/no-unsafe-url.static_message"),
                binding.range(),
                ctx.t("vue/no-unsafe-url.static_help"),
            );
        });
    }

    fn check_dynamic<'a>(
        ctx: &mut LintContext<'a>,
        element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        // Only check v-bind
        if binding.kind() != MarkupBindingKind::Bind {
            return;
        }

        // Bindings on <slot> are slot props, not URL-bearing DOM attributes.
        if is_slot_tag(element.tag()) {
            return;
        }

        // Get the attribute name
        let Some(attr_name) = binding.authored_argument(ctx.source) else {
            return;
        };
        if !binding.argument_is_static() {
            return;
        }

        // Check if this is a potentially unsafe attribute
        if !is_unsafe_url_sink(attr_name, element) {
            return;
        }

        // Skip if the element is router-link (it handles routing safely)
        if is_router_link_tag(element.tag()) {
            return;
        }

        let expression = binding.expression().or_else(|| {
            let retained = binding.retained_expression()?;
            let range = retained.authored_range(retained.ast.span())?;
            ctx.source.get(range.start as usize..range.end as usize)
        });
        if expression.map(str::trim).is_some_and(|expression| {
            (attr_name.eq_ignore_ascii_case("href") && is_hash_prefixed_expression(expression))
                || expression_is_sanitizer_call(expression)
        }) {
            return;
        }

        let help_message = if attr_name == "href" {
            ctx.t("vue/no-unsafe-url.help_href")
        } else {
            ctx.t("vue/no-unsafe-url.help")
        };

        ctx.warn_at_with_help(
            ctx.t_fmt("vue/no-unsafe-url.message", &[("attr", attr_name)]),
            binding.range(),
            help_message,
        );
    }
}

impl MarkupRule for NoUnsafeUrl {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::ElementsAndDirectives
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if !ctx.is_jsx_attribute_value() {
            Self::check_static(ctx.lint(), element);
        }
    }

    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
        if !ctx.is_jsx_attribute_value() {
            Self::check_dynamic(ctx.lint(), element, &directive.as_binding());
        }
    }
}

impl Rule for NoUnsafeUrl {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn template_hooks(&self) -> crate::rule::RuleHooks {
        crate::rule::RuleHooks::ELEMENT.union(crate::rule::RuleHooks::DIRECTIVE)
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn jsx_needs_lowering(&self) -> bool {
        // Preserve lowering-only parser diagnostics from the original JSX lane.
        true
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check_static(ctx, &MarkupElement::new(element));
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        Self::check_dynamic(
            ctx,
            &MarkupElement::new(element),
            &MarkupBinding::from_relief_directive(directive),
        );
    }
}

#[cfg(test)]
mod tests;
