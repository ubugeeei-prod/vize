use oxc_ast::{
    AstKind,
    ast::{
        AssignmentExpression, AssignmentTarget, BindingPattern, Expression, IdentifierReference,
        TSType,
    },
};
use oxc_ast_visit::{Visit, walk::walk_assignment_expression};
use oxc_parser::Parser;
use oxc_semantic::{Scoping, Semantic, SemanticBuilder, SymbolId};
use oxc_span::{ContentEq, GetSpan, SourceType, Span};

use super::{
    AssignmentIndex, keyof_indexed_object_from_cast, keyof_operand_from_cast, peel_expression,
};

impl AssignmentIndex {
    pub(in super::super) fn new(source: &str, tsx: bool) -> Self {
        if !source.contains("keyof") {
            return Self::default();
        }
        let allocator = oxc_allocator::Allocator::default();
        let source_type = if tsx {
            SourceType::tsx()
        } else {
            SourceType::ts()
        };
        let parsed = Parser::new(&allocator, source, source_type).parse();
        if !parsed.diagnostics.is_empty() {
            return Self::default();
        }
        let semantic = SemanticBuilder::new()
            .with_build_nodes(true)
            .build(&parsed.program);
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
            semantic: &semantic.semantic,
            types: &types,
            offsets: Vec::new(),
        };
        visitor.visit_program(&parsed.program);
        visitor.offsets.sort_unstable();
        visitor.offsets.dedup();
        Self {
            offsets: visitor.offsets,
        }
    }

    pub(in super::super) fn matches_at(&self, offset: u32) -> bool {
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

    pub(super) fn equivalent(&self, left: &TSType<'_>, right: &TSType<'_>) -> bool {
        if !left.content_eq(right) {
            return false;
        }
        let symbols = |span: Span| {
            let start = self
                .references
                .partition_point(|(reference, _)| reference.start < span.start);
            self.references
                .get(start..)
                .unwrap_or_default()
                .iter()
                .take_while(move |(reference, _)| reference.end <= span.end)
                .map(|(_, symbol)| *symbol)
        };
        // Equal spelling is insufficient when a nested scope shadows a type parameter.
        let left = symbols(left.span()).collect::<Option<Vec<_>>>();
        let right = symbols(right.span()).collect::<Option<Vec<_>>>();
        left.is_some() && left == right
    }
}

impl<'a> Visit<'a> for TypeReferences<'_> {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.references
            .push((identifier.span, self.symbol(identifier)));
    }
}

struct Visitor<'ctx, 'ast> {
    semantic: &'ctx Semantic<'ast>,
    types: &'ctx TypeReferences<'ctx>,
    offsets: Vec<u32>,
}

impl<'ast> Visitor<'_, 'ast> {
    fn value_object<'expr>(
        &'expr self,
        assignment: &'expr AssignmentExpression<'ast>,
    ) -> Option<&'expr TSType<'ast>> {
        if let Some(object) = keyof_indexed_object_from_cast(&assignment.right, self.types) {
            return Some(object);
        }
        let Expression::Identifier(identifier) = peel_expression(&assignment.right) else {
            return None;
        };
        let symbol = self.types.symbol(identifier)?;
        let AstKind::VariableDeclarator(declarator) =
            self.semantic.symbol_declaration(symbol).kind()
        else {
            return None;
        };
        if declarator.type_annotation.is_some()
            || !matches!(declarator.id, BindingPattern::BindingIdentifier(_))
            || declarator.span.end > assignment.span.start
            || !self.types.scoping.symbol_redeclarations(symbol).is_empty()
            || self
                .types
                .scoping
                .get_resolved_references(symbol)
                .any(|reference| reference.is_write())
        {
            return None;
        }
        keyof_indexed_object_from_cast(declarator.init.as_ref()?, self.types)
    }
}

impl<'a> Visit<'a> for Visitor<'_, 'a> {
    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if assignment.operator.is_assign()
            && let AssignmentTarget::ComputedMemberExpression(member) = &assignment.left
            && let Some(target) = keyof_operand_from_cast(&member.expression)
            && let Some(value) = self.value_object(assignment)
            && self.types.equivalent(target, value)
        {
            // Only the indexed LHS diagnostic is known to be a false positive, not nested RHS errors.
            self.offsets.push(assignment.left.span().start);
        }
        walk_assignment_expression(self, assignment);
    }
}
