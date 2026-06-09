//! Style block processing and scoped CSS.

use vize_carton::{String, ToCompactString};

use crate::types::{CssModuleMapping, SfcError, SfcStyleBlock, StyleCompileOptions};

pub(crate) struct StyleCompileResult {
    pub(crate) code: String,
    pub(crate) css_module: Option<CssModuleMapping>,
}

/// Compile a style block
pub fn compile_style(
    style: &SfcStyleBlock,
    options: &StyleCompileOptions,
) -> Result<String, SfcError> {
    compile_style_with_modules(style, options).map(|result| result.code)
}

pub(crate) fn compile_style_with_modules(
    style: &SfcStyleBlock,
    options: &StyleCompileOptions,
) -> Result<StyleCompileResult, SfcError> {
    if let Some(module_name) = style.module.as_ref() {
        let css_options = crate::css::CssCompileOptions {
            scope_id: Some(options.id.clone()),
            scoped: style.scoped || options.scoped,
            source_map: options.source_map,
            filename: Some(css_module_filename(style, options)),
            css_modules: true,
            ..Default::default()
        };
        let result = crate::css::compile_css(&style.content, &css_options);
        let mut output = result.code;
        if options.trim {
            output = output.trim().to_compact_string();
        }

        if let Some(error) = result.errors.into_iter().next() {
            return Err(SfcError {
                message: error,
                code: Some("CSS_MODULE_COMPILE_ERROR".into()),
                loc: Some(style.loc.clone()),
            });
        }

        let exports = result
            .exports
            .unwrap_or_default()
            .into_iter()
            .map(|(original, export)| (original, export.name))
            .collect();

        return Ok(StyleCompileResult {
            code: output,
            css_module: Some(CssModuleMapping {
                name: module_name.as_ref().into(),
                exports,
            }),
        });
    }

    let (mut output, _) = crate::css::transform_css_v_bind(&style.content, Some(&options.id));

    // Apply scoped transformation if needed
    if style.scoped || options.scoped {
        output = apply_scoped_css(&output, &options.id);
    }

    // Trim if requested
    if options.trim {
        output = output.trim().to_compact_string();
    }

    Ok(StyleCompileResult {
        code: output,
        css_module: None,
    })
}

fn css_module_filename(style: &SfcStyleBlock<'_>, options: &StyleCompileOptions) -> String {
    if let Some(src) = style.src.as_deref() {
        return src.into();
    }

    let lang = style.lang.as_deref().unwrap_or("css");
    let mut filename = String::with_capacity(options.id.len() + lang.len() + 24);
    use std::fmt::Write as _;
    filename.push_str(&options.id);
    let _ = write!(&mut filename, "-{}", style.loc.start);
    filename.push('.');
    filename.push_str(lang);
    filename
}

