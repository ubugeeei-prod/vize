//! Keep the established scoped CSS printer for flat rules while routing
//! nested and slotted selectors through the Vite selector model.

use vize_carton::String;

pub(crate) fn scope_css(css: &str, scope_id: &str) -> String {
    if needs_vite_scoping(css) {
        crate::vite_plugin::scope_css_for_pipeline(css, scope_id)
    } else {
        let bump = vize_carton::pool::acquire();
        super::scoped::apply_scoped_css(&bump, css, scope_id).into()
    }
}

pub(crate) fn scope_style_css(css: &str, scope_id: &str) -> String {
    if needs_vite_scoping(css) {
        crate::vite_plugin::scope_css_for_pipeline(css, scope_id)
    } else {
        crate::style::apply_scoped_css(css, scope_id)
    }
}

fn needs_vite_scoping(css: &str) -> bool {
    if css.contains(":slotted(") || css.contains("::v-slotted(") {
        return true;
    }

    let bytes = css.as_bytes();
    let mut index = 0;
    let mut header_start = 0;
    let mut style_rule_depth = 0;
    let mut rule_stack = Vec::new();
    while index < bytes.len() {
        match bytes.get(index).copied() {
            Some(b'/') if bytes.get(index + 1) == Some(&b'*') => {
                index += 2;
                while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/')
                {
                    index += 1;
                }
                index = (index + 2).min(bytes.len());
                continue;
            }
            Some(quote @ (b'\'' | b'"')) => {
                index += 1;
                while index < bytes.len() {
                    if bytes.get(index) == Some(&b'\\') {
                        index += 2;
                    } else if bytes.get(index) == Some(&quote) {
                        index += 1;
                        break;
                    } else {
                        index += 1;
                    }
                }
                continue;
            }
            Some(b'{') => {
                let header = css.get(header_start..index).unwrap_or_default().trim();
                let is_style = !header.starts_with('@');
                if is_style && style_rule_depth > 0 {
                    return true;
                }
                rule_stack.push(is_style);
                style_rule_depth += usize::from(is_style);
                header_start = index + 1;
            }
            Some(b'}') => {
                if rule_stack.pop() == Some(true) {
                    style_rule_depth -= 1;
                }
                header_start = index + 1;
            }
            Some(b';') => header_start = index + 1,
            _ => {}
        }
        index += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::needs_vite_scoping;

    #[test]
    fn selects_only_nested_rules_and_slotted_selectors() {
        assert!(needs_vite_scoping(".a { & .b { color: red; } }"));
        assert!(needs_vite_scoping(".a { color: red; .b { color: blue; } }"));
        assert!(needs_vite_scoping(".a > :slotted(.b) { color: red; }"));
        assert!(!needs_vite_scoping(
            "@keyframes spin { to { opacity: 0; } }"
        ));
        assert!(!needs_vite_scoping("@media screen { .a { color: red; } }"));
        assert!(!needs_vite_scoping(":where(.a) { content: '{'; }"));
    }
}
