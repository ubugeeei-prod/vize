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
use vize_carton::String;
use vize_relief::ast::{DirectiveNode, ElementNode, ExpressionNode, PropNode};

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

/// Attributes that can be exploited with unsafe URLs
const UNSAFE_URL_ATTRS: &[&str] = &[
    "href",
    "xlink:href",
    "src",
    "srcset",
    "action",
    "formaction",
    "data",
];

fn is_url_attr(name: &str) -> bool {
    UNSAFE_URL_ATTRS
        .iter()
        .any(|attr| name.eq_ignore_ascii_case(attr))
}

fn is_router_link_tag(tag: &str) -> bool {
    tag == "router-link" || tag == "RouterLink" || tag == "nuxt-link" || tag == "NuxtLink"
}

fn normalized_scheme(value: &str) -> Option<(String, &str)> {
    let mut scheme = String::default();
    let mut saw_scheme_char = false;

    for (index, ch) in value.char_indices() {
        if ch == ':' {
            return saw_scheme_char.then(|| (scheme, &value[index + 1..]));
        }

        if ch.is_ascii_whitespace() || ch.is_ascii_control() {
            continue;
        }

        if matches!(ch, '/' | '#' | '?') {
            return None;
        }

        if ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.') {
            saw_scheme_char = true;
            scheme.push(ch.to_ascii_lowercase());
            continue;
        }

        return None;
    }

    None
}

fn is_executable_data_url(rest: &str) -> bool {
    let media_type = rest
        .trim_start_matches(|ch: char| ch.is_ascii_whitespace() || ch.is_ascii_control())
        .split(',')
        .next()
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();

    matches!(
        media_type.as_str(),
        "text/html" | "application/xhtml+xml" | "image/svg+xml" | "text/xml" | "application/xml"
    )
}

fn is_unsafe_url(value: &str) -> bool {
    let Some((scheme, rest)) = normalized_scheme(value) else {
        return false;
    };

    match scheme.as_str() {
        "javascript" | "vbscript" => true,
        "data" => is_executable_data_url(rest),
        _ => false,
    }
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
            if !is_url_attr(attr_name) {
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

        // Get the attribute name
        let attr_name = match &directive.arg {
            Some(ExpressionNode::Simple(s)) => s.content.as_str(),
            _ => return,
        };

        // Check if this is a potentially unsafe attribute
        if !is_url_attr(attr_name) {
            return;
        }

        // Skip if the element is router-link (it handles routing safely)
        if is_router_link_tag(element.tag.as_str()) {
            return;
        }

        let help_message = if attr_name == "href" {
            ctx.t("vue/no-unsafe-url.help_href")
        } else {
            ctx.t("vue/no-unsafe-url.help")
        };

        ctx.warn_with_help(
            ctx.t("vue/no-unsafe-url.message"),
            &directive.loc,
            help_message,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::NoUnsafeUrl;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoUnsafeUrl));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_static_href() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<a href="/about">About</a>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_static_javascript_src() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<iframe src="javascript:alert(1)"></iframe>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_warns_static_obfuscated_javascript_href() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<a href="java&#x0A;script:alert(1)">Link</a>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_warns_static_vbscript_formaction() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<button formaction="vbscript:msgbox(1)">Submit</button>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_warns_static_executable_data_url() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<iframe src="data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg=="></iframe>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_allows_static_image_data_url() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<img src="data:image/png;base64,iVBORw0KGgo=">"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_static_unsafe_srcset_candidate() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<img srcset="/safe.png 1x, javascript:alert(1) 2x">"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_router_link() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<router-link :to="{ name: 'profile' }">Profile</router-link>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_dynamic_href() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<a :href="userUrl">Link</a>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_warns_dynamic_src() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<iframe :src="url"></iframe>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_class_binding() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="classes"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