/// Apply scoped CSS transformation
pub fn apply_scoped_css(css: &str, scope_id: &str) -> String {
    let mut attr_selector = String::with_capacity(scope_id.len() + 2);
    attr_selector.push('[');
    attr_selector.push_str(scope_id);
    attr_selector.push(']');
    let mut output = String::with_capacity(css.len() * 2);
    let mut chars = css.chars().peekable();
    let mut in_selector = true;
    let mut in_string = false;
    let mut string_char = '"';
    let mut in_comment = false;
    let mut in_at_rule = false; // Track if we're in an at-rule header
    let mut brace_depth: u32 = 0;
    let mut at_rule_depth: u32 = 0; // Track nested at-rule depth
    let mut last_selector_end = 0;
    let mut current = String::default();
    let mut pending_keyframes = false;
    let mut keyframes_brace_depth: Option<u32> = None;
    let mut saved_at_rule_depth: Option<u32> = None;

    while let Some(c) = chars.next() {
        current.push(c);

        if in_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
                in_comment = false;
            }
            continue;
        }

        if in_string {
            if c == string_char && !current.ends_with("\\\"") && !current.ends_with("\\'") {
                in_string = false;
            }
            if !in_selector && !in_at_rule {
                output.push(c);
            }
            continue;
        }

        match c {
            '"' | '\'' => {
                in_string = true;
                string_char = c;
                if !in_selector && !in_at_rule {
                    output.push(c);
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
                in_comment = true;
            }
            '{' => {
                brace_depth += 1;
                if in_at_rule {
                    // End of at-rule header (e.g., @media (...) {)
                    let at_rule_part = &current[last_selector_end..current.len() - 1];
                    output.push_str(at_rule_part.trim());
                    output.push('{');
                    in_at_rule = false;
                    if pending_keyframes {
                        saved_at_rule_depth = Some(at_rule_depth);
                        keyframes_brace_depth = Some(brace_depth);
                        pending_keyframes = false;
                    }
                    at_rule_depth = brace_depth;
                    in_selector = true;
                    last_selector_end = current.len();
                } else if keyframes_brace_depth.is_some_and(|d| brace_depth > d) {
                    // Inside @keyframes: stops (from/to/0%/100%) are not selectors
                    let kf_part = &current[last_selector_end..current.len() - 1];
                    output.push_str(kf_part.trim());
                    output.push('{');
                    in_selector = false;
                    last_selector_end = current.len();
                } else if in_selector && brace_depth == 1 {
                    // End of selector at root level, apply scope
                    let selector_part = &current[last_selector_end..current.len() - 1];
                    output.push_str(&scope_selector_with_leading_comments(
                        selector_part,
                        &attr_selector,
                    ));
                    output.push('{');
                    in_selector = false;
                    last_selector_end = current.len();
                } else if in_selector && at_rule_depth > 0 && brace_depth > at_rule_depth {
                    // End of selector inside at-rule (e.g., inside @media), apply scope
                    let selector_part = &current[last_selector_end..current.len() - 1];
                    output.push_str(&scope_selector_with_leading_comments(
                        selector_part,
                        &attr_selector,
                    ));
                    output.push('{');
                    in_selector = false;
                    last_selector_end = current.len();
                } else {
                    output.push(c);
                }
            }
            '}' => {
                brace_depth = brace_depth.saturating_sub(1);
                output.push(c);
                // Check @keyframes block end — restore parent at_rule_depth
                if keyframes_brace_depth.is_some_and(|d| brace_depth < d) {
                    keyframes_brace_depth = None;
                    if let Some(saved) = saved_at_rule_depth.take() {
                        at_rule_depth = saved;
                    }
                }
                if brace_depth == 0 {
                    in_selector = true;
                    at_rule_depth = 0;
                    last_selector_end = current.len();
                } else if at_rule_depth > 0 && brace_depth >= at_rule_depth {
                    // Inside at-rule, back to selector mode for next rule
                    in_selector = true;
                    last_selector_end = current.len();
                }
            }
            '@' if in_selector => {
                // Start of at-rule (e.g., @media, @keyframes, @supports)
                in_at_rule = true;
                in_selector = false;
                // Look ahead to detect @keyframes (including vendor prefixes)
                let css_remaining = &css[current.len()..];
                pending_keyframes = css_remaining.starts_with("keyframes")
                    || css_remaining.starts_with("-webkit-keyframes")
                    || css_remaining.starts_with("-moz-keyframes")
                    || css_remaining.starts_with("-o-keyframes");
            }
            ';' if in_at_rule => {
                // Statement at-rule (e.g., @import, @charset, @namespace)
                // Flush the entire at-rule including the semicolon
                let stmt = &current[last_selector_end..];
                output.push_str(stmt.trim());
                output.push('\n');
                in_at_rule = false;
                in_selector = true;
                pending_keyframes = false;
                last_selector_end = current.len();
            }
            _ if in_selector || in_at_rule => {
                // Still building selector or at-rule header
            }
            _ => {
                output.push(c);
            }
        }
    }

    // Handle any remaining content
    if !current[last_selector_end..].is_empty() && in_selector {
        output.push_str(&current[last_selector_end..]);
    }

    output
}

