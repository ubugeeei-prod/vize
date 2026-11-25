//! Span collection/merging, template-referenced-name discovery, and
//! `export default` rewriting used while assembling the virtual TypeScript.

#[cfg(test)]
mod hoisted_jsdoc_tests;
mod template_usage;

pub(super) use template_usage::template_usage;
use vize_carton::String;

pub(super) fn merge_overlapping_spans(mut spans: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    spans.retain(|(start, end)| start < end);
    spans.sort_by_key(|&(start, end)| (start, end));

    let mut merged: Vec<(u32, u32)> = Vec::with_capacity(spans.len());
    for (start, end) in spans {
        if let Some((_, previous_end)) = merged.last_mut()
            && start <= *previous_end
        {
            *previous_end = (*previous_end).max(end);
            continue;
        }
        merged.push((start, end));
    }
    merged
}

/// Widen a hoisted declaration's span over the JSDoc blocks that lead it.
///
/// TypeScript attaches a `/** … */` comment to the declaration that follows
/// it. A declaration moved to module scope without its JSDoc loses its
/// `@deprecated`, `@see` and documentation, and the comment left behind in
/// `__setup()` documents whichever statement comes next instead (#12985).
/// Only whole-line blocks separated from the span by whitespace move.
pub(super) fn with_leading_jsdoc(script: &str, (start, end): (u32, u32)) -> (u32, u32) {
    let mut cursor = start as usize;
    while let Some(open) = script.get(..cursor).and_then(leading_jsdoc_start) {
        cursor = open;
    }
    (u32::try_from(cursor).unwrap_or(start), end)
}

/// Where the JSDoc block that ends `before` (up to trailing whitespace) opens,
/// when that block opens its own line.
fn leading_jsdoc_start(before: &str) -> Option<usize> {
    let head = before.trim_end().strip_suffix("*/")?;
    let open = head.rfind("/**")?;
    let body = head.get(open + 3..)?;
    if body.contains("*/") {
        return None;
    }
    let line_start = head.get(..open)?.rfind('\n').map_or(0, |index| index + 1);
    head.get(line_start..open)?
        .trim()
        .is_empty()
        .then_some(open)
}

/// Byte length of the JSDoc blocks (and the whitespace after them) that lead
/// a hoisted declaration's text.
pub(super) fn leading_jsdoc_len(text: &str) -> usize {
    let mut offset = 0;
    loop {
        let Some(rest) = text.get(offset..) else {
            return offset;
        };
        let trimmed = rest.trim_start();
        let Some(close) = trimmed.strip_prefix("/**").and_then(|body| body.find("*/")) else {
            return offset;
        };
        // `/**`, the body, `*/`, then the whitespace before the declaration.
        let comment_end = rest.len() - trimmed.len() + 3 + close + 2;
        let after = rest.get(comment_end..).unwrap_or_default();
        offset += comment_end + (after.len() - after.trim_start().len());
    }
}

/// Generated identifier resolving Vue's `defineComponent` (see
/// `DEFINE_COMPONENT_HELPER`). Plain object-literal default exports are
/// wrapped in a call to it so TypeScript binds `this` inside
/// computed/methods via Vue's `ThisType` machinery instead of the bare
/// object literal (which produced TS2339 false positives).
pub(super) const DEFINE_COMPONENT_REF: &str = "__vizeDefineComponent";

/// Module-scope declaration for `DEFINE_COMPONENT_REF`. Uses the same
/// `import('vue')` reference form as the other generated vue helpers so it
/// never collides with user imports.
pub(super) const DEFINE_COMPONENT_HELPER: &str =
    "declare const __vizeDefineComponent: typeof import('vue').defineComponent;\n";

pub(super) fn rewrite_export_default_for_module_scope(
    text: &str,
    default_object: Option<(usize, usize, usize)>,
    default_expr: Option<(usize, usize, usize)>,
) -> String {
    // Plain `export default { ... }` (the Options API shape) is wrapped with
    // Vue's `defineComponent`; every other default-export shape is rewritten to
    // a bare `const __default__ = <expr>` using the AST-provided byte offsets.
    if let Some(output) = wrap_default_export_object(text, default_object) {
        return output;
    }
    if let Some(output) = rewrite_default_export_expression(text, default_expr) {
        return output;
    }

    // No rewriteable default export in this span (e.g. a re-export statement
    // such as `export { default } from './x'`): leave the text untouched.
    text.into()
}

