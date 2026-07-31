use std::path::Path;

use oxc_ast::ast::{
    AssignmentExpression, AssignmentTarget, BindingPattern, Expression, TSIndexedAccessType,
    TSType, TSTypeOperatorOperator, VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_assignment_expression, walk_variable_declarator},
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use vize_carton::{FxHashMap, String};

pub(super) fn matches_at(source: &str, path: &Path, offset: u32) -> bool {
    let allocator = oxc_allocator::Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if !parsed.diagnostics.is_empty() {
        return false;
    }

    let mut visitor = Visitor {
        source,
        offset,
        value_casts: FxHashMap::default(),
        found: false,
    };
    visitor.visit_program(&parsed.program);
    visitor.found
}

struct Visitor<'source> {
    source: &'source str,
    offset: u32,
    value_casts: FxHashMap<String, Span>,
    found: bool,
}

impl<'a> Visit<'a> for Visitor<'_> {
    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if let BindingPattern::BindingIdentifier(id) = &declarator.id
            && let Some(init) = &declarator.init
            && let Some(object) = keyof_indexed_object_from_cast(init, self.source)
        {
            self.value_casts
                .insert(String::from(id.name.as_str()), object);
        }
        walk_variable_declarator(self, declarator);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if self.found {
            return;
        }
        if span_contains(assignment.span, self.offset)
            && assignment.operator.is_assign()
            && is_explicit_keyof_indexed_assignment(assignment, self.source, &self.value_casts)
        {
            self.found = true;
            return;
        }
        walk_assignment_expression(self, assignment);
    }
}

fn is_explicit_keyof_indexed_assignment(
    assignment: &AssignmentExpression<'_>,
    source: &str,
    value_casts: &FxHashMap<String, Span>,
) -> bool {
    let AssignmentTarget::ComputedMemberExpression(member) = &assignment.left else {
        return false;
    };
    let Some(target_keyof_object) = keyof_operand_from_cast(&member.expression) else {
        return false;
    };
    let Some(value_indexed_object) =
        indexed_assignment_value_object(&assignment.right, source, value_casts)
    else {
        return false;
    };
    same_type_text(source, target_keyof_object, value_indexed_object)
}

fn indexed_assignment_value_object(
    expression: &Expression<'_>,
    source: &str,
    value_casts: &FxHashMap<String, Span>,
) -> Option<Span> {
    if let Some(object) = keyof_indexed_object_from_cast(expression, source) {
        return Some(object);
    }
    let Expression::Identifier(identifier) = peel_expression(expression) else {
        return None;
    };
    value_casts.get(identifier.name.as_str()).copied()
}

fn keyof_operand_from_cast(expression: &Expression<'_>) -> Option<Span> {
    let Expression::TSAsExpression(ts_as) = peel_expression(expression) else {
        return None;
    };
    keyof_operand(&ts_as.type_annotation)
}

fn keyof_indexed_object_from_cast(expression: &Expression<'_>, source: &str) -> Option<Span> {
    let Expression::TSAsExpression(ts_as) = peel_expression(expression) else {
        return None;
    };
    let TSType::TSIndexedAccessType(indexed) = peel_type(&ts_as.type_annotation) else {
        return None;
    };
    keyof_indexed_object(indexed, source)
}

fn keyof_indexed_object(indexed: &TSIndexedAccessType<'_>, source: &str) -> Option<Span> {
    let object = peel_type(&indexed.object_type);
    let keyof = keyof_operand(&indexed.index_type)?;
    let object = object_span(object);
    same_type_text(source, object, keyof).then_some(object)
}

fn keyof_operand(ty: &TSType<'_>) -> Option<Span> {
    let TSType::TSTypeOperatorType(operator) = peel_type(ty) else {
        return None;
    };
    (operator.operator == TSTypeOperatorOperator::Keyof)
        .then(|| object_span(peel_type(&operator.type_annotation)))
}

fn object_span(ty: &TSType<'_>) -> Span {
    match ty {
        TSType::TSParenthesizedType(parenthesized) => {
            object_span(peel_type(&parenthesized.type_annotation))
        }
        _ => ty.span(),
    }
}

fn peel_expression<'expr, 'ast>(expression: &'expr Expression<'ast>) -> &'expr Expression<'ast> {
    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            peel_expression(&parenthesized.expression)
        }
        _ => expression,
    }
}

fn peel_type<'ty, 'ast>(ty: &'ty TSType<'ast>) -> &'ty TSType<'ast> {
    match ty {
        TSType::TSParenthesizedType(parenthesized) => peel_type(&parenthesized.type_annotation),
        _ => ty,
    }
}

fn same_type_text(source: &str, left: Span, right: Span) -> bool {
    span_text(source, left).is_some_and(|left| {
        span_text(source, right).is_some_and(|right| left.trim() == right.trim())
    })
}

fn span_text(source: &str, span: Span) -> Option<&str> {
    source.get(span.start as usize..span.end as usize)
}

fn span_contains(span: Span, offset: u32) -> bool {
    span.start <= offset && offset < span.end
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::matches_at;

    #[test]
    fn detects_false_positive_shape() {
        let source = r#"type WithOptionalBooleans<T> = {
  [K in keyof T as [T[K]] extends [boolean] ? K : never]?: T[K];
} & {
  [K in keyof T as [T[K]] extends [boolean] ? never : K]: T[K];
};

export function pickDefinedProps<T extends Record<string, unknown>>(
  source: T,
  key: string
): WithOptionalBooleans<T> {
  const result = {} as WithOptionalBooleans<T>;
  const value = source[
    key
  ] as WithOptionalBooleans<T>[keyof WithOptionalBooleans<T>];

  result[key as keyof WithOptionalBooleans<T>] = value;

  return result;
}
"#;
        let offset = source.find("result[key").unwrap() as u32;

        assert!(matches_at(
            source,
            PathBuf::from("foo.ts").as_path(),
            offset
        ));
    }

    #[test]
    fn requires_matching_types() {
        let source = r#"type A = { one: string };
type B = { two: number };
declare const target: A;
declare const key: string;
declare const value: B[keyof B];
target[key as keyof A] = value as B[keyof B];
"#;
        let offset = source.find("target[key").unwrap() as u32;

        assert!(!matches_at(
            source,
            PathBuf::from("foo.ts").as_path(),
            offset
        ));
    }
}