/// Add scope to selector text while preserving leading CSS comments verbatim.
fn scope_selector_with_leading_comments(selector: &str, attr_selector: &str) -> String {
    let Some(prefix_end) = leading_css_comment_trivia_end(selector) else {
        return scope_selector(&normalize_deep_selectors(selector.trim()), attr_selector);
    };

    let mut output = String::with_capacity(selector.len() + attr_selector.len());
    output.push_str(&selector[..prefix_end]);

    let selector_body = selector[prefix_end..].trim();
    if !selector_body.is_empty() {
        output.push_str(&scope_selector(
            &normalize_deep_selectors(selector_body),
            attr_selector,
        ));
    }

    output
}

/// Normalize the pre-CSS-Scoped-syntax deep combinators (`>>>`, `/deep/`,
/// `::v-deep`, `::v-slotted`, `::v-global`) to the modern `:deep(...)` /
/// `:slotted(...)` / `:global(...)` function form so the downstream
/// scope-attr inserter handles them uniformly with Vue. (#971)
fn normalize_deep_selectors(selector: &str) -> String {
    if !looks_like_deep_legacy(selector) {
        return selector.to_compact_string();
    }

    const MARKERS: &[(&str, &str)] = &[
        ("::v-deep", ":deep"),
        ("::v-slotted", ":slotted"),
        ("::v-global", ":global"),
        (">>>", ":deep"),
        ("/deep/", ":deep"),
    ];

    // Find the earliest marker.
    let mut best: Option<(usize, &str, &str)> = None;
    for (needle, modern) in MARKERS {
        if let Some(pos) = selector.find(needle)
            && best.is_none_or(|(p, _, _)| pos < p)
        {
            best = Some((pos, *needle, *modern));
        }
    }
    let Some((pos, needle, modern)) = best else {
        return selector.to_compact_string();
    };

    let before = selector[..pos].trim_end();
    let after = selector[pos + needle.len()..].trim_start();

    // Function form (e.g. `::v-deep(.x)`): consume the parenthesised
    // argument and emit `modern(inner)<rest>`. Combinator form
    // (e.g. `.foo >>> .bar`): wrap the remainder of the selector in
    // `modern(...)`.
    let mut out = String::with_capacity(selector.len() + 8);
    out.push_str(before);
    if !before.is_empty() {
        out.push(' ');
    }
    if let Some(rest) = after.strip_prefix('(')
        && let Some(end) = rest.find(')')
    {
        let inner = rest[..end].trim();
        let trailing = &rest[end + 1..];
        out.push_str(modern);
        out.push('(');
        out.push_str(inner);
        out.push(')');
        out.push_str(trailing);
    } else {
        out.push_str(modern);
        out.push('(');
        out.push_str(after);
        out.push(')');
    }
    out
}

fn looks_like_deep_legacy(s: &str) -> bool {
    s.contains(">>>")
        || s.contains("/deep/")
        || s.contains("::v-deep")
        || s.contains("::v-slotted")
        || s.contains("::v-global")
}

fn leading_css_comment_trivia_end(value: &str) -> Option<usize> {
    let mut cursor = 0usize;
    let mut found_comment = false;

    loop {
        let ws_end = value[cursor..]
            .char_indices()
            .find(|(_, char)| !char.is_whitespace())
            .map_or(value.len(), |(index, _)| cursor + index);
        cursor = ws_end;

        if !value[cursor..].starts_with("/*") {
            return found_comment.then_some(cursor);
        }

        found_comment = true;
        let Some(end) = value[cursor + 2..].find("*/") else {
            return Some(value.len());
        };
        cursor += 2 + end + 2;
    }
}

