//! Free reads of the authored setup type parameters, including their scopes.
//! This fragment is outside the script Program. Parse it only on unused demand,
//! after the retained script's semantic pass finds an unread candidate.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_carton::{CompactString, FxHashSet, cstr};

pub(super) fn free_reads(generic: &str) -> Option<Vec<CompactString>> {
    // An anonymous function gives all type parameters their real lexical
    // scope, without inventing a binding that could shadow an authored import.
    let source = cstr!("(function<{generic}>() {{}});");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let [Statement::ExpressionStatement(statement)] = parsed.program.body.as_slice() else {
        return None;
    };
    let Expression::FunctionExpression(function) = statement.expression.get_inner_expression()
    else {
        return None;
    };
    let parameters = function.type_parameters.as_ref()?;
    // Require precisely the authored attribute between the wrapper brackets.
    // Embedded statements or a prematurely closed parameter list are refused.
    if parameters.span.start != 9 || parameters.span.end as usize != 11 + generic.len() {
        return None;
    }
    // OXC permits repeated type-parameter symbols. Their invalid authored scope
    // cannot prove an import unread, even when the semantic pass has no error.
    let mut names = FxHashSet::default();
    if parameters
        .params
        .iter()
        .any(|parameter| !names.insert(parameter.name.name.as_str()))
    {
        return None;
    }
    let built = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .with_build_nodes(true)
        .build(&parsed.program);
    if !built.diagnostics.is_empty() {
        return None;
    }
    let semantic = &built.semantic;
    let scoping = semantic.scoping();
    Some(
        scoping
            .root_unresolved_references_ids()
            .flatten()
            .filter_map(|id| {
                let reference = scoping.get_reference(id);
                let flags = reference.flags();
                (flags.is_read() || flags.is_type() || flags.is_value_as_type())
                    .then(|| CompactString::new(semantic.reference_name(reference)))
            })
            .collect(),
    )
}
