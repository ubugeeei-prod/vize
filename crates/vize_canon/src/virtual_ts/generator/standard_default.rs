//! Existing per-line projection of contiguous `export default` declarations.

use super::{options_api::OptionsApiScriptFacts, options_api_support, spans::DEFINE_COMPONENT_REF};
use std::borrow::Cow;
use vize_carton::cstr;

pub(super) struct Context<'a> {
    pub facts: &'a OptionsApiScriptFacts,
    pub has_script_setup: bool,
    pub inside_literal: bool,
}

#[derive(Default)]
pub(super) struct PendingDefault<'a> {
    pub wrap_close: Option<usize>,
    pub class_alias: Option<(usize, &'a str)>,
}

pub(super) fn rewrite_line<'s>(
    output_line: &mut Cow<'_, str>,
    line: &str,
    (line_start, line_end): (usize, usize),
    script: &'s str,
    context: Context<'_>,
    pending: &mut PendingDefault<'s>,
) -> bool {
    // Strip `export` from non-import lines inside setup scope
    let trimmed_line = output_line.trim_start();
    let Some(default_expr) = trimmed_line
        .strip_prefix("export default")
        .filter(|rest| options_api_support::follows_default_keyword(rest))
        .filter(|_| !context.inside_literal)
    else {
        return false;
    };
    let leading_ws = output_line.strip_suffix(trimmed_line).unwrap_or_default();
    // A class default export (the class-component shape) stays
    // a real class declaration so `@Component()` decorators
    // remain valid (a bare `const __default__ = class {}`
    // rewrite moves them onto a class expression — TS1206). The
    // `const __default__ = <Name>` alias is deferred until the
    // class body closes.
    let class_default =
        if context.has_script_setup || matches!(output_line, std::borrow::Cow::Owned(_)) {
            None
        } else {
            context.facts.default_export.class.filter(
                |&(export_start, class_start, class_end, _, _)| {
                    export_start >= line_start
                        && class_start >= line_start
                        && class_start < line_end
                        && class_end > line_start
                        && class_start - line_start <= line.len()
                        && line.is_char_boundary(class_start - line_start)
                },
            )
        };
    // Wrap a plain object-literal default export (Options
    // API) with `defineComponent` so `this` in
    // computed/methods gets Vue's instance typing. Applies
    // only to the plain <script> block; any other shape keeps
    // the bare `const __default__ =` rewrite.
    let wrap_object = if class_default.is_some()
        || context.has_script_setup
        || matches!(output_line, std::borrow::Cow::Owned(_))
    {
        None
    } else {
        context
            .facts
            .default_export
            .object
            .filter(|&(export_start, object_start, _)| {
                export_start >= line_start
                    && object_start >= line_start
                    && object_start < line_end
                    && object_start - line_start <= line.len()
                    && line
                        .get(object_start - line_start..)
                        .is_some_and(|rest| rest.starts_with('{'))
            })
    };
    if let Some((class_end, name, class_text)) = class_default.and_then(|(_, cs, ce, ns, ne)| {
        Some((ce, script.get(ns..ne)?, line.get(cs - line_start..)?))
    }) {
        // Drop the `export default ` keyword, keep the class
        // (and any same-line trailing decorator) verbatim.
        *output_line = std::borrow::Cow::Owned(cstr!("{leading_ws}{class_text}").into());
        pending.class_alias = Some((class_end, name));
    } else if let Some((object_column, object_end)) =
        wrap_object.and_then(|(_, object_start, object_end)| {
            let object_column = object_start - line_start;
            (line.len() - default_expr.len() <= object_column)
                .then_some((object_column, object_end))
        })
    {
        let keyword_end = line.len() - default_expr.len();
        let separator = line.get(keyword_end..object_column).unwrap_or_default();
        let single_line = (object_end <= line_end)
            .then(|| object_end - line_start)
            .and_then(|close_column| {
                Some((
                    line.get(object_column..close_column)?,
                    line.get(close_column..)?,
                ))
            });
        if let Some((object, after)) = single_line {
            // Single-line `export default { ... }`.
            *output_line = std::borrow::Cow::Owned(
                    cstr!(
                        "{leading_ws}const __default__ ={separator}{DEFINE_COMPONENT_REF}({object}){after}",
                    )
                    .into(),
                );
        } else {
            pending.wrap_close = Some(object_end);
            *output_line = std::borrow::Cow::Owned(
                cstr!(
                    "{leading_ws}const __default__ ={separator}{DEFINE_COMPONENT_REF}({}",
                    line.get(object_column..).unwrap_or_default(),
                )
                .into(),
            );
        }
    } else {
        *output_line = std::borrow::Cow::Owned(
            cstr!("{leading_ws}const __default__ ={}", default_expr).into(),
        );
    }
    true
}
