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
use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::operator::LogicalOperator;
use vize_l0::FxHashSet;
use vize_l0::String;
use vize_l0::ToCompactString;
use vize_relief::{
    ElementNode, ExpressionNode, PropNode, RootNode, SourceLocation, TemplateChildNode,
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
struct IfDirectiveInfo {
    has_v_if: bool,
    has_v_else_if: bool,
    has_v_else: bool,
    condition: Option<String>,
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
                if let Some(clauses) = info.condition.as_deref().and_then(condition_clauses) {
                    seen_clauses.extend(clauses);
                }
            } else if info.has_v_else_if && in_if_chain {
                // Continue the chain
                if let Some(clauses) = info.condition.as_deref().and_then(condition_clauses) {
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
fn get_if_directive_info(el: &ElementNode) -> IfDirectiveInfo {
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
                        info.condition = Some(get_expression_content(exp));
                    }
                    info.loc = Some(dir.loc.clone());
                }
                "else-if" => {
                    info.has_v_else_if = true;
                    if let Some(ref exp) = dir.exp {
                        info.condition = Some(get_expression_content(exp));
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

/// Get content from ExpressionNode
fn get_expression_content(expr: &ExpressionNode) -> String {
    match expr {
        ExpressionNode::Simple(s) => s.content.to_compact_string(),
        ExpressionNode::Compound(_) => "<compound>".to_compact_string(),
    }
}

/// Convert a condition to disjunctive clauses. Each clause is an unordered set
/// of conjunction atoms, so an earlier subset makes a later branch unreachable.
fn condition_clauses(condition: &str) -> Option<Vec<FxHashSet<String>>> {
    let allocator = Allocator::default();
    let expression = Parser::new(&allocator, condition, SourceType::ts())
        .parse_expression()
        .ok()?;
    clauses(&expression, condition)
}

fn clauses(expression: &Expression<'_>, source: &str) -> Option<Vec<FxHashSet<String>>> {
    match expression.get_inner_expression() {
        Expression::LogicalExpression(logical) if logical.operator == LogicalOperator::Or => {
            let mut left = clauses(&logical.left, source)?;
            left.extend(clauses(&logical.right, source)?);
            Some(left)
        }
        Expression::LogicalExpression(logical) if logical.operator == LogicalOperator::And => {
            let left = clauses(&logical.left, source)?;
            let right = clauses(&logical.right, source)?;
            let mut product = Vec::with_capacity(left.len().saturating_mul(right.len()));
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
            let span = atom.span();
            let raw = source.get(span.start as usize..span.end as usize)?;
            let key = strip_trivia(raw)?;
            let mut clause = FxHashSet::default();
            clause.insert(key);
            Some(vec![clause])
        }
    }
}

fn strip_trivia(source: &str) -> Option<String> {
    let bytes = source.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    let mut quote = None;
    let mut escaped = false;
    while let Some(&byte) = bytes.get(index) {
        if let Some(delimiter) = quote {
            output.push(byte);
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
            }
            index += 1;
            continue;
        }
        if matches!(byte, b'\'' | b'"' | b'`') {
            quote = Some(byte);
            output.push(byte);
            index += 1;
        } else if bytes.get(index..index + 2) == Some(b"//") {
            index += 2;
            while bytes
                .get(index)
                .is_some_and(|byte| !matches!(*byte, b'\n' | b'\r'))
            {
                index += 1;
            }
        } else if bytes.get(index..index + 2) == Some(b"/*") {
            index += 2;
            while index + 1 < bytes.len() && bytes.get(index..index + 2) != Some(b"*/") {
                index += 1;
            }
            if index + 1 >= bytes.len() {
                return None;
            }
            index += 2;
        } else {
            if !byte.is_ascii_whitespace() {
                output.push(byte);
            }
            index += 1;
        }
    }
    std::str::from_utf8(&output).ok().map(Into::into)
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
}