/// Add scope attribute to a selector
fn scope_selector(selector: &str, attr_selector: &str) -> String {
    // Normalize legacy deep combinators (`>>>`, `/deep/`, `::v-deep`, etc.)
    // to their modern `:deep(...)` / `:slotted(...)` / `:global(...)` form
    // before splitting on `,`. (#971)
    let normalized = normalize_deep_selectors(selector);
    // Split on top-level commas only — a comma inside `:not(a, b)` or
    // `:is(a, b)` is part of one selector, not a list separator.
    split_top_level_commas(&normalized)
        .into_iter()
        .map(|s| scope_single_selector(s.trim(), attr_selector))
        .collect::<Vec<_>>()
        .join(", ")
        .into()
}

fn split_top_level_commas(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut depth: i32 = 0;
    let mut last = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            b',' if depth == 0 => {
                out.push(&s[last..i]);
                last = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(&s[last..]);
    out
}

/// Add scope attribute to a single selector
fn scope_single_selector(selector: &str, attr_selector: &str) -> String {
    if selector.is_empty() {
        return selector.to_compact_string();
    }

    // Handle :deep(), :slotted(), :global()
    if selector.contains(":deep(") {
        return transform_deep(selector, attr_selector);
    }

    if selector.contains(":slotted(") {
        return transform_slotted(selector, attr_selector);
    }

    if selector.contains(":global(") {
        return transform_global(selector);
    }

    // Split into compound selectors at top-level whitespace only — whitespace
    // inside `:is(...)`, `:not(...)`, etc. is part of the same compound and
    // must stay verbatim. (#971)
    let parts: Vec<&str> = split_top_level_whitespace(selector);
    if parts.is_empty() {
        return selector.to_compact_string();
    }

    // Add scope to the last part
    let mut result = String::default();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }

        if i == parts.len() - 1 {
            // Last part - add scope
            result.push_str(&add_scope_to_element(part, attr_selector));
        } else {
            result.push_str(part);
        }
    }

    result
}

fn split_top_level_whitespace(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut depth: i32 = 0;
    let mut start: Option<usize> = None;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'(' | b'[' => {
                if start.is_none() {
                    start = Some(i);
                }
                depth += 1;
            }
            b')' | b']' => {
                depth -= 1;
            }
            b' ' | b'\t' | b'\n' | b'\r' if depth == 0 => {
                if let Some(s_pos) = start.take() {
                    out.push(&s[s_pos..i]);
                }
            }
            _ => {
                if start.is_none() {
                    start = Some(i);
                }
            }
        }
        i += 1;
    }
    if let Some(s_pos) = start {
        out.push(&s[s_pos..]);
    }
    out
}

/// Add scope attribute to an element selector
fn add_scope_to_element(selector: &str, attr_selector: &str) -> String {
    // Find the FIRST top-level pseudo-element or pseudo-class so the scope
    // attribute lands on the compound selector, not inside a functional
    // pseudo-class argument (e.g. `.x:not(:checked)` → `.x[attr]:not(:checked)`,
    // not `.x:not(:[attr]checked)`). Skip colons inside parentheses. (#971)
    if let Some(pseudo_pos) = find_top_level_pseudo(selector) {
        let before = &selector[..pseudo_pos];
        let after = &selector[pseudo_pos..];
        // Avoid splitting at a pseudo that is part of an escape sequence
        // (`\:`), which is rare but valid in CSS.
        if !before.ends_with('\\') {
            let mut result =
                String::with_capacity(before.len() + attr_selector.len() + after.len());
            result.push_str(before);
            result.push_str(attr_selector);
            result.push_str(after);
            return result;
        }
    }

    let mut result = String::with_capacity(selector.len() + attr_selector.len());
    result.push_str(selector);
    result.push_str(attr_selector);
    result
}

