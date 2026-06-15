//! vue/no-unsafe-url
//!
//! Warn about potentially unsafe URL bindings.
//!
//! Dynamic URLs in href and src attributes can be exploited for XSS
//! attacks using `javascript:` protocol or data URLs.
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
//! <img :src="imageUrl" />
//! ```
//!
//! ### Safe Patterns
//! ```vue
//! <!-- Trusted static URLs are safe -->
//! <a href="/about">About</a>
//!
//! <!-- Computed URLs with validation -->
//! <a :href="sanitizedUrl">Link</a>
//!
//! <!-- Using router-link instead of href -->
//! <router-link :to="{ name: 'profile', params: { id } }">Profile</router-link>
//! ```
//!
//! ## Best Practices
//!
//! 1. Sanitize URLs on the backend before storing
//! 2. Use `@braintree/sanitize-url` for frontend validation
//! 3. Prefer `<router-link>` over `<a :href="">`

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::url::is_unsafe_url;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode};

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

/// Attributes that are URL-bearing on any element (the name implies a URL even
/// on a custom component).
const GLOBAL_URL_ATTRS: &[&str] = &["href", "xlink:href", "src", "srcset"];

/// Attributes that are URL-bearing only on specific HTML elements. On any other
/// element — a `<div>`, or a custom component — they are ordinary props (for
/// example `<MyComponent :data="rows" />`), so treating them as URLs there is a
/// false positive.
fn is_element_scoped_url_attr(name: &str, tag: &str) -> bool {
    if name.eq_ignore_ascii_case("data") {
        // `<object data="…">` is the only element where `data` is a URL.
        tag.eq_ignore_ascii_case("object")
    } else if name.eq_ignore_ascii_case("action") {
        tag.eq_ignore_ascii_case("form")
    } else if name.eq_ignore_ascii_case("formaction") {
        tag.eq_ignore_ascii_case("button") || tag.eq_ignore_ascii_case("input")
    } else {
        false
    }
}

fn is_url_attr_on(name: &str, tag: &str) -> bool {
    GLOBAL_URL_ATTRS
        .iter()
        .any(|attr| name.eq_ignore_ascii_case(attr))
        || is_element_scoped_url_attr(name, tag)
}

fn is_router_link_tag(tag: &str) -> bool {
    tag == "router-link" || tag == "RouterLink" || tag == "nuxt-link" || tag == "NuxtLink"
}

fn is_slot_tag(tag: &str) -> bool {
    tag == "slot"
}

fn is_unsafe_static_attr_value(attr_name: &str, value: &str) -> bool {
    if attr_name.eq_ignore_ascii_case("srcset") {
        return value
            .split(',')
            .map(str::trim_start)
            .filter_map(|candidate| candidate.split_ascii_whitespace().next())
            .any(is_unsafe_url);
    }

    is_unsafe_url(value)
}

fn is_hash_only_href_binding(attr_name: &str, directive: &DirectiveNode) -> bool {
    if !attr_name.eq_ignore_ascii_case("href") {
        return false;
    }

    let Some(ExpressionNode::Simple(exp)) = directive.exp.as_ref() else {
        return false;
    };

    is_hash_prefixed_expression(exp.content.trim())
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
    for index in 1..bytes.len() {
        let byte = bytes[index];
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' {
            escaped = true;
            continue;
        }
        if byte == quote {
            return Some((&value[1..index], &value[index + 1..]));
        }
    }

    None
}

impl Rule for NoUnsafeUrl {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if is_router_link_tag(element.tag.as_str()) {
            return;
        }

        for prop in &element.props {
            let PropNode::Attribute(attr) = prop else {
                continue;
            };

            let attr_name = attr.name.as_str();
            if !is_url_attr_on(attr_name, element.tag.as_str()) {
                continue;
            }

            let Some(value) = &attr.value else {
                continue;
            };

            if !is_unsafe_static_attr_value(attr_name, value.content.as_str()) {
                continue;
            }

            ctx.warn_with_help(
                ctx.t("vue/no-unsafe-url.static_message"),
                &attr.loc,
                ctx.t("vue/no-unsafe-url.static_help"),
            );
        }
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // Only check v-bind
        if directive.name != "bind" {
            return;
        }

        // Bindings on <slot> are slot props, not URL-bearing DOM attributes.
        if is_slot_tag(element.tag.as_str()) {
            return;
        }

        // Get the attribute name
        let attr_name = match &directive.arg {
            Some(ExpressionNode::Simple(s)) => s.content.as_str(),
            _ => return,
        };

        // Check if this is a potentially unsafe attribute
        if !is_url_attr_on(attr_name, element.tag.as_str()) {
            return;
        }

        // Skip if the element is router-link (it handles routing safely)
        if is_router_link_tag(element.tag.as_str()) {
            return;
        }

        if is_hash_only_href_binding(attr_name, directive) {
            return;
        }

        let help_message = if attr_name == "href" {
            ctx.t("vue/no-unsafe-url.help_href")
        } else {
            ctx.t("vue/no-unsafe-url.help")
        };

        ctx.warn_with_help(
            ctx.t_fmt("vue/no-unsafe-url.message", &[("attr", attr_name)]),
            &directive.loc,
            help_message,
        );
    }
}

#[cfg(test)]
#[path = "no_unsafe_url_tests.rs"]
mod tests;