/// The copied value's generated start follows fixed synthetic text and the
/// preserved keyword trivia. Searching for its text can match a comment.
pub(super) fn rewritten_default_value_start(
    text: &str,
    default_object: Option<(usize, usize, usize)>,
    default_expr: Option<(usize, usize, usize)>,
) -> Option<usize> {
    let (start, inner, _) = default_object.or(default_expr)?;
    let keyword_end = super::irregular_default::keyword_end(text, start)?;
    let between = if text.get(start..keyword_end)? == "export default" {
        0
    } else {
        text.get(start + 6..keyword_end - 7)?.len()
    };
    let gap = inner.checked_sub(keyword_end)?;
    let wrapper = if default_object.is_some() {
        DEFINE_COMPONENT_REF.len() + 1
    } else {
        0
    };
    Some(start + "const __default__ =".len() + between + gap + wrapper)
}

/// Rewrite an arbitrary `export default <expr>` to `const __default__ = <expr>`
/// using the `(export_start, expr_start, expr_end)` offsets (relative to
/// `text`) located by the AST. The `export default` keyword run
/// (`export_start..expr_start`) is dropped and the exported expression
/// (`expr_start..expr_end`) is copied verbatim, so source formatting —
/// `export default{` with no space, multi-line calls, decorated/anonymous
/// classes — is preserved exactly. Returns `None` when the offsets do not
/// describe a default export inside `text`.
fn rewrite_default_export_expression(
    text: &str,
    default_expr: Option<(usize, usize, usize)>,
) -> Option<String> {
    const EXPORT_DEFAULT: &str = "export default";
    const REPLACEMENT: &str = "const __default__ =";

    let (export_start, expr_start, expr_end) = default_expr?;
    if expr_start >= expr_end || export_start >= expr_start {
        return None;
    }
    let keyword_end = super::irregular_default::keyword_end(text, export_start)?;
    if keyword_end > expr_start {
        return None;
    }
    let before = text.get(..export_start)?;
    // `keyword_end..expr_start` is the inter-token whitespace/comment run
    // between the keyword and the expression (empty for `export default{`).
    let gap = text.get(keyword_end..expr_start)?;
    let expression = text.get(expr_start..expr_end)?;
    let after = text.get(expr_end..)?;

    let mut output = String::with_capacity(text.len() + REPLACEMENT.len());
    output.push_str(before);
    output.push_str(REPLACEMENT);
    if text.get(export_start..keyword_end)? != EXPORT_DEFAULT {
        output.push_str(text.get(export_start + 6..keyword_end - 7)?);
    }
    output.push_str(gap);
    output.push_str(expression);
    output.push_str(after);
    Some(output)
}

