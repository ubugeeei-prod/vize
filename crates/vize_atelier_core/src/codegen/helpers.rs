//! Utility functions for code generation.

mod constant_expression;

pub use self::constant_expression::is_constant_simple_expression;
use crate::RuntimeHelper;
use vize_carton::ToCompactString;

/// Decode HTML numeric character references in hex or decimal form.
pub fn decode_html_entities(s: &str) -> String {
    // Numeric decoding only triggers on `&`; otherwise skip the state machine
    // and copy the input in a single pass.
    if !s.contains('&') {
        return String::from(s);
    }

    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '&' && chars.peek() == Some(&'#') {
            chars.next(); // consume '#'
            let is_hex = chars.peek() == Some(&'x') || chars.peek() == Some(&'X');
            if is_hex {
                chars.next(); // consume 'x' or 'X'
            }

            let mut num_str = String::default();
            while let Some(&ch) = chars.peek() {
                if ch == ';' {
                    chars.next(); // consume ';'
                    break;
                }
                let is_valid_char =
                    (is_hex && ch.is_ascii_hexdigit()) || (!is_hex && ch.is_ascii_digit());
                if is_valid_char {
                    num_str.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }

            if !num_str.is_empty() {
                let codepoint = if is_hex {
                    u32::from_str_radix(&num_str, 16).ok()
                } else {
                    num_str.parse::<u32>().ok()
                };

                if let Some(cp) = codepoint
                    && let Some(decoded_char) = char::from_u32(cp)
                {
                    result.push(decoded_char);
                    continue;
                }
            }

            // If decoding failed, output the original sequence
            result.push('&');
            result.push('#');
            if is_hex {
                result.push('x');
            }
            result.push_str(&num_str);
        } else {
            result.push(c);
        }
    }

    result
}

/// Returns true if a byte may belong to a character that `escape_js_string`
/// would rewrite (an escape-requiring char) or that `decode_html_entities`
/// might act on (`&`). Used as a cheap fast-path gate: when no byte of the
/// input matches, the string passes through both stages unchanged.
///
/// The control characters the slow path escapes are exactly the Unicode `Cc`
/// category — C0 (U+0000..=U+001F), DEL (U+007F), and C1 (U+0080..=U+009F) —
/// because the catch-all arm uses `char::is_control()`. In UTF-8 those are the
/// bytes `0x00..=0x1F`, `0x7F`, and the two-byte sequences `0xC2 0x80..=0xC2
/// 0x9F`. We flag every `0xC2` lead byte (a conservative superset of the C1
/// range) so a C1 control can never slip through the fast path; this is the
/// subtle case a naive `b < 0x20` check would miss. All other non-ASCII bytes
/// (lead `>= 0xC3` and continuation bytes) encode characters `>= U+00C0`, none
/// of which are control characters, so they are safe to pass through.
#[inline]
fn byte_may_need_js_escaping(b: u8) -> bool {
    b < 0x20 || b == b'"' || b == b'&' || b == b'\\' || b == 0x7F || b == 0xC2
}

/// Escape a string for use in JavaScript string literals
pub fn escape_js_string(s: &str) -> String {
    // Fast path: when no byte can require HTML-entity decoding or JS escaping,
    // the two-pass decode+escape would reproduce the input verbatim. Skip both
    // passes (and their allocations) and copy once. This is the overwhelmingly
    // common case — plain text, attribute values, and identifiers — on a path
    // hit for every text node, prop, comment, and slot name during codegen.
    if !s.bytes().any(byte_may_need_js_escaping) {
        return String::from(s);
    }

    // First decode HTML entities, then escape for JS
    let decoded = decode_html_entities(s);
    let mut result = String::with_capacity(decoded.len());
    fn push_hex4(out: &mut String, value: u32) {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        out.push_str("\\u");
        out.push(HEX[((value >> 12) & 0xF) as usize] as char);
        out.push(HEX[((value >> 8) & 0xF) as usize] as char);
        out.push(HEX[((value >> 4) & 0xF) as usize] as char);
        out.push(HEX[(value & 0xF) as usize] as char);
    }
    for c in decoded.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\x08' => result.push_str("\\b"), // backspace
            '\x0C' => result.push_str("\\f"), // form feed
            c if c.is_control() => {
                // Other control characters as unicode escape
                push_hex4(&mut result, c as u32);
            }
            c => result.push(c),
        }
    }
    result
}

/// Check if a string is a valid JavaScript identifier (doesn't need quoting)
pub fn is_valid_js_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    // First character must be a letter, underscore, or dollar sign
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    // Remaining characters can also include digits
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Convert a component/directive asset name into a valid JavaScript identifier.
pub fn to_valid_asset_identifier(kind: &str, name: &str) -> String {
    let mut ident = String::with_capacity(kind.len() + name.len() + 2);
    ident.push('_');
    ident.push_str(kind);
    ident.push('_');

    // Mirror Vue's `toValidAssetId` (compiler-core utils, issue #4422): word
    // characters pass through, `-` becomes `_`, and every other character is
    // replaced by its char code rendered as a decimal string.
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            ident.push(c);
        } else if c == '-' {
            ident.push('_');
        } else {
            ident.push_str(&(c as u32).to_compact_string());
        }
    }

    ident
}