/// Find the first top-level `:` introducing a pseudo-class or `::` introducing
/// a pseudo-element, skipping any colon that lives inside parentheses (i.e.
/// inside `:not(...)`, `:is(...)`, `:where(...)`, `:has(...)` arguments).
fn find_top_level_pseudo(selector: &str) -> Option<usize> {
    let bytes = selector.as_bytes();
    let mut depth: i32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b':' if depth == 0 => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

/// Transform :deep() to descendant selector
fn transform_deep(selector: &str, attr_selector: &str) -> String {
    // :deep(.child) -> [data-v-xxx] .child
    if let Some(start) = selector.find(":deep(") {
        let before = &selector[..start];
        let after = &selector[start + 6..];

        if let Some(end) = after.find(')') {
            let inner = &after[..end];
            let rest = &after[end + 1..];

            let scoped_before = scope_deep_prefix(before, attr_selector);

            let mut result =
                String::with_capacity(scoped_before.len() + inner.len() + rest.len() + 1);
            result.push_str(&scoped_before);
            result.push(' ');
            result.push_str(inner);
            result.push_str(rest);
            return result;
        }
    }

    selector.to_compact_string()
}

fn scope_deep_prefix(before: &str, attr_selector: &str) -> String {
    let before = before.trim_end();
    if before.is_empty() {
        return attr_selector.to_compact_string();
    }

    let Some(combinator_start) = trailing_combinator_start(before) else {
        return scope_single_selector(before.trim(), attr_selector);
    };

    let target_end = before[..combinator_start].trim_end().len();
    if target_end == 0 {
        let mut result = String::with_capacity(attr_selector.len() + before.len());
        result.push_str(attr_selector);
        result.push_str(&before[combinator_start..]);
        return result;
    }

    let scoped_target = scope_single_selector(&before[..target_end], attr_selector);
    let mut result = String::with_capacity(scoped_target.len() + before.len() - target_end);
    result.push_str(&scoped_target);
    result.push_str(&before[target_end..]);
    result
}

fn trailing_combinator_start(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    match bytes.last().copied()? {
        b'>' | b'+' | b'~' => Some(bytes.len() - 1),
        b'|' if bytes.len() >= 2 && bytes[bytes.len() - 2] == b'|' => Some(bytes.len() - 2),
        _ => None,
    }
}

/// Transform :slotted() for slot content
fn transform_slotted(selector: &str, attr_selector: &str) -> String {
    // :slotted(.child) -> .child[data-v-xxx-s]
    if let Some(start) = selector.find(":slotted(") {
        let after = &selector[start + 9..];

        if let Some(end) = after.find(')') {
            let inner = &after[..end];
            let rest = &after[end + 1..];

            let mut result =
                String::with_capacity(inner.len() + attr_selector.len() + rest.len() + 2);
            result.push_str(inner);
            result.push_str(attr_selector);
            result.push_str("-s");
            result.push_str(rest);
            return result;
        }
    }

    selector.to_compact_string()
}

/// Transform :global() to unscoped
fn transform_global(selector: &str) -> String {
    // :global(.class) -> .class
    if let Some(start) = selector.find(":global(") {
        let before = &selector[..start];
        let after = &selector[start + 8..];

        if let Some(end) = after.find(')') {
            let inner = &after[..end];
            let rest = &after[end + 1..];

            let mut result = String::with_capacity(before.len() + inner.len() + rest.len());
            result.push_str(before);
            result.push_str(inner);
            result.push_str(rest);
            return result;
        }
    }

    selector.to_compact_string()
}

/// Extract CSS v-bind() expressions
pub fn extract_css_vars(css: &str) -> Vec<String> {
    crate::css::transform_css_v_bind(css, None).1
}

#[cfg(test)]
mod tests {
    use super::{
        apply_scoped_css, extract_css_vars, scope_selector, transform_deep, transform_global,
    };

    #[test]
    fn test_scope_simple_selector() {
        let result = scope_selector(".foo", "[data-v-123]");
        assert_eq!(result, ".foo[data-v-123]");
    }

    #[test]
    fn test_scope_functional_pseudo_class_keeps_inner_intact() {
        // #971: a colon inside `:not(...)` / `:is(...)` / `:where(...)` /
        // `:has(...)` must not be where the scope attribute lands. The
        // scope attaches to the compound selector instead.
        let result = scope_selector(".x:not(:checked)", "[data-v-123]");
        assert_eq!(result, ".x[data-v-123]:not(:checked)");

        let result = scope_selector(".btn:is(:hover, :focus)", "[data-v-123]");
        assert_eq!(result, ".btn[data-v-123]:is(:hover, :focus)");
    }

    #[test]
    fn test_scope_v_deep_combinator_form() {
        // Legacy `::v-deep` combinator form normalizes to `:deep(...)`.
        let result = scope_selector(".foo ::v-deep .bar", "[data-v-123]");
        assert_eq!(result, ".foo[data-v-123] .bar");
    }

    #[test]
    fn test_scope_v_deep_function_form() {
        // Legacy `::v-deep(.x)` function form normalizes to `:deep(.x)`.
        let result = scope_selector(".foo ::v-deep(.bar)", "[data-v-123]");
        assert_eq!(result, ".foo[data-v-123] .bar");
    }

    #[test]
    fn test_scope_legacy_deep_combinators() {
        // Both `>>>` and `/deep/` normalize to `:deep(...)` wrapping the
        // remainder.
        let result = scope_selector(".foo >>> .bar", "[data-v-123]");
        assert_eq!(result, ".foo[data-v-123] .bar");

        let result = scope_selector(".foo /deep/ .bar", "[data-v-123]");
        assert_eq!(result, ".foo[data-v-123] .bar");
    }

    #[test]
    fn test_scope_v_slotted_function_form() {
        let result = scope_selector("::v-slotted(.bar)", "[data-v-123]");
        assert_eq!(result, ".bar[data-v-123]-s");
    }

    #[test]
    fn test_scope_descendant_selector() {
        let result = scope_selector(".foo .bar", "[data-v-123]");
        assert_eq!(result, ".foo .bar[data-v-123]");
    }

    #[test]
    fn test_scope_multiple_selectors() {
        let result = scope_selector(".foo, .bar", "[data-v-123]");
        assert_eq!(result, ".foo[data-v-123], .bar[data-v-123]");
    }

    #[test]
    fn test_transform_deep() {
        let result = transform_deep(":deep(.child)", "[data-v-123]");
        assert_eq!(result, "[data-v-123] .child");
    }

    #[test]
    fn test_transform_deep_after_child_combinator() {
        let result = transform_deep(".sponsors__item > :deep(.sponsor)", "[data-v-123]");
        assert_eq!(result, ".sponsors__item[data-v-123] > .sponsor");
    }

    #[test]
    fn test_apply_scoped_css_deep_after_child_combinator() {
        let css = ".sponsors__item > :deep(.sponsor) { width: 100%; }";
        let result = apply_scoped_css(css, "data-v-123");
        assert_eq!(
            result,
            ".sponsors__item[data-v-123] > .sponsor{ width: 100%; }"
        );
    }

    #[test]
    fn test_apply_scoped_css_preserves_deep_comment_before_selector() {
        let css = "/* override :deep(p) from the parent */\n.foo { color: red; }";
        let result = apply_scoped_css(css, "data-v-123");

        assert_eq!(
            result,
            "/* override :deep(p) from the parent */\n.foo[data-v-123]{ color: red; }"
        );
    }

    #[test]
    fn test_apply_scoped_css_preserves_deep_comment_inside_at_rule() {
        let css = "@media (min-width: 1px) { /* A <span>, not a <p>; ignore :deep(p). */\n.conferences__venue { display: block; } }";
        let result = apply_scoped_css(css, "data-v-abc");

        assert_eq!(
            result,
            "@media (min-width: 1px){ /* A <span>, not a <p>; ignore :deep(p). */\n.conferences__venue[data-v-abc]{ display: block; }}"
        );
    }

    #[test]
    fn test_apply_scoped_css_stray_closing_brace_keeps_scoping() {
        let css = "}.foo { color: red; }.bar { color: blue; }";
        let result = apply_scoped_css(css, "data-v-123");

        assert_eq!(
            result,
            "}.foo[data-v-123]{ color: red; }.bar[data-v-123]{ color: blue; }"
        );
    }

    #[test]
    fn test_transform_global() {
        let result = transform_global(":global(.foo)");
        assert_eq!(result, ".foo");
    }

    #[test]
    fn test_extract_css_vars() {
        let css = ".foo { color: v-bind(color); background: v-bind('bgColor'); }";
        let vars = extract_css_vars(css);
        assert_eq!(vars, vec!["color", "bgColor"]);
    }

    #[test]
    fn test_extract_css_vars_with_quoted_parentheses() {
        let css = r#"
.header {
  background-color: color(from v-bind("parentBg ?? 'var(--bg)'") srgb r g b / 0.85);
}
.textCountGraph {
  background-image: conic-gradient(
    var(--countColor) 0% v-bind("Math.min(100, textCountPercentage) + '%'"),
    rgba(0, 0, 0, .2) v-bind("Math.min(100, textCountPercentage) + '%'") 100%
  );
}
"#;
        let vars = extract_css_vars(css);
        assert_eq!(
            vars,
            vec![
                "parentBg ?? 'var(--bg)'",
                "Math.min(100, textCountPercentage) + '%'",
                "Math.min(100, textCountPercentage) + '%'",
            ]
        );
    }

    #[test]
    fn test_scope_media_query() {
        let css = "@media (max-width: 768px) { .foo { color: red; } }";
        let result = apply_scoped_css(css, "data-v-123");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_scope_media_query_with_comment() {
        let css = "/* Mobile responsive */\n@media (max-width: 768px) {\n  .glyph-playground {\n    grid-template-columns: 1fr;\n  }\n}";
        let result = apply_scoped_css(css, "data-v-123");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_scope_keyframes() {
        let css = "@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }";
        let result = apply_scoped_css(css, "data-v-123");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_scope_webkit_keyframes() {
        let css = "@-webkit-keyframes fade { 0% { opacity: 0; } 100% { opacity: 1; } }";
        let result = apply_scoped_css(css, "data-v-123");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_nested_css_media_passthrough() {
        // CSS nesting: @media (--mobile) inside a selector should pass through
        let css = "#pages-store {\n  display: grid;\n  row-gap: 1.5rem;\n  @media (--mobile) {\n    row-gap: 1rem;\n  }\n  h1 {\n    padding: 7.5rem 0;\n    @media (--mobile) {\n      padding: 2.5rem 0.75rem;\n    }\n  }\n}";
        let result = apply_scoped_css(css, "data-v-123");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_root_level_media_with_custom_query() {
        // Root-level @media with custom media query
        let css = ".foo { color: red; }\n@media (--mobile) { .foo { font-size: 12px; } }";
        let result = apply_scoped_css(css, "data-v-abc");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_apply_scoped_css_at_import() {
        let css = "@import \"~/assets/styles/custom-media-query.css\";\n\nfooter { width: 100%; }";
        let result = apply_scoped_css(css, "data-v-123");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_apply_scoped_css_at_import_with_nested_css() {
        let css = "@import \"custom.css\";\n\nfooter {\n  width: 100%;\n  @media (--mobile) {\n    padding: 1rem;\n  }\n}";
        let result = apply_scoped_css(css, "data-v-abc");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_scope_keyframes_inside_media() {
        let css = "@media (prefers-reduced-motion: no-preference) { @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } } .foo { color: red; } }";
        let result = apply_scoped_css(css, "data-v-123");
        insta::assert_snapshot!(result.as_str());
    }
}
