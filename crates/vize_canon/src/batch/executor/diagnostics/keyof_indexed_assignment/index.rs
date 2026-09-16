use std::path::Path;

use oxc_ast::ast::{
    AssignmentExpression, AssignmentTarget, BindingPattern, Expression, IdentifierReference,
    VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_assignment_expression, walk_variable_declarator},
};
use oxc_parser::Parser;
use oxc_semantic::{Scoping, SemanticBuilder, SymbolId};
use oxc_span::{GetSpan, SourceType, Span};
use vize_carton::FxHashMap;

use super::{
    AssignmentIndex, keyof_indexed_object_from_cast, keyof_operand_from_cast, peel_expression,
    same_type_text,
};

impl AssignmentIndex {
    pub(super) fn new(source: &str, path: &Path) -> Self {
        if !source.contains("keyof") {
            return Self::default();
        }
        let allocator = oxc_allocator::Allocator::default();
        let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());
        let parsed = Parser::new(&allocator, source, source_type).parse();
        if !parsed.diagnostics.is_empty() {
            return Self::default();
        }
        let semantic = SemanticBuilder::new().build(&parsed.program);
        if !semantic.diagnostics.is_empty() {
            return Self::default();
        }
        let scoping = semantic.semantic.scoping();
        let mut types = TypeReferences {
            scoping,
            references: Vec::new(),
        };
        types.visit_program(&parsed.program);
        types.references.sort_by_key(|(span, _)| span.start);
        let mut visitor = Visitor {
            source,
            types: &types,
            value_casts: FxHashMap::default(),
            offsets: Vec::new(),
        };
        visitor.visit_program(&parsed.program);
        visitor.offsets.sort_unstable();
        visitor.offsets.dedup();
        Self {
            offsets: visitor.offsets,
        }
    }

    pub(super) fn matches_at(&self, offset: u32) -> bool {
        self.offsets.binary_search(&offset).is_ok()
    }
}

pub(super) struct TypeReferences<'a> {
    scoping: &'a Scoping,
    references: Vec<(Span, Option<SymbolId>)>,
}

impl TypeReferences<'_> {
    fn symbol(&self, identifier: &IdentifierReference<'_>) -> Option<SymbolId> {
        self.scoping
            .get_reference(identifier.reference_id.get()?)
            .symbol_id()
    }

    pub(super) fn equivalent(&self, source: &str, left: Span, right: Span) -> bool {
        if !same_type_text(source, left, right) {
            return false;
        }
        let symbols = |span: Span| {
            let start = self
                .references
                .partition_point(|(reference, _)| reference.start < span.start);
            self.references[start..]
                .iter()
                .take_while(move |(reference, _)| reference.end <= span.end)
                .map(|(_, symbol)| *symbol)
        };
        // Equal spelling is insufficient when a nested scope shadows a type parameter.
        let left = symbols(left).collect::<Option<Vec<_>>>();
        let right = symbols(right).collect::<Option<Vec<_>>>();
        left.is_some() && left == right
    }
}

impl<'a> Visit<'a> for TypeReferences<'_> {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.references
            .push((identifier.span, self.symbol(identifier)));
    }
}

struct Visitor<'a> {
    source: &'a str,
    types: &'a TypeReferences<'a>,
    value_casts: FxHashMap<SymbolId, (Span, u32)>,
    offsets: Vec<u32>,
}

impl Visitor<'_> {
    fn value_object(&self, assignment: &AssignmentExpression<'_>) -> Option<Span> {
        if let Some(object) =
            keyof_indexed_object_from_cast(&assignment.right, self.source, self.types)
        {
            return Some(object);
        }
        let Expression::Identifier(identifier) = peel_expression(&assignment.right) else {
            return None;
        };
        let (object, declaration_end) = self.value_casts.get(&self.types.symbol(identifier)?)?;
        (*declaration_end <= assignment.span.start).then_some(*object)
    }
}

impl<'a> Visit<'a> for Visitor<'_> {
    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if declarator.type_annotation.is_none()
            && let BindingPattern::BindingIdentifier(id) = &declarator.id
            && let Some(symbol) = id.symbol_id.get()
            && !self
                .types
                .scoping
                .get_resolved_references(symbol)
                .any(|reference| reference.is_write())
            && let Some(init) = &declarator.init
            && let Some(object) = keyof_indexed_object_from_cast(init, self.source, self.types)
        {
            self.value_casts
                .insert(symbol, (object, declarator.span.end));
        }
        walk_variable_declarator(self, declarator);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if assignment.operator.is_assign()
            && let AssignmentTarget::ComputedMemberExpression(member) = &assignment.left
            && let Some(target) = keyof_operand_from_cast(&member.expression)
            && let Some(value) = self.value_object(assignment)
            && self.types.equivalent(self.source, target, value)
        {
            // Only the indexed LHS diagnostic is known to be a false positive, not nested RHS errors.
            self.offsets.push(assignment.left.span().start);
        }
        walk_assignment_expression(self, assignment);
    }
}
