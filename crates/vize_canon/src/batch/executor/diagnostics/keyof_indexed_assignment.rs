use std::path::Path;

use oxc_ast::ast::{Expression, TSIndexedAccessType, TSType, TSTypeOperatorOperator};

mod index;
use index::TypeReferences;

#[derive(Default)]
pub(super) struct AssignmentIndex {
    offsets: Vec<u32>,
}

impl super::DiagnosticMapper<'_> {
    pub(in super::super) fn is_keyof_indexed_assignment(
        &mut self,
        virtual_path: &Path,
        line: u32,
        column: u32,
    ) -> bool {
        let Some(file) = self.project.find_by_diagnostic_virtual(virtual_path) else {
            return false;
        };
        let Some(offset) = self.virtual_offset(file, line, column) else {
            return false;
        };
        self.keyof_assignments
            .entry(file.virtual_path.clone())
            .or_insert_with(|| AssignmentIndex::new(&file.content, &file.virtual_path))
            .matches_at(offset)
    }
}

#[cfg(test)]
fn matches_at(source: &str, path: &Path, offset: u32) -> bool {
    AssignmentIndex::new(source, path).matches_at(offset)
}

fn keyof_operand_from_cast<'expr, 'ast>(
    expression: &'expr Expression<'ast>,
) -> Option<&'expr TSType<'ast>> {
    let Expression::TSAsExpression(ts_as) = peel_expression(expression) else {
        return None;
    };
    keyof_operand(&ts_as.type_annotation)
}

fn keyof_indexed_object_from_cast<'expr, 'ast>(
    expression: &'expr Expression<'ast>,
    types: &TypeReferences,
) -> Option<&'expr TSType<'ast>> {
    let Expression::TSAsExpression(ts_as) = peel_expression(expression) else {
        return None;
    };
    let TSType::TSIndexedAccessType(indexed) = peel_type(&ts_as.type_annotation) else {
        return None;
    };
    keyof_indexed_object(indexed, types)
}

fn keyof_indexed_object<'ty, 'ast>(
    indexed: &'ty TSIndexedAccessType<'ast>,
    types: &TypeReferences,
) -> Option<&'ty TSType<'ast>> {
    let object = peel_type(&indexed.object_type);
    let keyof = keyof_operand(&indexed.index_type)?;
    types.equivalent(object, keyof).then_some(object)
}

fn keyof_operand<'ty, 'ast>(ty: &'ty TSType<'ast>) -> Option<&'ty TSType<'ast>> {
    let TSType::TSTypeOperatorType(operator) = peel_type(ty) else {
        return None;
    };
    (operator.operator == TSTypeOperatorOperator::Keyof)
        .then(|| peel_type(&operator.type_annotation))
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

#[cfg(test)]
#[path = "keyof_indexed_assignment/scope_tests.rs"]
mod scope_tests;

#[cfg(test)]
#[path = "keyof_indexed_assignment/mapping_tests.rs"]
mod mapping_tests;

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
