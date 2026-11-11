//! vue/no-dupe-v-else-if
//!
//! Disallow duplicate conditions in `v-if` / `v-else-if` chains.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-if="foo"></div>
//! <div v-else-if="foo"></div>
//!
//! <div v-if="a === 1"></div>
//! <div v-else-if="a === 2"></div>
//! <div v-else-if="a === 1"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-if="foo"></div>
//! <div v-else-if="bar"></div>
//! <div v-else></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{Expression, RegExpLiteral, StringLiteral, TemplateElement};
use oxc_ast_visit::Visit;
use oxc_span::GetSpan;
use oxc_syntax::operator::LogicalOperator;
use vize_l0::FxHashSet;
use vize_l0::String;
use vize_relief::{
    ElementNode, ExpressionNode, JsExpression, PropNode, RootNode, SourceLocation,
    TemplateChildNode,
};

static META: RuleMeta = RuleMeta {
    name: "vue/no-dupe-v-else-if",
    description: "Disallow duplicate conditions in `v-if` / `v-else-if` chains",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow duplicate v-else-if conditions
pub struct NoDupeVElseIf;

/// Info about v-if directive on an element
struct IfDirectiveInfo<'a> {
    has_v_if: bool,
    has_v_else_if: bool,
    has_v_else: bool,
    condition: Option<JsExpression<'a>>,
    loc: Option<SourceLocation>,
}

impl Rule for NoDupeVElseIf {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        // Check root children
        check_element_children(ctx, &root.children);
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Check element children
        check_element_children(ctx, &element.children);
    }
}

/// Check children for duplicate v-else-if conditions
fn check_element_children(ctx: &mut LintContext, children: &[TemplateChildNode]) {
    let mut seen_clauses: Vec<FxHashSet<String>> = Vec::new();
    let mut in_if_chain = false;

    for child in children.iter() {
        if let TemplateChildNode::Element(el) = child {
            let info = get_if_directive_info(el);

            if info.has_v_if {
                // Start a new chain
                seen_clauses.clear();
                in_if_chain = true;
                if let Some(clauses) = info.condition.and_then(condition_clauses) {
                    seen_clauses.extend(clauses);
                }
            } else if info.has_v_else_if && in_if_chain {
                // Continue the chain
                if let Some(clauses) = info.condition.and_then(condition_clauses) {
                    if clauses
                        .iter()
                        .all(|clause| seen_clauses.iter().any(|seen| seen.is_subset(clause)))
                    {
                        // Report the error
                        if let Some(loc) = info.loc {
                            ctx.error_with_help(
                                ctx.t("vue/no-dupe-v-else-if.message"),
                                &loc,
                                ctx.t("vue/no-dupe-v-else-if.help"),
                            );
                        }
                    }
                    seen_clauses.extend(clauses);
                }
            } else if info.has_v_else && in_if_chain {
                // End the chain
                in_if_chain = false;
                seen_clauses.clear();
            } else {
                // Not part of an if chain, reset
                in_if_chain = false;
                seen_clauses.clear();
            }
        }
        // Non-element nodes (text, comment, etc.) are ignored but chain continues
    }
}

/// Get v-if/v-else-if/v-else directive info from an element
fn get_if_directive_info<'a>(el: &ElementNode<'a>) -> IfDirectiveInfo<'a> {
    let mut info = IfDirectiveInfo {
        has_v_if: false,
        has_v_else_if: false,
        has_v_else: false,
        condition: None,
        loc: None,
    };

    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop {
            match dir.name {
                "if" => {
                    info.has_v_if = true;
                    if let Some(ref exp) = dir.exp {
                        info.condition = retained_expression(exp);
                    }
                    info.loc = Some(dir.loc.clone());
                }
                "else-if" => {
                    info.has_v_else_if = true;
                    if let Some(ref exp) = dir.exp {
                        info.condition = retained_expression(exp);
                    }
                    info.loc = Some(dir.loc.clone());
                }
                "else" => {
                    info.has_v_else = true;
                }
                _ => {}
            }
        }
    }

    info
}

