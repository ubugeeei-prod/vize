use std::borrow::Cow;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    ArrowFunctionExpression, Class, Expression, Function, IdentifierReference, StringLiteral,
    TemplateLiteral,
};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::scope::ScopeFlags;
use vize_carton::{String, append};
use vize_croquis::{
    Scope, ScopeData, TemplateExpression, VForScopeData, drawer::extract_identifiers_oxc,
};

pub(crate) fn append_ignored_vif_guard_open(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    guard: &str,
    purpose: &str,
) {
    let isolated = crate::virtual_ts::expressions::isolate_incomplete_expression(guard);
    let one_line = guard_on_ignored_line(isolated.as_str());
    let guard = one_line.as_ref();
    append!(
        *ts,
        "{indent}// @ts-ignore {purpose}; authored v-if checks own diagnostics.\n{indent}if ({guard}) {{\n"
    );
}

/// Only expression whitespace may move onto the ignored physical line.
/// Function/class bodies can contain ASI; templates and continued strings carry
/// newline-sensitive token values. Preserve those and every uncertain parse.
fn guard_on_ignored_line(guard: &str) -> Cow<'_, str> {
    if !guard.contains(['\n', '\r', '\u{2028}', '\u{2029}'])
        || guard.contains('/')
        || !vize_l0::expression_guard::expression_is_safe_to_parse(guard)
    {
        return Cow::Borrowed(guard);
    }
    // A slash can introduce a comment or regexp. The old comment scanner has
    // no regexp authority; retain all such guards, including division, exactly.
    let source = guard.trim();
    let allocator = Allocator::new();
    let Ok(expression) = Parser::new(&allocator, source, SourceType::ts()).parse_expression()
    else {
        return Cow::Borrowed(guard);
    };
    let span = expression.span();
    if source.get(span.start as usize..span.end as usize) != Some(source) {
        return Cow::Borrowed(guard);
    }
    let mut safety = GuardLineSafety {
        source,
        uncertain: false,
    };
    safety.visit_expression(&expression);
    if safety.uncertain {
        return Cow::Borrowed(guard);
    }
    Cow::Owned(
        source
            .chars()
            .map(|ch| {
                if matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
                    ' '
                } else {
                    ch
                }
            })
            .collect(),
    )
}

struct GuardLineSafety<'s> {
    source: &'s str,
    uncertain: bool,
}
impl<'a> Visit<'a> for GuardLineSafety<'_> {
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {
        self.uncertain = true;
    }
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {
        self.uncertain = true;
    }
    fn visit_class(&mut self, _: &Class<'a>) {
        self.uncertain = true;
    }
    fn visit_template_literal(&mut self, _: &TemplateLiteral<'a>) {
        self.uncertain = true;
    }
    fn visit_string_literal(&mut self, value: &StringLiteral<'a>) {
        self.uncertain |= self
            .source
            .get(value.span.start as usize..value.span.end as usize)
            .is_none_or(|raw| raw.contains(['\n', '\r', '\u{2028}', '\u{2029}']));
    }
}

/// Compute the enclosing v-if guard shared by all expressions in a v-for scope.
pub(super) fn common_vif_guard_prefix(exprs: &[&TemplateExpression]) -> Option<String> {
    let mut iter = exprs.iter();
    let first = iter.next()?.vif_guard.as_ref()?;
    let mut rest = Vec::new();
    for expr in iter {
        rest.push(expr.vif_guard.as_ref()?.as_str());
    }
    common_guard_prefix_from_terms(first.as_str(), rest.into_iter())
}

pub(super) fn common_vif_guard_prefix_for_guards(guards: &[&str]) -> Option<String> {
    let (first, rest) = guards.split_first()?;
    common_guard_prefix_from_terms(first, rest.iter().copied())
}

/// Replay every branch condition after a generated callback boundary.
/// Captured properties and mutable bindings also lose negative narrowing.
/// Generated-only comparison diagnostics belong to the existing guard ignores.
pub(super) fn callback_vif_guard(guard: &str, handler_binding: Option<&str>) -> Option<String> {
    if let Some(name) = handler_binding {
        let allocator = Allocator::default();
        if let Ok(expression) = Parser::new(&allocator, guard, SourceType::ts()).parse_expression()
        {
            let span = expression.span();
            if guard.get(span.start as usize..span.end as usize) == Some(guard.trim()) {
                let mut reference = GuardReference { name, used: false };
                reference.visit_expression(&expression);
                if !reference.used {
                    return None;
                }
            }
        }
    }
    Some(String::from(guard))
}

/// Keep only branch terms that narrow immutable callback-local aliases.
/// `v-for` aliases are emitted as `const` bindings and `v-slot` aliases are
/// parameters of the generated slot function, so TypeScript keeps their
/// narrowing across an authored callback boundary. Props and other render-time
/// values are deliberately excluded because they may change before the
/// listener runs (#11965, #11973, #12593).
pub(super) fn callback_vif_guard_for_stable_aliases(
    guard: &str,
    aliases: &[&str],
) -> Option<String> {
    if aliases.is_empty() {
        return None;
    }
    let terms: Vec<_> = split_guard_terms(guard)
        .into_iter()
        .filter(|term| references_any_alias(term, aliases))
        .collect();
    (!terms.is_empty()).then(|| String::from(terms.join(" && ").as_str()))
}

