//! Template expressions resolve names lexically. Handler parameters, slot and
//! v-for aliases, and handler-local declarations shadow browser globals; type
//! positions and a direct `typeof name` operand do not read them at runtime.

use super::NoBrowserGlobalsInSsr;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, IdentifierReference, Statement, TSType, UnaryExpression};
use oxc_ast_visit::{Visit, walk::walk_unary_expression};
use oxc_parser::{ParseOptions, Parser};
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::SourceType;
use oxc_syntax::operator::UnaryOperator;
use vize_l0::{String, cstr, expression_guard::expression_is_safe_to_parse};

impl NoBrowserGlobalsInSsr {
    /// Names `expr` reads that no binding inside the expression declares.
    /// Unparsable text yields nothing; the template compiler reports it.
    pub(super) fn runtime_identifiers(expr: &str, directive: Option<&str>) -> Vec<String> {
        if !expression_is_safe_to_parse(expr) {
            return Vec::new();
        }
        match directive {
            // Aliases are template scope; only the iterable is read here.
            Some("for") => {
                let (_, source) = vize_croquis::drawer::parse_v_for_expression(expr);
                free_reads(&cstr!("({source}\n);"), false)
            }
            // Slot props are a parameter list; only their defaults read.
            Some("slot") => free_reads(&cstr!("(({expr}\n) => {{}});"), false),
            _ => free_reads(&cstr!("({expr}\n);"), false).or_else(|| {
                // Inline handlers may be statement bodies.
                (directive == Some("on"))
                    .then(|| free_reads(expr, true))
                    .flatten()
            }),
        }
        .unwrap_or_default()
    }
}

fn free_reads(source: &str, statements: bool) -> Option<Vec<String>> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts())
        .with_options(ParseOptions {
            allow_return_outside_function: statements,
            ..ParseOptions::default()
        })
        .parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    // A wrapper must stay one expression; `a); (b` must not escape it.
    if !statements
        && !matches!(
            parsed.program.body.as_slice(),
            [Statement::ExpressionStatement(_)]
        )
    {
        return None;
    }
    let built = SemanticBuilder::new().build(&parsed.program);
    if !built.diagnostics.is_empty() {
        return None;
    }
    let mut reads = FreeReads {
        semantic: &built.semantic,
        names: Vec::new(),
    };
    reads.visit_program(&parsed.program);
    Some(reads.names)
}

struct FreeReads<'s, 'a> {
    semantic: &'s Semantic<'a>,
    names: Vec<String>,
}

impl<'a> Visit<'a> for FreeReads<'_, 'a> {
    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        let Some(reference) = id.reference_id.get() else {
            return;
        };
        let reference = self.semantic.scoping().get_reference(reference);
        if reference.is_value() && reference.symbol_id().is_none() {
            self.names.push(String::from(id.name.as_str()));
        }
    }

    fn visit_ts_type(&mut self, _: &TSType<'a>) {}

    fn visit_unary_expression(&mut self, expression: &UnaryExpression<'a>) {
        if expression.operator == UnaryOperator::Typeof
            && matches!(
                expression.argument.get_inner_expression(),
                Expression::Identifier(_)
            )
        {
            return;
        }
        walk_unary_expression(self, expression);
    }
}
