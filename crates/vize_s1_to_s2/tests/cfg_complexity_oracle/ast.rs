//! The naive evaluator's expression half: re-parse the folio text into a
//! fresh arena, record every AST node with a parent link, and derive each
//! operator tree and `?:` from those links alone (no inherited counters).

use oxc_ast::AstKind;
use oxc_ast::ast::LogicalOperator;
use oxc_ast_visit::Visit;
use oxc_span::{GetSpan as _, SourceType};
use vize_s0::Allocator;

/// One decision-bearing node of a re-parsed expression.
pub struct ExprFact {
    pub kind: &'static str,
    pub start: u32,
    pub end: u32,
    pub enclosing_conditionals: u32,
    pub decisions: u32,
    pub runs: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Logical(LogicalOperator, u32),
    Conditional,
    Paren,
    Other,
}

struct Node {
    kind: Kind,
    start: u32,
    end: u32,
    parent: Option<usize>,
}

#[derive(Default)]
struct Collect {
    nodes: Vec<Node>,
    stack: Vec<usize>,
}

impl<'a> Visit<'a> for Collect {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        let (node_kind, span) = match kind {
            AstKind::LogicalExpression(it) => {
                (Kind::Logical(it.operator, it.left.span().end), it.span)
            }
            AstKind::ConditionalExpression(it) => (Kind::Conditional, it.span),
            AstKind::ParenthesizedExpression(it) => (Kind::Paren, it.span),
            other => (Kind::Other, other.span()),
        };
        self.nodes.push(Node {
            kind: node_kind,
            start: span.start,
            end: span.end,
            parent: self.stack.last().copied(),
        });
        self.stack.push(self.nodes.len() - 1);
    }

    fn leave_node(&mut self, _kind: AstKind<'a>) {
        self.stack.pop();
    }
}

/// Re-parse `source` and derive every logical tree and `?:` from the
/// node list's parent links alone.
pub fn expression_facts(source: &str) -> Vec<ExprFact> {
    let allocator = Allocator::new();
    let parsed = oxc_parser::Parser::new(allocator.as_oxc(), source, SourceType::ts())
        .parse_expression()
        .expect("folio js(...) text re-parses: it was admitted once");
    let mut collect = Collect::default();
    collect.visit_expression(&parsed);
    let nodes = collect.nodes;
    let ancestors =
        |index: usize| std::iter::successors(nodes[index].parent, |&at| nodes[at].parent);
    let mut facts = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        let enclosing_conditionals = ancestors(index)
            .filter(|&at| nodes[at].kind == Kind::Conditional)
            .count() as u32;
        match node.kind {
            Kind::Conditional => facts.push(ExprFact {
                kind: "conditional",
                start: node.start,
                end: node.end,
                enclosing_conditionals,
                decisions: 1,
                runs: 0,
            }),
            Kind::Logical(..) => {
                let governed = ancestors(index)
                    .find(|&at| nodes[at].kind != Kind::Paren)
                    .is_some_and(|at| matches!(nodes[at].kind, Kind::Logical(..)));
                if governed {
                    continue;
                }
                // Tree members: logical nodes whose climb to this root
                // crosses only parentheses and logical nodes.
                let mut members: Vec<(u32, LogicalOperator)> = nodes
                    .iter()
                    .enumerate()
                    .filter_map(|(at, member)| match member.kind {
                        Kind::Logical(op, left_end) => (at == index
                            || ancestors(at)
                                .take_while(|&up| {
                                    matches!(nodes[up].kind, Kind::Paren | Kind::Logical(..))
                                })
                                .any(|up| up == index))
                        .then_some((left_end, op)),
                        _ => None,
                    })
                    .collect();
                members.sort_by_key(|(left_end, _)| *left_end);
                let runs = 1 + members
                    .windows(2)
                    .filter(|pair| pair[0].1 != pair[1].1)
                    .count() as u32;
                facts.push(ExprFact {
                    kind: "logical",
                    start: node.start,
                    end: node.end,
                    enclosing_conditionals,
                    decisions: members.len() as u32,
                    runs,
                });
            }
            Kind::Paren | Kind::Other => {}
        }
    }
    facts
}