/// Rewrite `export default { ... }` to
/// `const __default__ = __vizeDefineComponent({ ... })` using the
/// `(export_start, object_start, object_end)` offsets (relative to `text`)
/// located by the AST. Returns `None` when the offsets do not describe a
/// plain object-literal default export inside `text`.
fn wrap_default_export_object(
    text: &str,
    default_object: Option<(usize, usize, usize)>,
) -> Option<String> {
    const EXPORT_DEFAULT: &str = "export default";

    let (export_start, object_start, object_end) = default_object?;
    if object_start >= object_end {
        return None;
    }
    let keyword_end = super::irregular_default::keyword_end(text, export_start)?;
    if keyword_end > object_start {
        return None;
    }
    let before = text.get(..export_start)?;
    let gap = text.get(keyword_end..object_start)?;
    let object = text.get(object_start..object_end)?;
    let after = text.get(object_end..)?;
    if !object.starts_with('{') {
        return None;
    }

    let mut output =
        String::with_capacity(text.len() + DEFINE_COMPONENT_REF.len() + EXPORT_DEFAULT.len());
    output.push_str(before);
    output.push_str("const __default__ =");
    if text.get(export_start..keyword_end)? != EXPORT_DEFAULT {
        output.push_str(text.get(export_start + 6..keyword_end - 7)?);
    }
    output.push_str(gap);
    output.push_str(DEFINE_COMPONENT_REF);
    output.push('(');
    output.push_str(object);
    output.push(')');
    output.push_str(after);
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::rewrite_export_default_for_module_scope;
    use crate::virtual_ts::generator::options_api::find_default_export_targets;

    /// Drive the rewrite exactly as the generator does: classify the default
    /// export with the shared single-parse `find_default_export_targets`, then
    /// feed the resulting AST byte offsets (here the whole script is one span,
    /// so offsets need no re-basing) into the rewrite.
    fn rewrite(script: &str) -> super::String {
        let targets = find_default_export_targets(script);
        rewrite_export_default_for_module_scope(script, targets.object, targets.expr)
    }

    #[test]
    fn plain_object_literal_is_wrapped_with_define_component() {
        // The Options API object-literal shape is unchanged by this PR: it is
        // still wrapped in `__vizeDefineComponent(...)` via the object path.
        let out = rewrite("export default { name: 'A' }");
        assert_eq!(
            out.as_str(),
            "const __default__ = __vizeDefineComponent({ name: 'A' })"
        );
    }

    #[test]
    fn define_component_call_is_rewritten_via_span() {
        // `export default defineComponent({...})` used to fall to the line
        // scanner; it now slices on the call expression span and is NOT wrapped
        // again in `defineComponent`.
        let out = rewrite("export default defineComponent({ name: 'A' })");
        assert_eq!(
            out.as_str(),
            "const __default__ = defineComponent({ name: 'A' })"
        );
    }

    #[test]
    fn anonymous_class_is_rewritten_via_span() {
        // Anonymous default classes have no name to alias by, so the class
        // path declines them; the generic expr span handles them instead.
        let out = rewrite("export default class { x = 1 }");
        assert_eq!(out.as_str(), "const __default__ = class { x = 1 }");
    }

    #[test]
    fn named_class_is_left_to_the_per_line_class_path() {
        // A *named* class default export is captured as `class`, not `expr`,
        // and the module-scope rewrite leaves it untouched (the per-line
        // setup-scope path keeps decorators on a real class declaration —
        // PR #1434). The module-scope function is a no-op here.
        let script = "export default class Foo { x = 1 }";
        let out = rewrite(script);
        assert_eq!(out.as_str(), script);
    }

    #[test]
    fn export_default_with_no_space_is_rewritten_correctly() {
        // `export default{` (no space before the brace). The old line scanner
        // required `export default` be followed by whitespace, so it failed to
        // match and emitted `export default{ ... }` verbatim — invalid at
        // module scope. The span path drops the keyword run exactly.
        let out = rewrite("export default{ name: 'A' }");
        assert_eq!(
            out.as_str(),
            "const __default__ =__vizeDefineComponent({ name: 'A' })"
        );
    }

    #[test]
    fn define_component_call_with_no_space_is_rewritten_correctly() {
        // Same no-space hazard, but a call expression so it takes the generic
        // expr path rather than the object-wrap path.
        let out = rewrite("export default(defineComponent({ name: 'A' }))");
        assert_eq!(
            out.as_str(),
            "const __default__ =(defineComponent({ name: 'A' }))"
        );
    }

    #[test]
    fn multi_line_call_is_rewritten_across_lines() {
        // An awkwardly formatted multi-line default export. The old scanner
        // rewrote only the first physical line containing `export default`,
        // leaving trailing lines under a dangling `const`; the span path
        // replaces just the keyword run and keeps the expression — including
        // its newlines — verbatim.
        let script = "export default\n  defineComponent({\n    name: 'A',\n  })";
        let out = rewrite(script);
        assert_eq!(
            out.as_str(),
            "const __default__ =\n  defineComponent({\n    name: 'A',\n  })"
        );
    }

    #[test]
    fn commented_and_split_keywords_keep_comments_before_the_expression() {
        for header in [
            "export /* 日本 */ default",
            "export\r\ndefault",
            "export // @ts-ignore\ndefault",
        ] {
            let script = vize_carton::cstr!("{header} {{ name: 'A' }}");
            let out = rewrite(&script);
            let gap = &header[6..header.len() - 7];
            assert_eq!(
                out.as_str(),
                vize_carton::cstr!(
                    "const __default__ ={gap} __vizeDefineComponent({{ name: 'A' }})"
                )
            );
            let script = vize_carton::cstr!("{header} defineComponent({{ name: 'A' }})");
            assert_eq!(
                rewrite(&script).as_str(),
                vize_carton::cstr!("const __default__ ={gap} defineComponent({{ name: 'A' }})")
            );
        }
    }

    #[test]
    fn leading_code_before_export_default_is_preserved() {
        // Statements before the default export are copied verbatim; only the
        // keyword run is replaced.
        let script = "const a = 1;\nexport default defineComponent({ a })";
        let out = rewrite(script);
        assert_eq!(
            out.as_str(),
            "const a = 1;\nconst __default__ = defineComponent({ a })"
        );
    }

    #[test]
    fn re_export_default_is_left_untouched() {
        // `export { default } from './x'` is not a default-export declaration;
        // neither offset is produced and the text is returned unchanged.
        let script = "export { default } from './x'";
        let out = rewrite(script);
        assert_eq!(out.as_str(), script);
    }
}