fn retained_expression<'a>(expression: &ExpressionNode<'a>) -> Option<JsExpression<'a>> {
    let ExpressionNode::Simple(simple) = expression else {
        return None;
    };
    simple
        .js_ast
        .filter(|retained| retained.raw == simple.content)?
        .as_expression()
}

/// Convert a condition to disjunctive clauses. Each clause is an unordered set
/// of conjunction atoms, so an earlier subset makes a later branch unreachable.
fn condition_clauses(condition: JsExpression<'_>) -> Option<Vec<FxHashSet<String>>> {
    clauses(condition.ast, condition.raw)
}

const MAX_CLAUSES: usize = 256;

fn clauses(expression: &Expression<'_>, source: &str) -> Option<Vec<FxHashSet<String>>> {
    match expression.without_parentheses() {
        Expression::LogicalExpression(logical) if logical.operator == LogicalOperator::Or => {
            let mut left = clauses(&logical.left, source)?;
            left.extend(clauses(&logical.right, source)?);
            Some(left)
        }
        Expression::LogicalExpression(logical) if logical.operator == LogicalOperator::And => {
            let left = clauses(&logical.left, source)?;
            let right = clauses(&logical.right, source)?;
            let product_len = left.len().saturating_mul(right.len());
            if product_len > MAX_CLAUSES {
                return None;
            }
            let mut product = Vec::with_capacity(product_len);
            for left_clause in left {
                for right_clause in &right {
                    let mut clause = left_clause.clone();
                    clause.extend(right_clause.iter().cloned());
                    product.push(clause);
                }
            }
            Some(product)
        }
        atom => {
            let key = token_key(atom, source)?;
            let mut clause = FxHashSet::default();
            clause.insert(key);
            Some(vec![clause])
        }
    }
}

/// Spans copied verbatim: the inside of a literal is one token.
#[derive(Default)]
struct LiteralSpans(Vec<(usize, usize)>);

impl<'a> Visit<'a> for LiteralSpans {
    fn visit_string_literal(&mut self, literal: &StringLiteral<'a>) {
        self.0
            .push((literal.span.start as usize, literal.span.end as usize));
    }

    fn visit_template_element(&mut self, element: &TemplateElement<'a>) {
        self.0
            .push((element.span.start as usize, element.span.end as usize));
    }

    fn visit_reg_exp_literal(&mut self, literal: &RegExpLiteral<'a>) {
        self.0
            .push((literal.span.start as usize, literal.span.end as usize));
    }
}

/// The operand's tokens joined into one string, like eslint-plugin-vue's
/// `equalTokens`: comments and whitespace between tokens are dropped, except
/// where dropping them would merge two tokens (`typeof a` vs `typeofa`).
fn token_key(atom: &Expression<'_>, source: &str) -> Option<String> {
    let span = atom.span();
    let (start, end) = (span.start as usize, span.end as usize);
    let bytes = source.as_bytes().get(start..end)?;
    let mut literals = LiteralSpans::default();
    literals.visit_expression(atom);
    literals.0.sort_unstable();
    let mut literals = literals.0.into_iter().peekable();
    let mut output: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut gap = false;
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        while literals
            .peek()
            .is_some_and(|&(literal_start, _)| literal_start < start + index)
        {
            literals.next();
        }
        if let Some(&(literal_start, literal_end)) = literals.peek()
            && literal_start == start + index
        {
            let literal = bytes.get(index..literal_end.checked_sub(start)?)?;
            literals.next();
            if !literal.is_empty() {
                push_token(&mut output, literal, &mut gap);
                index += literal.len();
            }
            continue;
        }
        if byte.is_ascii_whitespace() {
            gap = true;
            index += 1;
        } else if bytes.get(index..index + 2) == Some(b"//") {
            gap = true;
            while bytes
                .get(index)
                .is_some_and(|byte| !matches!(*byte, b'\n' | b'\r'))
            {
                index += 1;
            }
        } else if bytes.get(index..index + 2) == Some(b"/*") {
            gap = true;
            index += 2;
            while index < bytes.len() && bytes.get(index..index + 2) != Some(b"*/") {
                index += 1;
            }
            index += 2;
        } else {
            push_token(&mut output, &[byte], &mut gap);
            index += 1;
        }
    }
    std::str::from_utf8(&output).ok().map(Into::into)
}