/// Default helper alias function
pub fn default_helper_alias(helper: RuntimeHelper) -> &'static str {
    match helper {
        // Core helpers
        RuntimeHelper::Fragment => "_Fragment",
        RuntimeHelper::Teleport => "_Teleport",
        RuntimeHelper::Suspense => "_Suspense",
        RuntimeHelper::KeepAlive => "_KeepAlive",
        RuntimeHelper::BaseTransition => "_BaseTransition",
        RuntimeHelper::Transition => "_Transition",
        RuntimeHelper::TransitionGroup => "_TransitionGroup",
        RuntimeHelper::OpenBlock => "_openBlock",
        RuntimeHelper::CreateBlock => "_createBlock",
        RuntimeHelper::CreateElementBlock => "_createElementBlock",
        RuntimeHelper::CreateVNode => "_createVNode",
        RuntimeHelper::CreateElementVNode => "_createElementVNode",
        RuntimeHelper::CreateComment => "_createCommentVNode",
        RuntimeHelper::CreateText => "_createTextVNode",
        RuntimeHelper::CreateStatic => "_createStaticVNode",
        RuntimeHelper::ResolveComponent => "_resolveComponent",
        RuntimeHelper::ResolveDynamicComponent => "_resolveDynamicComponent",
        RuntimeHelper::ResolveDirective => "_resolveDirective",
        RuntimeHelper::ResolveFilter => "_resolveFilter",
        RuntimeHelper::WithDirectives => "_withDirectives",
        RuntimeHelper::VShow => "_vShow",
        RuntimeHelper::VModelText => "_vModelText",
        RuntimeHelper::VModelCheckbox => "_vModelCheckbox",
        RuntimeHelper::VModelRadio => "_vModelRadio",
        RuntimeHelper::VModelSelect => "_vModelSelect",
        RuntimeHelper::VModelDynamic => "_vModelDynamic",
        RuntimeHelper::RenderList => "_renderList",
        RuntimeHelper::RenderSlot => "_renderSlot",
        RuntimeHelper::CreateSlots => "_createSlots",
        RuntimeHelper::ToDisplayString => "_toDisplayString",
        RuntimeHelper::MergeProps => "_mergeProps",
        RuntimeHelper::NormalizeClass => "_normalizeClass",
        RuntimeHelper::NormalizeStyle => "_normalizeStyle",
        RuntimeHelper::NormalizeProps => "_normalizeProps",
        RuntimeHelper::GuardReactiveProps => "_guardReactiveProps",
        RuntimeHelper::ToHandlers => "_toHandlers",
        RuntimeHelper::Camelize => "_camelize",
        RuntimeHelper::Capitalize => "_capitalize",
        RuntimeHelper::ToHandlerKey => "_toHandlerKey",
        RuntimeHelper::SetBlockTracking => "_setBlockTracking",
        RuntimeHelper::PushScopeId => "_pushScopeId",
        RuntimeHelper::PopScopeId => "_popScopeId",
        RuntimeHelper::WithCtx => "_withCtx",
        RuntimeHelper::Unref => "_unref",
        RuntimeHelper::IsRef => "_isRef",
        RuntimeHelper::WithMemo => "_withMemo",
        RuntimeHelper::IsMemoSame => "_isMemoSame",
        RuntimeHelper::WithModifiers => "_withModifiers",
        RuntimeHelper::WithKeys => "_withKeys",

        // SSR helpers
        RuntimeHelper::SsrInterpolate => "_ssrInterpolate",
        RuntimeHelper::SsrRenderVNode => "_ssrRenderVNode",
        RuntimeHelper::SsrRenderComponent => "_ssrRenderComponent",
        RuntimeHelper::SsrRenderSlot => "_ssrRenderSlot",
        RuntimeHelper::SsrRenderSlotInner => "_ssrRenderSlotInner",
        RuntimeHelper::SsrRenderAttrs => "_ssrRenderAttrs",
        RuntimeHelper::SsrRenderAttr => "_ssrRenderAttr",
        RuntimeHelper::SsrRenderDynamicAttr => "_ssrRenderDynamicAttr",
        RuntimeHelper::SsrIncludeBooleanAttr => "_ssrIncludeBooleanAttr",
        RuntimeHelper::SsrRenderClass => "_ssrRenderClass",
        RuntimeHelper::SsrRenderStyle => "_ssrRenderStyle",
        RuntimeHelper::SsrRenderDynamicModel => "_ssrRenderDynamicModel",
        RuntimeHelper::SsrGetDynamicModelProps => "_ssrGetDynamicModelProps",
        RuntimeHelper::SsrRenderList => "_ssrRenderList",
        RuntimeHelper::SsrLooseEqual => "_ssrLooseEqual",
        RuntimeHelper::SsrLooseContain => "_ssrLooseContain",
        RuntimeHelper::SsrGetDirectiveProps => "_ssrGetDirectiveProps",
        RuntimeHelper::SsrRenderTeleport => "_ssrRenderTeleport",
        RuntimeHelper::SsrRenderSuspense => "_ssrRenderSuspense",
    }
}