struct GuardReference<'s> {
    name: &'s str,
    used: bool,
}

impl<'a> Visit<'a> for GuardReference<'_> {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.used |= identifier.name == self.name;
    }
}

/// Remove a guard prefix that is already active in the surrounding generated
/// control flow.
///
/// Guards are compared term-by-term so conditions containing nested `&&`
/// expressions are not truncated accidentally. If `prefix` is not an exact
/// top-level prefix, the original guard is preserved.
pub(crate) fn remove_enclosing_vif_guard_prefix(guard: &str, prefix: &str) -> Option<String> {
    let guard_terms = split_guard_terms(guard);
    let prefix_terms = split_guard_terms(prefix);
    if prefix_terms.len() > guard_terms.len()
        || !prefix_terms
            .iter()
            .zip(guard_terms.iter())
            .all(|(prefix_term, guard_term)| prefix_term == guard_term)
    {
        return Some(String::from(guard));
    }

    let remaining = guard_terms.get(prefix_terms.len()..).unwrap_or_default();
    (!remaining.is_empty()).then(|| String::from(remaining.join(" && ").as_str()))
}

fn common_guard_prefix_from_terms<'a>(
    first: &'a str,
    rest: impl Iterator<Item = &'a str>,
) -> Option<String> {
    let mut common: Vec<&str> = split_guard_terms(first);

    for guard in rest {
        let terms = split_guard_terms(guard);
        let shared = common
            .iter()
            .zip(terms.iter())
            .take_while(|(a, b)| a == b)
            .count();
        common.truncate(shared);
        if common.is_empty() {
            return None;
        }
    }

    (!common.is_empty()).then(|| String::from(common.join(" && ").as_str()))
}

pub(super) fn common_vif_guard_prefix_outside_v_for(
    exprs: &[&TemplateExpression],
    data: &VForScopeData,
) -> Option<String> {
    let guard = common_vif_guard_prefix(exprs)?;
    trim_v_for_alias_guard_prefix(guard, data)
}

pub(super) fn common_vif_guard_prefix_outside_v_for_scope(
    exprs: &[&TemplateExpression],
    scope: &Scope,
) -> Option<String> {
    let ScopeData::VFor(data) = scope.data() else {
        return None;
    };
    common_vif_guard_prefix_outside_v_for(exprs, data)
}

pub(super) fn common_vif_guard_prefix_for_guards_outside_v_for(
    guards: &[&str],
    data: &VForScopeData,
) -> Option<String> {
    let guard = common_vif_guard_prefix_for_guards(guards)?;
    trim_v_for_alias_guard_prefix(guard, data)
}

fn trim_v_for_alias_guard_prefix(guard: String, data: &VForScopeData) -> Option<String> {
    let own_aliases = v_for_aliases(data);
    if own_aliases.is_empty() {
        return Some(guard);
    }

    let terms = split_guard_terms(guard.as_str());
    let outside_terms: Vec<&str> = terms
        .into_iter()
        .take_while(|term| !references_any_alias(term, own_aliases.as_slice()))
        .collect();

    (!outside_terms.is_empty()).then(|| String::from(outside_terms.join(" && ").as_str()))
}

pub(super) fn split_guard_terms(guard: &str) -> Vec<&str> {
    // Generated guards wrap each authored condition in parentheses. Preserve
    // those nodes: only the outer generated conjunction joins branch terms.
    // A byte-depth scan can split inside strings, comments, regexes or templates.
    let allocator = Allocator::default();
    let Ok(expression) = Parser::new(&allocator, guard, SourceType::ts()).parse_expression() else {
        return vec![guard];
    };
    // This parser API can accept a valid prefix without consuming trailing
    // tokens. Never discard them (or outside comments) when opening guards.
    let span = expression.span();
    if guard.get(span.start as usize..span.end as usize) != Some(guard.trim()) {
        return vec![guard];
    }
    let mut terms = Vec::new();
    collect_guard_terms(&expression, guard, &mut terms);
    terms
}

fn collect_guard_terms<'s>(expression: &Expression<'_>, source: &'s str, terms: &mut Vec<&'s str>) {
    if let Expression::LogicalExpression(logical) = expression
        && logical.operator.is_and()
    {
        collect_guard_terms(&logical.left, source, terms);
        collect_guard_terms(&logical.right, source, terms);
    } else {
        let span = expression.span();
        terms.push(
            source
                .get(span.start as usize..span.end as usize)
                .unwrap_or(source),
        );
    }
}

fn v_for_aliases(data: &VForScopeData) -> Vec<&str> {
    let mut aliases: Vec<&str> = data
        .value_bindings
        .iter()
        .map(|alias| alias.as_str())
        .collect();
    if let Some(alias) = data.key_alias.as_ref() {
        aliases.push(alias.as_str());
    }
    if let Some(alias) = data.index_alias.as_ref() {
        aliases.push(alias.as_str());
    }
    aliases
}