fn push_token(output: &mut Vec<u8>, token: &[u8], gap: &mut bool) {
    if std::mem::take(gap)
        && let (Some(&left), Some(&right)) = (output.last(), token.first())
        && tokens_merge(left, right)
    {
        output.push(b' ');
    }
    output.extend_from_slice(token);
}

/// Whether `left` and `right` written without whitespace lex as one token.
fn tokens_merge(left: u8, right: u8) -> bool {
    let word = |byte: u8| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'\\') || byte >= 0x80
    };
    (word(left) && word(right))
        || (right == b'=' && b"<>=!+-*/%&|^?".contains(&left))
        || matches!(
            (left, right),
            (b'.', b'.')
                | (b'=', b'>')
                | (b'+', b'+')
                | (b'-', b'-')
                | (b'*', b'*')
                | (b'<', b'<')
                | (b'>', b'>')
                | (b'&', b'&')
                | (b'|', b'|')
                | (b'?', b'?' | b'.')
                | (b'/', b'/' | b'*')
        )
}

#[cfg(test)]
mod tests {
    use super::NoDupeVElseIf;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDupeVElseIf));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_no_duplicates() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="foo"></div><div v-else-if="bar"></div><div v-else></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_condition() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="foo"></div><div v-else-if="foo"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_duplicate_in_chain() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="a === 1"></div><div v-else-if="a === 2"></div><div v-else-if="a === 1"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_respaced_reordered_and_covered_conditions() {
        let result = create_linter().lint_template(
            r#"<p v-if="status === 'ready'"/><p v-else-if="status==='ready'"/>
<p v-if="isOpen && isPinned"/><p v-else-if="isPinned && isOpen"/>
<p v-if="loading || failed"/><p v-else-if="failed"/>
<p v-if="isOpen"/><p v-else-if="isOpen && isPinned"/>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 4, "{:?}", result.diagnostics);
    }

    #[test]
    fn compares_operands_by_tokens() {
        let linter = create_linter();
        for source in [
            r#"<p v-if="s === `row-${a}`"/><p v-else-if="s === `row-${ a }`"/>"#,
            r#"<p v-if="/'/.test(s)"/><p v-else-if="/'/.test( s )"/>"#,
            r#"<p v-if="a /* note */ === 1"/><p v-else-if="a === 1"/>"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(result.error_count, 1, "{source}: {:?}", result.diagnostics);
        }
        for source in [
            r#"<p v-if="typeof a === 'string'"/><p v-else-if="typeofa === 'string'"/>"#,
            r#"<p v-if="s === 'a b'"/><p v-else-if="s === 'ab'"/>"#,
            r#"<p v-if="s.match(/ a /)"/><p v-else-if="s.match(/a/)"/>"#,
            r#"<p v-if="a + +b"/><p v-else-if="a++ + b"/>"#,
            r#"<p v-if="(a as boolean)"/><p v-else-if="a"/>"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(result.error_count, 0, "{source}: {:?}", result.diagnostics);
        }
    }

    #[test]
    fn skips_exponentially_large_clause_products() {
        let groups = (0..9)
            .map(|index| format!("(a{index} || b{index})"))
            .collect::<Vec<_>>()
            .join(" && ");
        let source = format!(r#"<p v-if="{groups}"/><p v-else-if="{groups}"/>"#);
        assert_eq!(
            create_linter()
                .lint_template(&source, "test.vue")
                .error_count,
            0
        );
    }
}