// Re-export from vize_carton for convenience
pub use vize_carton::{String, camelize, capitalize};

/// Capitalize first letter of a string (alias for capitalize)
#[inline]
pub fn capitalize_first(s: &str) -> String {
    capitalize(s)
}

/// Check if a component is a Vue built-in that should be imported directly.
/// Handles both PascalCase and kebab-case tag names.
pub fn is_builtin_component(name: &str) -> Option<RuntimeHelper> {
    match name {
        "Teleport" | "teleport" => Some(RuntimeHelper::Teleport),
        "Suspense" | "suspense" => Some(RuntimeHelper::Suspense),
        "KeepAlive" | "keep-alive" => Some(RuntimeHelper::KeepAlive),
        "BaseTransition" | "base-transition" => Some(RuntimeHelper::BaseTransition),
        "Transition" | "transition" => Some(RuntimeHelper::Transition),
        "TransitionGroup" | "transition-group" => Some(RuntimeHelper::TransitionGroup),
        _ => None,
    }
}

#[cfg(test)]
mod escape_tests {
    use super::{decode_html_entities, escape_js_string};

    /// Reference implementation of the JS-string escape that always runs the
    /// full char-by-char pass (no fast path). `escape_js_string`'s fast path
    /// must reproduce this exactly for every input.
    fn reference_escape(s: &str) -> String {
        let decoded = decode_html_entities(s);
        let mut out = String::new();
        for c in decoded.chars() {
            match c {
                '\\' => out.push_str("\\\\"),
                '"' => out.push_str("\\\""),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                '\u{08}' => out.push_str("\\b"),
                '\u{0C}' => out.push_str("\\f"),
                c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out
    }

    #[test]
    fn fast_path_matches_reference_for_every_control_char() {
        // Every Cc codepoint (C0 0x00..=0x1F, DEL 0x7F, C1 0x80..=0x9F) plus
        // surrounding printable bytes — the fast path must agree with the slow
        // path on all of them. This is the case the original fast-path proposal
        // got wrong (DEL and C1 controls).
        for cp in 0u32..=0x9Fu32 {
            let ch = char::from_u32(cp).unwrap();
            for sample in [
                format!("{ch}"),
                format!("a{ch}b"),
                format!("{ch}{ch}"),
                format!("pre {ch} post"),
            ] {
                assert_eq!(
                    escape_js_string(&sample),
                    reference_escape(&sample),
                    "mismatch for codepoint U+{cp:04X}"
                );
            }
        }
    }

    #[test]
    fn fast_path_matches_reference_for_assorted_strings() {
        let cases = [
            "",
            "hello world",
            "plain ascii identifier_123",
            "a\"b",
            "a\\b",
            "tab\tnewline\nreturn\r",
            "back\u{08}space form\u{0C}feed",
            "del\u{7f}char",
            "c1\u{80}\u{9f}controls",
            "café résumé naïve", // accented (0xC3 lead, fast path)
            "こんにちは世界",    // CJK (0xE3 lead, fast path)
            "© 2024 ® ±",        // Latin-1 supplement (0xC2 lead, slow path)
            "amp & without hash",
            "named &amp; entity",
            "numeric &#x41;&#66; entities",
            "quote entity &#34; here",
            "mix \"q\" & \u{80} \t end",
            "emoji 😀 passthrough",
        ];
        for case in cases {
            assert_eq!(
                escape_js_string(case),
                reference_escape(case),
                "mismatch for {case:?}"
            );
        }
    }

    #[test]
    fn plain_strings_pass_through_unchanged() {
        for s in ["hello", "café", "こんにちは", "a_b$c123", ""] {
            assert_eq!(escape_js_string(s), s);
        }
    }

    #[test]
    fn escapes_specific_known_outputs() {
        assert_eq!(escape_js_string("a\"b"), "a\\\"b");
        assert_eq!(escape_js_string("a\\b"), "a\\\\b");
        assert_eq!(escape_js_string("\u{7f}"), "\\u007f"); // DEL
        assert_eq!(escape_js_string("\u{80}"), "\\u0080"); // C1 start
        assert_eq!(escape_js_string("\u{9f}"), "\\u009f"); // C1 end
        assert_eq!(escape_js_string("\u{01}"), "\\u0001"); // C0
        assert_eq!(escape_js_string("&#x22;"), "\\\""); // entity -> quote -> escaped
    }

    #[test]
    fn decode_fast_path_matches_full_decode() {
        // No '&' -> verbatim; entities still decode correctly.
        assert_eq!(decode_html_entities("plain text"), "plain text");
        assert_eq!(decode_html_entities("café"), "café");
        assert_eq!(decode_html_entities("&#x41;&#66;"), "AB");
        assert_eq!(decode_html_entities("&amp; stays"), "&amp; stays");
        assert_eq!(decode_html_entities("&#zz; invalid"), "&#zz; invalid");
    }
}