fn references_any_alias(term: &str, aliases: &[&str]) -> bool {
    extract_identifiers_oxc(term)
        .iter()
        .any(|identifier| aliases.iter().any(|alias| identifier.as_str() == *alias))
}

#[cfg(test)]
mod tests {
    use super::{
        callback_vif_guard, callback_vif_guard_for_stable_aliases, guard_on_ignored_line,
        remove_enclosing_vif_guard_prefix, split_guard_terms,
    };

    #[test]
    fn multiline_narrowing_copy_preserves_asi_and_literal_semantics() {
        assert_eq!(
            guard_on_ignored_line("enabled &&\n items[0].label === 'x'"),
            "enabled &&  items[0].label === 'x'"
        );
        for source in [
            "(() => { return\n true })()",
            "(function*() { yield\n value })()",
            "(() => { value\n ++other; return true })()",
            "String.raw`first\nsecond` === value",
            "value === 'first\\\nsecond'",
            "ready\n trailing",
            "/[/*][*/]/.test(text) &&\n ready",
            "ready && // comment\n visible",
            "total /\n count > 1",
        ] {
            assert_eq!(guard_on_ignored_line(source), source, "{source}");
        }
    }

    #[test]
    fn branch_boundaries_come_from_parsed_expressions() {
        for condition in [
            "(item.kind === ') && (')",
            r#"(item.kind === ") && (")"#,
            "(item.kind === `) && ${'('}`)",
            "(/\\) && \\(/.test(item.kind))",
            "(item.kind /* ) && ( */ === 'a')",
            "(item.kind // ) && (\n === 'a')",
            "(ready && enabled)",
            "((item as { kind: string }).kind === 'a')",
            "((item satisfies { kind: string }).kind === 'a')",
        ] {
            let mut guard = vize_carton::String::from("(parent) && ");
            guard.push_str(condition);
            assert_eq!(split_guard_terms(guard.as_str()), ["(parent)", condition]);
        }
        for guard in [
            "(parent) && (",
            "(parent) && (ready) trailing",
            "(parent) && (ready) // retained outside comment",
            "/* retained outside comment */ (parent) && (ready)",
        ] {
            assert_eq!(split_guard_terms(guard), [guard]);
        }
    }

    #[test]
    fn callback_guard_keeps_positive_and_negative_narrowing_terms() {
        assert_eq!(
            callback_vif_guard(
                "!(item.type === 'text') && ready && (item.type === 'container')",
                None
            )
            .as_deref(),
            Some("!(item.type === 'text') && ready && (item.type === 'container')")
        );
    }

    #[test]
    fn callback_guard_replays_negated_branch_terms() {
        for guard in ["!(row.item.kind === 'header')", "!(!draft)"] {
            assert_eq!(callback_vif_guard(guard, None).as_deref(), Some(guard));
        }
    }

    #[test]
    fn callback_guard_omits_only_conditions_unrelated_to_a_bare_handler_binding() {
        assert_eq!(
            callback_vif_guard("state.kind === 'entry'", Some("onPress")),
            None
        );
        assert_eq!(callback_vif_guard("state.handler", Some("handler")), None);
        for guard in [
            "typeof handler === 'function'",
            "typeof \\u0068andler === 'function'",
        ] {
            assert_eq!(
                callback_vif_guard(guard, Some("handler")).as_deref(),
                Some(guard)
            );
        }
        assert_eq!(
            callback_vif_guard("handler trailing", Some("onPress")).as_deref(),
            Some("handler trailing")
        );
    }

    #[test]
    fn callback_guard_replays_only_stable_alias_terms() {
        assert_eq!(
            callback_vif_guard_for_stable_aliases("ready && isItem(entry) && handler", &["entry"])
                .as_deref(),
            Some("isItem(entry)")
        );
        assert_eq!(
            callback_vif_guard_for_stable_aliases(
                "isItem(outer) && row.kind === 'item'",
                &["outer", "row"]
            )
            .as_deref(),
            Some("isItem(outer) && row.kind === 'item'")
        );
        assert_eq!(
            callback_vif_guard_for_stable_aliases("ready && handler", &["entry"]),
            None
        );
        assert_eq!(
            callback_vif_guard_for_stable_aliases("isItem(entry)", &[]),
            None
        );
    }

    #[test]
    fn removes_an_exact_enclosing_guard() {
        assert_eq!(
            remove_enclosing_vif_guard_prefix(
                "!(item.type === 'hunk-bar')",
                "!(item.type === 'hunk-bar')"
            ),
            None
        );
    }

    #[test]
    fn removes_only_complete_top_level_guard_terms() {
        assert_eq!(
            remove_enclosing_vif_guard_prefix(
                "(ready && enabled) && !(item.type === 'hunk-bar')",
                "(ready && enabled)",
            )
            .as_deref(),
            Some("!(item.type === 'hunk-bar')")
        );
    }

    #[test]
    fn preserves_a_guard_when_the_prefix_does_not_match() {
        assert_eq!(
            remove_enclosing_vif_guard_prefix("(ready) && (visible)", "(enabled)").as_deref(),
            Some("(ready) && (visible)")
        );
    }
}
